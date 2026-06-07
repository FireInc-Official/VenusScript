use rustc_hash::FxHashMap;
use std::rc::Rc;
use std::cell::RefCell;
use crate::parser::ast::*;

// ═══════════════════════════════════════════════════════════
// Value: Runtime representation of every variable
// ═══════════════════════════════════════════════════════════

pub type SharedMap = Rc<RefCell<FxHashMap<String, Value>>>;

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<Value>),
    Vec2(f64, f64),
    Vec3(f64, f64, f64),
    Vec4(f64, f64, f64, f64),
    Function {
        name: String,
        params: Vec<String>,
        body: Rc<Vec<Node>>,
        is_native: bool,
    },
    Object(SharedMap),
    Buffer(Rc<RefCell<Vec<u8>>>),
    Tensor { shape: Vec<usize>, data: Rc<RefCell<Vec<f64>>> },
    Signal(Rc<RefCell<Vec<Value>>>),
    Task(Rc<RefCell<TaskState>>),
    ReturnValue(Box<Value>),
    Void,
}

#[derive(Clone)]
pub struct TaskState {
    pub env: Rc<RefCell<Environment>>,
    pub body: Rc<Vec<Node>>,
    pub finished: bool,
}

impl std::fmt::Debug for TaskState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskState {{ finished: {} }}", self.finished)
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Vec2(x, y) => write!(f, "vec2({}, {})", x, y),
            Value::Vec3(x, y, z) => write!(f, "vec3({}, {}, {})", x, y, z),
            Value::Vec4(x, y, z, w) => write!(f, "vec4({}, {}, {}, {})", x, y, z, w),
            Value::Function { name, .. } => write!(f, "<func {}>", name),
            Value::Object(map) => {
                let len = map.borrow().len();
                if len == 0 {
                    write!(f, "<object>")
                } else {
                    write!(f, "<object {{{} members}}>", len)
                }
            }
            Value::Buffer(b) => write!(f, "<buffer {} bytes>", b.borrow().len()),
            Value::Tensor { shape, .. } => write!(f, "<tensor {:?}>", shape),
            Value::Signal(s) => write!(f, "<signal with {} listeners>", s.borrow().len()),
            Value::Task(_) => write!(f, "<task>"),
            Value::ReturnValue(v) => write!(f, "<return {}>", v),
            Value::Void => write!(f, "void"),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Environment: Scoped variable storage
// ═══════════════════════════════════════════════════════════

pub struct Environment {
    pub variables: SharedMap,
    pub parent: Option<Rc<RefCell<Environment>>>,
}

impl Environment {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: Rc::new(RefCell::new(FxHashMap::default())),
            parent: None,
        }))
    }

    pub fn new_with_parent(parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: Rc::new(RefCell::new(FxHashMap::default())),
            parent: Some(parent),
        }))
    }

    pub fn new_with_map_and_parent(map: SharedMap, parent: Rc<RefCell<Environment>>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            variables: map,
            parent: Some(parent),
        }))
    }

    pub fn set(&mut self, name: String, value: Value) {
        self.variables.borrow_mut().insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        if let Some(val) = self.variables.borrow().get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.borrow().get(name)
        } else {
            None
        }
    }

    pub fn set_existing(&mut self, name: &str, value: Value) -> bool {
        if self.variables.borrow().contains_key(name) {
            self.variables.borrow_mut().insert(name.to_string(), value);
            true
        } else if let Some(parent) = &self.parent {
            parent.borrow_mut().set_existing(name, value)
        } else {
            false
        }
    }
}

// ═══════════════════════════════════════════════════════════
// Standard Library (baked into the binary)
// ═══════════════════════════════════════════════════════════

const STD_SOURCE: &str = include_str!("std.vs");

// ═══════════════════════════════════════════════════════════
// Evaluator: Walks the AST and executes
// ═══════════════════════════════════════════════════════════

pub type NativeFunc = Rc<dyn Fn(Vec<Value>) -> Result<Value, String>>;

pub struct Evaluator {
    pub env: Rc<RefCell<Environment>>,
    pub last_span: crate::lexer::token::Span,
    pub native_funcs: FxHashMap<String, NativeFunc>,
    pub last_condition_passed: bool,
}

impl Evaluator {
    pub fn new() -> Self {
        let env = Environment::new();
        let mut eval = Self { 
            env, 
            last_span: crate::lexer::token::Span::default(),
            native_funcs: FxHashMap::default(),
            last_condition_passed: false,
        };
        eval.init_std();
        eval
    }

    fn init_std(&mut self) {
        if let Ok(tokens) = crate::lexer::scanner::Scanner::new(STD_SOURCE).scan_all() {
            if let Ok(program) = crate::parser::parser::Parser::new(tokens).parse() {
                let _ = self.eval_program(&program);
            }
        }
    }



    pub fn eval_program(&mut self, program: &Program) -> Result<(), String> {
        for node in &program.nodes {
            self.eval_node(node)?;
        }
        Ok(())
    }

    // ── Node Evaluation ──

    fn eval_node(&mut self, node: &Node) -> Result<Value, String> {
        self.last_span = node.span().clone();
        match node {
            Node::Variable(var) => self.eval_variable(var),
            Node::ExprStmt(expr) => self.eval_expr(expr),
            Node::Assignment(a) => self.eval_assignment(a),
        }
    }



    // ── Variable ──

