//use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
extern crate vocrab;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{error::Error, io};
use structopt::StructOpt;
use textwrap::fill;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use vocrab::lemmatizer::*;

#[derive(StructOpt)]
struct Opt {
    // The json file to read from
    file: String,
}

#[derive(Clone, Copy)]
enum AppColumn {
    Lemmas,
    Forms,
    Usage,
}

impl AppColumn {
    fn next(self) -> Self {
        match self {
            AppColumn::Lemmas => AppColumn::Forms,
            AppColumn::Forms => AppColumn::Usage,
            AppColumn::Usage => AppColumn::Usage,
        }
    }

    fn prev(self) -> Self {
        match self {
            AppColumn::Lemmas => AppColumn::Lemmas,
            AppColumn::Forms => AppColumn::Lemmas,
            AppColumn::Usage => AppColumn::Forms,
        }
    }
}

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

// Application state
struct App<'a> {
    token_array: Vec<Vec<Token>>,
    lemma_map: &'a LemmaMap,
    lemma_vec: StatefulList<LemmaVecItem<'a>>,
    curr_lemma: Option<String>,
    form_vec: Option<StatefulList<FormVecItem<'a>>>,
    curr_form: Option<String>,
    usage_vec: Option<StatefulList<&'a (usize, usize)>>,
    column: AppColumn,
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load lemmatization
    let opt = Opt::from_args();
    let filepath = opt.file; //= "data/emos-vs-punks.json";
    let token_array = tokens_from_file(filepath).unwrap();
    let lemma_map = map_from_array(&token_array);
    let mut lemma_vec: LemmaVec = lemma_map.iter().collect();
    lemma_vec.sort_by(|a, b| b.1.word_count().cmp(&a.1.word_count()));

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App {
        token_array,
        lemma_map: &lemma_map,
        lemma_vec: StatefulList::with_items(lemma_vec),
        curr_lemma: None,
        form_vec: None,
        curr_form: None,
        usage_vec: None,
        column: AppColumn::Lemmas,
    };

    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn update_form(app: &mut App) {
    let (lemma, form_map) = match app.lemma_vec.state.selected() {
        Some(i) => app.lemma_vec.items[i],
        None => {
            app.lemma_vec.state.select(Some(0));
            app.lemma_vec.items[0]
        }
    };

    match &app.curr_lemma {
        Some(curr_lemma) if curr_lemma == lemma => {}
        _ => {
            let mut form_vec: FormVec = form_map.iter().collect();
            form_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
            app.form_vec = Some(StatefulList::with_items(form_vec));
            app.curr_lemma = Some(lemma.to_string());
            if let Some(form_vec) = &mut app.form_vec {
                form_vec.state.select(Some(0));
            }
        }
    };

    update_usage(app);
}

fn update_usage(app: &mut App) {
    let (form, usage_vec) = match &mut app.form_vec {
        Some(form_vec) => match form_vec.state.selected() {
            Some(i) => form_vec.items[i],
            None => {
                form_vec.state.select(Some(0));
                form_vec.items[0]
            }
        },
        None => {
            update_form(app);
            match &mut app.form_vec {
                Some(form_vec) => {
                    form_vec.state.select(Some(0));
                    form_vec.items[0]
                }
                None => {
                    eprintln!("Error: Unable to find form in update_usage");
                    std::process::exit(1);
                }
            }
        }
    };

    match &app.curr_form {
        Some(curr_form) if curr_form == form => {}
        _ => {
            let usage_vec: Vec<&(usize, usize)> = usage_vec.iter().collect();
            app.usage_vec = Some(StatefulList::with_items(usage_vec));
            app.curr_form = Some(form.to_string());
            if let Some(usage_vec) = &mut app.usage_vec {
                usage_vec.state.select(Some(0));
            }
        }
    };
}

