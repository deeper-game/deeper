use logos::{Lexer, Logos};

#[derive(Logos, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Token {
    #[error]
    #[regex(r"[ \t\f]+", logos::skip)]
    Error,

    #[regex("//[^\r\n]*", priority = 2)]
    #[regex("//[^\n]*", priority = 1)]
    Comment,

    #[token("\n")]
    #[token("\r\n")]
    Newline,

    #[token(" ")]
    Space,

    #[regex("[+-]?[0-9][0-9_]*")]
    #[regex("[+-]?0b[0-1][0-1_]*")]
    #[regex("[+-]?0x[0-9a-fA-F][0-9a-fA-F_]*")]
    Int,
    #[token("true")]
    #[token("false")]
    Bool,

    #[token("let")]
    Let,

    #[token("struct")]
    Struct,
    #[token("enum")]
    Enum,

    #[regex("_[A-Za-z0-9_]+|[A-Za-z][A-Za-z0-9_]*")]
    Ident,

    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,

    #[token("while")]
    While,
    #[token("for")]
    For,

    #[token("return")]
    Return,
    #[token("continue")]
    Continue,
    #[token("break")]
    Break,

    #[token("match")]
    Match,

    #[token("=")]
    Equal,
    #[token("==")]
    DoubleEqual,
    #[token("!=")]
    NotEqual,
    #[token(">=")]
    GreaterThanEqual,
    #[token("<=")]
    LessThanEqual,
    #[token("<")]
    LeftCaret,
    #[token(">")]
    RightCaret,

    #[token("!")]
    Bang,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    ForwardSlash,
    #[token("%")]
    PercentSign,
    #[token("^")]
    Caret,

    #[token("<<")]
    LeftShift,
    #[token(">>")]
    RightShift,
    #[token("&")]
    Ampersand,
    #[token("|")]
    Pipe,

    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("[")]
    LeftSquareBracket,
    #[token("]")]
    RightSquareBracket,
    #[token("{")]
    LeftCurlyBracket,
    #[token("}")]
    RightCurlyBracket,

    #[token("->")]
    RightArrow,
    #[token("<-")]
    LeftArrow,
    #[token("=>")]
    RightFatArrow,
    //#[token("<=")] Can match same as LessThanEqual
    //LeftFatArrow,

    #[token("@")]
    At,
    #[token(",")]
    Comma,
    #[token(";")]
    Semicolon,
    #[token(":")]
    Colon,
    #[token(".")]
    Dot,
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
           _ => todo!()
        }
    }
}
