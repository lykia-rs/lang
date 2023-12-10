use phf::phf_map;
use serde::{Deserialize, Serialize};

use super::ast::Literal;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Symbol {
    Comma,
    Colon,
    Semicolon,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    Bang,
    Dot,
    Minus,
    Plus,
    Slash,
    Star,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    Str,
    Num,
    Null,
    False,
    True,
    //
    Identifier { dollar: bool },
    //
    Symbol(Symbol),
    Keyword(Keyword),
    SqlKeyword(SqlKeyword),
    //
    Eof,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Keyword {
    And,
    Or,
    Class,
    Else,
    Fun,
    If,
    Break,
    Continue,
    Return,
    Super,
    This,
    Var,
    While,
    For,
    Loop,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum SqlKeyword {
    /*
        Bool,
        Boolean,
        Char,
        String,
        Varchar,
        Text,
        Time,
        Int,
        Integer,
        Float,
        Double,
    */
    //
    Begin,
    Transaction,
    Rollback,
    Commit,
    //
    Where,
    Having,
    Asc,
    Desc,
    Order,
    By,
    Explain,
    Is,
    Not,
    Null,
    Offset,
    Like,
    Limit,
    And,
    Or,
    //
    Join,
    Inner,
    Outer,
    Right,
    Left,
    On,
    //
    Create,
    Insert,
    Update,
    Delete,
    Drop,
    Into,
    Values,
    Index,
    Table,
    //
    Select,
    From,
    As,
    //
    Cross,
    Default,
    Group,
    Key,
    Of,
    Only,
    Primary,
    References,
    Set,
    System,
    Unique,
    Read,
    Write,
    //
    Union,
    All,
    Intersect,
    Except,
    Distinct,
}

#[macro_export]
macro_rules! kw {
    ($val: expr) => {
        TokenType::Keyword($val)
    };
}

#[macro_export]
macro_rules! skw {
    ($val: expr) => {
        TokenType::SqlKeyword($val)
    };
}

#[macro_export]
macro_rules! sym {
    ($val: expr) => {
        TokenType::Symbol($val)
    };
}

pub static SYMBOLS: phf::Map<char, TokenType> = phf_map! {
    '(' => sym!(Symbol::LeftParen),
    ')' => sym!(Symbol::RightParen),
    '{' => sym!(Symbol::LeftBrace),
    '}' => sym!(Symbol::RightBrace),
    ',' => sym!(Symbol::Comma),
    ':' => sym!(Symbol::Colon),
    '.' => sym!(Symbol::Dot),
    '-' => sym!(Symbol::Minus),
    '+' => sym!(Symbol::Plus),
    ';' => sym!(Symbol::Semicolon),
    '*' => sym!(Symbol::Star),
};

pub static CASE_SNS_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    "and" => kw!(Keyword::And),
    "class" => kw!(Keyword::Class),
    "else" => kw!(Keyword::Else),
    "for" => kw!(Keyword::For),
    "fun" => kw!(Keyword::Fun),
    "if" => kw!(Keyword::If),
    "or" => kw!(Keyword::Or),
    "break" => kw!(Keyword::Break),
    "continue" => kw!(Keyword::Continue),
    "return" => kw!(Keyword::Return),
    "super" => kw!(Keyword::Super),
    "this" => kw!(Keyword::This),
    "var" => kw!(Keyword::Var),
    "while" => kw!(Keyword::While),
    "loop" => kw!(Keyword::Loop),
    //
    "null" =>  TokenType::Null,
    "false" => TokenType::False,
    "true" => TokenType::True,
};

pub static CASE_INS_KEYWORDS: phf::Map<&'static str, TokenType> = phf_map! {
    //
    "ALL" => skw!(SqlKeyword::All),
    "DISTINCT" => skw!(SqlKeyword::Distinct),
    "UNION" => skw!(SqlKeyword::Union),
    "INTERSECT" => skw!(SqlKeyword::Intersect),
    "EXCEPT" => skw!(SqlKeyword::Except),
    "BEGIN" => skw!(SqlKeyword::Begin),
    "TRANSACTION" => skw!(SqlKeyword::Transaction),
    "ROLLBACK" => skw!(SqlKeyword::Rollback),
    "COMMIT" => skw!(SqlKeyword::Commit),
    "WHERE" => skw!(SqlKeyword::Where),
    "HAVING" => skw!(SqlKeyword::Having),
    "ASC" => skw!(SqlKeyword::Asc),
    "DESC" => skw!(SqlKeyword::Desc),
    "ORDER" => skw!(SqlKeyword::Order),
    "BY" => skw!(SqlKeyword::By),
    "AND" => skw!(SqlKeyword::And),
    "OR" => skw!(SqlKeyword::Or),
    "EXPLAIN" => skw!(SqlKeyword::Explain),
    "IS" => skw!(SqlKeyword::Is),
    "NOT" => skw!(SqlKeyword::Not),
    "NULL" => skw!(SqlKeyword::Null),
    "OFFSET" => skw!(SqlKeyword::Offset),
    "LIMIT" => skw!(SqlKeyword::Limit),
    "LIKE" => skw!(SqlKeyword::Like),
    "JOIN" => skw!(SqlKeyword::Join),
    "INNER" => skw!(SqlKeyword::Inner),
    "OUTER" => skw!(SqlKeyword::Outer),
    "RIGHT" => skw!(SqlKeyword::Right),
    "LEFT" => skw!(SqlKeyword::Left),
    "ON" => skw!(SqlKeyword::On),
    "CREATE" => skw!(SqlKeyword::Create),
    "INSERT" => skw!(SqlKeyword::Insert),
    "UPDATE" => skw!(SqlKeyword::Update),
    "DELETE" => skw!(SqlKeyword::Delete),
    "DROP" => skw!(SqlKeyword::Drop),
    "INTO" => skw!(SqlKeyword::Into),
    "VALUES" => skw!(SqlKeyword::Values),
    "INDEX" => skw!(SqlKeyword::Index),
    "SELECT" => skw!(SqlKeyword::Select),
    "FROM" => skw!(SqlKeyword::From),
    "AS" => skw!(SqlKeyword::As),
    "CROSS" => skw!(SqlKeyword::Cross),
    "DEFAULT" => skw!(SqlKeyword::Default),
    "GROUP" => skw!(SqlKeyword::Group),
    "KEY" => skw!(SqlKeyword::Key),
    "OF" => skw!(SqlKeyword::Of),
    "ONLY" => skw!(SqlKeyword::Only),
    "PRIMARY" => skw!(SqlKeyword::Primary),
    "REFERENCES" => skw!(SqlKeyword::References),
    "SET" => skw!(SqlKeyword::Set),
    "SYSTEM" => skw!(SqlKeyword::System),
    "TABLE" => skw!(SqlKeyword::Table),
    "UNIQUE" => skw!(SqlKeyword::Unique),
    "READ" => skw!(SqlKeyword::Read),
    "WRITE" => skw!(SqlKeyword::Write),
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Token {
    pub tok_type: TokenType,
    pub literal: Option<Literal>,
    pub lexeme: Option<String>,
    pub span: Span,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: u32,
    pub line_end: u32,
}

pub trait Spanned {
    fn get_span(&self) -> Span;
}

impl Spanned for Span {
    fn get_span(&self) -> Span {
        *self
    }
}

impl Span {
    pub fn default() -> Span {
        Span {
            start: 0,
            end: 0,
            line: 0,
            line_end: 0,
        }
    }
    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line.min(other.line),
            line_end: self.line_end.min(other.line_end),
        }
    }
}
