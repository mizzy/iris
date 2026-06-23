use std::io::{self, BufRead, Write};

mod diff;
mod highlight;

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = io::BufWriter::new(stdout.lock());

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        // TODO: parse diff, detect language, highlight, and output
        let _ = writeln!(out, "{line}");
    }

    let _ = out.flush();
}
