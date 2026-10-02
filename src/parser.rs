use crate::ast::*;
use crate::error::{BruteError, Result};
use crate::lexer::{Token, TokenType};

// ─────────────────────────────────────────────────────────────────────────────
// Parser struct
// ─────────────────────────────────────────────────────────────────────────────
pub struct Parser {
    tokens:  Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, current: 0 }
    }

    // ── public entry ──────────────────────────────────────────────────────────

    pub fn parse(&mut self) -> Result<Program> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        Ok(Program::new(stmts))
    }

    // ── token helpers ─────────────────────────────────────────────────────────

    fn peek(&self) -> &Token { &self.tokens[self.current] }
    fn previous(&self) -> &Token { &self.tokens[self.current - 1] }
    fn is_at_end(&self) -> bool { self.peek().token_type == TokenType::EOF }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() { self.current += 1; }
        self.previous()
    }

    fn check(&self, tt: &TokenType) -> bool {
        !self.is_at_end() && &self.peek().token_type == tt
    }

    fn check2(&self, tt: &TokenType) -> bool {
        if self.current + 1 >= self.tokens.len() { return false; }
        &self.tokens[self.current + 1].token_type == tt
    }

    fn match_tok(&mut self, types: &[TokenType]) -> bool {
        for tt in types {
            if self.check(tt) { self.advance(); return true; }
        }
        false
    }

    fn consume(&mut self, tt: TokenType, msg: &str) -> Result<&Token> {
        if self.check(&tt) { return Ok(self.advance()); }
        Err(self.error(msg))
    }

    fn error(&self, msg: &str) -> BruteError {
        let tok = self.peek();
        BruteError::syntax(tok.line, tok.column,
            format!("{} (got '{}')", msg, tok.lexeme))
    }

    fn error_prev(&self, msg: &str) -> BruteError {
        let tok = self.previous();
        BruteError::syntax(tok.line, tok.column, msg)
    }

    // ── optional semicolon ────────────────────────────────────────────────────
    fn optional_semi(&mut self) { self.match_tok(&[TokenType::Semicolon]); }

    // ═════════════════════════════════════════════════════════════════════════
    // TYPE PARSING
    // ═════════════════════════════════════════════════════════════════════════

    fn parse_type(&mut self) -> Result<Type> {
        // &T  /  &mut T
        if self.match_tok(&[TokenType::Ampersand]) {
            let mutable = self.match_tok(&[TokenType::Mut]);
            let inner = self.parse_type()?;
            return Ok(Type::Reference(Box::new(inner), mutable));
        }
        // [T] or [T; N]
        if self.match_tok(&[TokenType::LeftBracket]) {
            let inner = self.parse_type()?;
            if self.match_tok(&[TokenType::Semicolon]) {
                let n = self.consume(TokenType::IntLiteral, "Expected array size")?;
                let size: usize = n.lexeme.parse().map_err(|_| self.error_prev("Invalid array size"))?;
                self.consume(TokenType::RightBracket, "Expected ']'")?;
                return Ok(Type::Array(Box::new(inner), Some(size)));
            }
            self.consume(TokenType::RightBracket, "Expected ']'")?;
            return Ok(Type::List(Box::new(inner)));
        }
        // (T, U, …)  — tuple
        if self.match_tok(&[TokenType::LeftParen]) {
            if self.match_tok(&[TokenType::RightParen]) {
                return Ok(Type::Tuple(vec![]));
            }
            let mut types = vec![self.parse_type()?];
            while self.match_tok(&[TokenType::Comma]) {
                if self.check(&TokenType::RightParen) { break; }
                types.push(self.parse_type()?);
            }
            self.consume(TokenType::RightParen, "Expected ')' after tuple types")?;
            if types.len() == 1 { return Ok(types.remove(0)); }
            return Ok(Type::Tuple(types));
        }
        // fn(T) -> U
        if self.match_tok(&[TokenType::Fn]) {
            self.consume(TokenType::LeftParen, "Expected '(' after 'fn' in type")?;
            let mut params = Vec::new();
            while !self.check(&TokenType::RightParen) {
                params.push(self.parse_type()?);
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::RightParen, "Expected ')'")?;
            let ret = if self.match_tok(&[TokenType::Arrow]) {
                self.parse_type()?
            } else { Type::Void };
            return Ok(Type::Function(params, Box::new(ret)));
        }

        // Named type  (possibly generic)
        let base = self.parse_named_type()?;
        // Trailing ?  => Option<T>
        if self.match_tok(&[TokenType::Question]) {
            return Ok(Type::Option(Box::new(base)));
        }
        Ok(base)
    }

    fn parse_named_type(&mut self) -> Result<Type> {
        let name = match self.peek().token_type {
            TokenType::Int      => { self.advance(); return Ok(Type::Int); }
            TokenType::Float    => { self.advance(); return Ok(Type::Float); }
            TokenType::Bool     => { self.advance(); return Ok(Type::Bool); }
            TokenType::String_  => { self.advance(); return Ok(Type::String); }
            TokenType::Char_    => { self.advance(); return Ok(Type::Char); }
            TokenType::Void     => { self.advance(); return Ok(Type::Void); }
            TokenType::Self_    => { self.advance(); return Ok(Type::SelfType); }
            TokenType::Identifier => {
                let n = self.advance().lexeme.clone();
                n
            }
            _ => return Err(self.error("Expected type")),
        };

        // Generic args  Name<T, U>
        if self.match_tok(&[TokenType::Less]) {
            let mut args = Vec::new();
            while !self.check(&TokenType::Greater) && !self.is_at_end() {
                args.push(self.parse_type()?);
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::Greater, "Expected '>' after generic args")?;
            // Special-case well-known generic types
            return Ok(match name.as_str() {
                "Option" if args.len() == 1 => Type::Option(Box::new(args.remove(0))),
                "Result" if args.len() == 2 => {
                    let err = args.remove(1); let ok = args.remove(0);
                    Type::Result(Box::new(ok), Box::new(err))
                }
                "Future" if args.len() == 1 => Type::Future(Box::new(args.remove(0))),
                _ => Type::Generic(name, args),
            });
        }
        Ok(Type::Custom(name))
    }

    // ═════════════════════════════════════════════════════════════════════════
    // GENERIC PARAM LIST  <T, U, ...>
    // ═════════════════════════════════════════════════════════════════════════

    fn parse_generic_params(&mut self) -> Vec<String> {
        if !self.check(&TokenType::Less) { return vec![]; }
        self.advance(); // consume '<'
        let mut params = Vec::new();
        while !self.check(&TokenType::Greater) && !self.is_at_end() {
            if let TokenType::Identifier = self.peek().token_type {
                params.push(self.advance().lexeme.clone());
            }
            // Optional trait bounds:  T: ToString + Container
            if self.match_tok(&[TokenType::Colon]) {
                while !self.check(&TokenType::Comma)
                    && !self.check(&TokenType::Greater)
                    && !self.is_at_end()
                {
                    self.advance();
                }
            }
            if !self.match_tok(&[TokenType::Comma]) { break; }
        }
        let _ = self.consume(TokenType::Greater, "Expected '>' after generic params");
        params
    }

    /// Skip a `<...>` generic argument list (turbofish), e.g. in
    /// `Queue::<string>::new()` or `Collection::create<int>()`.
    /// Call only when the current token is `<`.
    fn skip_generic_args(&mut self) {
        if !self.check(&TokenType::Less) { return; }
        let mut depth = 0i32;
        while !self.is_at_end() {
            match self.peek().token_type {
                TokenType::Less       => depth += 1,
                TokenType::Greater    => depth -= 1,
                TokenType::RightShift => depth -= 2, // `>>` inside nested generics
                _ => {}
            }
            self.advance();
            if depth <= 0 { break; }
        }
    }

    /// `true` when the current `<` begins a generic argument list that is
    /// followed by `(` or `::` (a turbofish-style call), not a comparison.
    fn is_turbofish(&self) -> bool {
        if !self.check(&TokenType::Less) { return false; }
        let mut i = self.current;
        let mut depth = 0i32;
        loop {
            let Some(t) = self.tokens.get(i) else { return false };
            match t.token_type {
                TokenType::Less       => depth += 1,
                TokenType::RightShift => depth -= 2,
                TokenType::Greater    => {
                    depth -= 1;
                    if depth <= 0 {
                        return matches!(
                            self.tokens.get(i + 1).map(|t| &t.token_type),
                            Some(TokenType::LeftParen)
                                | Some(TokenType::DoubleColon)
                                | Some(TokenType::Dot)
                        );
                    }
                }
                TokenType::Identifier | TokenType::Comma
                | TokenType::Int | TokenType::Float | TokenType::Bool
                | TokenType::String_ | TokenType::Char_ | TokenType::Void
                | TokenType::Question | TokenType::Ampersand | TokenType::Mut
                | TokenType::LeftBracket | TokenType::RightBracket => {}
                _ => return false,
            }
            i += 1;
        }
    }

    // ═════════════════════════════════════════════════════════════════════════
    // DECLARATIONS
    // ═════════════════════════════════════════════════════════════════════════

    fn declaration(&mut self) -> Result<Stmt> {
        // pub …
        let is_public = self.match_tok(&[TokenType::Pub]);

        if self.match_tok(&[TokenType::Fn]) {
            return self.function_decl(is_public, false);
        }
        if self.match_tok(&[TokenType::Async]) {
            self.consume(TokenType::Fn, "Expected 'fn' after 'async'")?;
            return self.function_decl(is_public, true);
        }
        if self.match_tok(&[TokenType::Struct]) {
            return self.struct_decl(is_public);
        }
        if self.match_tok(&[TokenType::Enum]) {
            return self.enum_decl(is_public);
        }
        if self.match_tok(&[TokenType::Trait]) {
            return self.trait_decl(is_public);
        }
        if self.match_tok(&[TokenType::Impl]) {
            if is_public { return Err(self.error("'impl' cannot be public")); }
            return self.impl_decl();
        }
        if self.match_tok(&[TokenType::Const]) {
            return self.const_decl(is_public);
        }
        if self.match_tok(&[TokenType::Type]) {
            return self.type_alias_decl(is_public);
        }
        if self.match_tok(&[TokenType::Import]) {
            if is_public { return Err(self.error("'import' cannot be public")); }
            return self.import_decl();
        }
        if self.match_tok(&[TokenType::Use]) {
            if is_public { return Err(self.error("'use' cannot be public")); }
            return self.use_decl();
        }
        if is_public {
            return Err(self.error("'pub' must precede fn/struct/enum/trait/const/type"));
        }
        self.statement()
    }

    // ── function ──────────────────────────────────────────────────────────────

    fn function_decl(&mut self, is_public: bool, is_async: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected function name")?.lexeme.clone();
        let generic_params = self.parse_generic_params();

        self.consume(TokenType::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_params()?;
        self.consume(TokenType::RightParen, "Expected ')' after parameters")?;

        let return_type = if self.match_tok(&[TokenType::Arrow]) {
            self.parse_type()?
        } else { Type::Void };

        // Optional `where T: Trait` clause before the body
        if self.match_tok(&[TokenType::Where]) {
            while !self.check(&TokenType::LeftBrace) && !self.is_at_end() {
                self.advance();
            }
        }

        self.consume(TokenType::LeftBrace, "Expected '{' before function body")?;
        let body = self.block()?;

        Ok(Stmt::Function { name, generic_params, params, return_type, body, is_async, is_public })
    }

    fn parse_params(&mut self) -> Result<Vec<Param>> {
        let mut params = Vec::new();
        if self.check(&TokenType::RightParen) { return Ok(params); }
        loop {
            // `mut` marker on a parameter (mutability is dynamic — ignored)
            let _is_mut = self.match_tok(&[TokenType::Mut]);
            // self / this parameter
            if self.match_tok(&[TokenType::Self_]) || self.match_tok(&[TokenType::This]) {
                params.push(Param { name: "self".into(), ty: Type::SelfType, default: None });
            } else {
                let name = self.consume(TokenType::Identifier, "Expected parameter name")?.lexeme.clone();
                self.consume(TokenType::Colon, "Expected ':' after parameter name")?;
                let ty = self.parse_type()?;
                let default = if self.match_tok(&[TokenType::Equal]) {
                    Some(self.expression()?)
                } else { None };
                params.push(Param { name, ty, default });
            }
            if !self.match_tok(&[TokenType::Comma]) { break; }
            if self.check(&TokenType::RightParen) { break; }
        }
        Ok(params)
    }

    // ── struct ────────────────────────────────────────────────────────────────

    fn struct_decl(&mut self, is_public: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected struct name")?.lexeme.clone();
        let generic_params = self.parse_generic_params();
        self.consume(TokenType::LeftBrace, "Expected '{' after struct name")?;

        let mut fields  = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let field_pub = self.match_tok(&[TokenType::Pub]);
            let _is_static = self.match_tok(&[TokenType::Static]);
            if self.match_tok(&[TokenType::Fn]) {
                methods.push(self.function_decl(field_pub, false)?);
            } else if self.match_tok(&[TokenType::Async]) {
                self.consume(TokenType::Fn, "Expected 'fn'")?;
                methods.push(self.function_decl(field_pub, true)?);
            } else {
                let fname = self.consume(TokenType::Identifier, "Expected field name")?.lexeme.clone();
                self.consume(TokenType::Colon, "Expected ':' after field name")?;
                let ty = self.parse_type()?;
                self.match_tok(&[TokenType::Semicolon, TokenType::Comma]);
                fields.push(StructField { name: fname, ty, is_public: field_pub });
            }
        }
        self.consume(TokenType::RightBrace, "Expected '}' after struct body")?;
        Ok(Stmt::Struct { name, generic_params, fields, methods, is_public })
    }

    // ── enum ──────────────────────────────────────────────────────────────────

    fn enum_decl(&mut self, is_public: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected enum name")?.lexeme.clone();
        let generic_params = self.parse_generic_params();
        self.consume(TokenType::LeftBrace, "Expected '{' after enum name")?;

        let mut variants = Vec::new();
        let mut methods  = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Associated type declaration: `type Item;` / `type Item = T;` — ignored
            if self.match_tok(&[TokenType::Type]) {
                let _ = self.consume(TokenType::Identifier, "Expected associated type name");
                if self.match_tok(&[TokenType::Equal]) { let _ = self.parse_type(); }
                self.match_tok(&[TokenType::Semicolon, TokenType::Comma]);
                continue;
            }
            // Method members: `pub fn`, `static fn`, `async fn`, or plain `fn`
            let m_pub    = self.match_tok(&[TokenType::Pub]);
            let _static_ = self.match_tok(&[TokenType::Static]);
            let m_async  = self.match_tok(&[TokenType::Async]);
            if self.match_tok(&[TokenType::Fn]) {
                methods.push(self.function_decl(m_pub, m_async)?);
                self.match_tok(&[TokenType::Comma]);
                continue;
            }
            let vname = self.consume(TokenType::Identifier, "Expected variant name")?.lexeme.clone();
            let fields = if self.match_tok(&[TokenType::LeftParen]) {
                let mut types = Vec::new();
                while !self.check(&TokenType::RightParen) {
                    types.push(self.parse_type()?);
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightParen, "Expected ')'")?;
                EnumVariantFields::Tuple(types)
            } else if self.check(&TokenType::LeftBrace) {
                self.advance();
                let mut sfields = Vec::new();
                while !self.check(&TokenType::RightBrace) {
                    let fpub = self.match_tok(&[TokenType::Pub]);
                    let fn_ = self.consume(TokenType::Identifier, "Expected field name")?.lexeme.clone();
                    self.consume(TokenType::Colon, "Expected ':'")?;
                    let ty = self.parse_type()?;
                    self.match_tok(&[TokenType::Semicolon, TokenType::Comma]);
                    sfields.push(StructField { name: fn_, ty, is_public: fpub });
                }
                self.consume(TokenType::RightBrace, "Expected '}'")?;
                EnumVariantFields::Struct(sfields)
            } else {
                EnumVariantFields::Unit
            };
            variants.push(EnumVariant { name: vname, fields });
            if !self.match_tok(&[TokenType::Comma]) { break; }
        }
        self.consume(TokenType::RightBrace, "Expected '}' after enum body")?;
        Ok(Stmt::Enum { name, generic_params, variants, methods, is_public })
    }

    // ── trait ─────────────────────────────────────────────────────────────────

    fn trait_decl(&mut self, is_public: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected trait name")?.lexeme.clone();
        let generic_params = self.parse_generic_params();

        // optional super-traits: trait Foo: Bar + Baz
        let mut super_traits = Vec::new();
        if self.match_tok(&[TokenType::Colon]) {
            loop {
                super_traits.push(self.consume(TokenType::Identifier, "Expected trait name")?.lexeme.clone());
                if !self.match_tok(&[TokenType::Plus]) { break; }
            }
        }
        self.consume(TokenType::LeftBrace, "Expected '{' after trait name")?;
        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Associated type: `type ItemType;` — recorded only by name
            if self.match_tok(&[TokenType::Type]) {
                let _ = self.consume(TokenType::Identifier, "Expected associated type name");
                if self.match_tok(&[TokenType::Equal]) { let _ = self.parse_type(); }
                self.match_tok(&[TokenType::Semicolon, TokenType::Comma]);
                continue;
            }
            let _m_pub   = self.match_tok(&[TokenType::Pub]);
            let is_async = self.match_tok(&[TokenType::Async]);
            self.consume(TokenType::Fn, "Expected 'fn' in trait")?;
            let mname = self.consume(TokenType::Identifier, "Expected method name")?.lexeme.clone();
            self.consume(TokenType::LeftParen, "Expected '('")?;
            let params = self.parse_params()?;
            self.consume(TokenType::RightParen, "Expected ')'")?;
            let return_type = if self.match_tok(&[TokenType::Arrow]) { self.parse_type()? } else { Type::Void };
            let body = if self.match_tok(&[TokenType::LeftBrace]) {
                Some(self.block()?)
            } else {
                self.consume(TokenType::Semicolon, "Expected ';' or '{' after trait method signature")?;
                None
            };
            methods.push(TraitMethod { name: mname, params, return_type, body, is_async });
        }
        self.consume(TokenType::RightBrace, "Expected '}' after trait body")?;
        Ok(Stmt::Trait { name, generic_params, super_traits, methods, is_public })
    }

    // ── impl ──────────────────────────────────────────────────────────────────

    fn impl_decl(&mut self) -> Result<Stmt> {
        let generic_params = self.parse_generic_params();

        // impl Trait for Type  OR  impl Type — both names may carry generic
        // arguments (`impl Printable<int> for Box<int>`), which we keep in the
        // type name so specialised impls like `Box<int>::method` resolve.
        let first = self.consume(TokenType::Identifier, "Expected type or trait name")?
            .lexeme.clone();
        let first_args = self.generic_args_string();

        let (trait_name, type_name) = if self.match_tok(&[TokenType::For]) {
            let ty = self.consume(TokenType::Identifier, "Expected type name after 'for'")?
                .lexeme.clone();
            let ty_args = self.generic_args_string();
            (Some(first), format!("{}{}", ty, ty_args))
        } else {
            (None, format!("{}{}", first, first_args))
        };

        self.consume(TokenType::LeftBrace, "Expected '{' after type name")?;
        let mut methods = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Associated type assignment: `type ItemType = T;` — ignored
            if self.match_tok(&[TokenType::Type]) {
                let _ = self.consume(TokenType::Identifier, "Expected associated type name");
                if self.match_tok(&[TokenType::Equal]) { let _ = self.parse_type(); }
                self.match_tok(&[TokenType::Semicolon, TokenType::Comma]);
                continue;
            }
            let is_pub    = self.match_tok(&[TokenType::Pub]);
            let _is_static = self.match_tok(&[TokenType::Static]);
            let is_async  = self.match_tok(&[TokenType::Async]);
            self.consume(TokenType::Fn, "Expected 'fn' in impl")?;
            methods.push(self.function_decl(is_pub, is_async)?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after impl body")?;
        Ok(Stmt::Impl { generic_params, trait_name, type_name, methods })
    }

    /// Consume a `<T, U>` generic-argument list and return its normalised
    /// text (`<int,string>`), or an empty string when absent.
    fn generic_args_string(&mut self) -> String {
        if !self.check(&TokenType::Less) { return String::new(); }
        let mut depth = 0i32;
        let mut s = String::new();
        while !self.is_at_end() {
            let t = &self.peek().token_type;
            let lex = self.peek().lexeme.clone();
            match t {
                TokenType::Less       => { depth += 1; s.push('<'); }
                TokenType::RightShift => { depth -= 2; s.push_str(">>"); }
                TokenType::Greater    => { depth -= 1; s.push('>'); }
                TokenType::Comma      => s.push(','),
                _                     => s.push_str(&lex),
            }
            self.advance();
            if depth <= 0 { break; }
        }
        s
    }

    // ── const / type alias / import / use ─────────────────────────────────────

    fn const_decl(&mut self, is_public: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected constant name")?.lexeme.clone();
        self.consume(TokenType::Colon, "Expected ':' after constant name")?;
        let type_hint = self.parse_type()?;
        self.consume(TokenType::Equal, "Expected '=' after type")?;
        let value = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after constant")?;
        Ok(Stmt::Const { name, type_hint, value, is_public })
    }

    fn type_alias_decl(&mut self, is_public: bool) -> Result<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expected type name")?.lexeme.clone();
        let generic_params = self.parse_generic_params();
        self.consume(TokenType::Equal, "Expected '=' after type name")?;
        let alias_type = self.parse_type()?;
        self.consume(TokenType::Semicolon, "Expected ';' after type alias")?;
        Ok(Stmt::TypeAlias { name, generic_params, alias_type, is_public })
    }

    fn import_decl(&mut self) -> Result<Stmt> {
        // import { A, B as C } from "path"
        // import "path"
        if self.match_tok(&[TokenType::LeftBrace]) {
            let mut items = Vec::new();
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if self.match_tok(&[TokenType::Star]) {
                    items.push(ImportItem::All);
                } else {
                    let orig = self.consume(TokenType::Identifier, "Expected import name")?.lexeme.clone();
                    if self.match_tok(&[TokenType::As]) {
                        let alias = self.consume(TokenType::Identifier, "Expected alias name")?.lexeme.clone();
                        items.push(ImportItem::Alias(orig, alias));
                    } else {
                        items.push(ImportItem::Name(orig));
                    }
                }
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::RightBrace, "Expected '}'")?;
            self.consume(TokenType::From, "Expected 'from' after import list")?;
            let path = self.consume(TokenType::StringLiteral, "Expected module path string")?.lexeme.clone();
            self.optional_semi();
            return Ok(Stmt::Import { path, items });
        }
        // bare: import "path"
        let path = self.consume(TokenType::StringLiteral, "Expected module path string")?.lexeme.clone();
        self.optional_semi();
        Ok(Stmt::Import { path, items: vec![ImportItem::All] })
    }

    fn use_decl(&mut self) -> Result<Stmt> {
        let mut path = self.consume(TokenType::Identifier, "Expected module path")?.lexeme.clone();
        while self.match_tok(&[TokenType::DoubleColon]) || self.match_tok(&[TokenType::Dot]) {
            // Braced import list:  use a::b::{x, y as z}
            if self.check(&TokenType::LeftBrace) {
                self.advance();
                let mut items = Vec::new();
                while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                    if self.match_tok(&[TokenType::Star]) {
                        items.push(ImportItem::All);
                    } else {
                        let orig = self.consume(TokenType::Identifier, "Expected import name")?.lexeme.clone();
                        if self.match_tok(&[TokenType::As]) {
                            let alias = self.consume(TokenType::Identifier, "Expected alias name")?.lexeme.clone();
                            items.push(ImportItem::Alias(orig, alias));
                        } else {
                            items.push(ImportItem::Name(orig));
                        }
                    }
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightBrace, "Expected '}' after use list")?;
                self.optional_semi();
                return Ok(Stmt::Import { path, items });
            }
            // Glob import:  use a::b::*
            if self.match_tok(&[TokenType::Star]) {
                self.optional_semi();
                return Ok(Stmt::Import { path, items: vec![ImportItem::All] });
            }
            let seg = self.consume(TokenType::Identifier, "Expected identifier after '::'")?;
            path.push_str("::");
            path.push_str(&seg.lexeme);
        }
        let as_name = if self.match_tok(&[TokenType::As]) {
            Some(self.consume(TokenType::Identifier, "Expected alias name")?.lexeme.clone())
        } else { None };
        self.optional_semi();
        Ok(Stmt::Use { path, as_name })
    }

    // ═════════════════════════════════════════════════════════════════════════
    // STATEMENTS
    // ═════════════════════════════════════════════════════════════════════════

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_tok(&[TokenType::Let])      { return self.let_stmt(); }
        if self.match_tok(&[TokenType::Return])   { return self.return_stmt(); }
        if self.match_tok(&[TokenType::Break])    { return self.break_stmt(); }
        if self.match_tok(&[TokenType::Continue]) {
            self.optional_semi();
            return Ok(Stmt::Continue);
        }
        if self.match_tok(&[TokenType::If])       { return self.if_stmt(); }
        if self.match_tok(&[TokenType::While])    { return self.while_stmt(); }
        if self.match_tok(&[TokenType::For])      { return self.for_stmt(); }
        if self.match_tok(&[TokenType::Loop])     { return self.loop_stmt(); }
        if self.match_tok(&[TokenType::Match])    { return self.match_stmt(); }
        if self.match_tok(&[TokenType::Try])      { return self.try_stmt(); }
        if self.match_tok(&[TokenType::Async])    {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'async'")?;
            let body = self.block()?;
            return Ok(Stmt::Async { block: body });
        }
        // expression statement (including assignments)
        let expr = self.expression()?;
        self.optional_semi();
        Ok(Stmt::Expr(expr))
    }

    fn let_stmt(&mut self) -> Result<Stmt> {
        let mutable = self.match_tok(&[TokenType::Mut]);
        let name = self.consume(TokenType::Identifier, "Expected variable name")?.lexeme.clone();
        let type_hint = if self.match_tok(&[TokenType::Colon]) {
            Some(self.parse_type()?)
        } else { None };
        self.consume(TokenType::Equal, "Expected '=' in let binding")?;
        let value = self.expression()?;
        self.optional_semi();
        Ok(Stmt::Let { name, type_hint, value, mutable })
    }

    fn return_stmt(&mut self) -> Result<Stmt> {
        let value = if !self.check(&TokenType::Semicolon) && !self.check(&TokenType::RightBrace) {
            Some(self.expression()?)
        } else { None };
        self.optional_semi();
        Ok(Stmt::Return(value))
    }

    fn break_stmt(&mut self) -> Result<Stmt> {
        let value = if !self.check(&TokenType::Semicolon) && !self.check(&TokenType::RightBrace) {
            Some(self.expression()?)
        } else { None };
        self.optional_semi();
        Ok(Stmt::Break(value))
    }

    fn if_stmt(&mut self) -> Result<Stmt> {
        // Parentheses around condition are optional (Rust-style without, C-style with)
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let condition = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')' after condition")?; }

        self.consume(TokenType::LeftBrace, "Expected '{' before if body")?;
        let then_block = self.block()?;

        let else_block = if self.match_tok(&[TokenType::Else]) {
            if self.match_tok(&[TokenType::If]) {
                Some(vec![self.if_stmt()?])
            } else {
                self.consume(TokenType::LeftBrace, "Expected '{' before else body")?;
                Some(self.block()?)
            }
        } else { None };

        Ok(Stmt::If { condition, then_block, else_block })
    }

    fn while_stmt(&mut self) -> Result<Stmt> {
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let condition = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')'")?; }
        self.consume(TokenType::LeftBrace, "Expected '{' before while body")?;
        let body = self.block()?;
        Ok(Stmt::While { condition, body, label: None })
    }

    fn for_stmt(&mut self) -> Result<Stmt> {
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let var = self.consume(TokenType::Identifier, "Expected variable name in for loop")?.lexeme.clone();
        self.consume(TokenType::In, "Expected 'in' after variable")?;
        let iterator = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')'")?; }
        self.consume(TokenType::LeftBrace, "Expected '{' before for body")?;
        let body = self.block()?;
        Ok(Stmt::For { var, iterator, body, label: None })
    }

    fn loop_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftBrace, "Expected '{' before loop body")?;
        let body = self.block()?;
        Ok(Stmt::Loop { body, label: None })
    }

    fn match_stmt(&mut self) -> Result<Stmt> {
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let expr = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')'")?; }
        self.consume(TokenType::LeftBrace, "Expected '{' before match body")?;

        let mut arms = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            let guard = if self.match_tok(&[TokenType::If]) {
                Some(self.expression()?)
            } else { None };
            self.consume(TokenType::FatArrow, "Expected '=>' after pattern")?;
            let body = if self.match_tok(&[TokenType::LeftBrace]) {
                self.block()?
            } else if self.match_tok(&[TokenType::Return]) {
                // `pat => return expr` — early return from the enclosing fn
                let value = if self.check(&TokenType::Comma)
                    || self.check(&TokenType::RightBrace) {
                    None
                } else {
                    Some(self.expression()?)
                };
                vec![Stmt::Return(value)]
            } else {
                vec![Stmt::Expr(self.expression()?)]
            };
            arms.push((pattern, guard, body));
            self.match_tok(&[TokenType::Comma, TokenType::Semicolon]);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after match body")?;
        Ok(Stmt::Match { expr, arms })
    }

    fn try_stmt(&mut self) -> Result<Stmt> {
        self.consume(TokenType::LeftBrace, "Expected '{' after 'try'")?;
        let block = self.block()?;
        let mut catch_blocks = Vec::new();
        while self.match_tok(&[TokenType::Catch]) {
            let (binding, error_type) = if self.check(&TokenType::LeftParen) {
                self.advance();
                let name = self.consume(TokenType::Identifier, "Expected catch variable")?.lexeme.clone();
                let ety = if self.match_tok(&[TokenType::Colon]) { Some(self.parse_type()?) } else { None };
                self.consume(TokenType::RightParen, "Expected ')'")?;
                (Some(name), ety)
            } else if self.check(&TokenType::Identifier) {
                let name = self.advance().lexeme.clone();
                (Some(name), None)
            } else { (None, None) };
            self.consume(TokenType::LeftBrace, "Expected '{' after catch")?;
            let body = self.block()?;
            catch_blocks.push(CatchBlock { binding, error_type, body });
        }
        let finally_block = if self.match_tok(&[TokenType::Finally]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after 'finally'")?;
            Some(self.block()?)
        } else { None };
        Ok(Stmt::Try { block, catch_blocks, finally_block })
    }

    // ── block helper ──────────────────────────────────────────────────────────

    fn block(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            stmts.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after block")?;
        Ok(stmts)
    }

    // ═════════════════════════════════════════════════════════════════════════
    // EXPRESSIONS  (Pratt-style precedence climbing)
    // ═════════════════════════════════════════════════════════════════════════

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        // Bare arrow lambda `x => body` — only when the identifier directly
        // leads the expression, so `x == y => arm` in a match guard is not
        // mistaken for one.
        if self.check(&TokenType::Identifier) && self.check2(&TokenType::FatArrow) {
            let name = self.advance().lexeme.clone();
            self.advance(); // =>
            return Ok(Expr::Lambda {
                params: vec![(name, None)],
                return_type: None,
                body: Box::new(self.expression()?),
            });
        }
        let expr = self.null_coalesce()?;

        // Compound assignments: +=  -=  *=  /=  %=  **=  &=  |=  ^=
        let compound_op = match self.peek().token_type {
            TokenType::PlusEqual       => Some(CompoundOp::Add),
            TokenType::MinusEqual      => Some(CompoundOp::Sub),
            TokenType::StarEqual       => Some(CompoundOp::Mul),
            TokenType::SlashEqual      => Some(CompoundOp::Div),
            TokenType::PercentEqual    => Some(CompoundOp::Mod),
            TokenType::StarStarEqual   => Some(CompoundOp::Pow),
            TokenType::AndEqual        => Some(CompoundOp::BitAnd),
            TokenType::OrEqual         => Some(CompoundOp::BitOr),
            TokenType::XorEqual        => Some(CompoundOp::BitXor),
            _ => None,
        };
        if let Some(op) = compound_op {
            self.advance();
            let value = self.assignment()?;
            return Ok(Expr::CompoundAssign {
                target: Box::new(expr),
                op,
                value:  Box::new(value),
            });
        }

        // Simple assignment: =
        if self.match_tok(&[TokenType::Equal]) {
            let value = self.assignment()?;
            return Ok(Expr::Assign { target: Box::new(expr), value: Box::new(value) });
        }

        Ok(expr)
    }

    fn null_coalesce(&mut self) -> Result<Expr> {
        let mut expr = self.pipeline()?;
        while self.match_tok(&[TokenType::QuestionQuestion]) {
            let right = self.pipeline()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::NullCoalesce, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn pipeline(&mut self) -> Result<Expr> {
        let mut expr = self.logical_or()?;
        while self.match_tok(&[TokenType::Pipeline]) {
            let right = self.logical_or()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::Pipeline, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn logical_or(&mut self) -> Result<Expr> {
        let mut expr = self.logical_and()?;
        while self.match_tok(&[TokenType::Or]) {
            let right = self.logical_and()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::Or, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Result<Expr> {
        let mut expr = self.bitwise_or()?;
        while self.match_tok(&[TokenType::And]) {
            let right = self.bitwise_or()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::And, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn bitwise_or(&mut self) -> Result<Expr> {
        let mut expr = self.bitwise_xor()?;
        while self.match_tok(&[TokenType::Pipe]) {
            let right = self.bitwise_xor()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::BitOr, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn bitwise_xor(&mut self) -> Result<Expr> {
        let mut expr = self.bitwise_and()?;
        while self.match_tok(&[TokenType::Caret]) {
            let right = self.bitwise_and()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::BitXor, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn bitwise_and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;
        while self.match_tok(&[TokenType::Ampersand]) {
            let right = self.equality()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op: BinOp::BitAnd, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        loop {
            let op = match self.peek().token_type {
                TokenType::EqualEqual => BinOp::Eq,
                TokenType::NotEqual   => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.comparison()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.shift()?;
        loop {
            // `expr is TypeName` — type test; the right side is a type name
            if self.check(&TokenType::Is) {
                self.advance();
                let tname = match self.peek().token_type {
                    TokenType::Identifier => self.advance().lexeme.clone(),
                    TokenType::Int      => { self.advance(); "int".to_string() }
                    TokenType::Float    => { self.advance(); "float".to_string() }
                    TokenType::Bool     => { self.advance(); "bool".to_string() }
                    TokenType::String_  => { self.advance(); "string".to_string() }
                    TokenType::Char_    => { self.advance(); "char".to_string() }
                    TokenType::Void     => { self.advance(); "void".to_string() }
                    TokenType::None     => { self.advance(); "none".to_string() }
                    _ => return Err(self.error("Expected type name after 'is'")),
                };
                expr = Expr::BinaryOp {
                    left:  Box::new(expr),
                    op:    BinOp::Is,
                    right: Box::new(Expr::Identifier(tname)),
                };
                continue;
            }
            let op = match self.peek().token_type {
                TokenType::Greater      => BinOp::Gt,
                TokenType::GreaterEqual => BinOp::Ge,
                TokenType::Less         => BinOp::Lt,
                TokenType::LessEqual    => BinOp::Le,
                _ => break,
            };
            self.advance();
            let right = self.shift()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Result<Expr> {
        let mut expr = self.range()?;
        loop {
            let op = match self.peek().token_type {
                TokenType::LeftShift  => BinOp::Shl,
                TokenType::RightShift => BinOp::Shr,
                _ => break,
            };
            self.advance();
            let right = self.range()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn range(&mut self) -> Result<Expr> {
        let expr = self.term()?;
        if self.match_tok(&[TokenType::DotDotEqual]) {
            let end = self.term()?;
            return Ok(Expr::BinaryOp { left: Box::new(expr), op: BinOp::RangeInclusive, right: Box::new(end) });
        }
        if self.match_tok(&[TokenType::DotDot]) {
            let end = self.term()?;
            return Ok(Expr::BinaryOp { left: Box::new(expr), op: BinOp::Range, right: Box::new(end) });
        }
        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        loop {
            let op = match self.peek().token_type {
                TokenType::Plus  => BinOp::Add,
                TokenType::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.factor()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.power()?;
        loop {
            let op = match self.peek().token_type {
                TokenType::Star    => BinOp::Mul,
                TokenType::Slash   => BinOp::Div,
                TokenType::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.power()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right) };
        }
        Ok(expr)
    }

    fn power(&mut self) -> Result<Expr> {
        let base = self.unary()?;
        if self.match_tok(&[TokenType::StarStar]) {
            let exp = self.power()?; // right-associative
            return Ok(Expr::BinaryOp { left: Box::new(base), op: BinOp::Pow, right: Box::new(exp) });
        }
        Ok(base)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_tok(&[TokenType::Not]) {
            let e = self.unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOp::Not, expr: Box::new(e) });
        }
        if self.match_tok(&[TokenType::Minus]) {
            let e = self.unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOp::Neg, expr: Box::new(e) });
        }
        if self.match_tok(&[TokenType::Tilde]) {
            let e = self.unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOp::BitNot, expr: Box::new(e) });
        }
        // `*expr` — dereference (mutex/arc guard → inner value, else identity)
        if self.match_tok(&[TokenType::Star]) {
            let e = self.unary()?;
            return Ok(Expr::UnaryOp { op: UnaryOp::Deref, expr: Box::new(e) });
        }
        // await expr
        if self.match_tok(&[TokenType::Await]) {
            let e = self.unary()?;
            return Ok(Expr::Await { expr: Box::new(e) });
        }
        self.postfix()
    }

    // ── postfix: calls, indexing, field access, ? ─────────────────────────────

    fn postfix(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.match_tok(&[TokenType::LeftParen]) {
                let args = self.arg_list()?;
                expr = Expr::Call { func: Box::new(expr), args };
            } else if self.match_tok(&[TokenType::LeftBracket]) {
                let index = self.expression()?;
                self.consume(TokenType::RightBracket, "Expected ']'")?;
                expr = Expr::Index { target: Box::new(expr), index: Box::new(index) };
            } else if self.match_tok(&[TokenType::Dot]) {
                let field = self.consume(TokenType::Identifier, "Expected field or method name")?.lexeme.clone();
                if self.match_tok(&[TokenType::LeftParen]) {
                    let args = self.arg_list()?;
                    expr = Expr::MethodCall { object: Box::new(expr), method: field, args };
                } else {
                    expr = Expr::FieldAccess { object: Box::new(expr), field };
                }
            } else if self.check(&TokenType::DoubleColon) {
                // `expr::name` — namespace-style member access on dicts/objects
                self.advance();
                // Turbofish on the segment:  `expr::<T>` or `expr::name<T>`
                if self.check(&TokenType::Less) { self.skip_generic_args(); }
                let field = self.consume(TokenType::Identifier, "Expected name after '::'")?.lexeme.clone();
                if self.is_turbofish() { self.skip_generic_args(); }
                if self.match_tok(&[TokenType::LeftParen]) {
                    let args = self.arg_list()?;
                    expr = Expr::MethodCall { object: Box::new(expr), method: field, args };
                } else {
                    expr = Expr::FieldAccess { object: Box::new(expr), field };
                }
            } else if self.is_turbofish() {
                // `name<T>(args)` — generic call; the type args are erased
                self.skip_generic_args();
                if self.check(&TokenType::LeftParen) {
                    self.advance();
                    let args = self.arg_list()?;
                    expr = Expr::Call { func: Box::new(expr), args };
                }
            } else if self.match_tok(&[TokenType::Question]) {
                expr = Expr::Try { expr: Box::new(expr) };
            } else if self.match_tok(&[TokenType::As]) {
                let target_type = self.parse_type()?;
                expr = Expr::TypeCast { expr: Box::new(expr), target_type };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn arg_list(&mut self) -> Result<Vec<Expr>> {
        let mut args = Vec::new();
        while !self.check(&TokenType::RightParen) && !self.is_at_end() {
            args.push(self.expression()?);
            if !self.match_tok(&[TokenType::Comma]) { break; }
        }
        self.consume(TokenType::RightParen, "Expected ')' after arguments")?;
        Ok(args)
    }

    // ── primary ───────────────────────────────────────────────────────────────

    fn primary(&mut self) -> Result<Expr> {
        // Literals
        if self.match_tok(&[TokenType::True])  { return Ok(Expr::Literal(Literal::Bool(true))); }
        if self.match_tok(&[TokenType::False]) { return Ok(Expr::Literal(Literal::Bool(false))); }
        if self.match_tok(&[TokenType::None])  { return Ok(Expr::Literal(Literal::None)); }

        if self.match_tok(&[TokenType::IntLiteral]) {
            let v: i64 = self.previous().lexeme.parse().map_err(|_| self.error_prev("Invalid integer literal"))?;
            return Ok(Expr::Literal(Literal::Int(v)));
        }
        if self.match_tok(&[TokenType::FloatLiteral]) {
            let v: f64 = self.previous().lexeme.parse().map_err(|_| self.error_prev("Invalid float literal"))?;
            return Ok(Expr::Literal(Literal::Float(v)));
        }
        if self.match_tok(&[TokenType::StringLiteral]) {
            return Ok(Expr::Literal(Literal::String(self.previous().lexeme.clone())));
        }
        if self.match_tok(&[TokenType::CharLiteral]) {
            let ch = self.previous().lexeme.chars().next().unwrap_or('\0');
            return Ok(Expr::Literal(Literal::Char(ch)));
        }

        // Block expression  { … }  or dict literal  { key: value, … }
        if self.match_tok(&[TokenType::LeftBrace]) {
            if self.is_dict_literal() {
                let mut pairs = Vec::new();
                while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                    // Bare identifier keys become string keys: `{ name: v }`
                    let key = match self.peek().token_type {
                        TokenType::Identifier | TokenType::StringLiteral => {
                            Expr::Literal(Literal::String(self.advance().lexeme.clone()))
                        }
                        _ => self.expression()?,
                    };
                    self.consume(TokenType::Colon, "Expected ':' in dict literal")?;
                    let v = self.expression()?;
                    pairs.push((key, v));
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightBrace, "Expected '}' after dict literal")?;
                return Ok(Expr::Dict(pairs));
            }
            return self.block_expr();
        }

        // Anonymous function expression:  fn(params) -> T { body }
        if self.match_tok(&[TokenType::Fn]) {
            self.consume(TokenType::LeftParen, "Expected '(' after 'fn'")?;
            let raw_params = self.parse_params()?;
            self.consume(TokenType::RightParen, "Expected ')' after 'fn' params")?;
            let return_type = if self.match_tok(&[TokenType::Arrow]) {
                Some(self.parse_type()?)
            } else { None };
            let body = if self.match_tok(&[TokenType::LeftBrace]) {
                self.block_expr()?
            } else {
                self.expression()?
            };
            return Ok(Expr::Lambda {
                params: raw_params.iter().map(|p| (p.name.clone(), Some(p.ty.clone()))).collect(),
                return_type,
                body: Box::new(body),
            });
        }

        // Grouping, tuple, or parenthesised arrow lambda:
        //   (expr)  (expr, expr)  (x => body)  (a, b) => body  () => body
        if self.match_tok(&[TokenType::LeftParen]) {
            if self.match_tok(&[TokenType::RightParen]) {
                // `() => body`
                if self.match_tok(&[TokenType::FatArrow]) {
                    return Ok(Expr::Lambda {
                        params: vec![],
                        return_type: None,
                        body: Box::new(self.expression()?),
                    });
                }
                return Ok(Expr::Tuple(vec![]));
            }
            let first = self.expression()?;
            // `(x => body)`
            if self.match_tok(&[TokenType::FatArrow]) {
                let name = match first {
                    Expr::Identifier(n) => n,
                    _ => return Err(self.error_prev("Expected parameter name before '=>'")),
                };
                let body = self.expression()?;
                self.consume(TokenType::RightParen, "Expected ')'")?;
                return Ok(Expr::Lambda {
                    params: vec![(name, None)],
                    return_type: None,
                    body: Box::new(body),
                });
            }
            if self.match_tok(&[TokenType::Comma]) {
                let mut elems = vec![first];
                while !self.check(&TokenType::RightBrace) && !self.check(&TokenType::RightParen) {
                    elems.push(self.expression()?);
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightParen, "Expected ')'")?;
                // `(a, b) => body`
                if self.match_tok(&[TokenType::FatArrow]) {
                    let mut params = Vec::new();
                    for e in elems {
                        match e {
                            Expr::Identifier(n) => params.push((n, None)),
                            _ => return Err(self.error_prev("Lambda parameters must be names")),
                        }
                    }
                    return Ok(Expr::Lambda {
                        params,
                        return_type: None,
                        body: Box::new(self.expression()?),
                    });
                }
                return Ok(Expr::Tuple(elems));
            }
            self.consume(TokenType::RightParen, "Expected ')'")?;
            // `(x) => body`
            if self.match_tok(&[TokenType::FatArrow]) {
                let name = match first {
                    Expr::Identifier(n) => n,
                    _ => return Err(self.error_prev("Expected parameter name before '=>'")),
                };
                return Ok(Expr::Lambda {
                    params: vec![(name, None)],
                    return_type: None,
                    body: Box::new(self.expression()?),
                });
            }
            return Ok(first);
        }

        // List literal  [a, b, c]   or list comprehension  [expr for x in iter if cond]
        if self.match_tok(&[TokenType::LeftBracket]) {
            return self.list_or_comprehension();
        }

        // Lambda  |params| body  or  |params| -> T body
        if self.match_tok(&[TokenType::Pipe]) {
            return self.lambda_expr();
        }

        // if expression (when used as expr, not stmt)
        if self.match_tok(&[TokenType::If]) {
            return self.if_expr();
        }

        // match expression
        if self.match_tok(&[TokenType::Match]) {
            return self.match_expr();
        }

        // Identifier  (may be struct init or path)
        if self.check(&TokenType::Identifier) {
            let name = self.advance().lexeme.clone();
            // Path  Foo::Bar  (with optional turbofish `Foo::<T>::Bar`)
            if self.check(&TokenType::DoubleColon) {
                let mut path = vec![name];
                while self.match_tok(&[TokenType::DoubleColon]) {
                    if self.check(&TokenType::Less) {
                        self.skip_generic_args();
                        continue;
                    }
                    path.push(self.consume(TokenType::Identifier, "Expected identifier after '::'")?
                        .lexeme.clone());
                }
                return Ok(Expr::PathAccess { path });
            }
            // Struct init  Foo { field: val, … }   (shorthand `Foo { field }` allowed)
            if self.check(&TokenType::LeftBrace) {
                // Peek to disambiguate from a block statement:
                // If next non-brace token is  ident + colon/comma/brace  it's struct init
                if self.is_struct_init() {
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                        let fname = self.consume(TokenType::Identifier, "Expected field name")?.lexeme.clone();
                        let fval = if self.match_tok(&[TokenType::Colon]) {
                            self.expression()?
                        } else {
                            // shorthand `Foo { name }` — field bound to same-name var
                            Expr::Identifier(fname.clone())
                        };
                        fields.push((fname, fval));
                        if !self.match_tok(&[TokenType::Comma]) { break; }
                    }
                    self.consume(TokenType::RightBrace, "Expected '}' after struct fields")?;
                    return Ok(Expr::StructInit { name, fields });
                }
            }
            return Ok(Expr::Identifier(name));
        }

        // self / this
        if self.match_tok(&[TokenType::Self_]) || self.match_tok(&[TokenType::This]) {
            return Ok(Expr::Identifier("this".to_string()));
        }

        Err(self.error("Expected expression"))
    }

    /// Detect struct-init by looking two tokens ahead for `ident :`,
    /// `ident ,` or `ident }` (shorthand fields).
    fn is_struct_init(&self) -> bool {
        if self.current + 2 >= self.tokens.len() { return false; }
        matches!(self.tokens[self.current + 1].token_type, TokenType::Identifier)
            && matches!(self.tokens[self.current + 2].token_type,
                TokenType::Colon | TokenType::Comma | TokenType::RightBrace)
    }

    /// `{` at expression position is a dict literal when it starts with
    /// `key:` (`"k":`, `k:`, `123:` …) or is empty; otherwise it's a block.
    fn is_dict_literal(&self) -> bool {
        match self.peek().token_type {
            TokenType::RightBrace => true,
            TokenType::StringLiteral | TokenType::IntLiteral
            | TokenType::FloatLiteral | TokenType::CharLiteral
            | TokenType::Identifier => {
                matches!(self.tokens.get(self.current + 1).map(|t| &t.token_type),
                    Some(TokenType::Colon))
            }
            _ => false,
        }
    }

    fn block_expr(&mut self) -> Result<Expr> {
        let mut stmts = Vec::new();
        let mut trailing = None;
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let stmt = self.declaration()?;
            // If this is the last item and it's an expression without a semicolon
            // it becomes the block's value
            if self.check(&TokenType::RightBrace) {
                if let Stmt::Expr(e) = stmt {
                    trailing = Some(Box::new(e));
                    break;
                }
            }
            stmts.push(stmt);
        }
        self.consume(TokenType::RightBrace, "Expected '}'")?;
        Ok(Expr::Block { stmts, trailing })
    }

    fn list_or_comprehension(&mut self) -> Result<Expr> {
        if self.check(&TokenType::RightBracket) {
            self.advance();
            return Ok(Expr::List(vec![]));
        }
        let first = self.expression()?;
        // list comprehension  [expr for var in iter]
        if self.match_tok(&[TokenType::For]) {
            let var_name = self.consume(TokenType::Identifier, "Expected variable name")?.lexeme.clone();
            self.consume(TokenType::In, "Expected 'in'")?;
            let iterable = self.expression()?;
            let condition = if self.match_tok(&[TokenType::If]) { Some(Box::new(self.expression()?)) } else { None };
            self.consume(TokenType::RightBracket, "Expected ']'")?;
            return Ok(Expr::ListComprehension {
                expr: Box::new(first), var_name,
                iterable: Box::new(iterable), condition,
            });
        }
        // regular list
        let mut items = vec![first];
        while self.match_tok(&[TokenType::Comma]) {
            if self.check(&TokenType::RightBracket) { break; }
            items.push(self.expression()?);
        }
        self.consume(TokenType::RightBracket, "Expected ']'")?;
        Ok(Expr::List(items))
    }

    fn lambda_expr(&mut self) -> Result<Expr> {
        // |params| -> ReturnType body_expr
        let mut params = Vec::new();
        while !self.check(&TokenType::Pipe) && !self.is_at_end() {
            let name = self.consume(TokenType::Identifier, "Expected parameter name")?.lexeme.clone();
            let ty = if self.match_tok(&[TokenType::Colon]) { Some(self.parse_type()?) } else { None };
            params.push((name, ty));
            if !self.match_tok(&[TokenType::Comma]) { break; }
        }
        self.consume(TokenType::Pipe, "Expected '|' after lambda parameters")?;
        let return_type = if self.match_tok(&[TokenType::Arrow]) { Some(self.parse_type()?) } else { None };
        let body = if self.match_tok(&[TokenType::LeftBrace]) {
            self.block_expr()?
        } else {
            self.expression()?
        };
        Ok(Expr::Lambda { params, return_type, body: Box::new(body) })
    }

    fn if_expr(&mut self) -> Result<Expr> {
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let cond = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')'")?; }
        self.consume(TokenType::LeftBrace, "Expected '{'")?;
        let then_expr = self.block_expr()?;
        let else_expr = if self.match_tok(&[TokenType::Else]) {
            if self.match_tok(&[TokenType::If]) {
                Some(Box::new(self.if_expr()?))
            } else {
                self.consume(TokenType::LeftBrace, "Expected '{'")?;
                Some(Box::new(self.block_expr()?))
            }
        } else { None };
        Ok(Expr::If {
            condition: Box::new(cond),
            then_expr: Box::new(then_expr),
            else_expr,
        })
    }

    fn match_expr(&mut self) -> Result<Expr> {
        let paren = self.match_tok(&[TokenType::LeftParen]);
        let subject = self.expression()?;
        if paren { self.consume(TokenType::RightParen, "Expected ')'")?; }
        self.consume(TokenType::LeftBrace, "Expected '{' before match arms")?;
        let mut arms = Vec::new();
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            let guard = if self.match_tok(&[TokenType::If]) { Some(Box::new(self.expression()?)) } else { None };
            self.consume(TokenType::FatArrow, "Expected '=>'")?;
            let body = if self.match_tok(&[TokenType::LeftBrace]) {
                self.block_expr()?
            } else {
                self.expression()?
            };
            arms.push(MatchArm { pattern, guard, body: Box::new(body) });
            self.match_tok(&[TokenType::Comma, TokenType::Semicolon]);
        }
        self.consume(TokenType::RightBrace, "Expected '}' after match arms")?;
        Ok(Expr::Match { expr: Box::new(subject), arms })
    }

    // ═════════════════════════════════════════════════════════════════════════
    // PATTERNS
    // ═════════════════════════════════════════════════════════════════════════

    fn parse_pattern(&mut self) -> Result<Pattern> {
        let mut pats = vec![self.parse_single_pattern()?];
        while self.match_tok(&[TokenType::Pipe]) {
            pats.push(self.parse_single_pattern()?);
        }
        if pats.len() == 1 { Ok(pats.remove(0)) } else { Ok(Pattern::Or(pats)) }
    }

    fn parse_single_pattern(&mut self) -> Result<Pattern> {
        // Wildcard _
        if self.check(&TokenType::Identifier) && self.peek().lexeme == "_" {
            self.advance();
            return Ok(Pattern::Wildcard);
        }
        // .. rest pattern (optionally named: `..rest` / `...rest`)
        if self.match_tok(&[TokenType::DotDot]) {
            self.match_tok(&[TokenType::Dot]); // `...` lexes as `..` + `.`
            if self.check(&TokenType::Identifier) {
                let name = self.advance().lexeme.clone();
                return Ok(Pattern::Binding { name, pattern: Box::new(Pattern::Rest) });
            }
            return Ok(Pattern::Rest);
        }

        // Literal patterns
        if self.match_tok(&[TokenType::True])  { return Ok(Pattern::Literal(Literal::Bool(true))); }
        if self.match_tok(&[TokenType::False]) { return Ok(Pattern::Literal(Literal::Bool(false))); }
        if self.match_tok(&[TokenType::None])  { return Ok(Pattern::Literal(Literal::None)); }

        if self.match_tok(&[TokenType::IntLiteral]) {
            let v: i64 = self.previous().lexeme.parse().unwrap_or(0);
            // Range pattern  1..=5  or  1..5
            if self.match_tok(&[TokenType::DotDotEqual]) {
                let end_expr = Expr::Literal(Literal::Int(
                    self.consume(TokenType::IntLiteral, "Expected range end")?.lexeme.parse().unwrap_or(0)
                ));
                return Ok(Pattern::Range { start: Box::new(Expr::Literal(Literal::Int(v))), end: Box::new(end_expr), inclusive: true });
            }
            if self.match_tok(&[TokenType::DotDot]) {
                let end_expr = Expr::Literal(Literal::Int(
                    self.consume(TokenType::IntLiteral, "Expected range end")?.lexeme.parse().unwrap_or(0)
                ));
                return Ok(Pattern::Range { start: Box::new(Expr::Literal(Literal::Int(v))), end: Box::new(end_expr), inclusive: false });
            }
            return Ok(Pattern::Literal(Literal::Int(v)));
        }
        if self.match_tok(&[TokenType::Minus]) {
            let v: i64 = self.consume(TokenType::IntLiteral, "Expected integer after '-'")?.lexeme.parse().unwrap_or(0);
            return Ok(Pattern::Literal(Literal::Int(-v)));
        }
        if self.match_tok(&[TokenType::StringLiteral]) {
            return Ok(Pattern::Literal(Literal::String(self.previous().lexeme.clone())));
        }
        if self.match_tok(&[TokenType::CharLiteral]) {
            let ch = self.previous().lexeme.chars().next().unwrap_or('\0');
            return Ok(Pattern::Literal(Literal::Char(ch)));
        }

        // Tuple pattern  (a, b)
        if self.match_tok(&[TokenType::LeftParen]) {
            let mut pats = Vec::new();
            while !self.check(&TokenType::RightParen) {
                pats.push(self.parse_pattern()?);
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::RightParen, "Expected ')'")?;
            return Ok(Pattern::Tuple(pats));
        }

        // List pattern  [a, b, ..]
        if self.match_tok(&[TokenType::LeftBracket]) {
            let mut pats = Vec::new();
            while !self.check(&TokenType::RightBracket) {
                pats.push(self.parse_pattern()?);
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::RightBracket, "Expected ']'")?;
            return Ok(Pattern::List(pats));
        }

        // Anonymous struct/dict pattern  { field, field: pat, .. }
        if self.match_tok(&[TokenType::LeftBrace]) {
            let mut fields = Vec::new();
            let mut rest   = false;
            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                if self.match_tok(&[TokenType::DotDot]) {
                    self.match_tok(&[TokenType::Dot]);
                    // optional binding name after `..` — still just marks rest
                    if self.check(&TokenType::Identifier) { self.advance(); }
                    rest = true;
                    break;
                }
                let fname = self.consume(TokenType::Identifier, "Expected field name")?.lexeme.clone();
                let fpat  = if self.match_tok(&[TokenType::Colon]) {
                    self.parse_pattern()?
                } else {
                    Pattern::Identifier(fname.clone())
                };
                fields.push((fname, fpat));
                if !self.match_tok(&[TokenType::Comma]) { break; }
            }
            self.consume(TokenType::RightBrace, "Expected '}'")?;
            return Ok(Pattern::Struct { name: String::new(), fields, rest });
        }

        // Named patterns: Identifier / path / struct / enum variant
        if self.check(&TokenType::Identifier) {
            let name = self.advance().lexeme.clone();

            // Path pattern  Foo::Bar(…)
            let mut path = vec![name.clone()];
            while self.match_tok(&[TokenType::DoubleColon]) {
                path.push(self.consume(TokenType::Identifier, "Expected identifier")?.lexeme.clone());
            }

            // Enum variant with tuple fields  Variant(a, b)
            if self.match_tok(&[TokenType::LeftParen]) {
                let mut fields = Vec::new();
                while !self.check(&TokenType::RightParen) {
                    fields.push(self.parse_pattern()?);
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightParen, "Expected ')'")?;
                return Ok(Pattern::EnumVariant { path, fields });
            }

            // Struct pattern  Variant { field: pat, … }
            if self.check(&TokenType::LeftBrace) {
                self.advance();
                let mut fields = Vec::new();
                let mut rest   = false;
                while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                    if self.match_tok(&[TokenType::DotDot]) { rest = true; break; }
                    let fname = self.consume(TokenType::Identifier, "Expected field name")?.lexeme.clone();
                    let fpat  = if self.match_tok(&[TokenType::Colon]) {
                        self.parse_pattern()?
                    } else {
                        Pattern::Identifier(fname.clone())
                    };
                    fields.push((fname, fpat));
                    if !self.match_tok(&[TokenType::Comma]) { break; }
                }
                self.consume(TokenType::RightBrace, "Expected '}'")?;
                return Ok(Pattern::Struct { name: path.join("::"), fields, rest });
            }

            // Plain identifier  (binding)
            if path.len() == 1 {
                return Ok(Pattern::Identifier(name));
            }
            // Multi-segment path without parens = unit enum variant
            return Ok(Pattern::EnumVariant { path, fields: vec![] });
        }

        Err(self.error("Expected pattern"))
    }
} // end impl Parser
