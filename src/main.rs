use std::{env, error::Error};

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
        println!("Usage: ./duolingo <username> <password>");
        Err("Womp")
    } else {
        let token;
        match api::login(&args[1], &args[2]).await {
            Some(t) => token = t,
            None => panic!("Could not log in"),
        }

        let selected_words = pick_words(&token, 1).await;

        match selected_words {
            Ok(v) => display_words(v).await,
            Err(_) => println!("Can't display!"),
        }

        Ok(())
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

    vocab.sort_by(|a, b| a.strength.partial_cmp(&b.strength).unwrap());

    for i in 0..count {
        selected_words.push(vocab[i as usize].clone());
    }

    Ok(selected_words)
}

async fn display_words(words: Vec<api::VocabWord>) {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 16_000_000, Mode::Mode0).unwrap();
    let pins = Gpio::new().unwrap();

    let cs = pins.get(8).unwrap().into_output(); // pin 24
    let busy = pins.get(25).unwrap().into_input(); // pin 22
    let dc = pins.get(24).unwrap().into_output(); // pin 18
    let rst = pins.get(23).unwrap().into_output(); // pin 16

    let mut delay = Delay::new();

    // Setup the epd
    let mut epd = Epd2in9::new(&mut spi, cs, busy, dc, rst, &mut delay);

    // Setup the graphics
    let mut display = Display2in9::default();

    // Draw some text
    /*display.draw(
    let _ = Text::new("Hello Rust!", Point::new(x, y))
    .draw(display);
    );

    // Transfer the frame data to the epd and display it
    epd.update_and_display_frame( & mut spi, & display.buffer()) ?;*/

    println!("Selected words: {:?}", words);
}

/*struct U8Delay {
    delay: delay::Ets
}

impl embedded_hal::blocking::delay::DelayMs<u8> for U8Delay {
    fn delay_ms(&mut self, ms: u8) {
        unsafe {
            //delay((ms as u32 * 1000));
            let mut delay = delay::Ets;
            delay.delay_ms(ms as u32 * 1);
        }
    }
}*/

fn draw_text(display: &mut Display2in9, text: &str, x: i32, y: i32) {
    let style = MonoTextStyleBuilder::new()
        .font(&embedded_graphics::mono_font::ascii::FONT_6X10)
        .text_color(Black)
        .background_color(White)
        .build();

    let text_style = TextStyleBuilder::new().baseline(Baseline::Top).build();

    let _ = Text::with_text_style(text, Point::new(x, y), style, text_style).draw(display);
}
