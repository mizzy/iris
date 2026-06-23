use std::env;
use std::io::{self, BufRead, Write};

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

mod diff;
mod highlight;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--list-themes") {
        let ts = ThemeSet::load_defaults();
        for name in ts.themes.keys() {
            println!("{name}");
        }
        return;
    }

    let theme_name = env::var("IRIS_THEME").unwrap_or_else(|_| "base16-ocean.dark".to_string());
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let theme = ts
        .themes
        .get(&theme_name)
        .unwrap_or_else(|| ts.themes.values().next().expect("no themes available"));

    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    let mut parser = diff::DiffParser::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        parser.feed(&line);
    }

    let blocks = parser.finish();

    for block in &blocks {
        match block {
            diff::Block::Plain(lines) => {
                for line in lines {
                    let _ = writeln!(out, "{line}");
                }
            }
            diff::Block::Diff(file) => {
                highlight::print_file(file, &ss, theme, &mut out);
            }
        }
    }

    let _ = out.flush();
}
