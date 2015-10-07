use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::char;

use globals::Globals;

enum State {
    Raw,
    Replacement,
    ReplacementEnd,
}

fn get_replacement<'g>(sequence: &[u8], globals: &'g Globals) -> &'g [u8] {
    match sequence {
        b"css" => globals.get_css_links(),
        b"js" => globals.get_js_links(),
        other => {
            match globals.get(other) {
                Some(v) => v.as_bytes(),
                None => panic!("unexpected key {:?}", String::from_utf8_lossy(other)),
            }
        },
    }
}

pub fn parse(file: &str, globals: &Globals) -> Vec<u8> {
    let path = Path::new(file);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        Ok(file) => file,
    };

    let mut result: Vec<u8> = Vec::new();
    let mut buf: [u8; 256] = [0; 256];
    let mut state = State::Raw;
    let mut prev_ch: Option<u8> = None;
    let mut replacement: Vec<u8> = Vec::new();

    loop {
        match file.read(&mut buf) {
            Ok(0) => if let State::Replacement = state {
                panic!("unclosed replacement token!");
            } else {
                break;
            },
            Ok(len) => {
                for ch in buf.iter().take(len) {
                    match state {
                        State::Raw => match (prev_ch, *ch) {
                            (Some(b'{'), b'{') => {
                                state = State::Replacement;
                                replacement.clear();
                            },
                            (Some(prev), ch) => {
                                result.push(prev);
                                prev_ch = Some(ch);
                            },
                            (_, ch) => {
                                prev_ch = Some(ch);
                            },
                        },
                        State::Replacement => match (prev_ch, *ch) {
                            (_, b'}') => {
                                state = State::ReplacementEnd;
                                prev_ch = Some(*ch);
                            },
                            (_, ch) => {
                                match char::from_u32(ch as u32) {
                                    Some(c) if !c.is_whitespace() => replacement.push(ch),
                                    _ => {},
                                }
                                prev_ch = Some(ch);
                            },
                        },
                        State::ReplacementEnd => match (prev_ch, *ch) {
                            (Some(b'}'), b'}') => {
                                result.extend(get_replacement(&replacement, &globals));
                                state = State::Raw;
                                prev_ch = None;
                            },
                            _ => {
                                panic!("expected }}, found something else");
                            },
                        }
                    }
                }
            },
            Err(why) => panic!("couldn't open {}: {}", display, Error::description(&why)),
        }
    }

    if let Some(prev) = prev_ch {
        result.push(prev);
    }

    result
}
