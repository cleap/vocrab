use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
//use cpython::{ObjectProtocol, PyModule, PyObject, PyResult, PySet, Python};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use thiserror::Error;

pub type LemmaVecItem<'a> = (&'a String, &'a HashMap<String, Vec<(usize, usize)>>);
pub type LemmaVec<'a> = Vec<LemmaVecItem<'a>>;
pub type FormVecItem<'a> = (&'a String, &'a Vec<(usize, usize)>);
pub type FormVec<'a> = Vec<FormVecItem<'a>>;
pub type LemmaMap = HashMap<String, HashMap<String, Vec<(usize, usize)>>>;
pub type FormMap = HashMap<String, Vec<(usize, usize)>>;

#[derive(Deserialize, Debug)]
pub struct Token {
    text: String,
    lemma: String,
    pos: String,
}

pub trait WordCount {
    fn word_count(&self) -> usize;
}

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

#[derive(Error, Debug)]
pub enum LemmatizerError {
    #[error("Could not open file")]
    FileIOFailed(std::io::Error),
    #[error("JSON parsing failed")]
    JSONParseFailed(serde_json::Error),
}

pub struct Lemmatizer {
    tokens: Vec<Vec<Token>>,
    lemma_map: LemmaMap,
}

impl Lemmatizer {
    pub fn new() -> Lemmatizer {
        Lemmatizer {
            tokens: Vec::new(),
            lemma_map: HashMap::new(),
        }
    }

    pub fn load_file<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<&mut Lemmatizer, LemmatizerError> {
        self.tokens.append(&mut tokens_from_file(path)?); // TODO Append instead of overwrite
        self.lemma_map = map_from_array(&self.tokens);
        Ok(self)
    }

    pub fn get_lemmas(&self) -> Vec<String> {
        let mut lemma_vec: LemmaVec = self.lemma_map.iter().collect();
        lemma_vec.sort_by(|a, b| b.1.word_count().cmp(&a.1.word_count()));
        lemma_vec
            .into_iter()
            .map(|(lemma, _)| lemma.to_string())
            .collect()
    }

    pub fn get_forms(&self, lemma: &str) -> Vec<String> {
        match self.lemma_map.get(lemma) {
            Some(map) => {
                let mut form_vec: FormVec = map.into_iter().collect();
                form_vec.sort_by(|a, b| b.1.len().cmp(&a.1.len()));
                form_vec
                    .into_iter()
                    .map(|(form, _)| form.to_string())
                    .collect()
            }
            None => Vec::new(),
        }
    }

    pub fn get_usages(&self, lemma: &str, form: &str) -> Vec<(String, String, String)> {
        match self.lemma_map.get(lemma) {
            Some(map) => match map.get(form) {
                Some(forms) => forms
                    .into_iter()
                    .map(|(sentence_i, token_i)| {
                        get_sentence_split(&self.tokens, *sentence_i, *token_i)
                    })
                    .collect(),
                None => Vec::new(),
            },
            None => Vec::new(),
        }
    }
}

pub fn tokens_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<Vec<Token>>, LemmatizerError> {
    let file = File::open(path).map_err(|e| LemmatizerError::FileIOFailed(e))?;
    let reader = BufReader::new(file);
    let values: JsonValue =
        serde_json::from_reader(reader).map_err(|e| LemmatizerError::JSONParseFailed(e))?;
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

pub fn map_from_array(token_array: &Vec<Vec<Token>>) -> LemmaMap {
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

pub fn get_sentence_split(
    token_array: &Vec<Vec<Token>>,
    sentence_i: usize,
    token_i: usize,
) -> (String, String, String) {
    let mut before: String = String::from("");
    let word: String = String::from(&token_array[sentence_i][token_i].text);
    let mut after: String = String::from("");

    for (i, token) in token_array[sentence_i].iter().enumerate() {
        if i != 0 && token.pos != "PUNCT" {
            if i < token_i {
                before.push(' ');
            } else if i > token_i {
                after.push(' ');
            }
        }
        if i < token_i {
            before.push_str(&token.text);
        } else if i > token_i {
            after.push_str(&token.text);
        }
    }
    before.push(' ');
    (before, word, after)
}

pub fn get_sentence_bolded(
    token_array: &Vec<Vec<Token>>,
    sentence_i: usize,
    token_i: usize,
) -> String {
    let mut sentence: String = String::from("");

    for (i, token) in token_array[sentence_i].iter().enumerate() {
        if i != 0 && token.pos != "PUNCT" {
            sentence.push(' ');
        }
        if i == token_i {
            sentence.push(0o033 as char);
            sentence.push_str("[1;4m");
        }
        sentence.push_str(&token.text);
        if i == token_i {
            sentence.push(0o033 as char);
            sentence.push_str("[22;24m");
        }
    }

    sentence
}
