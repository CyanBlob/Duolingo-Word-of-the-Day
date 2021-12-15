use std::{collections::HashMap, env};

use reqwest::header::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VocabResponse {
    language_string: String,
    learning_language: Option<String>,
    from_language: Option<String>,
    language_information: Option<LanguageInformation>,
    vocab_overview: Option<Vec<VocabWord>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageInformation {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VocabWord {
    pub strength_bars: Option<i32>,
    pub infinitive: Option<String>,
    pub normalized_string: Option<String>,
    pub pos: Option<String>,
    pub last_pacticed_ms: Option<i64>,
    pub skill: Option<String>,
    pub related_lexemes: Option<Vec<String>>,
    pub last_practiced: Option<String>,
    pub strength: Option<f32>,
    pub skill_url_title: Option<String>,
    pub gender: Option<String>,
    pub id: Option<String>,
    pub lexeme_id: Option<String>,
    pub word_string: Option<String>,
    pub translation: Option<Vec<String>>,
}

static URL: &str = "https://www.duolingo.com";

pub async fn login(username: &str, password: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let builder: reqwest::RequestBuilder;

    builder = client.get(format!(
        "{}/{}?login={}&password={}",
        URL, "login", username, password
    ));

    let resp_text;
    match builder.send().await {
        Ok(x) => resp_text = x.headers().clone(),
        Err(_) => return None,
    }

    let token;
    match resp_text.get("jwt") {
        Some(t) => match t.to_str() {
            Ok(s) => token = s,
            Err(_) => return None,
        },
        None => return None,
    }

    Some(token.into())
}

pub async fn get_vocab(token: &str) -> Result<Vec<VocabWord>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let builder: reqwest::RequestBuilder;

    builder = client.get(format!("{}/{}", URL, "vocabulary/overview?_=1638505469098"));

    let resp_text = builder
        .headers(get_headers(&token))
        .send()
        .await?
        .text()
        .await?;

    let response: Result<VocabResponse, serde_json::Error> = serde_json::from_str(&resp_text);

    return match response {
        Ok(r) => match r.vocab_overview {
            Some(v) => Ok(v),
            None => Err("Could not decode vocab response".into()),
        },
        Err(_) => Err("Could not decode vocab response".into()),
    };
}

pub async fn add_translations<'a>(
    token: &str,
    vocab_words: &'a mut Vec<VocabWord>,
) -> &'a mut Vec<VocabWord> {
    let client = reqwest::Client::new();
    let builder: reqwest::RequestBuilder;

    let mut word_list: String = String::from("[");

    for vocab_word in vocab_words.iter_mut() {
        word_list = format!(
            "{} \"{}\",",
            &word_list,
            vocab_word.word_string.as_ref().unwrap()
        )
        .to_owned();
    }

    word_list = word_list[0..word_list.len() - 1].to_string();

    word_list = format!("{}]", &word_list);

    let query = format!(
        "{}/{}/{}?tokens={}",
        "https://d2.duolingo.com/api/1/dictionary/hints", "es", "en", word_list
    );

    builder = client.get(query);

    let resp_text = builder
        .headers(get_headers(&token))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let response: HashMap<String, Option<Vec<String>>> = serde_json::from_str(&resp_text).unwrap();

    for vocab_word in vocab_words.iter_mut() {
        let key = vocab_word.word_string.as_ref().unwrap();

        vocab_word.translation = response.get(key).unwrap().to_owned();
    }

    vocab_words
}

fn get_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.append(
        AUTHORIZATION,
        HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap(),
    );

    headers
}
