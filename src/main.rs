use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::{Command, Stdio};

use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

mod diff;
mod highlight;

fn main() {
    // Reset SIGPIPE to default so broken pipe kills the process cleanly
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

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

    if io::stdout().is_terminal() {
        let mut child = Command::new("less")
            .arg("-R")
            .stdin(Stdio::piped())
            .spawn()
            .expect("failed to start less");

        let pipe = child.stdin.take().expect("failed to open less stdin");
        let mut out = io::BufWriter::new(pipe);
        process_stdin(&ss, theme, &mut out);
        let _ = out.flush();
        drop(out);

        let _ = child.wait();
    } else {
        let stdout = io::stdout();
        let mut out = io::BufWriter::new(stdout.lock());
        process_stdin(&ss, theme, &mut out);
        let _ = out.flush();
    }
}

fn process_stdin<W: Write>(ss: &SyntaxSet, theme: &syntect::highlighting::Theme, out: &mut W) {
    let stdin = io::stdin();
    let mut parser = diff::DiffParser::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        for block in parser.feed(&line) {
            if write_block(&block, ss, theme, out).is_err() {
                return;
            }
        }
    }

    for block in parser.finish() {
        if write_block(&block, ss, theme, out).is_err() {
            return;
        }
    }
}

fn write_block<W: Write>(
    block: &diff::Block,
    ss: &SyntaxSet,
    theme: &syntect::highlighting::Theme,
    out: &mut W,
) -> io::Result<()> {
    match block {
        diff::Block::Plain(lines) => {
            for line in lines {
                writeln!(out, "{line}")?;
            }
        }
        diff::Block::Diff(file) => {
            highlight::print_file(file, ss, theme, out)?;
        }
    }
    Ok(())
}