    fn eval_variable(&mut self, var: &Variable) -> Result<Value, String> {
        if let TypeExpr::Named(ty) = &var.type_expr {
            match ty.as_str() {
                "func" => return self.eval_func_decl(var),
                "object" | "struct" | "behaviour" => return self.eval_object_decl(var),
                "enum" => return self.eval_enum_decl(var),
                "signal" => return self.eval_signal_decl(var),
                "task" => return self.eval_task_decl(var),
                "if" => return self.eval_if_var(var),
                "elif" => return self.eval_elif_var(var),
                "else" => return self.eval_else_var(var),
                "for" => return self.eval_for_var(var),
                "while" => return self.eval_while_var(var),
                "return" => return self.eval_return_var(var),
                "import" => return self.eval_import_var(var),
                _ => {}
            }
        }

        let mut assigned_val = None;
        if let Some(Node::Variable(anon_var)) = var.content.first() {
            if anon_var.name.is_none() {
                if let Some(Node::ExprStmt(val_expr)) = anon_var.content.first() {
                    assigned_val = Some(self.eval_expr(val_expr)?);
                }
            }
        }

        let val = if let Some(v) = assigned_val {
            v
        } else if let TypeExpr::Named(ty_name) = &var.type_expr {
            let blueprint_opt = self.env.borrow().get(ty_name);
            if let Some(Value::Object(blueprint)) = blueprint_opt {
                let new_obj = Rc::new(RefCell::new(blueprint.borrow().clone()));
                if !var.content.is_empty() {
                    let obj_env = Environment::new_with_map_and_parent(new_obj.clone(), self.env.clone());
                    let old_env = self.env.clone();
                    self.env = obj_env;
                    for child in &var.content {
                        if let Node::Variable(child_var) = child {
                            let res = self.eval_variable(child_var)?;
                            if let Value::ReturnValue(_) = res { return Ok(res); }
                        }
                    }
                    self.env = old_env;
                }
                Value::Object(new_obj)
            } else {
                match ty_name.as_str() {
                    "int" => Value::Int(0),
                    "float" => Value::Float(0.0),
                    "string" => Value::String("".to_string()),
                    "bool" => Value::Bool(false),
                    "vec2" | "vec3" | "vec4" => {
                        let mut vals = vec![0.0, 0.0, 0.0, 0.0];
                        if let Some(args) = &var.arguments {
                            for (i, arg) in args.args.iter().enumerate() {
                                if let Arg::Positional(expr) = arg {
                                    if let Value::Float(f) = self.eval_expr(expr)? {
                                        if i < 4 { vals[i] = f; }
                                    }
                                }
                            }
                        }
                        match ty_name.as_str() {
                            "vec2" => Value::Vec2(vals[0], vals[1]),
                            "vec3" => Value::Vec3(vals[0], vals[1], vals[2]),
                            "vec4" => Value::Vec4(vals[0], vals[1], vals[2], vals[3]),
                            _ => unreachable!(),
                        }
                    }
                    _ => {
                        if !var.content.is_empty() {
                            Value::Object(Rc::new(RefCell::new(FxHashMap::default())))
                        } else {
                            Value::Void
                        }
                    }
                }
            }
        } else if !var.content.is_empty() {
            Value::Object(Rc::new(RefCell::new(FxHashMap::default())))
        } else {
            Value::Void
        };

        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), val.clone());
        Ok(val)
    }

    fn eval_func_decl(&mut self, var: &Variable) -> Result<Value, String> {
        let mut params = Vec::new();
        if let Some(args) = &var.arguments {
            for arg in &args.args {
                if let Arg::Typed { name, .. } = arg {
                    params.push(name.clone());
                }
            }
        }
        
        let mut is_native = var.content.is_empty();
        for dec in &var.decorators {
            if dec.path.join(".") == "native" {
                is_native = true;
                break;
            }
        }

        let func_val = Value::Function {
            name: var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()),
            params,
            body: Rc::new(var.content.clone()),
            is_native,
        };
        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), func_val);
        Ok(Value::Void)
    }

    fn eval_object_decl(&mut self, var: &Variable) -> Result<Value, String> {
        let members = Rc::new(RefCell::new(FxHashMap::default()));
        
        // Push object environment so its properties can reference each other if needed
        let obj_env = Environment::new_with_map_and_parent(members.clone(), self.env.clone());
        let old_env = self.env.clone();
        self.env = obj_env.clone();

        for child in &var.content {
            if let Node::Variable(child_var) = child {
                let res = self.eval_variable(child_var)?;
                if let Value::ReturnValue(_) = res { return Ok(res); }
            }
        }
        
        self.env = old_env;
        
        let obj = Value::Object(members);
        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), obj.clone());
        Ok(obj)
    }

    fn eval_enum_decl(&mut self, var: &Variable) -> Result<Value, String> {
        let enum_map = Rc::new(RefCell::new(FxHashMap::default()));
        let mut index = 0;
        
        for child in &var.content {
            if let Node::ExprStmt(Expr::Identifier(name, _)) = child {
                enum_map.borrow_mut().insert(name.clone(), Value::Int(index));
                index += 1;
            }
        }
        
        let obj = Value::Object(enum_map);
        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), obj.clone());
        Ok(obj)
    }



    fn eval_signal_decl(&mut self, var: &Variable) -> Result<Value, String> {
        let val = Value::Signal(Rc::new(RefCell::new(Vec::new())));
        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), val.clone());
        Ok(val)
    }

    fn eval_task_decl(&mut self, var: &Variable) -> Result<Value, String> {
        let mut params = Vec::new();
        if let Some(args) = &var.arguments {
            for arg in &args.args {
                if let Arg::Typed { name, .. } = arg {
                    params.push(name.clone());
                }
            }
        }
        let task = TaskState {
            env: self.env.clone(),
            body: Rc::new(var.content.clone()),
            finished: false,
        };
        let val = Value::Task(Rc::new(RefCell::new(task)));
        self.env.borrow_mut().set(var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()), val.clone());
        Ok(val)
    }

    // ── Expression Evaluation ──

    fn eval_assignment(&mut self, a: &Assignment) -> Result<Value, String> {
        let raw_val = self.eval_expr(&a.value)?;
        
        match &a.target {
            Expr::Identifier(name, _) => {
                let current_val = self.env.borrow().get(name).unwrap_or(Value::Void);
                let new_val = self.apply_assign_op(&current_val, raw_val, &a.op)?;
                
                if !self.env.borrow_mut().set_existing(name, new_val.clone()) {
                    self.env.borrow_mut().set(name.clone(), new_val.clone());
                }
                Ok(new_val)
            }
            Expr::MemberAccess { object, member, .. } => {
                let obj_val = self.eval_expr(object)?;
                match obj_val {
                    Value::Object(map) => {
                        let current_val = map.borrow().get(member).cloned().unwrap_or(Value::Void);
                        let new_val = self.apply_assign_op(&current_val, raw_val, &a.op)?;
                        map.borrow_mut().insert(member.clone(), new_val.clone());
                        Ok(new_val)
                    }
                    Value::Vec2(x, y) => {
                        let current_val = match member.as_str() { "x" => Value::Float(x), "y" => Value::Float(y), _ => Value::Void };
                        let new_val = self.apply_assign_op(&current_val, raw_val, &a.op)?;
                        let new_f = if let Value::Float(f) = new_val { f } else { 0.0 };
                        let updated = match member.as_str() {
                            "x" => Value::Vec2(new_f, y),
                            "y" => Value::Vec2(x, new_f),
                            _ => return Err("Invalid member".to_string()),
                        };
                        self.set_member_base(object, updated.clone())?;
                        Ok(new_val)
                    }
                    Value::Vec3(x, y, z) => {
                        let current_val = match member.as_str() { "x" => Value::Float(x), "y" => Value::Float(y), "z" => Value::Float(z), _ => Value::Void };
                        let new_val = self.apply_assign_op(&current_val, raw_val, &a.op)?;
                        let new_f = if let Value::Float(f) = new_val { f } else { 0.0 };
                        let updated = match member.as_str() {
                            "x" => Value::Vec3(new_f, y, z),
                            "y" => Value::Vec3(x, new_f, z),
                            "z" => Value::Vec3(x, y, new_f),
                            _ => return Err("Invalid member".to_string()),
                        };
                        self.set_member_base(object, updated.clone())?;
                        Ok(new_val)
                    }
                    Value::Vec4(x, y, z, w) => {
                        let current_val = match member.as_str() { "x"| "r" => Value::Float(x), "y"| "g" => Value::Float(y), "z"| "b" => Value::Float(z), "w"| "a" => Value::Float(w), _ => Value::Void };
                        let new_val = self.apply_assign_op(&current_val, raw_val, &a.op)?;
                        let new_f = if let Value::Float(f) = new_val { f } else { 0.0 };
                        let updated = match member.as_str() {
                            "x" | "r" => Value::Vec4(new_f, y, z, w),
                            "y" | "g" => Value::Vec4(x, new_f, z, w),
                            "z" | "b" => Value::Vec4(x, y, new_f, w),
                            "w" | "a" => Value::Vec4(x, y, z, new_f),
                            _ => return Err("Invalid member".to_string()),
                        };
                        self.set_member_base(object, updated.clone())?;
                        Ok(new_val)
                    }
                    _ => Err("Invalid assignment target".to_string())
                }
            }
            _ => Err("Invalid assignment target".to_string())
        }
    }

    fn set_member_base(&mut self, object_expr: &Expr, updated_val: Value) -> Result<(), String> {
        if let Expr::Identifier(name, _) = object_expr {
            if !self.env.borrow_mut().set_existing(name, updated_val.clone()) {
                self.env.borrow_mut().set(name.clone(), updated_val);
            }
            Ok(())
        } else {
            Err("Nested member assignments on vectors not fully supported yet".to_string())
        }
    }

    fn apply_assign_op(&self, current: &Value, right: Value, op: &AssignOp) -> Result<Value, String> {
        match op {
            AssignOp::Assign => Ok(right),
            AssignOp::AddAssign => self.eval_binary_vals(current, right, BinOp::Add),
            AssignOp::SubAssign => self.eval_binary_vals(current, right, BinOp::Sub),
            AssignOp::MulAssign => self.eval_binary_vals(current, right, BinOp::Mul),
            AssignOp::DivAssign => self.eval_binary_vals(current, right, BinOp::Div),
        }
    }

    // ── Control Flow (As Variables) ──

    fn eval_return_var(&mut self, var: &Variable) -> Result<Value, String> {
        let mut val = Value::Void;
        if let Some(args) = &var.arguments {
            if let Some(Arg::Positional(expr)) = args.args.first() {
                val = self.eval_expr(expr)?;
            }
        } else if let Some(Node::Variable(anon)) = var.content.first() {
            if let Some(Node::ExprStmt(expr)) = anon.content.first() {
                val = self.eval_expr(expr)?;
            }
        }
        Ok(Value::ReturnValue(Box::new(val)))
    }

    fn eval_if_var(&mut self, var: &Variable) -> Result<Value, String> {
        let cond = if let Some(args) = &var.arguments {
            if let Some(Arg::Positional(expr)) = args.args.first() {
                self.eval_expr(expr)?
            } else { Value::Bool(false) }
        } else { Value::Bool(false) };
        
        self.last_condition_passed = false;
        if let Value::Bool(true) = cond {
            self.last_condition_passed = true;
            for child in &var.content {
                let res = self.eval_node(child)?;
                if let Value::ReturnValue(_) = res { return Ok(res); }
            }
        }
        Ok(Value::Void)
    }

    fn eval_elif_var(&mut self, var: &Variable) -> Result<Value, String> {
        if self.last_condition_passed { return Ok(Value::Void); }
        let cond = if let Some(args) = &var.arguments {
            if let Some(Arg::Positional(expr)) = args.args.first() {
                self.eval_expr(expr)?
            } else { Value::Bool(false) }
        } else { Value::Bool(false) };
        
        if let Value::Bool(true) = cond {
            self.last_condition_passed = true;
            for child in &var.content {
                let res = self.eval_node(child)?;
                if let Value::ReturnValue(_) = res { return Ok(res); }
            }
        }
        Ok(Value::Void)
    }

    fn eval_else_var(&mut self, var: &Variable) -> Result<Value, String> {
        if self.last_condition_passed { return Ok(Value::Void); }
        for child in &var.content {
            let res = self.eval_node(child)?;
            if let Value::ReturnValue(_) = res { return Ok(res); }
        }
        Ok(Value::Void)
    }

    fn eval_for_var(&mut self, var: &Variable) -> Result<Value, String> {
        if let Some(args) = &var.arguments {
            if let Some(Arg::Positional(expr)) = args.args.first() {
                if let Expr::BinaryOp { left, op: BinOp::In, right, .. } = expr {
                    if let (Expr::Identifier(item_name, _), iterable_expr) = (left.as_ref(), right.as_ref()) {
                        let iterable = self.eval_expr(iterable_expr)?;
                        if let Value::Array(arr) = iterable {
                            for item in arr {
                                self.env.borrow_mut().set(item_name.clone(), item);
                                for child in &var.content {
                                    let res = self.eval_node(child)?;
                                    if let Value::ReturnValue(_) = res { return Ok(res); }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(Value::Void)
    }

    fn eval_while_var(&mut self, var: &Variable) -> Result<Value, String> {
        loop {
            let cond = if let Some(args) = &var.arguments {
                if let Some(Arg::Positional(expr)) = args.args.first() {
                    self.eval_expr(expr)?
                } else { Value::Bool(false) }
            } else { Value::Bool(false) };
            
            if let Value::Bool(b) = cond {
                if !b { break; }
            } else {
                break;
            }
            
            for child in &var.content {
                let res = self.eval_node(child)?;
                if let Value::ReturnValue(_) = res { return Ok(res); }
            }
        }
        Ok(Value::Void)
    }
    
    fn eval_import_var(&mut self, var: &Variable) -> Result<Value, String> {
        let module_name = var.name.as_deref().unwrap_or("");
        if module_name.is_empty() { return Err("Import requires a module name".to_string()); }
        
        let source = match module_name {
            "std" => STD_SOURCE.to_string(),
            other => {
                let path = format!("{}.vs", other);
                std::fs::read_to_string(&path).map_err(|e| format!("Failed to load module '{}': {}", other, e))?
            }
        };

        let mut scanner = crate::lexer::scanner::Scanner::new(&source);
        let tokens = scanner.scan_all().map_err(|e| format!("Stdlib lex error: {}", e))?;
        let mut parser = crate::parser::parser::Parser::new(tokens);
        let program = parser.parse().map_err(|e| format!("Stdlib parse error: {}", e))?;

        let mut module_eval = Evaluator::new();
        let module_exports = std::rc::Rc::new(std::cell::RefCell::new(rustc_hash::FxHashMap::default()));
        
        for node in &program.nodes {
            if let Node::Variable(child_var) = node {
                let res = module_eval.eval_variable(child_var)?;
                if let Value::ReturnValue(_) = res { return Ok(res); }
                if child_var.is_export {
                    let child_name = child_var.name.as_deref().unwrap_or("<anonymous>");
                    if let Some(val) = module_eval.env.borrow().get(child_name) {
                        module_exports.borrow_mut().insert(child_name.to_string(), val);
                    }
                }
            } else {
                module_eval.eval_node(node)?;
            }
        }

        self.env.borrow_mut().set(module_name.to_string(), Value::Object(module_exports));
        Ok(Value::Void)
    }

    // ═══════════════════════════════════════════════════════════
    // Expression Evaluation
    // ═══════════════════════════════════════════════════════════

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        self.last_span = expr.span().clone();
        match expr {
            Expr::IntLiteral(n, _) => Ok(Value::Int(*n)),
            Expr::FloatLiteral(n, _) => Ok(Value::Float(*n)),
            Expr::StringLiteral(s, _) => Ok(Value::String(s.clone())),
            Expr::BoolLiteral(b, _) => Ok(Value::Bool(*b)),
            Expr::ArrayLiteral(items, _) => {
                let mut arr = Vec::new();
                for item in items {
                    arr.push(self.eval_expr(item)?);
                }
                Ok(Value::Array(arr))
            }
            Expr::Identifier(name, _) => {
                if let Some(v) = self.env.borrow().get(name) {
                    Ok(v)
                } else {
                    Err(format!("Undefined identifier: {}", name))
                }
            }
            Expr::BinaryOp { left, op, right, .. } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary_vals(&l, r, op.clone())
            }
            Expr::UnaryOp { op, operand, .. } => self.eval_unary(op, operand),
            Expr::MemberAccess { object, member, .. } => self.eval_member_access(object, member),
            Expr::IndexAccess { object, index, .. } => self.eval_index_access(object, index),
            Expr::Call { callee, args, .. } => self.eval_call(callee, args),
        }
    }

    fn eval_binary_vals(&self, l: &Value, r: Value, op: BinOp) -> Result<Value, String> {
        macro_rules! math_op {
            ($a:expr, $b:expr, $variant:ident) => {
                match op {
                    BinOp::Add => Ok(Value::$variant($a + $b)),
                    BinOp::Sub => Ok(Value::$variant($a - $b)),
                    BinOp::Mul => Ok(Value::$variant($a * $b)),
                    BinOp::Div => Ok(Value::$variant($a / $b)),
                    BinOp::Mod => Ok(Value::$variant($a % $b)),
                    BinOp::Eq => Ok(Value::Bool($a == $b)),
                    BinOp::NotEq => Ok(Value::Bool($a != $b)),
                    BinOp::Lt => Ok(Value::Bool($a < $b)),
                    BinOp::Gt => Ok(Value::Bool($a > $b)),
                    BinOp::LtEq => Ok(Value::Bool($a <= $b)),
                    BinOp::GtEq => Ok(Value::Bool($a >= $b)),
                    _ => Err("Unsupported binary operation".to_string())
                }
            }
        }
        
        match (l, r) {
            (Value::Int(a), Value::Int(b)) => math_op!(*a, b, Int),
            (Value::Float(a), Value::Float(b)) => math_op!(*a, b, Float),
            
            // Basic Bool ops
            (Value::Bool(a), Value::Bool(b)) => match op {
                BinOp::Eq => Ok(Value::Bool(*a == b)),
                BinOp::NotEq => Ok(Value::Bool(*a != b)),
                BinOp::And => Ok(Value::Bool(*a && b)),
                BinOp::Or => Ok(Value::Bool(*a || b)),
                _ => Err("Unsupported binary operation".to_string())
            },
            
            // Basic String ops
            (Value::String(a), Value::String(b)) => match op {
                BinOp::Add => Ok(Value::String(format!("{}{}", a, b))),
                BinOp::Eq => Ok(Value::Bool(a == &b)),
                _ => Err("Unsupported binary operation".to_string())
            },
            
            // Vectors
            (Value::Vec4(ax, ay, az, aw), Value::Vec4(bx, by, bz, bw)) => match op {
                BinOp::Add => Ok(Value::Vec4(ax + bx, ay + by, az + bz, aw + bw)),
                BinOp::Sub => Ok(Value::Vec4(ax - bx, ay - by, az - bz, aw - bw)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Vec4(ax, ay, az, aw), Value::Float(f)) => match op {
                BinOp::Mul => Ok(Value::Vec4(ax * f, ay * f, az * f, aw * f)),
                BinOp::Div => Ok(Value::Vec4(ax / f, ay / f, az / f, aw / f)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Float(f), Value::Vec4(ax, ay, az, aw)) => match op {
                BinOp::Mul => Ok(Value::Vec4(ax * f, ay * f, az * f, aw * f)),
                _ => Err("Unsupported".to_string())
            },
            
            (Value::Vec3(ax, ay, az), Value::Vec3(bx, by, bz)) => match op {
                BinOp::Add => Ok(Value::Vec3(ax + bx, ay + by, az + bz)),
                BinOp::Sub => Ok(Value::Vec3(ax - bx, ay - by, az - bz)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Vec3(ax, ay, az), Value::Float(f)) => match op {
                BinOp::Mul => Ok(Value::Vec3(ax * f, ay * f, az * f)),
                BinOp::Div => Ok(Value::Vec3(ax / f, ay / f, az / f)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Float(f), Value::Vec3(ax, ay, az)) => match op {
                BinOp::Mul => Ok(Value::Vec3(ax * f, ay * f, az * f)),
                _ => Err("Unsupported".to_string())
            },
            
            (Value::Vec2(ax, ay), Value::Vec2(bx, by)) => match op {
                BinOp::Add => Ok(Value::Vec2(ax + bx, ay + by)),
                BinOp::Sub => Ok(Value::Vec2(ax - bx, ay - by)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Vec2(ax, ay), Value::Float(f)) => match op {
                BinOp::Mul => Ok(Value::Vec2(ax * f, ay * f)),
                BinOp::Div => Ok(Value::Vec2(ax / f, ay / f)),
                _ => Err("Unsupported".to_string())
            },
            (Value::Float(f), Value::Vec2(ax, ay)) => match op {
                BinOp::Mul => Ok(Value::Vec2(ax * f, ay * f)),
                _ => Err("Unsupported".to_string())
            },

            _ => Err("Unsupported binary operation".to_string())
        }
    }

    fn eval_unary(&mut self, op: &UnOp, operand: &Expr) -> Result<Value, String> {
        let val = self.eval_expr(operand)?;
        match (op, val) {
            (UnOp::Negate, Value::Int(n)) => Ok(Value::Int(-n)),
            (UnOp::Negate, Value::Float(n)) => Ok(Value::Float(-n)),
            (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
            _ => Err("Unsupported unary operation".to_string()),
        }
    }

    fn eval_member_access(&mut self, object: &Expr, member: &str) -> Result<Value, String> {
        let obj_val = self.eval_expr(object)?;
        
        match &obj_val {
            Value::Object(map) => {
                if let Some(val) = map.borrow().get(member) {
                    Ok(val.clone())
                } else {
                    Err(format!("Object has no member '{}'", member))
                }
            }
            Value::Vec2(x, y) => match member {
                "x" => Ok(Value::Float(*x)),
                "y" => Ok(Value::Float(*y)),
                "to_string" => Ok(Value::Function { name: "__native_vec2_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                _ => Err(format!("Vec2 has no member '{}'", member))
            },
            Value::Vec3(x, y, z) => match member {
                "x" => Ok(Value::Float(*x)),
                "y" => Ok(Value::Float(*y)),
                "z" => Ok(Value::Float(*z)),
                "to_string" => Ok(Value::Function { name: "__native_vec3_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                _ => Err(format!("Vec3 has no member '{}'", member))
            },
            Value::Vec4(x, y, z, w) => match member {
                "x" | "r" => Ok(Value::Float(*x)),
                "y" | "g" => Ok(Value::Float(*y)),
                "z" | "b" => Ok(Value::Float(*z)),
                "w" | "a" => Ok(Value::Float(*w)),
                "to_string" => Ok(Value::Function { name: "__native_vec4_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                _ => Err(format!("Vec4 has no member '{}'", member))
            },
            Value::Bool(_) => {
                match member {
                    "to_string" => Ok(Value::Function { name: "__native_bool_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Bool has no method '{}'", member)),
                }
            }
            Value::String(_) => {
                match member {
                    "len" => Ok(Value::Function { name: "__native_string_len".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "upper" => Ok(Value::Function { name: "__native_string_upper".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "lower" => Ok(Value::Function { name: "__native_string_lower".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "contains" => Ok(Value::Function { name: "__native_string_contains".to_string(), params: vec!["substr".to_string()], body: Rc::new(vec![]), is_native: true }),
                    "replace" => Ok(Value::Function { name: "__native_string_replace".to_string(), params: vec!["from".to_string(), "to".to_string()], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("String has no method '{}'", member)),
                }
            }
            Value::Array(_) => {
                match member {
                    "len" => Ok(Value::Function { name: "__native_array_len".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "push" => Ok(Value::Function { name: "__native_array_push".to_string(), params: vec!["item".to_string()], body: Rc::new(vec![]), is_native: true }),
                    "pop" => Ok(Value::Function { name: "__native_array_pop".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "clear" => Ok(Value::Function { name: "__native_array_clear".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "remove" => Ok(Value::Function { name: "__native_array_remove".to_string(), params: vec!["index".to_string()], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Array has no method '{}'", member)),
                }
            }
            Value::Int(_) => {
                match member {
                    "to_string" => Ok(Value::Function { name: "__native_int_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    "abs" => Ok(Value::Function { name: "__native_int_abs".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Int has no method '{}'", member)),
                }
            }
            Value::Float(_) => {
                match member {
                    "to_string" => Ok(Value::Function { name: "__native_float_to_string".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Float has no method '{}'", member)),
                }
            }
            Value::Buffer(_) => {
                match member {
                    "read_u8" => Ok(Value::Function { name: "__native_buffer_read_u8".to_string(), params: vec!["index".to_string()], body: Rc::new(vec![]), is_native: true }),
                    "write_u8" => Ok(Value::Function { name: "__native_buffer_write_u8".to_string(), params: vec!["index".to_string(), "value".to_string()], body: Rc::new(vec![]), is_native: true }),
                    "len" => Ok(Value::Function { name: "__native_buffer_len".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Buffer has no method '{}'", member)),
                }
            }
            Value::Signal(_) => {
                match member {
                    "connect" => Ok(Value::Function { name: "__native_signal_connect".to_string(), params: vec!["listener".to_string()], body: Rc::new(vec![]), is_native: true }),
                    "emit" => Ok(Value::Function { name: "__native_signal_emit".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Signal has no method '{}'", member)),
                }
            }
            Value::Task(_) => {
                match member {
                    "resume" => Ok(Value::Function { name: "__native_task_resume".to_string(), params: vec![], body: Rc::new(vec![]), is_native: true }),
                    _ => Err(format!("Task has no method '{}'", member)),
                }
            }
            _ => Err(format!("Cannot access member '{}' on {:?}", member, obj_val)),
        }
    }

    fn eval_index_access(&mut self, object: &Expr, index: &Expr) -> Result<Value, String> {
        let obj_val = self.eval_expr(object)?;
        let idx_val = self.eval_expr(index)?;
        match (obj_val, idx_val) {
            (Value::Array(arr), Value::Int(i)) => {
                let idx = if i < 0 { (arr.len() as i64 + i) as usize } else { i as usize };
                arr.get(idx).cloned().ok_or_else(|| format!("Index {} out of bounds", i))
            }
            _ => Err("Invalid index access".to_string()),
        }
    }

    fn eval_call(&mut self, callee: &Expr, args: &[Expr]) -> Result<Value, String> {
        if let Expr::Identifier(name, _) = callee {
            match name.as_str() {
                "vec2" | "vec3" | "vec4" => {
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(self.eval_expr(arg)?);
                    }
                    return self.call_native(name, &arg_vals, None);
                }
                _ => {}
            }
        }

        let (func_val, receiver) = if let Expr::MemberAccess { object, member, .. } = callee {
            let obj = self.eval_expr(object)?;
            let method = self.eval_member_access(object, member)?;
            (method, Some((*object.clone(), obj)))
        } else {
            (self.eval_expr(callee)?, None)
        };

        let mut arg_vals = Vec::new();
        for arg in args {
            arg_vals.push(self.eval_expr(arg)?);
        }

        if let Value::Function { name, params, body, is_native } = func_val {
            if is_native {
                return self.call_native(&name, &arg_vals, receiver);
            }

            if params.len() != arg_vals.len() {
                return Err(format!("Expected {} args, got {}", params.len(), arg_vals.len()));
            }

            // Method Call Scope Chain:
            // Local Env (params) -> Object Env (properties) -> Global Env (self.env)
            let call_env = if let Some((_, Value::Object(ref obj_map))) = receiver {
                let obj_env = Environment::new_with_map_and_parent(obj_map.clone(), self.env.clone());
                Environment::new_with_parent(obj_env)
            } else {
                Environment::new_with_parent(self.env.clone())
            };

            for (i, param) in params.iter().enumerate() {
                call_env.borrow_mut().set(param.clone(), arg_vals[i].clone());
            }

            let old_env = self.env.clone();
            self.env = call_env;

            let mut ret_val = Value::Void;
            for node in body.as_ref() {
                let res = self.eval_node(&node)?;
                if let Value::ReturnValue(v) = res {
                    ret_val = *v;
                    break;
                }
            }

            self.env = old_env;
            Ok(ret_val)
        } else {
            Err("Attempted to call a non-function".to_string())
        }
    }

    fn call_native(
        &mut self,
        name: &str,
        args: &[Value],
        receiver: Option<(Expr, Value)>,
    ) -> Result<Value, String> {
        if let Some(func) = self.native_funcs.get(name) {
            // Native closures expect `Vec<Value>`
            let mut all_args = args.to_vec();
            if let Some((_, ref r_val)) = receiver {
                all_args.insert(0, r_val.clone()); // pass receiver as first arg
            }
            return func(all_args);
        }

        match name {
            "__native_print" | "__native_console_print" | "print" | "log" => {
                if let Some(msg) = args.first() { println!("{}", msg); }
                Ok(Value::Void)
            }
            "sin" => {
                if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.sin())) } else { Err("Expected float".to_string()) }
            }

            "tan" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.tan())) } else { Err("Expected float".to_string()) } }
            "asin" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.asin())) } else { Err("Expected float".to_string()) } }
            "acos" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.acos())) } else { Err("Expected float".to_string()) } }
            "atan" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.atan())) } else { Err("Expected float".to_string()) } }
            "pow" => { if let (Some(Value::Float(a)), Some(Value::Float(b))) = (args.get(0), args.get(1)) { Ok(Value::Float(a.powf(*b))) } else { Err("Expected 2 floats".to_string()) } }
            "sqrt" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.sqrt())) } else { Err("Expected float".to_string()) } }
            "abs" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.abs())) } else { Err("Expected float".to_string()) } }
            "min" => { if let (Some(Value::Float(a)), Some(Value::Float(b))) = (args.get(0), args.get(1)) { Ok(Value::Float(a.min(*b))) } else { Err("Expected 2 floats".to_string()) } }
            "floor" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.floor())) } else { Err("Expected float".to_string()) } }
            "ceil" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.ceil())) } else { Err("Expected float".to_string()) } }
            "round" => { if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.round())) } else { Err("Expected float".to_string()) } }
            "cos" => {
                if let Some(Value::Float(f)) = args.first() { Ok(Value::Float(f.cos())) } else { Err("Expected float".to_string()) }
            }
            "max" => {
                if let (Some(Value::Float(a)), Some(Value::Float(b))) = (args.get(0), args.get(1)) { Ok(Value::Float(a.max(*b))) } else { Err("Expected 2 floats".to_string()) }
            }
            "type_of" => {
                if let Some(val) = args.first() {
                    let ty_str = match val {
                        Value::Int(_) => "int",
                        Value::Float(_) => "float",
                        Value::String(_) => "string",
                        Value::Bool(_) => "bool",
                        Value::Array(_) => "array",
                        Value::Vec2(..) => "vec2",
                        Value::Vec3(..) => "vec3",
                        Value::Vec4(..) => "vec4",
                        Value::Buffer(_) => "buffer",
                        Value::Object(_) => "object",
                        _ => "unknown"
                    };
                    Ok(Value::String(ty_str.to_string()))
                } else {
                    Err("Expected 1 argument".to_string())
                }
            }
            "assert" => {
                if let Some(Value::Bool(b)) = args.first() {
                    if !b { return Err("Assertion failed".to_string()); }
                    Ok(Value::Void)
                } else { Err("Expected bool".to_string()) }
            }

            "time" => {
                if let Ok(n) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                    Ok(Value::Float(n.as_millis() as f64))
                } else {
                    Ok(Value::Float(0.0))
                }
            }
            "sleep" => {
                if let Some(Value::Float(ms)) = args.first() {
                    std::thread::sleep(std::time::Duration::from_millis(*ms as u64));
                    Ok(Value::Void)
                } else {
                    Err("Expected float ms".to_string())
                }
            }

            "read_text" => {
                if let Some(Value::String(path)) = args.first() {
                    match std::fs::read_to_string(path) {
                        Ok(content) => Ok(Value::String(content)),
                        Err(e) => Err(format!("Failed to read file: {}", e)),
                    }
                } else { Err("Expected string path".to_string()) }
            }
            "write_text" => {
                if let (Some(Value::String(path)), Some(Value::String(content))) = (args.get(0), args.get(1)) {
                    match std::fs::write(path, content) {
                        Ok(_) => Ok(Value::Bool(true)),
                        Err(_) => Ok(Value::Bool(false)),
                    }
                } else { Err("Expected string path and string content".to_string()) }
            }
            "exists" => {
                if let Some(Value::String(path)) = args.first() {
                    Ok(Value::Bool(std::path::Path::new(path).exists()))
                } else { Err("Expected string path".to_string()) }
            }
            "alloc" => {
                if let Some(Value::Int(size)) = args.first() {
                    Ok(Value::Buffer(Rc::new(RefCell::new(vec![0; *size as usize]))))
                } else { Err("Expected int size".to_string()) }
            }
            "__native_string_len" => {
                if let Some((_, Value::String(s))) = &receiver { Ok(Value::Int(s.len() as i64)) } else { Err("Error".to_string()) }
            }
            "__native_string_upper" => {
                if let Some((_, Value::String(s))) = &receiver { Ok(Value::String(s.to_uppercase())) } else { Err("Error".to_string()) }
            }

            "__native_string_lower" => {
                if let Some((_, Value::String(s))) = &receiver { Ok(Value::String(s.to_lowercase())) } else { Err("Error".to_string()) }
            }
            "__native_string_replace" => {
                if let Some((_, Value::String(s))) = &receiver {
                    if let (Some(Value::String(from)), Some(Value::String(to))) = (args.get(0), args.get(1)) {
                        Ok(Value::String(s.replace(from, to)))
                    } else { Err("Err".to_string()) }
                } else { Err("Err".to_string()) }
            }
            "__native_string_contains" => {
                if let Some((_, Value::String(s))) = &receiver {
                    if let Some(Value::String(substr)) = args.first() { Ok(Value::Bool(s.contains(substr.as_str()))) } else { Err("Err".to_string()) }
                } else { Err("Err".to_string()) }
            }
            "__native_array_len" => {
                if let Some((_, Value::Array(arr))) = &receiver { Ok(Value::Int(arr.len() as i64)) } else { Err("Err".to_string()) }
            }
            "__native_array_push" => {
                if let Some((ref expr, Value::Array(mut arr))) = receiver {
                    if let Expr::Identifier(var_name, _) = expr {
                        if let Some(item) = args.first() {
                            arr.push(item.clone());
                            // Need to update the array in env
                            if !self.env.borrow_mut().set_existing(var_name, Value::Array(arr.clone())) {
                                self.env.borrow_mut().set(var_name.clone(), Value::Array(arr)); // fallback
                            }
                            return Ok(Value::Void);
                        }
                    }
                    Err("push failed".to_string())
                } else { Err("Err".to_string()) }
            }

            "__native_array_pop" => {
                if let Some((ref expr, Value::Array(mut arr))) = receiver {
                    if let Expr::Identifier(var_name, _) = expr {
                        let res = arr.pop().unwrap_or(Value::Void);
                        if !self.env.borrow_mut().set_existing(var_name, Value::Array(arr.clone())) {
                            self.env.borrow_mut().set(var_name.clone(), Value::Array(arr));
                        }
                        return Ok(res);
                    }
                    Err("pop failed".to_string())
                } else { Err("Err".to_string()) }
            }
            "__native_array_clear" => {
                if let Some((ref expr, Value::Array(mut arr))) = receiver {
                    if let Expr::Identifier(var_name, _) = expr {
                        arr.clear();
                        if !self.env.borrow_mut().set_existing(var_name, Value::Array(arr.clone())) {
                            self.env.borrow_mut().set(var_name.clone(), Value::Array(arr));
                        }
                        return Ok(Value::Void);
                    }
                    Err("clear failed".to_string())
                } else { Err("Err".to_string()) }
            }
            "__native_array_remove" => {
                if let Some((ref expr, Value::Array(mut arr))) = receiver {
                    if let Expr::Identifier(var_name, _) = expr {
                        if let Some(Value::Int(idx)) = args.first() {
                            let idx = *idx as usize;
                            if idx < arr.len() {
                                arr.remove(idx);
                                if !self.env.borrow_mut().set_existing(var_name, Value::Array(arr.clone())) {
                                    self.env.borrow_mut().set(var_name.clone(), Value::Array(arr));
                                }
                            }
                            return Ok(Value::Void);
                        }
                    }
                    Err("remove failed".to_string())
                } else { Err("Err".to_string()) }
            }
            "__native_int_to_string" => {
                if let Some((_, Value::Int(n))) = &receiver { Ok(Value::String(n.to_string())) } else { Err("Err".to_string()) }
            }
            "__native_int_abs" | "abs" => {
                if let Some((_, Value::Int(n))) = &receiver { Ok(Value::Int(n.abs())) } else { Err("Err".to_string()) }
            }
            "__native_float_to_string" => {
                if let Some((_, Value::Float(n))) = &receiver { Ok(Value::String(n.to_string())) } else { Err("Err".to_string()) }
            }
            "__native_bool_to_string" => {
                if let Some((_, Value::Bool(b))) = &receiver { Ok(Value::String(b.to_string())) } else { Err("Err".to_string()) }
            }
            "__native_vec2_constructor" | "vec2" => {
                let x = if let Some(Value::Float(n)) = args.get(0) { *n } else { 0.0 };
                let y = if let Some(Value::Float(n)) = args.get(1) { *n } else { 0.0 };
                Ok(Value::Vec2(x, y))
            }
            "__native_vec3_constructor" | "vec3" => {
                let x = if let Some(Value::Float(n)) = args.get(0) { *n } else { 0.0 };
                let y = if let Some(Value::Float(n)) = args.get(1) { *n } else { 0.0 };
                let z = if let Some(Value::Float(n)) = args.get(2) { *n } else { 0.0 };
                Ok(Value::Vec3(x, y, z))
            }
            "__native_vec4_constructor" | "vec4" => {
                let x = if let Some(Value::Float(n)) = args.get(0) { *n } else { 0.0 };
                let y = if let Some(Value::Float(n)) = args.get(1) { *n } else { 0.0 };
                let z = if let Some(Value::Float(n)) = args.get(2) { *n } else { 0.0 };
                let w = if let Some(Value::Float(n)) = args.get(3) { *n } else { 0.0 };
                Ok(Value::Vec4(x, y, z, w))
            }
            "__native_vec2_to_string" => {
                if let Some((_, Value::Vec2(x, y))) = &receiver { Ok(Value::String(format!("vec2({}, {})", x, y))) } else { Err("Err".to_string()) }
            }
            "__native_vec3_to_string" => {
                if let Some((_, Value::Vec3(x, y, z))) = &receiver { Ok(Value::String(format!("vec3({}, {}, {})", x, y, z))) } else { Err("Err".to_string()) }
            }
            "__native_vec4_to_string" => {
                if let Some((_, Value::Vec4(x, y, z, w))) = &receiver { Ok(Value::String(format!("vec4({}, {}, {}, {})", x, y, z, w))) } else { Err("Err".to_string()) }
            }
            "__native_math_max" | "max" => {
                match (args.get(0), args.get(1)) {
                    (Some(Value::Int(a)), Some(Value::Int(b))) => Ok(Value::Int(*a.max(b))),
                    (Some(Value::Float(a)), Some(Value::Float(b))) => Ok(Value::Float(a.max(*b))),
                    _ => Err("Err".to_string())
                }
            }
            "__native_math_min" | "min" => {
                match (args.get(0), args.get(1)) {
                    (Some(Value::Int(a)), Some(Value::Int(b))) => Ok(Value::Int(*a.min(b))),
                    (Some(Value::Float(a)), Some(Value::Float(b))) => Ok(Value::Float(a.min(*b))),
                    _ => Err("Err".to_string())
                }
            }
            "__native_buffer_read_u8" => {
                if let Some((_, Value::Buffer(b))) = &receiver {
                    if let Some(Value::Int(idx)) = args.get(0) {
                        let i = *idx as usize;
                        if i < b.borrow().len() { return Ok(Value::Int(b.borrow()[i] as i64)); }
                    }
                }
                Err("Buffer read error".to_string())
            }
            "__native_buffer_write_u8" => {
                if let Some((_, Value::Buffer(b))) = &receiver {
                    if let (Some(Value::Int(idx)), Some(Value::Int(val))) = (args.get(0), args.get(1)) {
                        let i = *idx as usize;
                        if i < b.borrow().len() {
                            b.borrow_mut()[i] = *val as u8;
                            return Ok(Value::Void);
                        }
                    }
                }
                Err("Buffer write error".to_string())
            }
            "__native_buffer_len" => {
                if let Some((_, Value::Buffer(b))) = &receiver { Ok(Value::Int(b.borrow().len() as i64)) } else { Err("Err".to_string()) }
            }
            "__native_signal_connect" => {
                if let Some((_, Value::Signal(s))) = &receiver {
                    if let Some(func) = args.get(0) {
                        s.borrow_mut().push(func.clone());
                        return Ok(Value::Void);
                    }
                }
                Err("Signal connect error".to_string())
            }
            "__native_signal_emit" => {
                if let Some((_, Value::Signal(s))) = &receiver {
                    let listeners = s.borrow().clone();
                    for func in listeners {
                        if let Value::Function { body, is_native: false, .. } = func {
                            let call_env = Environment::new_with_parent(self.env.clone());
                            let old_env = self.env.clone();
                            self.env = call_env;
                            for node in body.as_ref() {
                                let _ = self.eval_node(&node); // Ignore return values from listeners for now
                            }
                            self.env = old_env;
                        }
                    }
                    return Ok(Value::Void);
                }
                Err("Signal emit error".to_string())
            }
            "__native_task_resume" => {
                // To resume a task, we would need to execute its body.
                // Same problem: `call_native` doesn't have an easy way to recursively call `eval_node` without borrowing `self`.
                // Actually, `call_native` receives `&mut self`! So we CAN call `self.eval_node`!
                if let Some((_, Value::Task(t))) = &receiver {
                    let task = t.borrow().clone();
                    if task.finished { return Ok(Value::Void); }
                    
                    let old_env = self.env.clone();
                    self.env = task.env.clone();
                    
                    let mut ret_val = Value::Void;
                    for node in task.body.as_ref() {
                        let res = self.eval_node(&node)?;
                        if let Value::ReturnValue(v) = res {
                            ret_val = *v;
                            break;
                        }
                    }
                    self.env = old_env;
                    t.borrow_mut().finished = true;
                    return Ok(ret_val);
                }
                Err("Task resume error".to_string())
            }
            "__native_system_type_of" | "type_of" => {
                if let Some(val) = args.first() {
                    let type_name = match val {
                        Value::Int(_) => "int", Value::Float(_) => "float", Value::String(_) => "string",
                        Value::Bool(_) => "bool", Value::Array(_) => "array", Value::Function{..} => "func",
                        Value::Object(_) => "object", Value::Vec3(..) => "vec3", Value::Vec4(..) => "vec4",
                        _ => "unknown"
                    };
                    Ok(Value::String(type_name.to_string()))
                } else { Err("Err".to_string()) }
            }
            "__native_system_assert" | "assert" => {
                match args.first() {
                    Some(Value::Bool(true)) => Ok(Value::Void),
                    Some(Value::Bool(false)) => Err("Assertion failed!".to_string()),
                    _ => Err("Err".to_string())
                }
            }
            _ => Err(format!("Unknown native: {}", name))
        }
    }
}
