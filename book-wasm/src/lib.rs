mod utils;

use image::{DynamicImage, GenericImage, GenericImageView};
use serde::Deserialize;
use std::{collections::HashMap, io::Cursor, panic};
use wasm_bindgen::prelude::*;

use js_sys::Uint8Array;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Debug, Deserialize)]
struct MappedChar {
    image: Option<String>,
    // x y w h
    pos: (u32, u32, u32, u32),
    ascent: u32,
}

#[derive(Debug, Deserialize)]
struct FontData {
    images: HashMap<String, String>,
    chars: HashMap<char, MappedChar>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Color {
    Black,
    DarkBlue,
    DarkGreen,
    DarkAqua,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Hex(u8, u8, u8),
}

impl Color {
    fn to_rgb(self) -> (u8, u8, u8) {
        match self {
            Color::Black => (0, 0, 0),
            Color::DarkBlue => (0, 0, 170),
            Color::DarkGreen => (0, 170, 0),
            Color::DarkAqua => (0, 170, 170),
            Color::DarkRed => (170, 0, 0),
            Color::DarkPurple => (170, 0, 170),
            Color::Gold => (255, 170, 0),
            Color::Gray => (170, 170, 170),
            Color::DarkGray => (85, 85, 85),
            Color::Blue => (85, 85, 255),
            Color::Green => (85, 255, 85),
            Color::Aqua => (85, 255, 255),
            Color::Red => (255, 85, 85),
            Color::LightPurple => (255, 85, 255),
            Color::Yellow => (255, 255, 85),
            Color::White => (255, 255, 255),
            Color::Hex(r, g, b) => (r, g, b),
        }
    }