fn enter_behavior(app: &mut App) {
    match app.column {
        AppColumn::Lemmas => {
            update_form(app);
            app.column = AppColumn::Forms;
        }
        AppColumn::Forms => {
            update_usage(app);
            app.column = AppColumn::Usage;
        }
        AppColumn::Usage => {}
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    update_form(&mut app);
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => return Ok(()),
                KeyCode::Char('h') => app.column = app.column.prev(),
                KeyCode::Char('j') => match app.column {
                    AppColumn::Lemmas => {
                        app.lemma_vec.next();
                        update_form(&mut app);
                    }
                    AppColumn::Forms => match &mut app.form_vec {
                        Some(vec) => {
                            vec.next();
                            update_usage(&mut app);
                        }
                        None => {}
                    },
                    AppColumn::Usage => match &mut app.usage_vec {
                        Some(usage) => usage.next(),
                        None => {}
                    },
                },
                KeyCode::Char('k') => match app.column {
                    AppColumn::Lemmas => {
                        app.lemma_vec.prev();
                        update_form(&mut app);
                    }
                    AppColumn::Forms => match &mut app.form_vec {
                        Some(vec) => {
                            vec.prev();
                            update_usage(&mut app);
                        }
                        None => {}
                    },
                    AppColumn::Usage => match &mut app.usage_vec {
                        Some(usage) => usage.prev(),
                        None => {}
                    },
                },
                KeyCode::Char('l') => enter_behavior(&mut app),
                KeyCode::Enter => enter_behavior(&mut app),
                _ => {}
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let size = f.size();

    let selected_style = Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("📚🦀 Vocrab 🦀📚")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded);
    f.render_widget(block, size);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(f.size());

    let title_style = match &app.column {
        AppColumn::Lemmas => selected_style,
        _ => Style::default(),
    };

    let lemmas: Vec<ListItem> = app
        .lemma_vec
        .items
        .iter()
        .enumerate()
        .filter_map(|(i, (lemma, _))| {
            let content = vec![Spans::from(Span::raw(format!("{:4}: {}", i + 1, lemma)))];
            Some(ListItem::new(content))
        })
        .collect();
    let lemmas = List::new(lemmas)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Lemmas", title_style))
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(title_style.add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");
    f.render_stateful_widget(lemmas, chunks[0], &mut app.lemma_vec.state);

    let title_style = match &app.column {
        AppColumn::Forms => selected_style,
        _ => Style::default(),
    };

    match &mut app.form_vec {
        Some(form_vec) => {
            let forms: Vec<ListItem> = form_vec
                .items
                .iter()
                .enumerate()
                .filter_map(|(i, (form, _))| {
                    let content = vec![Spans::from(Span::raw(format!("{:4}: {}", i + 1, form)))];
                    Some(ListItem::new(content))
                })
                .collect();
            let lemma: String = match &app.curr_lemma {
                Some(lemma) => lemma.to_string(),
                None => "".to_string(),
            };
            let forms = List::new(forms)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled(format!("Forms: {}", lemma), title_style))
                        .title_alignment(Alignment::Left)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(title_style.add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            f.render_stateful_widget(forms, chunks[1], &mut form_vec.state);
        }
        _ => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Forms", title_style))
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded);
            f.render_widget(block, chunks[1]);
        }
    }

    let title_style = match &app.column {
        AppColumn::Usage => selected_style,
        _ => Style::default(),
    };

    let usage = match &app.usage_vec {
        Some(usage_vec) => {
            let usage: Vec<ListItem> = usage_vec
                .items
                .iter()
                .enumerate()
                .filter_map(|(_i, (sentence_i, token_i))| {
                    let (before, word, after) =
                        get_sentence_split(&app.token_array, *sentence_i, *token_i);

                    let para_width = chunks[2].width as usize - 3;

                    let mut paragraph: Vec<Span> = match before.len() {
                        0 => vec![],
                        _ => fill(&before, para_width)
                            .split("\n")
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|line| Span::from(String::from(*line)))
                            .collect(),
                    };

                    match paragraph.last() {
                        Some(item) if item.content.len() != 0 => paragraph.push(Span::from(" ")),
                        _ => {}
                    }
                    paragraph.push(Span::styled(word, Style::default().fg(Color::Red)));

                    paragraph.append(
                        &mut fill(&after, para_width)
                            .split("\n")
                            .collect::<Vec<&str>>()
                            .iter()
                            .map(|line| Span::from(String::from(*line)))
                            .collect(),
                    );
                    // TODO: wrap text
                    let content = vec![Spans::from(paragraph)];
                    Some(ListItem::new(content))
                })
                .collect();
            let form: String = match &app.curr_form {
                Some(form) => form.to_string(),
                None => "".to_string(),
            };
            let usage = List::new(usage)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(Span::styled(format!("Usage: {}", form), title_style))
                        .title_alignment(Alignment::Left)
                        .border_type(BorderType::Rounded),
                )
                .highlight_style(title_style.add_modifier(Modifier::BOLD))
                .highlight_symbol("> ");
            Some(usage)
        }
        _ => {
            let block = Block::default()
                .borders(Borders::ALL)
                .title(Span::styled("Usage", title_style))
                .title_alignment(Alignment::Left)
                .border_type(BorderType::Rounded);
            f.render_widget(block, chunks[2]);
            None
        }
    };

    match &mut app.usage_vec {
        Some(usage_vec) => {
            if let Some(list) = usage {
                f.render_stateful_widget(list, chunks[2], &mut usage_vec.state);
            }
        }
        None => {}
    };

    /*
    let title_style = match &app.column {
        AppColumn::Usage => selected_style,
        _ => Style::default(),
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled("Usage", title_style))
        .title_alignment(Alignment::Left)
        .border_type(BorderType::Rounded);
    f.render_widget(block, chunks[2]);
    */

    /*
    let block = Block::default()
        .title(vec![
            Span::styled("With", Style::default().fg(Color::Yellow)),
            Span::from(" background"),
        ])
        .title_alignment(Alignment::Center)
        .style(Style::default().bg(Color::Green));
    f.render_widget(block, top_chunks[0]);

    let block = Block::default()
        .title(Span::styled(
            "Styled title",
            Style::default()
                .fg(Color::White)
                .bg(Color::Red)
                .add_modifier(Modifier::BOLD),
        ))
        .title_alignment(Alignment::Right);
    f.render_widget(block, top_chunks[1]);
    */
}

fn cli() {
    let opt = Opt::from_args();
    let filepath = opt.file; //= "data/emos-vs-punks.json";
    println!();
    println!("Reading from {}.", filepath);
    let token_array = tokens_from_file(filepath).unwrap();
    let lemma_map = map_from_array(&token_array);

    let mut lemma_vec: LemmaVec = lemma_map.iter().collect();
    lemma_vec.sort_by(|a, b| b.1.word_count().cmp(&a.1.word_count()));

    let word_count = lemma_map.word_count();
    println!("Word count: {}", word_count);

    let mut count: usize = 0;
    for (i, (lemma, form_map)) in lemma_vec.into_iter().enumerate() {
        count += form_map.word_count();
        let percentage = 100.0 * count as f32 / word_count as f32;
        println!(
            "[{:5.1}%] {:4}: {} × {}",
            percentage,
            i + 1,
            lemma,
            form_map.word_count()
        );

        let mut form_vec: FormVec = form_map.iter().collect();
        form_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (j, (form, poss)) in form_vec.into_iter().enumerate() {
            println!("               {:3}: {} × {}", j + 1, form, poss.len());
            print!("                      \t");
            println!(
                "{}",
                get_sentence_bolded(&token_array, poss[0].0, poss[0].1)
            );
        }
    }
}
