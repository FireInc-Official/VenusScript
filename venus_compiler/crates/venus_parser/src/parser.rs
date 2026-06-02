use venus_lexer::token::{Token, TokenKind, Span};
use crate::ast::*;

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
            TokenKind::KwImport => self.parse_import(false).map(Node::Import),
            TokenKind::KwFrom => self.parse_import(true).map(Node::Import),
            TokenKind::KwIf => self.parse_if_chain().map(Node::IfChain),
            TokenKind::KwFor => self.parse_for_loop().map(Node::ForLoop),
            TokenKind::KwWhile => self.parse_while_loop().map(Node::WhileLoop),
            TokenKind::KwReturn => self.parse_return().map(Node::Return),
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
                    if self.match_kind(TokenKind::Equal) || self.match_kind(TokenKind::PlusEqual) || 
                       self.match_kind(TokenKind::MinusEqual) || self.match_kind(TokenKind::StarEqual) || 
                       self.match_kind(TokenKind::SlashEqual) {
                        let op = match self.previous().kind {
                            TokenKind::Equal => AssignOp::Assign,
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
                if self.match_kind(TokenKind::Equal) || self.match_kind(TokenKind::PlusEqual) || 
                   self.match_kind(TokenKind::MinusEqual) || self.match_kind(TokenKind::StarEqual) || 
                   self.match_kind(TokenKind::SlashEqual) {
                    let op = match self.previous().kind {
                        TokenKind::Equal => AssignOp::Assign,
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
        let type_expr = self.parse_type_expr()?;
        
        let name_token = self.consume_ident("Expected variable name")?;
        let name = name_token.lexeme.clone();

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

        let mut value = None;
        if self.match_kind(TokenKind::Equal) {
            value = Some(self.parse_expr()?);
        }

        // Must end with newline
        self.consume(TokenKind::Newline, "Expected newline after variable declaration")?;

        while self.match_kind(TokenKind::Newline) {}

        let mut content = Vec::new();
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
            value,
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

    fn parse_import(&mut self, is_from: bool) -> Result<ImportStmt, String> {
        let start = self.advance().span.start; // consume 'import' or 'from'
        
        let module_name;
        let mut items = Vec::new();

        if is_from {
            let name_token = self.consume_ident("Expected module name after 'from'")?;
            module_name = name_token.lexeme.clone();
            self.consume(TokenKind::KwImport, "Expected 'import' after module name")?;
            
            if self.match_kind(TokenKind::Star) {
                items.push("*".to_string());
            } else {
                loop {
                    let item_token = self.consume_ident("Expected item to import")?;
                    items.push(item_token.lexeme.clone());
                    if !self.match_kind(TokenKind::Comma) { break; }
                }
            }
        } else {
            let name_token = self.consume_ident("Expected module name after 'import'")?;
            module_name = name_token.lexeme.clone();
        }

        self.consume(TokenKind::Newline, "Expected newline after import statement")?;
        Ok(ImportStmt {
            module_name,
            items,
            is_from,
            span: Span { start, end: self.previous().span.end, line: 0, column: 0 },
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
            TokenKind::Identifier(name) => name.clone(),
            _ => return Err("Expected a type".to_string()),
        };

        let mut ty = TypeExpr::Named(base_type);

        while self.match_kind(TokenKind::LeftBracket) {
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

    // ── Control Flow ──
    
    fn parse_if_chain(&mut self) -> Result<IfChain, String> {
        self.advance(); // consume 'if'
        let condition = self.parse_expr()?;
        self.consume(TokenKind::Newline, "Expected newline after if condition")?;
        
        while self.match_kind(TokenKind::Newline) {}
        self.consume(TokenKind::Indent, "Expected block after if")?;
        let mut then_body = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.match_kind(TokenKind::Newline) { continue; }
            then_body.push(self.parse_node()?);
        }
        self.consume(TokenKind::Dedent, "Expected dedent")?;

        let mut elif_branches = Vec::new();
        while self.match_kind(TokenKind::KwElif) {
            let cond = self.parse_expr()?;
            self.consume(TokenKind::Newline, "Expected newline")?;
            while self.match_kind(TokenKind::Newline) {}
            self.consume(TokenKind::Indent, "Expected block")?;
            let mut body = Vec::new();
            while !self.check(TokenKind::Dedent) && !self.is_at_end() {
                if self.match_kind(TokenKind::Newline) { continue; }
                body.push(self.parse_node()?);
            }
            self.consume(TokenKind::Dedent, "Expected dedent")?;
            elif_branches.push((cond, body));
        }

        let mut else_body = None;
        if self.match_kind(TokenKind::KwElse) {
            self.consume(TokenKind::Newline, "Expected newline")?;
            self.consume(TokenKind::Indent, "Expected block")?;
            let mut body = Vec::new();
            while !self.check(TokenKind::Dedent) && !self.is_at_end() {
                if self.match_kind(TokenKind::Newline) { continue; }
                body.push(self.parse_node()?);
            }
            self.consume(TokenKind::Dedent, "Expected dedent")?;
            else_body = Some(body);
        }

        Ok(IfChain { condition, then_body, elif_branches, else_body, span: Span::default() })
    }

    fn parse_for_loop(&mut self) -> Result<ForLoop, String> {
        self.advance(); // 'for'
        let var_name = self.consume_ident("Expected loop variable")?.lexeme.clone();
        self.consume(TokenKind::KwIn, "Expected 'in' after loop variable")?;
        let iterable = self.parse_expr()?;
        self.consume(TokenKind::Newline, "Expected newline after for condition")?;
        
        while self.match_kind(TokenKind::Newline) {}
        self.consume(TokenKind::Indent, "Expected block")?;
        let mut body = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.match_kind(TokenKind::Newline) { continue; }
            body.push(self.parse_node()?);
        }
        self.consume(TokenKind::Dedent, "Expected dedent")?;
        Ok(ForLoop { var_name, iterable, body, span: Span::default() })
    }

    fn parse_while_loop(&mut self) -> Result<WhileLoop, String> {
        self.advance(); // 'while'
        let condition = self.parse_expr()?;
        self.consume(TokenKind::Newline, "Expected newline")?;
        
        while self.match_kind(TokenKind::Newline) {}
        self.consume(TokenKind::Indent, "Expected block")?;
        let mut body = Vec::new();
        while !self.check(TokenKind::Dedent) && !self.is_at_end() {
            if self.match_kind(TokenKind::Newline) { continue; }
            body.push(self.parse_node()?);
        }
        self.consume(TokenKind::Dedent, "Expected dedent")?;
        Ok(WhileLoop { condition, body, span: Span::default() })
    }

    fn parse_return(&mut self) -> Result<ReturnStmt, String> {
        self.advance(); // 'return'
        let mut value = None;
        if !self.check(TokenKind::Newline) {
            value = Some(self.parse_expr()?);
        }
        self.consume(TokenKind::Newline, "Expected newline")?;
        Ok(ReturnStmt { value, span: Span::default() })
    }

    // ── Expressions ──
    // Extremely simplified expression parser for now
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_equality()
    }

    fn parse_equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_comparison()?;
        while self.match_kind(TokenKind::EqualEqual) || self.match_kind(TokenKind::BangEqual) {
            let op = match self.previous().kind {
                TokenKind::EqualEqual => BinOp::Eq,
                TokenKind::BangEqual => BinOp::NotEq,
                _ => unreachable!(),
            };
            let right = self.parse_comparison()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right), span: Span::default() };
        }
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_term()?;
        while self.match_kind(TokenKind::Less) || self.match_kind(TokenKind::Greater) || 
              self.match_kind(TokenKind::LessEqual) || self.match_kind(TokenKind::GreaterEqual) {
            let op = match self.previous().kind {
                TokenKind::Less => BinOp::Lt,
                TokenKind::Greater => BinOp::Gt,
                TokenKind::LessEqual => BinOp::LtEq,
                TokenKind::GreaterEqual => BinOp::GtEq,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right), span: Span::default() };
        }
        Ok(expr)
    }

    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_factor()?;
        while self.match_kind(TokenKind::Plus) || self.match_kind(TokenKind::Minus) {
            let op = match self.previous().kind {
                TokenKind::Plus => BinOp::Add,
                TokenKind::Minus => BinOp::Sub,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right), span: Span::default() };
        }
        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.parse_unary()?;
        while self.match_kind(TokenKind::Star) || self.match_kind(TokenKind::Slash) {
            let op = match self.previous().kind {
                TokenKind::Star => BinOp::Mul,
                TokenKind::Slash => BinOp::Div,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            expr = Expr::BinaryOp { left: Box::new(expr), op, right: Box::new(right), span: Span::default() };
        }
        Ok(expr)
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
