use std::fmt;
use crate::error::{BruteError, Result, SourceLocation};

// ─────────────────────────────────────────────────────────────────────────────
// Token types
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Literals
    Identifier,
    IntLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,
    // Keywords
    Let, Mut, Fn, If, Else, While, For, In, Match, Return,
    Break, Continue, Struct, Enum, Import, From, As, Pub,
    True, False, None, Async, Await, Try, Catch, Finally,
    Static, Type, Trait, Impl, Self_, This, Use, Const,
    Loop, Where, Mod, Extern, Unsafe, Ref, Box_, Is,
    // Built-in types
    Int, Float, Bool, String_, Char_, Void,
    // Arithmetic
    Plus, Minus, Star, Slash, Percent, StarStar,
    // Comparison
    EqualEqual, NotEqual, Greater, GreaterEqual, Less, LessEqual,
    // Assignment
    Equal, PlusEqual, MinusEqual, StarEqual, SlashEqual,
    PercentEqual, AndEqual, OrEqual, XorEqual,
    StarStarEqual, SlashSlashEqual,
    // Logical
    And, Or, Not,
    // Bitwise
    Ampersand, Pipe, Caret, Tilde, LeftShift, RightShift,
    // Special operators
    Arrow, FatArrow, DotDot, DotDotEqual, QuestionQuestion,
    Pipeline, At, Question, Colon, DoubleColon,
    // Delimiters
    LeftParen, RightParen, LeftBrace, RightBrace,
    LeftBracket, RightBracket, Comma, Dot, Semicolon, Hash,
    // End of file
    EOF,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            TokenType::Identifier      => "identifier",
            TokenType::IntLiteral      => "integer",
            TokenType::FloatLiteral    => "float",
            TokenType::StringLiteral   => "string",
            TokenType::CharLiteral     => "char",
            TokenType::Let             => "let",
            TokenType::Mut             => "mut",
            TokenType::Fn              => "fn",
            TokenType::If              => "if",
            TokenType::Else            => "else",
            TokenType::While           => "while",
            TokenType::For             => "for",
            TokenType::In              => "in",
            TokenType::Match           => "match",
            TokenType::Return          => "return",
            TokenType::Break           => "break",
            TokenType::Continue        => "continue",
            TokenType::Struct          => "struct",
            TokenType::Enum            => "enum",
            TokenType::Import          => "import",
            TokenType::From            => "from",
            TokenType::As              => "as",
            TokenType::Pub             => "pub",
            TokenType::True            => "true",
            TokenType::False           => "false",
            TokenType::None            => "none",
            TokenType::Async           => "async",
            TokenType::Await           => "await",
            TokenType::Try             => "try",
            TokenType::Catch           => "catch",
            TokenType::Finally         => "finally",
            TokenType::Static          => "static",
            TokenType::Type            => "type",
            TokenType::Trait           => "trait",
            TokenType::Impl            => "impl",
            TokenType::Self_           => "self",
            TokenType::This            => "this",
            TokenType::Use             => "use",
            TokenType::Const           => "const",
            TokenType::Loop            => "loop",
            TokenType::Where           => "where",
            TokenType::Mod             => "mod",
            TokenType::Extern          => "extern",
            TokenType::Unsafe          => "unsafe",
            TokenType::Ref             => "ref",
            TokenType::Box_            => "box",
            TokenType::Is              => "is",
            TokenType::Int             => "int",
            TokenType::Float           => "float",
            TokenType::Bool            => "bool",
            TokenType::String_         => "string",
            TokenType::Char_           => "char",
            TokenType::Void            => "void",
            TokenType::Plus            => "+",
            TokenType::Minus           => "-",
            TokenType::Star            => "*",
            TokenType::Slash           => "/",
            TokenType::Percent         => "%",
            TokenType::StarStar        => "**",
            TokenType::EqualEqual      => "==",
            TokenType::NotEqual        => "!=",
            TokenType::Greater         => ">",
            TokenType::GreaterEqual    => ">=",
            TokenType::Less            => "<",
            TokenType::LessEqual       => "<=",
            TokenType::Equal           => "=",
            TokenType::PlusEqual       => "+=",
            TokenType::MinusEqual      => "-=",
            TokenType::StarEqual       => "*=",
            TokenType::SlashEqual      => "/=",
            TokenType::PercentEqual    => "%=",
            TokenType::AndEqual        => "&=",
            TokenType::OrEqual         => "|=",
            TokenType::XorEqual        => "^=",
            TokenType::StarStarEqual   => "**=",
            TokenType::SlashSlashEqual => "//=",
            TokenType::And             => "&&",
            TokenType::Or              => "||",
            TokenType::Not             => "!",
            TokenType::Ampersand       => "&",
            TokenType::Pipe            => "|",
            TokenType::Caret           => "^",
            TokenType::Tilde           => "~",
            TokenType::LeftShift       => "<<",
            TokenType::RightShift      => ">>",
            TokenType::Arrow           => "->",
            TokenType::FatArrow        => "=>",
            TokenType::DotDot          => "..",
            TokenType::DotDotEqual     => "..=",
            TokenType::QuestionQuestion => "??",
            TokenType::Pipeline        => "|>",
            TokenType::At              => "@",
            TokenType::Question        => "?",
            TokenType::Colon           => ":",
            TokenType::DoubleColon     => "::",
            TokenType::LeftParen       => "(",
            TokenType::RightParen      => ")",
            TokenType::LeftBrace       => "{",
            TokenType::RightBrace      => "}",
            TokenType::LeftBracket     => "[",
            TokenType::RightBracket    => "]",
            TokenType::Comma           => ",",
            TokenType::Dot             => ".",
            TokenType::Semicolon       => ";",
            TokenType::Hash            => "#",
            TokenType::EOF             => "<EOF>",
        };
        write!(f, "{}", s)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Token
