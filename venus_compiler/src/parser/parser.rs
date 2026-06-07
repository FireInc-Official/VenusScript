use crate::lexer::token::{Token, TokenKind, Span};
use crate::parser::ast::*;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut nodes = Vec::new();
        while !self.is_at_end() {
            // skip extra newlines at the top level
            if self.match_kind(TokenKind::Newline) {
                continue;
            }
            nodes.push(self.parse_node()?);
        }
        Ok(Program { nodes })
    }

    fn parse_node(&mut self) -> Result<Node, String> {
        // Collect decorators
        let mut decorators = Vec::new();
        while self.match_kind(TokenKind::At) {
            decorators.push(self.parse_decorator()?);
            self.consume(TokenKind::Newline, "Expected newline after decorator")?;
        }

        let peek = self.peek();
        match &peek.kind {
            TokenKind::KwExport => {
                self.advance(); // consume 'export'
                // After export, we expect a variable declaration
                self.parse_variable_exported(decorators).map(Node::Variable)
            }
            kind if kind.is_type_keyword() => {
                let is_var_decl = if let TokenKind::Identifier(_) = kind {
                    // If it's just an identifier, it's a Variable Declaration ONLY IF the next token is also an identifier
                    matches!(self.peek_next().kind, TokenKind::Identifier(_))
                } else {
                    // Built-in types (int, func, object) are always variable declarations
                    true
                };

                if is_var_decl {
                    self.parse_variable(decorators).map(Node::Variable)
                } else {
                    // Assignment or Expression Statement
                    let expr = self.parse_expr()?;
                    if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::PlusEqual) || 
                       self.match_kind(TokenKind::MinusEqual) || self.match_kind(TokenKind::StarEqual) || 
                       self.match_kind(TokenKind::SlashEqual) {
                        let op = match self.previous().kind {
                            TokenKind::Colon => AssignOp::Assign,
                            TokenKind::PlusEqual => AssignOp::AddAssign,
                            TokenKind::MinusEqual => AssignOp::SubAssign,
                            TokenKind::StarEqual => AssignOp::MulAssign,
                            TokenKind::SlashEqual => AssignOp::DivAssign,
                            _ => unreachable!(),
                        };
                        let value = self.parse_expr()?;
                        self.consume(TokenKind::Newline, "Expected newline after assignment")?;
                        Ok(Node::Assignment(Assignment {
                            target: expr.clone(),
                            op,
                            value,
                            span: expr.span(),
                        }))
                    } else {
                        self.consume(TokenKind::Newline, "Expected newline after expression")?;
                        Ok(Node::ExprStmt(expr))
                    }
                }
            }
            _ => {
                // Fallback for literals or other expressions
                let expr = self.parse_expr()?;
                if self.match_kind(TokenKind::Colon) || self.match_kind(TokenKind::PlusEqual) || 
                   self.match_kind(TokenKind::MinusEqual) || self.match_kind(TokenKind::StarEqual) || 
                   self.match_kind(TokenKind::SlashEqual) {
                    let op = match self.previous().kind {
                        TokenKind::Colon => AssignOp::Assign,
                        TokenKind::PlusEqual => AssignOp::AddAssign,
                        TokenKind::MinusEqual => AssignOp::SubAssign,
                        TokenKind::StarEqual => AssignOp::MulAssign,
                        TokenKind::SlashEqual => AssignOp::DivAssign,
                        _ => unreachable!(),
                    };
                    let value = self.parse_expr()?;
                    self.consume(TokenKind::Newline, "Expected newline after assignment")?;
                    Ok(Node::Assignment(Assignment {
                        target: expr.clone(),
                        op,
                        value,
                        span: expr.span(),
                    }))
                } else {
                    self.consume(TokenKind::Newline, "Expected newline after expression")?;
                    Ok(Node::ExprStmt(expr))
                }
            }
        }
    }

    fn parse_variable(&mut self, decorators: Vec<Decorator>) -> Result<Variable, String> {
        self.parse_variable_inner(decorators, false)
    }

    fn parse_variable_exported(&mut self, decorators: Vec<Decorator>) -> Result<Variable, String> {
        self.parse_variable_inner(decorators, true)
    }

    fn parse_variable_inner(&mut self, decorators: Vec<Decorator>, is_export: bool) -> Result<Variable, String> {
        let span_start = self.peek().span.start;
        
        let is_const = self.match_kind(TokenKind::KwConst);
        let is_ref = self.match_kind(TokenKind::KwRef);
        // support exclude
        if self.match_kind(TokenKind::KwExclude) {}
        
        let type_expr = self.parse_type_expr()?;
        
        let mut name = None;
        if let TokenKind::Identifier(ident) = &self.peek().kind {
            name = Some(ident.clone());
            self.advance();
        } else if self.peek().kind.is_type_keyword() {
            name = Some(self.advance().lexeme.clone());
        }

        let mut uses = Vec::new();
        if self.match_kind(TokenKind::KwUses) {
            uses.push(self.consume_ident("Expected identifier after 'uses'")?.lexeme.clone());
        }

        let mut arguments = None;
        if self.match_kind(TokenKind::LeftParen) {
            arguments = Some(self.parse_arg_list()?);
        }

        let mut return_type = None;
        if self.match_kind(TokenKind::Arrow) {
            return_type = Some(self.parse_type_expr()?);
        }

        let mut content = Vec::new();
        
        // Handle colon assignment
        if self.match_kind(TokenKind::Colon) {
            let value_expr = self.parse_expr()?;
            // Wrap in anonymous variable
            content.push(Node::Variable(Variable {
                type_expr: TypeExpr::Named("anonymous".to_string()),
                name: None,
                arguments: None,
                content: vec![Node::ExprStmt(value_expr)],
                return_type: None,
                decorators: vec![],
                uses: vec![],
                is_const: false,
                is_export: false,
                is_ref: false,
                span: Span::default(),
            }));
        }

        // Must end with newline
        self.consume(TokenKind::Newline, "Expected newline after variable declaration")?;

        while self.match_kind(TokenKind::Newline) {}

        if self.match_kind(TokenKind::Indent) {
            while !self.check(TokenKind::Dedent) && !self.is_at_end() {
                if self.match_kind(TokenKind::Newline) { continue; }
                content.push(self.parse_node()?);
            }
            self.consume(TokenKind::Dedent, "Expected dedent after block")?;
        }

        Ok(Variable {
            type_expr,
            name,
            arguments,
            content,
            return_type,
            decorators,
            uses,
            is_const,
            is_export,
            is_ref,
            span: Span { start: span_start, end: self.previous().span.end, line: 0, column: 0 },
        })
    }



    // ── Components ──

    fn parse_type_expr(&mut self) -> Result<TypeExpr, String> {
        let base_type = match &self.advance().kind {
            TokenKind::KwInt => "int".to_string(),
            TokenKind::KwFloat => "float".to_string(),
            TokenKind::KwString => "string".to_string(),
            TokenKind::KwBool => "bool".to_string(),
            TokenKind::KwStruct => "struct".to_string(),
            TokenKind::KwObject => "object".to_string(),
            TokenKind::KwBehaviour => "behaviour".to_string(),
            TokenKind::KwFunc => "func".to_string(),
            TokenKind::KwBuffer => "buffer".to_string(),
            TokenKind::KwEnum => "enum".to_string(),
            TokenKind::KwTask => "task".to_string(),
            TokenKind::KwSignal => "signal".to_string(),
            TokenKind::KwVec2 => "vec2".to_string(),
            TokenKind::KwVec3 => "vec3".to_string(),
            TokenKind::KwVec4 => "vec4".to_string(),
            TokenKind::KwMat2 => "mat2".to_string(),
            TokenKind::KwMat3 => "mat3".to_string(),
            TokenKind::KwMat4 => "mat4".to_string(),
            TokenKind::KwTensor => "tensor".to_string(),
            TokenKind::KwIf => "if".to_string(),
            TokenKind::KwElse => "else".to_string(),
            TokenKind::KwElif => "elif".to_string(),
            TokenKind::KwFor => "for".to_string(),
            TokenKind::KwWhile => "while".to_string(),
            TokenKind::KwReturn => "return".to_string(),
            TokenKind::KwImport => "import".to_string(),
            TokenKind::KwFrom => "from".to_string(),
            TokenKind::Identifier(name) => name.clone(),
            _ => return Err("Expected a type".to_string()),
        };

        let mut ty = TypeExpr::Named(base_type);

        while self.match_kind(TokenKind::LeftBracket) {
            if !self.check(TokenKind::RightBracket) {
                // allow `array[int]` by parsing a type inside and discarding it or putting it in the array type
                let _inner_ty = self.parse_type_expr()?;
            }
            self.consume(TokenKind::RightBracket, "Expected ']' after '['")?;
            ty = TypeExpr::Array(Box::new(ty));
        }

        Ok(ty)
    }

    fn parse_decorator(&mut self) -> Result<Decorator, String> {
        let start = self.previous().span.start;
        let mut path = Vec::new();
        path.push(self.consume_ident("Expected identifier after '@'")?.lexeme.clone());
        while self.match_kind(TokenKind::Dot) {
            path.push(self.consume_ident("Expected identifier after '.'")?.lexeme.clone());
        }
        Ok(Decorator { path, span: Span { start, end: self.previous().span.end, line: 0, column: 0 } })
    }

    fn parse_arg_list(&mut self) -> Result<ArgList, String> {
        let start = self.previous().span.start;
        let mut args = Vec::new();
        
        if !self.check(TokenKind::RightParen) {
            loop {
                args.push(self.parse_arg()?);
                if !self.match_kind(TokenKind::Comma) { break; }
            }
        }
        
        self.consume(TokenKind::RightParen, "Expected ')' after arguments")?;
        Ok(ArgList { args, span: Span { start, end: self.previous().span.end, line: 0, column: 0 } })
    }

    fn parse_arg(&mut self) -> Result<Arg, String> {
        // Can be Typed (int a), Named (w=800), or Positional (expr)
        let mut is_typed = false;
        let mut check_idx = self.current;
        let is_ref = if self.tokens[check_idx].kind == TokenKind::KwRef {
            check_idx += 1;
            true
        } else {
            false
        };

        if check_idx < self.tokens.len() && self.tokens[check_idx].kind.is_type_keyword() {
            if check_idx + 1 < self.tokens.len() && matches!(self.tokens[check_idx + 1].kind, TokenKind::Identifier(_)) {
                is_typed = true;
            }
        }

        if is_typed {
            if is_ref { self.advance(); } // Consume 'ref'
            let ty = self.parse_type_expr()?;
            let name_tok = self.consume_ident("Expected parameter name")?;
            return Ok(Arg::Typed {
                ty,
                name: name_tok.lexeme.clone(),
                is_ref,
                span: name_tok.span.clone(),
            });
        }

        // Check for Named (id = expr)
        if let TokenKind::Identifier(name) = &self.peek().kind {
            if let TokenKind::Equal = &self.peek_next().kind {
                let name = name.clone();
                let start_span = self.advance().span.clone(); // id
                self.advance(); // =
                let value = self.parse_expr()?;
                return Ok(Arg::Named { name, value, span: start_span });
            }
        }

        // Positional
        let expr = self.parse_expr()?;
        Ok(Arg::Positional(expr))
    }



    // ── Expressions ──
    // Extremely simplified expression parser for now
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_expr_pratt(0)
    }

    fn get_binop(kind: &TokenKind) -> Option<(u8, BinOp)> {
        match kind {
            TokenKind::KwIn => Some((1, BinOp::In)),
            TokenKind::EqualEqual => Some((1, BinOp::Eq)),
            TokenKind::BangEqual => Some((1, BinOp::NotEq)),
            TokenKind::Less => Some((2, BinOp::Lt)),
            TokenKind::Greater => Some((2, BinOp::Gt)),
            TokenKind::LessEqual => Some((2, BinOp::LtEq)),
            TokenKind::GreaterEqual => Some((2, BinOp::GtEq)),
            TokenKind::Plus => Some((3, BinOp::Add)),
            TokenKind::Minus => Some((3, BinOp::Sub)),
            TokenKind::Star => Some((4, BinOp::Mul)),
            TokenKind::Slash => Some((4, BinOp::Div)),
            _ => None,
        }
    }

    fn parse_expr_pratt(&mut self, min_prec: u8) -> Result<Expr, String> {
        let mut left = self.parse_unary()?;
        
        while let Some((prec, op)) = Self::get_binop(&self.peek().kind) {
            if prec < min_prec { break; }
            self.advance();
            let right = self.parse_expr_pratt(prec + 1)?;
            left = Expr::BinaryOp { left: Box::new(left), op, right: Box::new(right), span: Span::default() };
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, String> {
        if self.match_kind(TokenKind::Minus) || self.match_kind(TokenKind::Bang) {
            let op = match self.previous().kind {
                TokenKind::Minus => UnOp::Negate,
                TokenKind::Bang => UnOp::Not,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            return Ok(Expr::UnaryOp { op, operand: Box::new(right), span: Span::default() });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, String> {
        if self.match_kind(TokenKind::LeftBracket) {
            let mut items = Vec::new();
            if !self.check(TokenKind::RightBracket) {
                loop {
                    items.push(self.parse_expr()?);
                    if !self.match_kind(TokenKind::Comma) { break; }
                }
            }
            self.consume(TokenKind::RightBracket, "Expected ']' after array items")?;
            return Ok(Expr::ArrayLiteral(items, Span::default()));
        }

        let token = self.advance().clone();
        let mut expr = match token.kind {
            TokenKind::IntLiteral(n) => Expr::IntLiteral(n, token.span),
            TokenKind::FloatLiteral(n) => Expr::FloatLiteral(n, token.span),
            TokenKind::StringLiteral(s) => Expr::StringLiteral(s, token.span),
            TokenKind::BoolLiteral(b) => Expr::BoolLiteral(b, token.span),
            TokenKind::Identifier(name) => Expr::Identifier(name, token.span),
            TokenKind::LeftParen => {
                let e = self.parse_expr()?;
                self.consume(TokenKind::RightParen, "Expected ')'")?;
                e
            }
            kind if kind.is_type_keyword() => {
                Expr::Identifier(token.lexeme.clone(), token.span)
            }
            _ => return Err(format!("Expected expression at line {} (found {:?})", token.span.line, token.kind)),
        };

        // Handle trailing calls or member access
        loop {
            if self.match_kind(TokenKind::LeftParen) {
                let mut args = Vec::new();
                if !self.check(TokenKind::RightParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_kind(TokenKind::Comma) { break; }
                    }
                }
                self.consume(TokenKind::RightParen, "Expected ')' after call args")?;
                expr = Expr::Call { callee: Box::new(expr), args, span: Span::default() };
            } else if self.match_kind(TokenKind::Dot) {
                let member = self.consume_ident("Expected property name")?.lexeme.clone();
                expr = Expr::MemberAccess { object: Box::new(expr), member, span: Span::default() };
            } else if self.match_kind(TokenKind::LeftBracket) {
                let index = self.parse_expr()?;
                self.consume(TokenKind::RightBracket, "Expected ']' after index")?;
                expr = Expr::IndexAccess { object: Box::new(expr), index: Box::new(index), span: Span::default() };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // ── Utilities ──

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, err: &str) -> Result<&Token, String> {
        if self.check(kind) {
            Ok(self.advance())
        } else {
            let peek = self.peek();
            Err(format!("{} at line {}, col {} (found {:?})", err, peek.span.line, peek.span.column, peek.kind))
        }
    }

    fn consume_ident(&mut self, err: &str) -> Result<&Token, String> {
        if let TokenKind::Identifier(_) = self.peek().kind {
            Ok(self.advance())
        } else if self.peek().kind.is_type_keyword() {
            Ok(self.advance())
        } else {
            Err(err.to_string())
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        if self.is_at_end() { return false; }
        // We only compare the variant type, not the exact payload if it has one.
        // For Identifier/Literals we shouldn't use check() directly this way if we need exact match,
        // but it's safe for keywords and punctuation.
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn peek_next(&self) -> &Token {
        if self.current + 1 >= self.tokens.len() {
            self.peek()
        } else {
            &self.tokens[self.current + 1]
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
