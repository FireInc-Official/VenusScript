use std::collections::HashMap;
use crate::parser::ast::{Program, Node, Variable, Assignment, Expr, BinOp, TypeExpr};
use crate::analyzer::error::{VenusError, VenusErrorHandler};

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
        if let Some(name) = &var.name {
            self.set_var_type(name, &declared_ty);
        }

        match declared_ty.as_str() {
            "if" | "for" | "while" | "return" | "import" => {
                if !var.content.is_empty() {
                    self.enter_scope();
                    
                    if declared_ty == "for" {
                        if let Some(args) = &var.arguments {
                            if let Some(crate::parser::ast::Arg::Positional(Expr::BinaryOp { left, op: BinOp::In, right, .. })) = args.args.first() {
                                if let Expr::Identifier(item_name, _) = left.as_ref() {
                                    let right_ty = self.get_type_of_expr(right);
                                    let item_ty = if right_ty.starts_with("array[") {
                                        right_ty.replace("array[", "").replace("]", "")
                                    } else {
                                        "unknown".to_string()
                                    };
                                    self.set_var_type(item_name, &item_ty);
                                }
                            }
                        }
                    }

                    for n in &var.content { self.visit_node(n); }
                    self.exit_scope();
                }
                return;
            }
            _ => {}
        }

        if let Some(Node::Variable(anon_var)) = var.content.first() {
            if anon_var.name.is_none() {
                if let Some(Node::ExprStmt(val_expr)) = anon_var.content.first() {
                    let val_ty = self.get_type_of_expr(val_expr);
                    if !self.is_compatible(&declared_ty, &val_ty) {
                        self.errors.push(VenusError {
                            title: "Type Mismatch".to_string(),
                            message: format!("Ви намагаєтесь записати '{}' у змінну типу '{}'.", val_ty, declared_ty),
                            hint: Some(format!("Змініть тип змінної '{}' на '{}', або передайте правильне значення.", var.name.as_deref().unwrap_or("<anonymous>"), val_ty)),
                            span: val_expr.span(),
                        });
                    }
                }
            }
        }

        if !var.content.is_empty() {
            self.enter_scope();
            
            if let Some(args) = &var.arguments {
                for arg in &args.args {
                    if let crate::parser::ast::Arg::Typed { ty, name, .. } = arg {
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
        
        // Strict typing: No implicit conversions allowed for scalar primitives.
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
                    match name.as_str() {
                        "vec2" | "vec3" | "vec4" | "print" | "log" => return name.clone(),
                        _ => {}
                    }
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

    fn resolve_binary_op(&mut self, left_ty: &str, right_ty: &str, op: &BinOp, span: &crate::lexer::token::Span) -> String {
        if left_ty == "unknown" || right_ty == "unknown" { return "unknown".to_string(); }

        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                if left_ty == "int" && right_ty == "int" { return "int".to_string(); }
                if left_ty == "float" && right_ty == "float" { return "float".to_string(); }
                if (left_ty == "int" && right_ty == "float") || (left_ty == "float" && right_ty == "int") { return "float".to_string(); }
                if left_ty == "string" && right_ty == "string" && *op == BinOp::Add { return "string".to_string(); }
                
                // Vectors, Matrices, Tensors
                if left_ty.starts_with("vec") && left_ty == right_ty { return left_ty.to_string(); }
                if left_ty.starts_with("vec") && (right_ty == "float" || right_ty == "int") { return left_ty.to_string(); }
                
                if left_ty.starts_with("mat") && left_ty == right_ty { return left_ty.to_string(); }
                if left_ty.starts_with("mat") && (right_ty == "float" || right_ty == "int" || right_ty.starts_with("vec")) { return left_ty.to_string(); }
                
                if left_ty == "tensor" && right_ty == "tensor" { return "tensor".to_string(); }
                if left_ty == "tensor" && (right_ty == "float" || right_ty == "int") { return "tensor".to_string(); }

                self.errors.push(VenusError {
                    title: "Invalid Math Operation".to_string(),
                    message: format!("Неможливо виконати операцію над несумісними типами: '{}' та '{}'.", left_ty, right_ty),
                    hint: Some(format!("Спробуйте використати явне перетворення або broadcasting (наприклад '{} * float').", left_ty)),
                    span: span.clone(),
                });
                "unknown".to_string()
            }
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => "bool".to_string(),
            BinOp::And | BinOp::Or | BinOp::In => "bool".to_string(),
        }
    }
}
