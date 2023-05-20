#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Illegal,
    EOF,

    // identifiers + literals
    Identifier(String),
    Int(String),

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

pub fn LookupIdentifier(ident: &str) -> Token {
    let tok: Token;
    match ident {
        | "fn" => tok = Token::Function,
        | "let" => tok = Token::Let,
        | _ => tok = Token::Identifier(ident.to_string()),
    }
    return tok;
}

// NOTE: can try to add ? or ! here like in rust
pub fn is_letter(char: char) -> bool {
    if ('a'..='z').contains(&char) || ('A'..='Z').contains(&char) || char == '_' {
        return true;
    }
    return false;
}
pub fn is_digit(char: char) -> bool {
    if ('0'..='9').contains(&char) {
        return true;
    }
    return false;
}
