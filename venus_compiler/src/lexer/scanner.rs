use std::collections::VecDeque;
use crate::lexer::token::{Token, TokenKind, Span};

pub struct Scanner {
    source: Vec<char>,
    pos: usize,
    line: u32,
    column: u32,
    indent_stack: Vec<usize>,
    pending_tokens: VecDeque<Token>,
}

impl Scanner {
    pub fn new(input: &str) -> Self {
        Self {
            source: input.chars().collect(),
            pos: 0,
            line: 1,
            column: 1,
            indent_stack: vec![0], // Base indentation is 0
            pending_tokens: VecDeque::new(),
        }
    }

    pub fn scan_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens: Vec<Token> = Vec::new();
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            
            if is_eof {
                if let Some(last) = tokens.last() {
                    match last.kind {
                        TokenKind::Newline | TokenKind::Dedent | TokenKind::Indent => {}
                        _ => tokens.push(Token { kind: TokenKind::Newline, lexeme: "".to_string(), span: token.span.clone() })
                    }
                }
                tokens.push(token);
                break;
            }
            tokens.push(token);
        }
        Ok(tokens)
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        if let Some(token) = self.pending_tokens.pop_front() {
            return Ok(token);
        }

        self.skip_whitespace();

        if self.is_at_end() {
            // Process remaining dedents
            while self.indent_stack.len() > 1 {
                self.indent_stack.pop();
                self.pending_tokens.push_back(self.make_token(TokenKind::Dedent, ""));
            }
            self.pending_tokens.push_back(self.make_token(TokenKind::Eof, ""));
            return Ok(self.pending_tokens.pop_front().unwrap());
        }

        let c = self.advance();

        match c {
            '\n' => self.handle_newline(),
            '(' => Ok(self.make_token(TokenKind::LeftParen, "(")),
            ')' => Ok(self.make_token(TokenKind::RightParen, ")")),
            '[' => Ok(self.make_token(TokenKind::LeftBracket, "[")),
            ']' => Ok(self.make_token(TokenKind::RightBracket, "]")),
            ',' => Ok(self.make_token(TokenKind::Comma, ",")),
            ':' => Ok(self.make_token(TokenKind::Colon, ":")),
            '.' => Ok(self.make_token(TokenKind::Dot, ".")),
            '@' => Ok(self.make_token(TokenKind::At, "@")),
            '-' => {
                if self.match_char('>') {
                    Ok(self.make_token(TokenKind::Arrow, "->"))
                } else if self.match_char('=') {
                    Ok(self.make_token(TokenKind::MinusEqual, "-="))
                } else {
                    Ok(self.make_token(TokenKind::Minus, "-"))
                }
            }
            '+' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::PlusEqual, "+="))
                } else {
                    Ok(self.make_token(TokenKind::Plus, "+"))
                }
            }
            '*' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::StarEqual, "*="))
                } else {
                    Ok(self.make_token(TokenKind::Star, "*"))
                }
            }
            '/' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::SlashEqual, "/="))
                } else {
                    Ok(self.make_token(TokenKind::Slash, "/"))
                }
            }
            '%' => Ok(self.make_token(TokenKind::Percent, "%")),
            '=' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::EqualEqual, "=="))
                } else {
                    Ok(self.make_token(TokenKind::Equal, "="))
                }
            }
            '!' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::BangEqual, "!="))
                } else {
                    Ok(self.make_token(TokenKind::Bang, "!"))
                }
            }
            '<' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::LessEqual, "<="))
                } else {
                    Ok(self.make_token(TokenKind::Less, "<"))
                }
            }
            '>' => {
                if self.match_char('=') {
                    Ok(self.make_token(TokenKind::GreaterEqual, ">="))
                } else {
                    Ok(self.make_token(TokenKind::Greater, ">"))
                }
            }
            '&' => {
                if self.match_char('&') {
                    Ok(self.make_token(TokenKind::And, "&&"))
                } else {
                    Err(format!("Expected '&' at line {}, column {}", self.line, self.column))
                }
            }
            '|' => {
                if self.match_char('|') {
                    Ok(self.make_token(TokenKind::Or, "||"))
                } else {
                    Err(format!("Expected '|' at line {}, column {}", self.line, self.column))
                }
            }
            '"' | '\'' => self.string_literal(c),
            '#' => {
                // Comment, consume until end of line
                while !self.is_at_end() && self.peek() != '\n' {
                    self.advance();
                }
                self.next_token() // Recursively get the next actual token
            }
            c if c.is_ascii_digit() => self.number_literal(),
            c if c.is_alphabetic() || c == '_' => self.identifier(),
            _ => Err(format!("Unexpected character '{}' at line {}, column {}", c, self.line, self.column)),
        }
    }

    fn handle_newline(&mut self) -> Result<Token, String> {
        let newline_token = self.make_token(TokenKind::Newline, "\\n");
        self.line += 1;
        self.column = 1;

        // Count spaces for indentation
        let mut indent_count = 0;
        while !self.is_at_end() {
            let c = self.peek();
            if c == ' ' {
                indent_count += 1;
                self.advance();
            } else if c == '\t' {
                return Err(format!("Tabs are not allowed for indentation at line {}", self.line));
            } else {
                break;
            }
        }

        // If line is entirely blank or just a comment, ignore its indentation
        if self.is_at_end() || self.peek() == '\n' || self.peek() == '#' {
            return Ok(newline_token); // Just return the newline token without Indent/Dedent
        }

        let current_indent = *self.indent_stack.last().unwrap();

        if indent_count > current_indent {
            self.indent_stack.push(indent_count);
            self.pending_tokens.push_back(self.make_token(TokenKind::Indent, ""));
        } else if indent_count < current_indent {
            while let Some(&top) = self.indent_stack.last() {
                if top > indent_count {
                    self.indent_stack.pop();
                    self.pending_tokens.push_back(self.make_token(TokenKind::Dedent, ""));
                } else if top < indent_count {
                    return Err(format!("Indentation error at line {}", self.line));
                } else {
                    break;
                }
            }
        }

        // Return the newline, subsequent calls will fetch the Indent/Dedent from pending_tokens
        Ok(newline_token)
    }

    fn string_literal(&mut self, quote: char) -> Result<Token, String> {
        let start_pos = self.pos - 1;
        let start_column = self.column - 1;
        let start_line = self.line;
        
        let mut value = String::new();
        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            value.push(self.advance());
        }

        if self.is_at_end() {
            return Err(format!("Unterminated string literal at line {}", self.line));
        }

        // Consume the closing quote
        self.advance();
        Ok(Token {
            kind: TokenKind::StringLiteral(value),
            lexeme: "".to_string(), // Can be improved later
            span: Span {
                start: start_pos,
                end: self.pos,
                line: start_line,
                column: start_column,
            }
        })
    }

    fn number_literal(&mut self) -> Result<Token, String> {
        let start_pos = self.pos - 1;
        let mut is_float = false;

        while !self.is_at_end() && self.peek().is_ascii_digit() {
            self.advance();
        }

        if !self.is_at_end() && self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            self.advance(); // consume '.'
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let text: String = self.source[start_pos..self.pos].iter().collect();
        if is_float {
            Ok(self.make_token(TokenKind::FloatLiteral(text.parse().unwrap()), &text))
        } else {
            Ok(self.make_token(TokenKind::IntLiteral(text.parse().unwrap()), &text))
        }
    }

    fn identifier(&mut self) -> Result<Token, String> {
        let start_pos = self.pos - 1;

        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            self.advance();
        }

        let text: String = self.source[start_pos..self.pos].iter().collect();

        let kind = match text.as_str() {
            "int" => TokenKind::KwInt,
            "float" => TokenKind::KwFloat,
            "string" => TokenKind::KwString,
            "bool" => TokenKind::KwBool,
            "struct" => TokenKind::KwStruct,
            "object" => TokenKind::KwObject,
            "behaviour" => TokenKind::KwBehaviour,
            "func" => TokenKind::KwFunc,
            "buffer" => TokenKind::KwBuffer,
            "ref" => TokenKind::KwRef,
            "enum" => TokenKind::KwEnum,
            "task" => TokenKind::KwTask,
            "signal" => TokenKind::KwSignal,
            "const" => TokenKind::KwConst,
            "export" => TokenKind::KwExport,
            "exclude" => TokenKind::KwExclude,
            "import" => TokenKind::KwImport,
            "from" => TokenKind::KwFrom,
            "uses" => TokenKind::KwUses,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "elif" => TokenKind::KwElif,
            "for" => TokenKind::KwFor,
            "while" => TokenKind::KwWhile,
            "in" => TokenKind::KwIn,
            "return" => TokenKind::KwReturn,
            "true" => TokenKind::BoolLiteral(true),
            "false" => TokenKind::BoolLiteral(false),
            _ => TokenKind::Identifier(text.clone()),
        };

        Ok(self.make_token(kind, &text))
    }

    // ── Utility Methods ──

    fn advance(&mut self) -> char {
        let c = self.source[self.pos];
        self.pos += 1;
        self.column += 1;
        c
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() { return false; }
        if self.source[self.pos] != expected { return false; }
        self.pos += 1;
        self.column += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() { '\0' } else { self.source[self.pos] }
    }

    fn peek_next(&self) -> char {
        if self.pos + 1 >= self.source.len() { '\0' } else { self.source[self.pos + 1] }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.source.len()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            let c = self.peek();
            if c == ' ' || c == '\r' || c == '\t' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn make_token(&self, kind: TokenKind, lexeme: &str) -> Token {
        Token {
            kind,
            lexeme: lexeme.to_string(),
            span: Span {
                start: self.pos.saturating_sub(lexeme.len()),
                end: self.pos,
                line: self.line,
                column: self.column.saturating_sub(lexeme.len() as u32),
            },
        }
    }
}
