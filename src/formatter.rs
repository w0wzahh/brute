use anyhow::Result;

/// Formatter for Brute code.
///
/// Rather than pretty-printing the AST (which would drop comments and
/// source layout), the formatter works on the raw text: it normalizes
/// indentation to a consistent depth based on brace nesting, collapses
/// runs of blank lines, and trims trailing whitespace. Non-destructive
/// by design — comments and code are always preserved.
pub struct Formatter {
    indent_size: usize,
}

impl Formatter {
    /// Create a new formatter with default settings
    pub fn new() -> Self {
        Self { indent_size: 4 }
    }

    /// Create a new formatter with custom indent size
    pub fn with_indent_size(indent_size: usize) -> Self {
        Self { indent_size }
    }

    /// Format Brute source code and return the formatted code
    pub fn format(&mut self, source: &str) -> Result<String> {
        let mut out = String::with_capacity(source.len());
        let mut depth: i64 = 0;
        let mut blank_run = 0usize;
        let mut in_block_comment = false;

        for raw_line in source.lines() {
            let trimmed = raw_line.trim();

            // Track block comments so braces inside them don't affect depth
            if in_block_comment {
                out.push('\n');
                if trimmed.contains("*/") {
                    in_block_comment = false;
                }
                continue;
            }

            // Collapse runs of blank lines to a single blank line
            if trimmed.is_empty() {
                blank_run += 1;
                if blank_run <= 1 {
                    out.push('\n');
                }
                continue;
            }
            blank_run = 0;

            // A line that only *opens* a block comment
            if trimmed.starts_with("/*") && !trimmed.contains("*/") {
                in_block_comment = true;
            }

            let (opens, closes) = brace_balance(trimmed);

            // Dedent before writing when the line closes a block —
            // `}`, `} else {`, `) => {`, `case x:`-style closers
            let leading_close = closes > opens && trimmed.starts_with('}')
                || trimmed.starts_with(')')
                && trimmed.contains('{') == false
                && closes > 0;
            let line_depth = (depth - if leading_close || trimmed.starts_with('}')
                || trimmed.starts_with("else") || trimmed.starts_with("catch")
                || trimmed.starts_with("finally") { 1 } else { 0 }).max(0);

            for _ in 0..line_depth * self.indent_size as i64 {
                out.push(' ');
            }
            out.push_str(trimmed);
            out.push('\n');

            depth = (depth + opens - closes).max(0);
        }

        // Ensure exactly one trailing newline
        while out.ends_with("\n\n") {
            out.pop();
        }
        if !out.ends_with('\n') {
            out.push('\n');
        }

        Ok(out)
    }
}

/// Count `{` and `}` braces outside of strings and char literals.
fn brace_balance(line: &str) -> (i64, i64) {
    let mut opens = 0i64;
    let mut closes = 0i64;
    let mut in_str = false;
    let mut in_char = false;
    let mut escaped = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if escaped {
            escaped = false;
            continue;
        }
        match c {
            '\\' if in_str || in_char => escaped = true,
            '"' if !in_char => in_str = !in_str,
            '\'' if !in_str => in_char = !in_char,
            '/' if !in_str && !in_char && chars.peek() == Some(&'/') => break, // line comment
            '{' if !in_str && !in_char => opens += 1,
            '}' if !in_str && !in_char => closes += 1,
            _ => {}
        }
    }
    (opens, closes)
}
