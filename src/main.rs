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
use std::fs::DirEntry;

fn is_valid_path(input: &str) -> bool {
    return path::Path::new(input).exists();
}

fn get_path_from_input(input: &str) -> String {
    lazy_static! {
        static ref PATH_REGEX: Regex = Regex::new(r"/\w*$").unwrap();
    }
    return PATH_REGEX.replace(input, "").into();
}

fn get_subfolder(input: &String) -> Vec<fs::DirEntry> {
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
    )
        .unwrap();
    stdout.lock().flush().unwrap();
}

// fn get_guess(input: &String, subfolders: Vec<DirEntry>) -> String {
fn get_guess(text: &String) -> String {
    let path = get_path_from_input(&text);
    if !is_valid_path(&path) { return String::new(); }
    let subfolders = get_subfolder(&path);
    if subfolders.len() == 0 { return String::new(); }
    let last_input = text
        .replace(&path, "")
        .replace("/", "");

    let sorted_subfolders = subfolders
        .iter()
        .map(|folder| {
            folder
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
                .to_lowercase()
        })
        .sorted()
        .collect::<Vec<String>>();
    let mut guess = String::new();
    // write!(stdout, "\n{:?}", sorted_subfolders).unwrap();
    for name in sorted_subfolders {
        if name.cmp(&last_input).is_ge() {
            let input_len = last_input.len();
            guess = name[input_len.to_owned()..].to_string();
            break;
        }
    }
    return guess;
}

fn main() {
    let stdin = stdin();
    assert!(
        termion::is_tty(&stdin),
        "The terminal is not TTY compatible"
    );
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut text = String::new();
    write_header(&mut stdout);
    for c in stdin.keys() {
        write_header(&mut stdout);
        let key = c.unwrap();
        match key {
            Key::Ctrl('c') => break,
            Key::Backspace => { text.pop(); }
            Key::Char('\t') => { text.push_str(&get_guess(&text)) }
            Key::Char(' ') => { text.push_str("\\ ") }
            _ => { if let Key::Char(k) = key { text.push(k) } }
        }
        write!(stdout, "{}", text).unwrap();
        let guess = get_guess(&text);
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
