use std::env;

use reqwest::header::*;

static URL: &str = "https://www.duolingo.com";
static USER: &str = "andrewjamest1993@gmail.com";
static PASS: &str = "Zelada7417";

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage: ./duolingo <username> <password>");
        Err("Womp")
    } else {
        let token = login(&args[1], &args[2]).await;
        simple_query(&token.unwrap()).await;

        Ok(())
    }
}

pub async fn login(username: &str, password: &str) -> Option<String> {
    let client = reqwest::Client::new();
    let mut builder: reqwest::RequestBuilder;

    builder = client.get(format!(
        "{}/{}?login={}&password={}",
        URL, "login", USER, PASS
    ));

    let resp_text = builder.send().await.unwrap().headers().clone();

    //let response: Result<ApiResponse, serde_json::Error> = serde_json::from_str(&resp_text);

    let token = resp_text.get("jwt").unwrap().to_str().unwrap();

    println!("Got token!!\n{:?}", &token);

    Some(token.into())

    /*match response {
        Ok(r) => Ok(r.result),
        Err(e) => Err(e.into()),
    }*/
}

pub async fn simple_query(token: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut builder: reqwest::RequestBuilder;

    builder = client.get(format!("{}/{}", URL, "vocabulary/overview?_=1638505469098"));

    let resp_text = builder
        .headers(get_headers(&token))
        .send()
        .await?
        .text()
        .await?;

    //let response: Result<ApiResponse, serde_json::Error> = serde_json::from_str(&resp_text);

    println!("Got string!\n{:?}", &resp_text);

    /*match response {
        Ok(r) => Ok(r.result),
        Err(e) => Err(e.into()),
    }*/
    Ok(resp_text.clone())
}

fn get_headers(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();

    headers.append(
        AUTHORIZATION,
        HeaderValue::from_str(&(String::from("Bearer ") + token)).unwrap(),
    );

    headers
}