// ─────────────────────────────────────────────────────────────────────────────
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme:     String,
    pub line:       usize,
    pub column:     usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize, column: usize) -> Self {
        Token { token_type, lexeme, line, column }
    }

    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.line, self.column)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'", self.lexeme)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Lexer
// ─────────────────────────────────────────────────────────────────────────────
pub struct Lexer {
    /// Source stored as a Vec<char> so every index operation is O(1) and
    /// works correctly with multi-byte Unicode characters.
    source:   Vec<char>,
    tokens:   Vec<Token>,
    start:    usize,
    current:  usize,
    line:     usize,
    column:   usize,
    filename: Option<String>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Lexer {
            source:   source.chars().collect(),
            tokens:   Vec::new(),
            start:    0,
            current:  0,
            line:     1,
            column:   1,
            filename: None,
        }
    }

    pub fn with_filename(source: &str, filename: String) -> Self {
        let mut l = Lexer::new(source);
        l.filename = Some(filename);
        l
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn current_location(&self) -> SourceLocation {
        match &self.filename {
            Some(f) => SourceLocation::with_file(self.line, self.column, f.clone()),
            None    => SourceLocation::new(self.line, self.column),
        }
    }

    fn error(&self, msg: impl Into<String>) -> BruteError {
        BruteError::SyntaxError {
            location: self.current_location(),
            message:  msg.into(),
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.current] }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() { '\0' }
        else { self.source[self.current + 1] }
    }

    fn advance(&mut self) -> char {
        let c = self.source[self.current];
        self.current += 1;
        self.column  += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            return false;
        }
        self.current += 1;
        self.column  += 1;
        true
    }

    fn add_token(&mut self, tt: TokenType) {
        let lexeme: String = self.source[self.start..self.current].iter().collect();
        let col = self.column.saturating_sub(self.current - self.start);
        self.tokens.push(Token::new(tt, lexeme, self.line, col));
    }

    fn add_token_with_lexeme(&mut self, tt: TokenType, lexeme: String) {
        let col = self.column.saturating_sub(self.current - self.start);
        self.tokens.push(Token::new(tt, lexeme, self.line, col));
    }

    // ── public entry point ────────────────────────────────────────────────────

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }
        self.tokens.push(Token::new(
            TokenType::EOF, String::new(), self.line, self.column,
        ));
        Ok(self.tokens.clone())
    }

    // ── main dispatch ─────────────────────────────────────────────────────────

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            // Single-character tokens
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            '[' => self.add_token(TokenType::LeftBracket),
            ']' => self.add_token(TokenType::RightBracket),
            ',' => self.add_token(TokenType::Comma),
            ';' => self.add_token(TokenType::Semicolon),
            '~' => self.add_token(TokenType::Tilde),
            '@' => self.add_token(TokenType::At),
            '#' => self.add_token(TokenType::Hash),
            // Dot / range
            '.' => {
                if self.match_char('.') {
                    if self.match_char('=') { self.add_token(TokenType::DotDotEqual); }
                    else                    { self.add_token(TokenType::DotDot); }
                } else {
                    self.add_token(TokenType::Dot);
                }
            }
            // Colon / double-colon
            ':' => {
                if self.match_char(':') { self.add_token(TokenType::DoubleColon); }
                else                   { self.add_token(TokenType::Colon); }
            }
            // Arithmetic
            '+' => {
                if self.match_char('=') { self.add_token(TokenType::PlusEqual); }
                else                   { self.add_token(TokenType::Plus); }
            }
            '-' => {
                if      self.match_char('>') { self.add_token(TokenType::Arrow); }
                else if self.match_char('=') { self.add_token(TokenType::MinusEqual); }
                else                        { self.add_token(TokenType::Minus); }
            }
            '*' => {
                if self.match_char('*') {
                    if self.match_char('=') { self.add_token(TokenType::StarStarEqual); }
                    else                   { self.add_token(TokenType::StarStar); }
                } else if self.match_char('=') {
                    self.add_token(TokenType::StarEqual);
                } else {
                    self.add_token(TokenType::Star);
                }
            }
            '%' => {
                if self.match_char('=') { self.add_token(TokenType::PercentEqual); }
                else                   { self.add_token(TokenType::Percent); }
            }
            // Slash / comments
            '/' => {
                if self.match_char('/') {
                    if self.match_char('=') {
                        self.add_token(TokenType::SlashSlashEqual);
                    } else {
                        // Line comment — consume until newline
                        while self.peek() != '\n' && !self.is_at_end() { self.advance(); }
                    }
                } else if self.match_char('*') {
                    self.block_comment()?;
                } else if self.match_char('=') {
                    self.add_token(TokenType::SlashEqual);
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            // Logical / bitwise
            '!' => {
                if self.match_char('=') { self.add_token(TokenType::NotEqual); }
                else                   { self.add_token(TokenType::Not); }
            }
            '=' => {
                if      self.match_char('=') { self.add_token(TokenType::EqualEqual); }
                else if self.match_char('>') { self.add_token(TokenType::FatArrow); }
                else                        { self.add_token(TokenType::Equal); }
            }
            '<' => {
                if      self.match_char('<') { self.add_token(TokenType::LeftShift); }
                else if self.match_char('=') { self.add_token(TokenType::LessEqual); }
                else                        { self.add_token(TokenType::Less); }
            }
            '>' => {
                if      self.match_char('>') { self.add_token(TokenType::RightShift); }
                else if self.match_char('=') { self.add_token(TokenType::GreaterEqual); }
                else                        { self.add_token(TokenType::Greater); }
            }
            '&' => {
                if      self.match_char('&') { self.add_token(TokenType::And); }
                else if self.match_char('=') { self.add_token(TokenType::AndEqual); }
                else                        { self.add_token(TokenType::Ampersand); }
            }
            '|' => {
                if      self.match_char('|') { self.add_token(TokenType::Or); }
                else if self.match_char('>') { self.add_token(TokenType::Pipeline); }
                else if self.match_char('=') { self.add_token(TokenType::OrEqual); }
                else                        { self.add_token(TokenType::Pipe); }
            }
            '^' => {
                if self.match_char('=') { self.add_token(TokenType::XorEqual); }
                else                   { self.add_token(TokenType::Caret); }
            }
            '?' => {
                if self.match_char('?') { self.add_token(TokenType::QuestionQuestion); }
                else                   { self.add_token(TokenType::Question); }
            }
            // String literals
            '"' => self.string()?,
            // Char literals
            '\'' => self.char_literal()?,
            // Whitespace
            ' ' | '\r' | '\t' => {}
            '\n' => {
                self.line   += 1;
                self.column  = 1;
            }
            // Numbers & identifiers
            _ => {
                if c == '0' && (self.peek() == 'x' || self.peek() == 'X') {
                    self.advance(); // consume x/X
                    self.hex_number()?;
                } else if c == '0' && (self.peek() == 'b' || self.peek() == 'B') {
                    self.advance();
                    self.bin_number()?;
                } else if c.is_ascii_digit() {
                    self.number()?;
                } else if c.is_alphabetic() || c == '_' {
                    self.identifier();
                } else {
                    return Err(self.error(format!("Unexpected character '{}'", c)));
                }
            }
        }
        Ok(())
    }

    // ── comments ──────────────────────────────────────────────────────────────

    fn block_comment(&mut self) -> Result<()> {
        let mut depth = 1usize;
        while depth > 0 {
            if self.is_at_end() {
                return Err(self.error("Unterminated block comment"));
            }
            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance(); self.advance(); depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); self.advance(); depth -= 1;
            } else {
                if self.peek() == '\n' { self.line += 1; self.column = 0; }
                self.advance();
            }
        }
        Ok(())
    }

    // ── string literals ───────────────────────────────────────────────────────

    fn string(&mut self) -> Result<()> {
        let mut value = String::new();
        loop {
            if self.is_at_end() {
                return Err(self.error("Unterminated string literal"));
            }
            match self.peek() {
                '"' => { self.advance(); break; }
                '\\' => {
                    self.advance(); // consume backslash
                    let esc = self.advance();
                    match esc {
                        'n'  => value.push('\n'),
                        'r'  => value.push('\r'),
                        't'  => value.push('\t'),
                        '\\' => value.push('\\'),
                        '"'  => value.push('"'),
                        '\'' => value.push('\''),
                        '0'  => value.push('\0'),
                        'u'  => {
                            // \u{XXXX}
                            if !self.match_char('{') {
                                return Err(self.error("Expected '{' after \\u"));
                            }
                            let mut hex = String::new();
                            while self.peek() != '}' && !self.is_at_end() {
                                hex.push(self.advance());
                            }
                            if !self.match_char('}') {
                                return Err(self.error("Expected '}' to close \\u{...}"));
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| self.error(format!("Invalid Unicode escape \\u{{{}}}", hex)))?;
                            let ch = char::from_u32(code)
                                .ok_or_else(|| self.error(format!("Invalid Unicode codepoint {:x}", code)))?;
                            value.push(ch);
                        }
                        other => {
                            return Err(self.error(format!("Unknown escape sequence '\\{}'", other)));
                        }
                    }
                }
                '\n' => {
                    self.line += 1; self.column = 0;
                    value.push(self.advance());
                }
                _ => { value.push(self.advance()); }
            }
        }
        self.add_token_with_lexeme(TokenType::StringLiteral, value);
        Ok(())
    }

    // ── char literals ─────────────────────────────────────────────────────────

    fn char_literal(&mut self) -> Result<()> {
        let ch = if self.peek() == '\\' {
            self.advance(); // consume backslash
            match self.advance() {
                'n'  => '\n',
                'r'  => '\r',
                't'  => '\t',
                '\\' => '\\',
                '\'' => '\'',
                '"'  => '"',
                '0'  => '\0',
                other => return Err(self.error(format!("Unknown escape '\\{}'", other))),
            }
        } else if self.peek() == '\'' {
            return Err(self.error("Empty character literal"));
        } else {
            self.advance()
        };

        if !self.match_char('\'') {
            return Err(self.error("Character literal may only contain one character"));
        }
        self.add_token_with_lexeme(TokenType::CharLiteral, ch.to_string());
        Ok(())
    }

    // ── numeric literals ──────────────────────────────────────────────────────

    fn number(&mut self) -> Result<()> {
        // Integer part (first digit already consumed)
        while self.peek().is_ascii_digit() || self.peek() == '_' { self.advance(); }

        // Float?
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance(); // consume '.'
            while self.peek().is_ascii_digit() || self.peek() == '_' { self.advance(); }
            // Exponent
            if self.peek() == 'e' || self.peek() == 'E' {
                self.advance();
                if self.peek() == '+' || self.peek() == '-' { self.advance(); }
                if !self.peek().is_ascii_digit() {
                    return Err(self.error("Expected digits after exponent"));
                }
                while self.peek().is_ascii_digit() { self.advance(); }
            }
            let raw: String = self.source[self.start..self.current]
                .iter().filter(|&&c| c != '_').collect();
            self.add_token_with_lexeme(TokenType::FloatLiteral, raw);
        } else {
            // Optional integer suffix (e.g. 42u64 — we strip the suffix for now)
            while self.peek().is_ascii_alphanumeric() { self.advance(); }
            let raw: String = self.source[self.start..self.current]
                .iter().filter(|&&c| c != '_' && !c.is_alphabetic()).collect();
            self.add_token_with_lexeme(TokenType::IntLiteral, raw);
        }
        Ok(())
    }

    fn hex_number(&mut self) -> Result<()> {
        if !self.peek().is_ascii_hexdigit() {
            return Err(self.error("Expected hex digits after '0x'"));
        }
        while self.peek().is_ascii_hexdigit() || self.peek() == '_' { self.advance(); }
        let raw: String = self.source[self.start + 2..self.current]
            .iter().filter(|&&c| c != '_').collect();
        let value = i64::from_str_radix(&raw, 16)
            .map_err(|_| self.error(format!("Invalid hex literal '{}'", raw)))?;
        self.add_token_with_lexeme(TokenType::IntLiteral, value.to_string());
        Ok(())
    }

    fn bin_number(&mut self) -> Result<()> {
        if self.peek() != '0' && self.peek() != '1' {
            return Err(self.error("Expected binary digits after '0b'"));
        }
        while self.peek() == '0' || self.peek() == '1' || self.peek() == '_' { self.advance(); }
        let raw: String = self.source[self.start + 2..self.current]
            .iter().filter(|&&c| c != '_').collect();
        let value = i64::from_str_radix(&raw, 2)
            .map_err(|_| self.error(format!("Invalid binary literal '{}'", raw)))?;
        self.add_token_with_lexeme(TokenType::IntLiteral, value.to_string());
        Ok(())
    }

    // ── identifiers / keywords ────────────────────────────────────────────────

    fn identifier(&mut self) {
        while self.peek().is_alphanumeric() || self.peek() == '_' { self.advance(); }
        let text: String = self.source[self.start..self.current].iter().collect();
        let tt = keyword_or_ident(&text);
        self.add_token(tt);
    }
}

