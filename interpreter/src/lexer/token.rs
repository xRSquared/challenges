#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Illegal,
    EOF,

    // identifiers + literals
    Ident,
    Int,

    //operatorS
    Assign,
    Plus,

    // Delimiters
    Comma,
    SemiColon,
    LParen,
    RParen,
    LBrace,
    RBrace,

    //keywords
    Function,
    Let,
}
