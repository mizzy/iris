# iris

A syntax highlighter for `git diff` and `git log -p` output.

iris reads unified diff from stdin, detects the language of each changed file, and prints the diff with syntax-highlighted code while preserving the familiar diff format (headers, `+`/`-` markers, and hunk ranges).

## Install

```
cargo install --path .
```

## Usage

```bash
# Highlight git diff output
git diff | iris

# Highlight git log -p output
git log -p | iris

# Use as a git pager
git config --global pager.diff iris
git config --global pager.log iris
git config --global pager.show iris
```

## Configuration

iris uses the `IRIS_THEME` environment variable to select a color theme. Defaults to `base16-ocean.dark`.

```bash
export IRIS_THEME="Solarized (dark)"
```

To list available themes:

```bash
iris --list-themes
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