    fn from_color_code(code: char) -> Option<Self> {
        match code {
            '0' => Some(Color::Black),
            '1' => Some(Color::DarkBlue),
            '2' => Some(Color::DarkGreen),
            '3' => Some(Color::DarkAqua),
            '4' => Some(Color::DarkRed),
            '5' => Some(Color::DarkPurple),
            '6' => Some(Color::Gold),
            '7' => Some(Color::Gray),
            '8' => Some(Color::DarkGray),
            '9' => Some(Color::Blue),
            'a' => Some(Color::Green),
            'b' => Some(Color::Aqua),
            'c' => Some(Color::Red),
            'd' => Some(Color::LightPurple),
            'e' => Some(Color::Yellow),
            'f' => Some(Color::White),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Format {
    Obfuscated,
    Bold,
    Strikethrough,
    Underlined,
    Italic,
}

impl Format {
    fn from_format_code(code: char) -> Option<Self> {
        match code {
            'k' => Some(Format::Obfuscated),
            'l' => Some(Format::Bold),
            'm' => Some(Format::Strikethrough),
            'n' => Some(Format::Underlined),
            'o' => Some(Format::Italic),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct PlacingImage {
    image: String,
    pos: (u32, u32),
    character: (u32, u32, u32, u32),
    color: Color,
    formats: Vec<Format>,
}

const X_MIN: u32 = 16;
const Y_MIN: u32 = 36;
const Y_STEP: u32 = 9;
const MAX_WIDTH: u32 = 129;
const MAX_HEIGHT: u32 = Y_MIN + (Y_STEP * 14) - 1;

#[wasm_bindgen]
pub fn generate_image(font: String, text: String) -> Result<Uint8Array, JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let font_data: FontData = match serde_json::from_str(&font) {
        Ok(data) => data,
        Err(e) => {
            return Err(JsValue::from_str(&format!(
                "Failed to parse font data: {}",
                e
            )))
        }
    };

    let text = text.replace('\n', " \n ");
    let words = text.split(' ');

    let mut x: u32 = X_MIN;
    let mut y: u32 = Y_MIN;

    let mut placing_images = Vec::<PlacingImage>::new();

    let mut start_of_line = true;
    let mut color = Color::Black;
    let mut formats = Vec::<Format>::new();
    let mut next_is_modifier;

    for word in words {
        let mut word = format!("{} ", word);

        let mut first_try = true;
        next_is_modifier = false;

        'word: loop {
            if y > MAX_HEIGHT {
                break;
            }
            let mut word_placing = Vec::<PlacingImage>::new();
            let mut word_width = 0;
            let space_left: i32 = (MAX_WIDTH - x) as i32;

            let mut splitting_word_index: Option<usize> = None;

            for (i, c) in word.chars().enumerate() {
                if c == '§' {
                    next_is_modifier = true;
                    continue;
                }
                if next_is_modifier {
                    if c == 'r' {
                        color = Color::Black;
                        formats.clear();
                    }
                    if let Some(new_color) = Color::from_color_code(c) {
                        color = new_color;
                        formats.clear();
                    }
                    if let Some(new_format) = Format::from_format_code(c) {
                        formats.push(new_format);
                    }

                    next_is_modifier = false;
                    continue;
                }
                if c == '\n' {
                    x = X_MIN;
                    y += Y_STEP;
                    start_of_line = true;
                    break;
                }
                let char_data = font_data.chars.get(&c).unwrap_or_else(|| {
                    font_data.chars.get(&char::from_u32(0x00).unwrap()).unwrap()
                });

                if char_data.image.is_none() {
                    word_width += char_data.pos.2
                        + if formats.contains(&Format::Bold) {
                            1
                        } else {
                            0
                        };
                    continue;
                }

                if ((word_width + char_data.pos.2) as i32) > space_left {
                    // word is too long, break it
                    if first_try {
                        first_try = false;
                        x = X_MIN;
                        if !start_of_line {
                            y += Y_STEP;
                        }
                        continue 'word;
                    } else {
                        splitting_word_index = Some(i);
                        break;
                    }
                }

                word_placing.push(PlacingImage {
                    image: char_data.image.as_ref().unwrap().clone(),
                    character: char_data.pos,
                    pos: (x + word_width, y - char_data.ascent),
                    color,
                    formats: formats.clone(),
                });
                word_width += char_data.pos.2
                    + 1
                    + if formats.contains(&Format::Bold) {
                        1
                    } else {
                        0
                    };
                start_of_line = false;
            }

            x += word_width;
            placing_images.append(&mut word_placing);
            if let Some(swi) = splitting_word_index {
                word = word.chars().skip(swi).collect();
                x = X_MIN;
                y += Y_STEP;
                continue;
            } else {
                break;
            }
        }
    }

    let mut book = image::load_from_memory(
        &base64::decode(font_data.images.get("gui/book.png").unwrap()).unwrap(),
    )
    .unwrap();

    let mut images = HashMap::<String, DynamicImage>::new();

    for placing_image in placing_images.iter_mut() {
        let image = images
            .entry(placing_image.image.clone())
            .or_insert_with(|| {
                let image_data =
                    base64::decode(font_data.images.get(&placing_image.image).unwrap());
                image::load_from_memory(&image_data.unwrap()).unwrap()
            });

        let mut char_image = image.crop(
            placing_image.character.0,
            placing_image.character.1,
            placing_image.character.2,
            placing_image.character.3,
        );

        if placing_image.color == Color::Black {
            char_image.invert();
        } else if placing_image.color != Color::White {
            let (r, g, b) = placing_image.color.to_rgb();
            for x in 0..char_image.width() {
                for y in 0..char_image.height() {
                    let pixel = char_image.get_pixel(x, y);
                    if pixel[3] != 0 {
                        char_image.put_pixel(x, y, image::Rgba([r, g, b, pixel[3]]));
                    }
                }
            }
        }

        if placing_image.formats.contains(&Format::Bold) {
            let mut bold_image =
                image::DynamicImage::new_rgba8(char_image.width() + 1, char_image.height());
            bold_image.copy_from(&char_image, 0, 0).unwrap();
            image::imageops::overlay(&mut bold_image, &char_image, 1, 0);
            char_image = bold_image;
        }

        if placing_image.formats.contains(&Format::Italic) {
            let mut italic_image =
                image::DynamicImage::new_rgba8(char_image.width() + 2, char_image.height());
            italic_image
                .copy_from(
                    &char_image.sub_image(0, 0, char_image.width(), 2).to_image(),
                    2,
                    0,
                )
                .unwrap();
            italic_image
                .copy_from(
                    &char_image
                        .sub_image(0, 2, char_image.width(), char_image.height() - 4)
                        .to_image(),
                    1,
                    2,
                )
                .unwrap();

            italic_image
                .copy_from(
                    &char_image
                        .sub_image(0, char_image.height() - 2, char_image.width(), 1)
                        .to_image(),
                    0,
                    char_image.height() - 2,
                )
                .unwrap();

            placing_image.pos.0 -= 1;

            char_image = italic_image;
        }

        // TODO: add support for formatting

        image::imageops::overlay(
            &mut book,
            &char_image,
            placing_image.pos.0 as i64,
            placing_image.pos.1 as i64,
        );
    }

    let mut buffer = Vec::new();
    let mut cursor = Cursor::new(&mut buffer);
    book.write_to(&mut cursor, image::ImageOutputFormat::Png)
        .unwrap();

    Ok(Uint8Array::from(&buffer[..]))
}
