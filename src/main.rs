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

#[derive(Debug, Serialize, Deserialize)]
pub struct VocabWord {
    strength_bars: Option<i32>,
    infinitive: Option<String>,
    normalized_string: Option<String>,
    pos: Option<String>,
    last_pacticed_ms: Option<i64>,
    skill: Option<String>,
    related_lexemes: Option<Vec<String>>,
    last_practiced: Option<String>,
    strength: Option<f32>,
    skill_url_title: Option<String>,
    gender: Option<String>,
    id: Option<String>,
    lexeme_id: Option<String>,
    word_string: Option<String>,
    translation: Option<Vec<String>>,
}

static URL: &str = "https://www.duolingo.com";

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: ./duolingo <username> <password>");
        Err("Womp")
    } else {
        let token;
        match login(&args[1], &args[2]).await {
            Some(t) => token = t,
            None => panic!("Could not log in"),
        }

        let mut vocab;
        match get_vocab(&token).await {
            Ok(v) => vocab = v,
            Err(e) => panic!("Could not fetch vocab: {}", e),
        }

        let vocab = add_translations(&token, &mut vocab).await;

        println!("VOCAB:\n{:?}", vocab);

        Ok(())
    }
}

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

    for i in 0..vocab_words.len() {
        word_list = format!(
            "{} \"{}\",",
            &word_list,
            vocab_words[i].word_string.as_ref().unwrap()
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

    for i in 0..vocab_words.len() {
        let key = vocab_words[i].word_string.as_ref().unwrap();

        vocab_words[i].translation = response.get(key).unwrap().to_owned();
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
