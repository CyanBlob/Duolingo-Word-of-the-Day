use std::env;

use reqwest::header::*;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct VocabResponse {
    language_string: String,
    learning_language: Option<String>,
    from_language: Option<String>,
    language_information: Option<LanguageInformation>,
    vocab_overview: Option<Vec<VocabWord>>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LanguageInformation {
}

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
    translation: Option<String>
}

static URL: &str = "https://www.duolingo.com";

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: ./duolingo <username> <password>");
        Err("Womp")
    } else {
        let token = login(&args[1], &args[2]).await;
        get_vocab(&token.unwrap()).await;

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

    let resp_text = builder.send().await.unwrap().headers().clone();

    //let response: Result<ApiResponse, serde_json::Error> = serde_json::from_str(&resp_text);

    let token = resp_text.get("jwt").unwrap().to_str().unwrap();

    Some(token.into())

    /*match response {
        Ok(r) => Ok(r.result),
        Err(e) => Err(e.into()),
    }*/
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
    
    for word in response.unwrap().vocab_overview.unwrap() {
        println!("{}", &word.word_string.unwrap())
    }
    
    let vocab: Vec<VocabWord> = Vec::<VocabWord>::new();

    Ok(vocab)
}

pub async fn add_translations(token: &str) {
}

fn get_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.append(
        AUTHORIZATION,
        HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap(),
    );

    headers
}
