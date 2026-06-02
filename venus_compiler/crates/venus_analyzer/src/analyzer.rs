use std::collections::HashMap;
use venus_parser::ast::{Program, Node, Variable, Assignment, Expr, BinOp, TypeExpr};
use crate::error::{VenusError, VenusErrorHandler};

pub struct SemanticAnalyzer<'a> {
    scopes: Vec<HashMap<String, String>>, // Variable name -> Type string
    pub errors: Vec<VenusError>,
    error_handler: VenusErrorHandler<'a>,
}

impl<'a> SemanticAnalyzer<'a> {
    pub fn new(source_code: &'a str, file_name: &'a str) -> Self {
        Self {
            scopes: vec![HashMap::new()], // Global scope
            errors: Vec::new(),
            error_handler: VenusErrorHandler::new(source_code, file_name),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> bool {
        for node in &program.nodes {
            self.visit_node(node);
        }
        
        if !self.errors.is_empty() {
            for err in &self.errors {
                self.error_handler.report(err);
            }
            return false; // Has errors
        }
        true // Valid
    }

    fn visit_import(&mut self, imp: &venus_parser::ast::ImportStmt) {
        if imp.module_name == "std" {
            if imp.is_from {
                let global = self.scopes.last_mut().unwrap();
                global.insert("class".to_string(), "object".to_string());
                global.insert("console".to_string(), "object".to_string());
                global.insert("math".to_string(), "object".to_string());
                global.insert("system".to_string(), "object".to_string());
                global.insert("hardware".to_string(), "object".to_string());
                global.insert("UI".to_string(), "object".to_string());
                global.insert("string".to_string(), "object".to_string());
                global.insert("int".to_string(), "object".to_string());
                global.insert("float".to_string(), "object".to_string());
                global.insert("bool".to_string(), "object".to_string());
                global.insert("array".to_string(), "object".to_string());
                global.insert("buffer".to_string(), "object".to_string());
                global.insert("tensor".to_string(), "object".to_string());
                global.insert("signal".to_string(), "object".to_string());
                global.insert("task".to_string(), "object".to_string());
                global.insert("alloc".to_string(), "func".to_string());
                global.insert("spawn".to_string(), "func".to_string());
                global.insert("vec2".to_string(), "struct".to_string());
                global.insert("vec3".to_string(), "struct".to_string());
                global.insert("vec4".to_string(), "struct".to_string());
                global.insert("mat2".to_string(), "struct".to_string());
                global.insert("mat3".to_string(), "struct".to_string());
                global.insert("mat4".to_string(), "struct".to_string());
                global.insert("print".to_string(), "func".to_string());
            } else {
                let global = self.scopes.last_mut().unwrap();
                global.insert("std".to_string(), "object".to_string());
            }
        } else {
            let path = format!("{}.vs", imp.module_name);
            if let Ok(source) = std::fs::read_to_string(&path) {
                if let Ok(tokens) = venus_lexer::scanner::Scanner::new(&source).scan_all() {
                    if let Ok(program) = venus_parser::parser::Parser::new(tokens).parse() {
                        for node in &program.nodes {
                            if let Node::Variable(var) = node {
                                if var.is_export {
                                    if imp.is_from && (imp.items.contains(&"*".to_string()) || imp.items.contains(&var.name)) {
                                        let ty = self.type_expr_to_string(&var.type_expr);
                                        self.scopes.last_mut().unwrap().insert(var.name.clone(), ty);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    fn set_var_type(&mut self, name: &str, ty: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ty.to_string());
        }
    }

    fn get_var_type(&self, name: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }

    fn visit_node(&mut self, node: &Node) {
        match node {
            Node::Variable(var) => self.visit_variable(var),
            Node::Assignment(assign) => self.visit_assignment(assign),
            Node::ExprStmt(expr) => { self.get_type_of_expr(expr); }
            Node::IfChain(if_chain) => {
                let cond_ty = self.get_type_of_expr(&if_chain.condition);
                if cond_ty != "bool" && cond_ty != "unknown" {
                    self.errors.push(VenusError {
                        title: "Type Mismatch".to_string(),
                        message: format!("Умова 'if' очікує 'bool', але отримано '{}'", cond_ty),
                        hint: Some("Використовуйте оператори порівняння (==, <, >).".to_string()),
                        span: if_chain.condition.span(),
                    });
                }
                self.enter_scope();
                for n in &if_chain.then_body { self.visit_node(n); }
                self.exit_scope();
                
                for (cond, body) in &if_chain.elif_branches {
                    self.get_type_of_expr(cond);
                    self.enter_scope();
                    for n in body { self.visit_node(n); }
                    self.exit_scope();
                }
                if let Some(else_body) = &if_chain.else_body {
                    self.enter_scope();
                    for n in else_body { self.visit_node(n); }
                    self.exit_scope();
                }
            }
            Node::WhileLoop(while_loop) => {
                self.get_type_of_expr(&while_loop.condition);
                self.enter_scope();
                for n in &while_loop.body { self.visit_node(n); }
                self.exit_scope();
            }
            Node::ForLoop(for_loop) => {
                let iter_ty = self.get_type_of_expr(&for_loop.iterable);
                let element_ty = if iter_ty.starts_with("array[") {
                    iter_ty.replace("array[", "").replace("]", "")
                } else {
                    "unknown".to_string()
                };
                self.enter_scope();
                self.set_var_type(&for_loop.var_name, &element_ty);
                for n in &for_loop.body { self.visit_node(n); }
                self.exit_scope();
            }
            Node::Import(imp) => self.visit_import(imp),
            _ => {} // Ignore returns for now
        }
    }

    fn type_expr_to_string(&self, expr: &TypeExpr) -> String {
        match expr {
            TypeExpr::Named(n) => n.clone(),
            TypeExpr::Array(inner) => format!("array[{}]", self.type_expr_to_string(inner)),
        }
    }

    fn visit_variable(&mut self, var: &Variable) {
        let declared_ty = self.type_expr_to_string(&var.type_expr);
        self.set_var_type(&var.name, &declared_ty);

        if let Some(val_expr) = &var.value {
            let val_ty = self.get_type_of_expr(val_expr);
            if !self.is_compatible(&declared_ty, &val_ty) {
                self.errors.push(VenusError {
                    title: "Type Mismatch".to_string(),
                    message: format!("Ви намагаєтесь записати '{}' у змінну типу '{}'.", val_ty, declared_ty),
                    hint: Some(format!("Змініть тип змінної '{}' на '{}', або передайте правильне значення.", var.name, val_ty)),
                    span: val_expr.span(),
                });
            }
        }

        if !var.content.is_empty() {
            self.enter_scope();
            
            if let Some(args) = &var.arguments {
                for arg in &args.args {
                    if let venus_parser::ast::Arg::Typed { ty, name, .. } = arg {
                        let arg_ty = self.type_expr_to_string(ty);
                        self.set_var_type(name, &arg_ty);
                    }
                }
            }

            for n in &var.content {
                if declared_ty == "enum" {
                    continue; // Skip analyzing enum properties as normal expressions
                }
                self.visit_node(n);
            }
            self.exit_scope();
        }
    }

    fn visit_assignment(&mut self, assign: &Assignment) {
        let target_ty = self.get_type_of_expr(&assign.target);
        let val_ty = self.get_type_of_expr(&assign.value);

        if !self.is_compatible(&target_ty, &val_ty) {
            self.errors.push(VenusError {
                title: "Invalid Assignment".to_string(),
                message: format!("Неможливо записати значення типу '{}' у ціль типу '{}'.", val_ty, target_ty),
                hint: Some("Перевірте типи даних.".to_string()),
                span: assign.value.span(),
            });
        }
    }

    fn is_compatible(&self, expected: &str, actual: &str) -> bool {
        if expected == "unknown" || actual == "unknown" { return true; } // Fallback to avoid cascading errors
        if actual == "array[unknown]" && expected.starts_with("array[") { return true; }
        if expected == actual { return true; }
        
        // Allowed implicit conversions
        if expected == "float" && actual == "int" { return true; }
        
        false
    }

    fn get_type_of_expr(&mut self, expr: &Expr) -> String {
        match expr {
            Expr::IntLiteral(..) => "int".to_string(),
            Expr::FloatLiteral(..) => "float".to_string(),
            Expr::StringLiteral(..) => "string".to_string(),
            Expr::BoolLiteral(..) => "bool".to_string(),
            Expr::ArrayLiteral(items, _) => {
                if items.is_empty() { return "array[unknown]".to_string(); }
                let first_ty = self.get_type_of_expr(&items[0]);
                format!("array[{}]", first_ty)
            }
            Expr::Identifier(name, span) => {
                if let Some(ty) = self.get_var_type(name) {
                    ty
                } else {
                    self.errors.push(VenusError {
                        title: "Undefined Variable".to_string(),
                        message: format!("Змінна '{}' не знайдена у цій області видимості.", name),
                        hint: Some("Переконайтеся, що ви ініціалізували змінну перед використанням.".to_string()),
                        span: span.clone(),
                    });
                    "unknown".to_string()
                }
            }
            Expr::BinaryOp { left, op, right, .. } => {
                let left_ty = self.get_type_of_expr(left);
                let right_ty = self.get_type_of_expr(right);
                self.resolve_binary_op(&left_ty, &right_ty, op, &expr.span())
            }
            Expr::MemberAccess { object, member, span: _ } => {
                let _obj_ty = self.get_type_of_expr(object);
                // Currently returning 'unknown' for members because we need to look up object declarations
                // to know their actual fields. For now, allow everything via 'unknown'.
                if member == "len" { return "int".to_string(); }
                if member == "to_string" { return "string".to_string(); }
                "unknown".to_string()
            }
            Expr::Call { callee, args, .. } => {
                let callee_ty = self.get_type_of_expr(callee);
                for arg in args { self.get_type_of_expr(arg); }
                
                if let Expr::Identifier(name, _) = callee.as_ref() {
                    match name.as_str() {
                        "alloc" => return "buffer".to_string(),
                        "spawn" => return "task".to_string(),
                        "tensor" => return "tensor".to_string(),
                        "signal" => return "signal".to_string(),
                        _ => {}
                    }
                }

                if callee_ty == "func" {
                    "unknown".to_string()
                } else {
                    match callee_ty.as_str() {
                        "unknown" => "unknown".to_string(),
                        _ => callee_ty
                    }
                }
            }
            Expr::IndexAccess { object, index, .. } => {
                let obj_ty = self.get_type_of_expr(object);
                self.get_type_of_expr(index);
                if obj_ty.starts_with("array[") {
                    obj_ty.replace("array[", "").replace("]", "")
                } else {
                    "unknown".to_string()
                }
            }
            _ => "unknown".to_string()
        }
    }

    fn resolve_binary_op(&mut self, left_ty: &str, right_ty: &str, op: &BinOp, span: &venus_lexer::token::Span) -> String {
        if left_ty == "unknown" || right_ty == "unknown" { return "unknown".to_string(); }

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if left_ty == "int" && right_ty == "int" { return "int".to_string(); }
                if left_ty == "float" && right_ty == "float" { return "float".to_string(); }
                if (left_ty == "int" && right_ty == "float") || (left_ty == "float" && right_ty == "int") { return "float".to_string(); }
                if left_ty == "string" && right_ty == "string" && *op == BinOp::Add { return "string".to_string(); }
                
                // Vectors
                if left_ty.starts_with("vec") && left_ty == right_ty { return left_ty.to_string(); }
                if left_ty.starts_with("vec") && (right_ty == "float" || right_ty == "int") { return left_ty.to_string(); }

                self.errors.push(VenusError {
                    title: "Invalid Math Operation".to_string(),
                    message: format!("Неможливо виконати операцію над несумісними типами: '{}' та '{}'.", left_ty, right_ty),
                    hint: Some("Використовуйте сумісні типи для математики.".to_string()),
                    span: span.clone(),
                });
                "unknown".to_string()
            }
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => "bool".to_string(),
            BinOp::And | BinOp::Or => "bool".to_string(),
        }
    }
}