fn keyword_or_ident(text: &str) -> TokenType {
    match text {
        "let"       => TokenType::Let,
        "mut"       => TokenType::Mut,
        "fn"        => TokenType::Fn,
        "if"        => TokenType::If,
        "else"      => TokenType::Else,
        "while"     => TokenType::While,
        "for"       => TokenType::For,
        "in"        => TokenType::In,
        "match"     => TokenType::Match,
        "return"    => TokenType::Return,
        "break"     => TokenType::Break,
        "continue"  => TokenType::Continue,
        "struct"    => TokenType::Struct,
        "enum"      => TokenType::Enum,
        "import"    => TokenType::Import,
        "from"      => TokenType::From,
        "as"        => TokenType::As,
        "pub"       => TokenType::Pub,
        "true"      => TokenType::True,
        "false"     => TokenType::False,
        "none"      => TokenType::None,
        "async"     => TokenType::Async,
        "await"     => TokenType::Await,
        "try"       => TokenType::Try,
        "catch"     => TokenType::Catch,
        "finally"   => TokenType::Finally,
        "static"    => TokenType::Static,
        "type"      => TokenType::Type,
        "trait"     => TokenType::Trait,
        "impl"      => TokenType::Impl,
        "self"      => TokenType::Self_,
        "this"      => TokenType::This,
        "use"       => TokenType::Use,
        "const"     => TokenType::Const,
        "loop"      => TokenType::Loop,
        "where"     => TokenType::Where,
        "mod"       => TokenType::Mod,
        "extern"    => TokenType::Extern,
        "unsafe"    => TokenType::Unsafe,
        "ref"       => TokenType::Ref,
        "box"       => TokenType::Box_,
        "is"        => TokenType::Is,
        "int"       => TokenType::Int,
        "float"     => TokenType::Float,
        "bool"      => TokenType::Bool,
        "string"    => TokenType::String_,
        "char"      => TokenType::Char_,
        "void"      => TokenType::Void,
        _           => TokenType::Identifier,
    }
}
