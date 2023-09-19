use std::fmt::{self, Display, Formatter};

use crate::StringId;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operator {
    LParen,
    RParen,
    LBracket,
    RBracket,
    LDoubleCurly,
    RDoubleCurly,
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    PlusEqual,
    MinusEqual,
    MulEqual,
    DivEqual,
    ModEqual,
    Equal,
    EqualEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    Not,
    And,
    Or,
    Dot,
    Comma,
    Colon,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::LParen => write!(f, "("),
            Self::RParen => write!(f, ")"),
            Self::LBracket => write!(f, "["),
            Self::RBracket => write!(f, "]"),
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Mul => write!(f, "*"),
            Self::Div => write!(f, "/"),
            Self::Mod => write!(f, "%"),
            Self::PlusEqual => write!(f, "+="),
            Self::MinusEqual => write!(f, "-="),
            Self::MulEqual => write!(f, "*="),
            Self::DivEqual => write!(f, "/="),
            Self::ModEqual => write!(f, "%="),
            Self::Equal => write!(f, "="),
            Self::EqualEqual => write!(f, "=="),
            Self::LessThan => write!(f, "<"),
            Self::LessThanOrEqual => write!(f, "<="),
            Self::GreaterThan => write!(f, ">"),
            Self::GreaterThanOrEqual => write!(f, ">="),
            Self::Not => write!(f, "!"),
            Self::And => write!(f, "&&"),
            Self::Or => write!(f, "||"),
            Self::Dot => write!(f, "."),
            Self::Comma => write!(f, ","),
            Self::Colon => write!(f, ":"),
            Self::LDoubleCurly => write!(f, "{{"),
            Self::RDoubleCurly => write!(f, "}}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Value {
    Hex(u8, u8, u8),
    Index(usize),
    Number(u64),
    Float(f64),
    String(StringId),
    Ident(StringId),
    Bool(bool),
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hex(r, g, b) => write!(f, "r:{r} g:{g} b:{b}"),
            Self::Index(idx) => write!(f, "<idx {idx}>"),
            Self::Number(num) => write!(f, "{num}"),
            Self::Float(num) => write!(f, "{num}"),
            Self::String(s) => write!(f, "\"{s}\""),
            Self::Ident(id) => write!(f, "{id}"),
            Self::Bool(b) => write!(f, "{b}"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub(crate) enum Kind {
    For,
    In,
    If,
    Else,
    View,
    Newline,
    Indent(usize),

    Value(Value),
    Op(Operator),

    Eof,
}

impl Kind {
    pub(crate) fn to_token(self, pos: usize) -> Token {
        Token(self, pos)
    }
}

impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::For => write!(f, "for"),
            Self::In => write!(f, "in"),
            Self::If => write!(f, "if"),
            Self::Else => write!(f, "else"),
            Self::View => write!(f, "<view>"),
            Self::Newline => write!(f, "\\n"),
            Self::Indent(s) => write!(f, "<indent {s}>"),
            Self::Value(v) => write!(f, "<value {v}>"),
            Self::Op(o) => write!(f, "<op {o}>"),
            Self::Eof => write!(f, "<Eof>"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Token(pub(crate) Kind, pub(crate) usize);

pub(crate) struct Tokens {
    inner: Vec<Token>,
    index: usize,
    eof: usize,
}

impl Tokens {
    pub fn new(inner: Vec<Token>, eof: usize) -> Self {
        Self { inner, index: 0, eof }
    }

    pub fn consume(&mut self) {
        let _ = self.next();
    }

    pub fn next(&mut self) -> Token {
        match self.inner.get(self.index).copied() {
            Some(token) => {
                self.index += 1;
                token
            }
            None => Token(Kind::Eof, self.eof),
        }
    }

    pub fn next_no_indent(&mut self) -> Token {
        loop {
            let token = self.next();

            if let Kind::Indent(_) = token.0 {
                continue
            }

            break token
        }
    }

    pub fn consume_indent(&mut self) {
        loop {
            if matches!(self.inner.get(self.index), Some(Token(Kind::Indent(_), _))) {
                self.index += 1;
                continue;
            }
            break
        }
    }

    pub fn consume_newlines(&mut self) {
        loop {
            if matches!(self.inner.get(self.index), Some(Token(Kind::Newline, _))) {
                self.index += 1;
                continue;
            }
            break
        }
    }

    pub fn peek(&self) -> Token {
        self.inner.get(self.index).copied().unwrap_or(Token(Kind::Eof, self.eof))
    }

    pub fn peek_skip_indent(&mut self) -> Token {
        loop {
            let token = self.peek();

            if let Kind::Indent(_) = token.0 {
                self.index += 1;
                continue;
            }

            break token
        }
    }
}