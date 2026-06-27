use std::env;
use std::io::{self, BufRead, IsTerminal, Write};
use std::process::{Command, Stdio};

use syntect::highlighting::ThemeSet;
use syntect::parsing::{SyntaxSet, SyntaxSetBuilder};

mod diff;
mod highlight;

fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    if args.iter().any(|a| a == "--list-themes") {
        let ts = ThemeSet::load_defaults();
        for name in ts.themes.keys() {
            println!("{name}");
        }
        return;
    }

    let theme_name = env::var("IRIS_THEME").unwrap_or_else(|_| "base16-ocean.dark".to_string());
    let ss = load_syntax_set();
    let ts = ThemeSet::load_defaults();

    let theme = ts
        .themes
        .get(&theme_name)
        .unwrap_or_else(|| ts.themes.values().next().expect("no themes available"));

    let is_git_pager = env::var("GIT_PAGER_IN_USE").is_ok();
    let need_pager = !is_git_pager && io::stdout().is_terminal();

    if need_pager {
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

fn load_syntax_set() -> SyntaxSet {
    let bundled_bytes = include_bytes!(concat!(env!("OUT_DIR"), "/syntax_set.bin"));

    if let Ok(syntax_set) = syntect::dumps::from_uncompressed_data::<SyntaxSet>(bundled_bytes) {
        let user_dir = iris_config_dir().join("syntaxes");
        if user_dir.exists() {
            let mut builder: SyntaxSetBuilder = syntax_set.into_builder();
            let _ = builder.add_from_folder(&user_dir, true);
            return builder.build();
        }
        return syntax_set;
    }

    SyntaxSet::load_defaults_newlines()
}

fn iris_config_dir() -> std::path::PathBuf {
    if let Some(home) = env::var_os("HOME") {
        return std::path::PathBuf::from(home).join(".config/iris");
    }
    std::path::PathBuf::from(".config/iris")
}

fn print_help() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "\
iris {version} — syntax highlighter for git output

USAGE:
    git diff | iris
    git log -p | iris
    git config --global core.pager iris

OPTIONS:
    -h, --help          Show this help
    --list-themes       List available color themes

ENVIRONMENT:
    IRIS_THEME          Color theme (default: base16-ocean.dark)"
    );
}

fn process_stdin<W: Write>(ss: &SyntaxSet, theme: &syntect::highlighting::Theme, out: &mut W) {
    let stdin = io::stdin();
    let mut parser = diff::DiffParser::new();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.replace('\t', "    ");
        let stripped = strip_ansi(&line);

        for block in parser.feed(&stripped, &line) {
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

fn strip_ansi(s: &str) -> String {
    let bytes = strip_ansi_escapes::strip(s);
    String::from_utf8(bytes).unwrap_or_else(|_| s.to_string())
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
