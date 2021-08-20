extern crate lazy_static;
extern crate termion;

use lazy_static::lazy_static;
use regex::Regex;
use std::fs;
use std::io::*;
use std::io::{stdin, stdout, Write};
use std::path;

use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use itertools::Itertools;
use std::cmp::{Ordering};
use std::process::Command;

fn is_valid_path(input: &str) -> bool {
    return path::Path::new(input).exists();
}

fn get_last_input(input: &str) -> &str {
    lazy_static! {
        static ref INPUT_REGEX: Regex = match Regex::new(r#"/([\w.\-\\\\ ]*)$"#) {
            Ok(re) => re,
            Err(err) => panic!("{}", err)
        };
    }
    let groups =  INPUT_REGEX.captures(input);
    return match groups {
        None => "",
        Some(groups) => { groups.get(1).map_or("", |m| m.as_str()) }
    }
}

fn get_path_from_input(input: &str) -> &str {
    lazy_static! {
        static ref INPUT_REGEX: Regex = match Regex::new(r"^.*/") {
            Ok(re) => re,
            Err(err) => panic!("{}", err)
        };
    }
    let groups = INPUT_REGEX.captures(input);
    return match groups {
        None => "",
        Some(groups) => { groups.get(0).map_or("", |m| m.as_str()) }
    }
}

fn get_subfolder(input: &str) -> Vec<fs::DirEntry> {
    return fs::read_dir(input)
        .unwrap()
        .filter_map(Result::ok)
        .filter(|path| path.path().is_dir())
        .collect();
}

fn write_header(stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    const HEADER: &str = "GOTO : ";
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        HEADER
    ).unwrap();
    stdout.lock().flush().unwrap();
}

fn get_guess(text: &String, mut offset: i8) -> String {
    let path = get_path_from_input(&text);
    if !is_valid_path(&path) { return String::new(); }
    let subfolders = get_subfolder(path);
    if subfolders.len() == 0 { return String::new(); }
    let last_input = &get_last_input(&text).to_lowercase();

    let sorted_subfolders = subfolders
        .iter()
        .map(|folder| {
            folder
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
                // .to_lowercase()
        })
        .sorted_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()))
        .sorted_by(|a, b| {
            if b.starts_with('.') {
                if a.starts_with('.') { return a.to_lowercase().cmp(&b.to_lowercase()); };
                return Ordering::Less;
            }
            return Ordering::Greater;
        })
        .collect::<Vec<String>>();

    let nb_subfolders = sorted_subfolders.len();

    if last_input == "" {
        if offset < 0 { offset += nb_subfolders as i8 }
        return String::from(&sorted_subfolders[offset as usize % nb_subfolders]);
    }

    let mut guess = String::new();
    for i in 0..nb_subfolders {
        let name = &sorted_subfolders[i] as &str;
        if name.to_lowercase().cmp(last_input).is_ge() && name.to_lowercase().starts_with(last_input) {
            let input_len = last_input.len();
            if input_len <= name.len() {
                guess = name[input_len..].to_string();
                break
            }
        }
    }
    return guess;
}

fn fill_guess(text: &mut String, offset: i8) {
    let ref guess = get_guess(text, offset);
    if guess == "" { return; };
    text.push_str(&(guess.to_owned() + "/"));
}

fn travel_to(path: &String) {
    if !is_valid_path(path) { return }
    Command::new("open")
        .args(&[path])
        .output()
        .expect("Failed to open a Finder window");

}

fn run_in_terminal() {
    let stdin = stdin();
    assert!(termion::is_tty(&stdin), "The terminal is not TTY compatible");
    let mut stdout = stdout().into_raw_mode().unwrap();
    write!(stdout, "{}", termion::clear::All).unwrap();
    let mut text = String::new();
    let mut offset: i8 = 0;
    write_header(&mut stdout);
    for c in stdin.keys() {
        write_header(&mut stdout);
        let key = c.unwrap();
        match key {
            Key::Ctrl('c') => break,
            Key::Backspace => { text.pop(); }
            Key::Char('\t') | Key::Right => { fill_guess(&mut text, offset); offset = 0; }
            Key::Down => { offset += 1 }
            Key::Up => { offset -= 1 }
            Key::Char('\n') => { travel_to(&text) }
            _ => { if let Key::Char(k) = key { text.push(k) } }
        }
        write!(stdout, "{}", text).unwrap();
        let guess = get_guess(&text, offset);
        write!(
            stdout,
            "{color}{guess}{reset}",
            color = color::Fg(color::White),
            guess = guess,
            reset = color::Fg(color::Reset)
        ).unwrap();
        stdout.lock().flush().unwrap();
    }
}

fn main() {
    run_in_terminal();
}