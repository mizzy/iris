pub enum Block {
    Plain(Vec<String>),
    Diff(DiffFile),
}

pub enum DiffLine {
    Context(String),
    Added(String),
    Removed(String),
    HunkHeader(String),
}

pub struct DiffFile {
    pub header_lines: Vec<String>,
    pub filename: Option<String>,
    pub lines: Vec<DiffLine>,
}

pub struct DiffParser {
    pending: Vec<Block>,
    plain_lines: Vec<String>,
    current_header: Vec<String>,
    current_filename: Option<String>,
    current_lines: Vec<DiffLine>,
    in_file: bool,
}

impl DiffParser {
    pub fn new() -> Self {
        Self {
            pending: Vec::new(),
            plain_lines: Vec::new(),
            current_header: Vec::new(),
            current_filename: None,
            current_lines: Vec::new(),
            in_file: false,
        }
    }

    pub fn feed(&mut self, line: &str, raw_line: &str) -> Vec<Block> {
        self.pending.clear();

        if line.starts_with("diff ") {
            self.flush_file();
            self.flush_plain();
            self.current_header.push(line.to_string());
            self.in_file = true;
            return std::mem::take(&mut self.pending);
        }

        if !self.in_file {
            self.plain_lines.push(raw_line.to_string());
            return std::mem::take(&mut self.pending);
        }

        if line.starts_with("+++ b/") || line.starts_with("+++ /dev/null") {
            if line.starts_with("+++ b/") {
                self.current_filename = Some(line[6..].to_string());
            }
            self.current_header.push(line.to_string());
            return std::mem::take(&mut self.pending);
        }

        if is_header_line(line) {
            self.current_header.push(line.to_string());
            return std::mem::take(&mut self.pending);
        }

        if line.starts_with("@@") {
            self.current_lines
                .push(DiffLine::HunkHeader(line.to_string()));
            return std::mem::take(&mut self.pending);
        }

        if line.starts_with('+') {
            self.current_lines
                .push(DiffLine::Added(line[1..].to_string()));
        } else if line.starts_with('-') {
            self.current_lines
                .push(DiffLine::Removed(line[1..].to_string()));
        } else if line.starts_with(' ') {
            self.current_lines
                .push(DiffLine::Context(line[1..].to_string()));
        } else {
            self.current_lines
                .push(DiffLine::Context(line.to_string()));
        }

        std::mem::take(&mut self.pending)
    }

    fn flush_plain(&mut self) {
        if !self.plain_lines.is_empty() {
            self.pending
                .push(Block::Plain(std::mem::take(&mut self.plain_lines)));
        }
    }

    fn flush_file(&mut self) {
        if !self.current_header.is_empty() || !self.current_lines.is_empty() {
            self.pending.push(Block::Diff(DiffFile {
                header_lines: std::mem::take(&mut self.current_header),
                filename: self.current_filename.take(),
                lines: std::mem::take(&mut self.current_lines),
            }));
            self.in_file = false;
        }
    }

    pub fn finish(mut self) -> Vec<Block> {
        self.pending.clear();
        self.flush_file();
        self.flush_plain();
        self.pending
    }
}

