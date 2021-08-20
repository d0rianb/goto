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
    if !input.ends_with('/') { return false; }
    return path::Path::new(input).exists();
}

fn get_last_input(input: &str) -> &str {
    lazy_static! {
        static ref INPUT_REGEX: Regex = Regex::new(r#"/([\w.\-\\\\ ]*)$"#).unwrap();
    }
    let groups =  INPUT_REGEX.captures(input);
    return match groups {
        None => "",
        Some(groups) => { groups.get(1).map_or("", |m| m.as_str()) }
    }
}

fn get_path_from_input(input: &str) -> &str {
    lazy_static! {
        static ref PATH_REGEX: Regex = Regex::new(r"^.*/").unwrap();
    }
    let groups = PATH_REGEX.captures(input);
    return match groups {
        None => "",
        Some(groups) => { groups.get(0).map_or("", |m| m.as_str()) }
    }
}

fn get_sorted_subfolder(input: &str) -> Vec<String> {
    return fs::read_dir(input)
        .expect("Unable to access the sub directories")
        .filter_map(Result::ok)
        .filter(|path| path.path().is_dir())
        .map(|folder| {
            folder
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
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
}

fn write_header(header: &str, stdout: &mut termion::raw::RawTerminal<std::io::Stdout>) {
    write!(
        stdout,
        "{}{}{}",
        termion::clear::CurrentLine,
        termion::cursor::Goto(1, 1),
        header
    ).unwrap();
    stdout.lock().flush().unwrap();
}

fn get_guess(path: &str, last_input: &str, sorted_subfolders: &Vec<String>, mut offset: i8) -> String {
    if !is_valid_path(&path)|| sorted_subfolders.len() == 0  { return String::new(); }

    let nb_subfolders = sorted_subfolders.len();

    if last_input == "" {
        if offset < 0 { offset += nb_subfolders as i8 }
        return sorted_subfolders[offset as usize % nb_subfolders].to_string();
    }

    let mut guess = String::new();
    for i in 0..nb_subfolders {
        let name = &sorted_subfolders[i] as &str;
        if (&name.to_lowercase() as &str).cmp(last_input).is_ge() && name.to_lowercase().starts_with(last_input) {
            let input_len = last_input.len();
            if input_len <= name.len() {
                guess = name[input_len..].to_string();
                break
            }
        }
    }
    return guess;
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
    let mut sorted_subfolders = Vec::<String>::new();
    let mut previous_path = String::new();
    let mut guess = String::new();
    let light_white_color = color::AnsiValue::rgb(1, 1, 1);

    const HEADER: &str = "GOTO : ";

    write_header(HEADER, &mut stdout);

    for c in stdin.keys() {
        let key = c.unwrap();
        match key {
            Key::Ctrl('c') => break,
            Key::Backspace => { text.pop(); }
            Key::Char('\t') | Key::Right => {
                if guess != "" {
                    text.push_str(&(guess.to_owned() + "/"));
                    offset = 0;
                }
            }
            Key::Down => { offset += 1 }
            Key::Up => { offset -= 1 }
            Key::Char('\n') => { travel_to(&text) }
            _ => { if let Key::Char(k) = key { text.push(k) } }
        }


        let path = get_path_from_input(&text);
        let last_input = get_last_input(&text);

        if previous_path != path && is_valid_path(&path) {
            sorted_subfolders = get_sorted_subfolder(&path);
            previous_path = String::from(path);
        }

        guess = get_guess(&path, last_input, &sorted_subfolders, offset);

        write_header(HEADER, &mut stdout);
        write!(stdout, "{}", text).unwrap();
        write!(
            stdout,
            "{color}{guess}{reset}{cursor}",
            color = color::Fg(light_white_color),
            guess = guess,
            reset = color::Fg(color::Reset),
            cursor = termion::cursor::Goto((text.len() + HEADER.len() + 1) as u16, 1)

        ).unwrap();
        stdout.lock().flush().unwrap();
    }
}

fn main() {
    run_in_terminal();
}