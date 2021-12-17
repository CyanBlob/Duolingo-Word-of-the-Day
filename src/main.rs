use std::{env, error::Error};

use rand::Rng;

pub mod api;

use embedded_graphics::{
    mono_font::MonoTextStyleBuilder,
    prelude::Point,
    text::{Baseline, Text, TextStyleBuilder},
    Drawable,
};
use epd_waveshare::{
    color::*,
    epd2in9_v2::{Display2in9, Epd2in9},
    graphics::DisplayRotation,
    prelude::*,
};
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{Bus, Mode, SlaveSelect, Spi},
};

#[derive(Debug)]
enum ErrorCode {
    ApiError,
}

impl std::error::Error for ErrorCode {}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::ApiError => write!(f, "Api Error"),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        Err("Usage: ./duolingo <username> <password>")
    } else {
        let mut interval_timer =
            tokio::time::interval(chrono::Duration::hours(1).to_std().unwrap());

        let username = args[1].to_owned();
        let password = args[2].to_owned();

        loop {
            interval_timer.tick().await;

            new_word(&username, &password).await;
        }
    }
}

async fn new_word(username: &str, password: &str) {
    println!("New word! {:?}", std::time::SystemTime::now());
    let token;
    match api::login(&username, &password).await {
        Some(t) => token = t,
        None => panic!("Could not log in"),
    }

    let selected_words = pick_words(&token, 4).await;

    match selected_words {
        Ok(v) => display_words(v).await,
        Err(_) => println!("Can't display!"),
    }
}

async fn pick_words(token: &str, count: i32) -> Result<Vec<api::VocabWord>, Box<dyn Error>> {
    let mut vocab;
    match api::get_vocab(&token).await {
        Ok(v) => vocab = v,
        Err(_) => return Err(Box::new(ErrorCode::ApiError)),
    }

    let mut selected_words = Vec::<api::VocabWord>::new();

    let vocab = api::add_translations(&token, &mut vocab).await;

    vocab.sort_by(|a, b| match a.strength.partial_cmp(&b.strength) {
        Some(v) => return v,
        None => return std::cmp::Ordering::Equal,
    });

    let mut rng = rand::thread_rng();

    for _i in 0..count {
        let index = rng.gen_range(0..vocab.len() / 3); // take from weakest 1/3 of words
        selected_words.push(vocab[index as usize].clone());
    }

    Ok(selected_words)
}

async fn display_words(words: Vec<api::VocabWord>) {
    let mut spi;
    match Spi::new(Bus::Spi0, SlaveSelect::Ss0, 10_000_000, Mode::Mode0) {
        Ok(s) => spi = s,
        Err(e) => panic!("Could not get access to SPI: {:?}", e),
    }

    let pins = Gpio::new().unwrap();

    let cs =   pins.get(8).unwrap().into_output();  // pin 24
    let busy = pins.get(25).unwrap().into_input();  // pin 22
    let dc =   pins.get(24).unwrap().into_output(); // pin 18
    let rst =  pins.get(23).unwrap().into_output(); // pin 16

    let mut delay = Delay::new();

    // Setup the epd
    let mut epd = Epd2in9::new(&mut spi, cs, busy, dc, rst, &mut delay).unwrap();

    // Setup the graphics
    let mut display = Display2in9::default();

    display.set_rotation(DisplayRotation::Rotate90);

    for i in 0..words.len() {
        draw_text(
            &mut display,
            &words[i].word_string.as_ref().unwrap(),
            0,
            0 + 31 * i as i32,
            &embedded_graphics::mono_font::iso_8859_1::FONT_9X18_BOLD,
        );
        draw_text(
            &mut display,
            &words[i].translation.as_ref().unwrap().join(", "),
            0,
            19 + 31 * i as i32,
            &embedded_graphics::mono_font::iso_8859_1::FONT_8X13,
        );
    }

    if let Err(e) = epd.update_and_display_frame(&mut spi, &display.buffer(), &mut delay) {
        println!("Failed to refresh display: {:?}", e);
    }
}

fn draw_text(
    display: &mut Display2in9,
    text: &str,
    x: i32,
    y: i32,
    font: &embedded_graphics::mono_font::MonoFont,
) {
    let style = MonoTextStyleBuilder::new()
        .font(&font)
        .text_color(Black)
        .background_color(White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}
