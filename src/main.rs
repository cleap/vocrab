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

fn main() {
    let filepath = "data/emos-vs-punks.json";
    println!();
    println!("Reading from {}.", filepath);
    let token_array = tokens_from_file(filepath).unwrap();

    let lemma_map: HashMap<String, (usize, usize)> = HashMap::new();

    for (sentence_i, sentence) in token_array.iter().enumerate() {
        for (token_i, token) in sentence.iter().enumerate() {
            if token.pos == "PUNCT" {
                continue;
            }
            println!(
                "[{:03}:{:03}] {:20} | {}",
                sentence_i + 1,
                token_i + 1,
                token.text,
                token.pos
            );
        }
        if sentence_i >= 5 {
            break;
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
