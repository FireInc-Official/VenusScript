// ═══════════════════════════════════════════════════════════
// Token: the atoms that the scanner produces
// ═══════════════════════════════════════════════════════════

/// Pinpoints a token in source code for error messages
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Span {
    pub start: usize,       // byte offset from file start
    pub end: usize,
    pub line: u32,          // 1-indexed
    pub column: u32,        // 1-indexed
}

/// A single lexical token with its source location
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,     // raw source text
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ── Structure (the heartbeat of indentation-based parsing) ──
    Newline,                // end of a logical line
    Indent,                 // indentation level increased (→ start of content)
    Dedent,                 // indentation level decreased (→ end of content)

    // ── Literals (raw data values) ──
    IntLiteral(i64),        // 42, -7, 0xFF
    FloatLiteral(f64),      // 3.14, -0.5
    StringLiteral(String),  // "hello world"
    BoolLiteral(bool),      // true, false

    // ── Identifier (variable names, type names, everything) ──
    Identifier(String),     // age, MainScreen, Vector3, players

    // ── Primitive Type Keywords (Value Types — no () allowed) ──
    KwInt,                  // int
    KwFloat,                // float  
    KwString,               // string
    KwBool,                 // bool

    // ── Composite Type Keywords ──
    KwStruct,               // struct
    KwObject,               // object
    KwBehaviour,            // behaviour

    // ── Callable Type Keyword ──
    KwFunc,                 // func

    // ── Memory & Reference ──
    KwBuffer,               // buffer
    KwRef,                  // ref (modifier)

    // ── Grouping & States ──
    KwEnum,                 // enum

    // ── Async & Events ──
    KwTask,                 // task
    KwSignal,               // signal

    // ── SIMD & AI Math ──
    KwVec2, KwVec3, KwVec4,
    KwMat2, KwMat3, KwMat4,
    KwTensor,               // tensor

    // ── Modifier Keywords ──
    KwConst,                // const
    KwExport,               // export (visibility modifier)
    KwExclude,              // exclude (private modifier)
    KwUses,                 // struct Player uses IEntity

    // ── Module Keywords ──
    KwImport,               // import std
    KwFrom,                 // from std import *

    // ── Control Flow Keywords ──
    KwIf,
    KwElse,
    KwElif,                 // else if shorthand
    KwFor,
    KwWhile,
    KwIn,
    KwReturn,

    // ── Operators ──
    Plus,                   // +
    Minus,                  // -
    Star,                   // *
    Slash,                  // /
    Percent,                // %
    
    Equal,                  // =   (assignment / named arg)
    PlusEqual,              // +=
    MinusEqual,             // -=
    StarEqual,              // *=
    SlashEqual,             // /=
    
    EqualEqual,             // ==  (comparison)
    BangEqual,              // !=
    Less,                   // <
    Greater,                // >
    LessEqual,              // <=
    GreaterEqual,           // >=
    
    Arrow,                  // ->  (return type annotation)
    
    And,                    // &&
    Or,                     // ||
    Bang,                   // !
    
    Dot,                    // .   (member access, decorator paths)
    At,                     // @   (decorator prefix)

    // ── Delimiters ──
    LeftParen,              // (   ← arguments start
    RightParen,             // )   ← arguments end
    LeftBracket,            // [   ← array literal / index
    RightBracket,           // ]
    Comma,                  // ,   ← argument separator
    Colon,                  // :   (reserved for future slicing)

    // ── End of File ──
    Eof,
}

impl TokenKind {
    /// Check if this token is a type keyword that starts a Variable declaration
    pub fn is_type_keyword(&self) -> bool {
        matches!(self,
            TokenKind::KwInt | TokenKind::KwFloat | TokenKind::KwString |
            TokenKind::KwBool | TokenKind::KwStruct | TokenKind::KwObject |
            TokenKind::KwBehaviour | TokenKind::KwFunc |
            TokenKind::KwBuffer | TokenKind::KwEnum | TokenKind::KwTask | TokenKind::KwSignal |
            TokenKind::KwVec2 | TokenKind::KwVec3 | TokenKind::KwVec4 |
            TokenKind::KwMat2 | TokenKind::KwMat3 | TokenKind::KwMat4 |
            TokenKind::KwTensor |
            TokenKind::KwIf | TokenKind::KwElse | TokenKind::KwElif |
            TokenKind::KwFor | TokenKind::KwWhile | TokenKind::KwReturn |
            TokenKind::KwImport | TokenKind::KwFrom |
            TokenKind::Identifier(_)
        )
    }
}
