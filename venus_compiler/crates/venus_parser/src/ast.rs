// ═══════════════════════════════════════════════════════════
// AST: The Big List — Everything is a Variable
// ═══════════════════════════════════════════════════════════

use venus_lexer::token::Span;

/// The root of a VenusScript file: just a list of nodes.
/// A program IS the Big List.
#[derive(Debug, Clone)]
pub struct Program {
    pub nodes: Vec<Node>,
}

// ── The Universal Node ─────────────────────────────────────

/// Every item in the Big List is a Node.
/// Most nodes are Variables. Control flow and bare statements
/// also live here because they appear inside Variable content.
#[derive(Debug, Clone)]
pub enum Node {
    /// THE universal entity. A function, a struct, an object,
    /// a UI component, an int — they are ALL Variables.
    Variable(Variable),

    /// Control flow constructs (live inside a Variable's content)
    ForLoop(ForLoop),
    WhileLoop(WhileLoop),
    IfChain(IfChain),
    
    /// Statements (inside func content)
    Return(ReturnStmt),
    Assignment(Assignment),
    
    /// Module import directive (e.g. `import std`)
    Import(ImportStmt),
    
    /// Bare expression as a statement (e.g. a function call)
    ExprStmt(Expr),
}

impl Node {
    pub fn span(&self) -> Span {
        match self {
            Node::Variable(v) => v.span.clone(),
            Node::ForLoop(f) => f.span.clone(),
            Node::WhileLoop(w) => w.span.clone(),
            Node::IfChain(i) => i.span.clone(),
            Node::Return(r) => r.span.clone(),
            Node::Assignment(a) => a.span.clone(),
            Node::Import(i) => i.span.clone(),
            Node::ExprStmt(e) => e.span(),
        }
    }
}

// ═══════════════════════════════════════════════════════════
// THE Variable — the single most important type in the AST
// ═══════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct Variable {
    // ── Element 1: TYPE ──
    pub type_expr: TypeExpr,
    
    // ── Element 2: NAME ──
    pub name: String,
    
    // ── Element 3: ARGUMENTS (optional) ──
    pub arguments: Option<ArgList>,
    
    // ── Element 4: CONTENT ──
    pub value: Option<Expr>,
    pub content: Vec<Node>,
    
    // ── Metadata ──
    pub return_type: Option<TypeExpr>,
    pub decorators: Vec<Decorator>,
    pub uses: Vec<String>,
    pub is_const: bool,
    pub is_export: bool,
    pub is_ref: bool,
    pub span: Span,
}

// ── Type Expressions ───────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named(String),
    Array(Box<TypeExpr>),
}

// ── Arguments ──────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ArgList {
    pub args: Vec<Arg>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Arg {
    Typed {
        ty: TypeExpr,
        name: String,
        is_ref: bool,
        span: Span,
    },
    Named {
        name: String,
        value: Expr,
        span: Span,
    },
    Positional(Expr),
}

// ── Decorators ─────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Decorator {
    pub path: Vec<String>,
    pub span: Span,
}

// ── Control Flow ───────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ForLoop {
    pub var_name: String,
    pub iterable: Expr,
    pub body: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct WhileLoop {
    pub condition: Expr,
    pub body: Vec<Node>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct IfChain {
    pub condition: Expr,
    pub then_body: Vec<Node>,
    pub elif_branches: Vec<(Expr, Vec<Node>)>,
    pub else_body: Option<Vec<Node>>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Expr,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub module_name: String,
    pub items: Vec<String>, // if empty and not is_from, it's `import x`. If `["*"]`, it's `from x import *`.
    pub is_from: bool,
    pub span: Span,
}

// ── Expressions ────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Expr {
    IntLiteral(i64, Span),
    FloatLiteral(f64, Span),
    StringLiteral(String, Span),
    BoolLiteral(bool, Span),
    ArrayLiteral(Vec<Expr>, Span),
    Identifier(String, Span),
    BinaryOp {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
        span: Span,
    },
    UnaryOp {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MemberAccess {
        object: Box<Expr>,
        member: String,
        span: Span,
    },
    IndexAccess {
        object: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnOp {
    Negate,
    Not,
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::IntLiteral(_, s) => s.clone(),
            Expr::FloatLiteral(_, s) => s.clone(),
            Expr::StringLiteral(_, s) => s.clone(),
            Expr::BoolLiteral(_, s) => s.clone(),
            Expr::ArrayLiteral(_, s) => s.clone(),
            Expr::Identifier(_, s) => s.clone(),
            Expr::BinaryOp { left, right, .. } => {
                let l = left.span();
                let r = right.span();
                Span { start: l.start, end: r.end, line: l.line, column: l.column }
            },
            Expr::UnaryOp { operand, span, .. } => {
                let op_s = operand.span();
                let line = if span.line == 0 { op_s.line } else { span.line };
                let col = if span.column == 0 { op_s.column } else { span.column };
                Span { start: span.start, end: op_s.end, line, column: col }
            },
            Expr::Call { callee, span: _, .. } => {
                let c_s = callee.span();
                Span { start: c_s.start, end: c_s.end, line: c_s.line, column: c_s.column }
            },
            Expr::MemberAccess { object, .. } => object.span(),
            Expr::IndexAccess { object, .. } => object.span(),
        }
    }
}
