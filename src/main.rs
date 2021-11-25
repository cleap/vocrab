//use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

//use cpython::{ObjectProtocol, PyModule, PyObject, PyResult, PySet, Python};
use serde::Deserialize;
use serde_json::Value as JsonValue;

#[derive(Deserialize, Debug)]
struct Token {
    text: String,
    lemma: String,
    pos: String,
}

trait WordCount {
    fn word_count(&self) -> usize;
}

type LemmaMap = HashMap<String, HashMap<String, Vec<(usize, usize)>>>;
type FormMap = HashMap<String, Vec<(usize, usize)>>;

impl WordCount for FormMap {
    fn word_count(&self) -> usize {
        let mut len: usize = 0;
        for (_, list) in self {
            len += list.len();
        }
        len
    }
}

impl WordCount for LemmaMap {
    fn word_count(&self) -> usize {
        let mut count: usize = 0;
        for (_, form_map) in self {
            count += form_map.word_count();
        }
        count
    }
}

fn tokens_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<Token>>, Box<dyn Error>> {
    let file = File::open(path).expect("Could not open file");
    let reader = BufReader::new(file);
    let values: JsonValue =
        serde_json::from_reader(reader).expect("Could not convert reader to JSON Value");
    let sentences: Vec<JsonValue> = values
        .get("sentences")
        .unwrap()
        .as_array()
        .unwrap()
        .to_vec();

    let mut token_array: Vec<Vec<Token>> = Vec::new();

    for (sentence_i, sentence) in sentences.iter().enumerate() {
        token_array.push(Vec::new());

        let tokens: Vec<JsonValue> = sentence.as_array().unwrap().to_vec();
        for token in tokens.into_iter() {
            let token_struct: Token = serde_json::from_value(token).unwrap();
            token_array[sentence_i].push(token_struct);
        }
    }
    Ok(token_array)
}

fn add_to_map(token: &Token, pos: (usize, usize), lemma_map: &mut LemmaMap) {
    let lemma_key = token.lemma.to_lowercase();
    let form_key = token.text.to_lowercase();

    match lemma_map.get_mut(&lemma_key) {
        Some(form_map) => match form_map.get_mut(&form_key) {
            Some(list) => list.push(pos),
            None => {
                form_map.insert(form_key, vec![pos]);
            }
        },
        None => {
            lemma_map.insert(lemma_key, HashMap::from([(form_key, vec![pos])]));
        }
    }
}

fn map_from_array(token_array: &Vec<Vec<Token>>) -> LemmaMap {
    let mut lemma_map: LemmaMap = HashMap::new();

    for (sentence_i, sentence) in token_array.iter().enumerate() {
        for (token_i, token) in sentence.iter().enumerate() {
            if token.pos == "PUNCT" {
                continue;
            }

            add_to_map(&token, (sentence_i, token_i), &mut lemma_map);
        }
    }
    lemma_map
}

type LemmaVec<'a> = Vec<(&'a String, &'a HashMap<String, Vec<(usize, usize)>>)>;

fn main() {
    let filepath = "data/emos-vs-punks.json";
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

        let mut form_vec: Vec<(&String, &Vec<(usize, usize)>)> = form_map.iter().collect();
        form_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

        for (j, (form, poss)) in form_vec.into_iter().enumerate() {
            println!("               {:3}: {} × {}", j + 1, form, poss.len());
            print!("                      \t");
            let (sentence_i, token_i) = poss[0];
            for (k, token) in token_array[sentence_i].iter().enumerate() {
                if k != 0 && token.pos != "PUNCT" {
                    print!(" ");
                }
                if k == token_i {
                    print!("{}[1;4m", 0o033 as char);
                }
                print!("{}", token.text);
                if k == token_i {
                    print!("{}[22;24m", 0o033 as char);
                }
            }
            println!();
        }
    }
    /*
    let app = app::App::default().with_scheme(app::Scheme::Gtk);
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    wind.end();
    wind.show();
    but.set_callback(move |_| frame.set_label("Hello from rust"));
    app.run().unwrap();
    */
}
