//use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
extern crate vocrab;

use vocrab::lemmatizer::*;

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