fn is_header_line(line: &str) -> bool {
    line.starts_with("--- ")
        || line.starts_with("index ")
        || line.starts_with("new file")
        || line.starts_with("deleted file")
        || line.starts_with("old mode")
        || line.starts_with("new mode")
        || line.starts_with("similarity index")
        || line.starts_with("rename from")
        || line.starts_with("rename to")
        || line.starts_with("copy from")
        || line.starts_with("copy to")
        || line.starts_with("Binary files")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_all(input: &str) -> Vec<Block> {
        let mut parser = DiffParser::new();
        let mut blocks = Vec::new();
        for line in input.lines() {
            blocks.extend(parser.feed(line, line));
        }
        blocks.extend(parser.finish());
        blocks
    }

    #[test]
    fn parse_simple_diff() {
        let input = "\
diff --git a/foo.rs b/foo.rs
index 1234567..abcdefg 100644
--- a/foo.rs
+++ b/foo.rs
@@ -1,3 +1,3 @@
 fn main() {
-    old();
+    new();
 }";
        let blocks = parse_all(input);

        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Diff(file) => {
                assert_eq!(file.filename.as_deref(), Some("foo.rs"));
                assert_eq!(file.header_lines.len(), 4);
                assert_eq!(file.lines.len(), 5);
            }
            _ => panic!("expected Diff block"),
        }
    }

    #[test]
    fn parse_git_log_with_commit_header() {
        let input = "\
commit abc123
Author: Test <test@example.com>
Date:   Mon Jan 1 00:00:00 2026 +0000

    Some commit message

diff --git a/foo.rs b/foo.rs
index 1234567..abcdefg 100644
--- a/foo.rs
+++ b/foo.rs
@@ -1 +1 @@
-old
+new";
        let blocks = parse_all(input);

        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            Block::Plain(lines) => {
                assert_eq!(lines.len(), 6);
                assert!(lines[0].starts_with("commit"));
            }
            _ => panic!("expected Plain block"),
        }
        match &blocks[1] {
            Block::Diff(file) => {
                assert_eq!(file.filename.as_deref(), Some("foo.rs"));
            }
            _ => panic!("expected Diff block"),
        }
    }

    #[test]
    fn parse_multiple_files() {
        let input = "\
diff --git a/a.rs b/a.rs
--- a/a.rs
+++ b/a.rs
@@ -1 +1 @@
-old_a
+new_a
diff --git a/b.py b/b.py
--- a/b.py
+++ b/b.py
@@ -1 +1 @@
-old_b
+new_b";
        let blocks = parse_all(input);

        assert_eq!(blocks.len(), 2);
        match &blocks[0] {
            Block::Diff(f) => assert_eq!(f.filename.as_deref(), Some("a.rs")),
            _ => panic!("expected Diff"),
        }
        match &blocks[1] {
            Block::Diff(f) => assert_eq!(f.filename.as_deref(), Some("b.py")),
            _ => panic!("expected Diff"),
        }
    }

    #[test]
    fn plain_only_input() {
        let input = "just some text\nno diff here";
        let blocks = parse_all(input);

        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Plain(lines) => assert_eq!(lines.len(), 2),
            _ => panic!("expected Plain block"),
        }
    }

    #[test]
    fn deleted_file() {
        let input = "\
diff --git a/gone.rs b/gone.rs
deleted file mode 100644
index 1234567..0000000
--- a/gone.rs
+++ /dev/null
@@ -1 +0,0 @@
-bye";
        let blocks = parse_all(input);

        assert_eq!(blocks.len(), 1);
        match &blocks[0] {
            Block::Diff(file) => {
                assert!(file.filename.is_none());
                assert!(file.header_lines.iter().any(|h| h.starts_with("deleted file")));
            }
            _ => panic!("expected Diff block"),
        }
    }

    #[test]
    fn streaming_emits_blocks_incrementally() {
        let mut parser = DiffParser::new();

        let blocks = parser.feed("commit abc123", "commit abc123");
        assert!(blocks.is_empty());

        let blocks = parser.feed("diff --git a/foo.rs b/foo.rs", "diff --git a/foo.rs b/foo.rs");
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Plain(_)));

        let blocks = parser.feed("+++ b/foo.rs", "+++ b/foo.rs");
        assert!(blocks.is_empty());

        let blocks = parser.feed("@@ -1 +1 @@", "@@ -1 +1 @@");
        assert!(blocks.is_empty());

        let blocks = parser.feed("+new", "+new");
        assert!(blocks.is_empty());

        let blocks = parser.finish();
        assert_eq!(blocks.len(), 1);
        assert!(matches!(&blocks[0], Block::Diff(_)));
    }
}
