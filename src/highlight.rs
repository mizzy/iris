use std::io::Write;

use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::diff::{DiffFile, DiffLine};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";

pub fn print_file<W: Write>(file: &DiffFile, ss: &SyntaxSet, theme: &Theme, out: &mut W) {
    for header in &file.header_lines {
        let _ = writeln!(out, "{BOLD}{header}{RESET}");
    }

    let syntax = file
        .filename
        .as_deref()
        .and_then(|name| ss.find_syntax_for_file(name).ok().flatten())
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, theme);

    for diff_line in &file.lines {
        match diff_line {
            DiffLine::HunkHeader(text) => {
                let _ = writeln!(out, "{CYAN}{text}{RESET}");
            }
            DiffLine::Context(text) => {
                let highlighted = highlight_line(&mut h, ss, text);
                let _ = writeln!(out, " {highlighted}{RESET}");
            }
            DiffLine::Added(text) => {
                let highlighted = highlight_line(&mut h, ss, text);
                let _ = writeln!(out, "{GREEN}+{RESET}{highlighted}{RESET}");
            }
            DiffLine::Removed(text) => {
                let highlighted = highlight_line(&mut h, ss, text);
                let _ = writeln!(out, "{RED}-{RESET}{highlighted}{RESET}");
            }
        }
    }
}

fn highlight_line(h: &mut HighlightLines, ss: &SyntaxSet, text: &str) -> String {
    let line = if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    };

    let regions = h.highlight_line(&line, ss).unwrap_or_default();
    as_24_bit_terminal_escaped(&regions, false)
        .trim_end_matches('\n')
        .to_string()
}
