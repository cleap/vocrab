//use fltk::{app, button::Button, frame::Frame, prelude::*, window::Window};
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use cpython::{Python, PyResult, PyModule, PyObject, PySet, ObjectProtocol};
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
struct Token {
    text: String,
    lemma: String,
    pos: String
}

fn tokens_from_file<P: AsRef<Path>>(path: P) -> Result<bool, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let values: serde_json::Map<String, serde_json::Value> = serde_json::from_reader(reader)?;
    let temp = values.get("sentences").unwrap();
    Ok(true)
}

fn main() {

    let filepath = "data/emos-vs-punks.json";
    println!("Reading from {}.", filepath);
    tokens_from_file(filepath).unwrap();

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
