use std::io::{self, Write};

use similar::{ChangeTag, TextDiff};
use syntect::easy::HighlightLines;
use syntect::highlighting::Theme;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;

use crate::diff::{DiffFile, DiffLine};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const CYAN: &str = "\x1b[36m";

const ADDITION_BG: &str = "\x1b[48;2;0;50;0m";
const ADDITION_HIGHLIGHT_BG: &str = "\x1b[48;2;0;80;0m";
const DELETION_BG: &str = "\x1b[48;2;50;0;0m";
const DELETION_HIGHLIGHT_BG: &str = "\x1b[48;2;120;0;0m";

const BOLD_GREEN: &str = "\x1b[1;32m";
const BOLD_RED: &str = "\x1b[1;31m";

const WORD_DIFF_SIMILARITY_THRESHOLD: f32 = 0.6;

pub fn print_file<W: Write>(
    file: &DiffFile,
    ss: &SyntaxSet,
    theme: &Theme,
    out: &mut W,
) -> io::Result<()> {
    for header in &file.header_lines {
        writeln!(out, "{BOLD}{header}{RESET}")?;
    }

    let syntax = file
        .filename
        .as_deref()
        .and_then(|name| ss.find_syntax_for_file(name).ok().flatten())
        .unwrap_or_else(|| ss.find_syntax_plain_text());

    let mut h = HighlightLines::new(syntax, theme);

    let lines = &file.lines;
    let mut i = 0;

    while i < lines.len() {
        match &lines[i] {
            DiffLine::HunkHeader(text) => {
                writeln!(out, "{CYAN}{text}{RESET}")?;
                i += 1;
            }
            DiffLine::Context(text) => {
                let highlighted = highlight_line(&mut h, ss, text);
                writeln!(out, " {highlighted}{RESET}")?;
                i += 1;
            }
            DiffLine::Removed(_) => {
                let (removed, added) = collect_change_group(lines, &mut i);
                print_change_group(&removed, &added, &mut h, ss, out)?;
            }
            DiffLine::Added(text) => {
                let highlighted = highlight_line(&mut h, ss, text);
                writeln!(out, "{BOLD_GREEN}+{ADDITION_BG}{highlighted}{RESET}")?;
                i += 1;
            }
        }
    }

    Ok(())
}

fn collect_change_group<'a>(lines: &'a [DiffLine], i: &mut usize) -> (Vec<&'a str>, Vec<&'a str>) {
    let mut removed = Vec::new();
    let mut added = Vec::new();

    while *i < lines.len() {
        match &lines[*i] {
            DiffLine::Removed(text) => {
                removed.push(text.as_str());
                *i += 1;
            }
            _ => break,
        }
    }

    while *i < lines.len() {
        match &lines[*i] {
            DiffLine::Added(text) => {
                added.push(text.as_str());
                *i += 1;
            }
            _ => break,
        }
    }

    (removed, added)
}

fn print_change_group<W: Write>(
    removed: &[&str],
    added: &[&str],
    h: &mut HighlightLines,
    ss: &SyntaxSet,
    out: &mut W,
) -> io::Result<()> {
    if removed.len() == added.len() && are_similar_enough(removed, added) {
        for (old, new) in removed.iter().zip(added.iter()) {
            print_word_diff_line(old, new, h, ss, out)?;
        }
    } else {
        for text in removed {
            let highlighted = highlight_line(h, ss, text);
            writeln!(out, "{BOLD_RED}-{DELETION_BG}{highlighted}{RESET}")?;
        }
        for text in added {
            let highlighted = highlight_line(h, ss, text);
            writeln!(out, "{BOLD_GREEN}+{ADDITION_BG}{highlighted}{RESET}")?;
        }
    }
    Ok(())
}

fn are_similar_enough(removed: &[&str], added: &[&str]) -> bool {
    for (old, new) in removed.iter().zip(added.iter()) {
        let diff = TextDiff::from_chars(*old, *new);
        if diff.ratio() < WORD_DIFF_SIMILARITY_THRESHOLD {
            return false;
        }
    }
    true
}

fn print_word_diff_line<W: Write>(
    old: &str,
    new: &str,
    h: &mut HighlightLines,
    ss: &SyntaxSet,
    out: &mut W,
) -> io::Result<()> {
    let diff = TextDiff::from_words(old, new);

    write!(out, "{BOLD_RED}-{DELETION_BG}")?;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                let highlighted = highlight_inline(h, ss, change.value());
                write!(out, "{DELETION_BG}{highlighted}")?;
            }
            ChangeTag::Delete => {
                let highlighted = highlight_inline(h, ss, change.value());
                write!(out, "{DELETION_HIGHLIGHT_BG}{highlighted}")?;
            }
            ChangeTag::Insert => {}
        }
    }
    writeln!(out, "{RESET}")?;

    write!(out, "{BOLD_GREEN}+{ADDITION_BG}")?;
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => {
                let highlighted = highlight_inline(h, ss, change.value());
                write!(out, "{ADDITION_BG}{highlighted}")?;
            }
            ChangeTag::Insert => {
                let highlighted = highlight_inline(h, ss, change.value());
                write!(out, "{ADDITION_HIGHLIGHT_BG}{highlighted}")?;
            }
            ChangeTag::Delete => {}
        }
    }
    writeln!(out, "{RESET}")?;

    Ok(())
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

fn highlight_inline(h: &mut HighlightLines, ss: &SyntaxSet, text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let padded = if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{text}\n")
    };
    let regions = h.highlight_line(&padded, ss).unwrap_or_default();
    as_24_bit_terminal_escaped(&regions, false)
        .trim_end_matches('\n')
        .to_string()
}
