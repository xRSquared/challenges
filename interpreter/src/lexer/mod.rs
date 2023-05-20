pub mod token;
use anyhow::Result;
use token::Token;

struct Lexer {
    input: Vec<char>,
    position: usize,      // current position in input (points to current char)
    read_position: usize, // current reading position in input (after current char)
    char: char,           // current char under examination
}

impl Lexer {
    pub fn new(input: Vec<char>) -> Lexer {
        let mut lex = Lexer {
            input,
            position: 0,
            read_position: 0,
            char: 0 as char,
        };

        lex.read_char();
        return lex;
    }
    fn read_char(&mut self) -> () {
        if self.read_position >= self.input.len() {
            self.char = 0 as char; // NOTE: should this be EOF?
        } else {
            self.char = self.input[self.read_position]
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    fn skip_whitespace(&mut self) -> () {
        while [' ', '\t', '\n', '\r'].contains(&self.char) {
            self.read_char();
        }
    }
    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while token::is_letter(self.char) {
            self.read_char();
        }
        return self.input[position..self.position].iter().collect();
    }
    // TODO: handle floats
    fn read_number(&mut self) -> String {
        let position = self.position;
        while token::is_digit(self.char) {
            self.read_char();
        }
        return self.input[position..self.position].iter().collect();
    }
    // TODO: make this an iterator
    fn next_token(&mut self) -> Result<Token> {
        let tok: token::Token;
        self.skip_whitespace();

        match self.char {
            | '=' => tok = Token::Assign,
            | ';' => tok = Token::SemiColon,
            | '(' => tok = Token::LParen,
            | ')' => tok = Token::RParen,
            | ',' => tok = Token::Comma,
            | '+' => tok = Token::Plus,
            | '{' => tok = Token::LBrace,
            | '}' => tok = Token::RBrace,
            | '\0' => tok = Token::EOF,
            | _ => {
                if token::is_letter(self.char) {
                    let ident = self.read_identifier();
                    return Ok(token::LookupIdentifier(&ident));
                } else if token::is_digit(self.char) {
                    return Ok(Token::Int(self.read_number()));
                } else {
                    return Ok(Token::Illegal);
                }
            },
        }
        self.read_char();
        return Ok(tok);
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::{Lexer, Token};

    fn iterate_through_tokens(input: &str, tokens: Vec<Token>) -> Result<()> {
        let mut lexer = Lexer::new(input.chars().collect());

        for token in tokens {
            let lexed_token = lexer.next_token()?;
            println!("{:?},{:?}", token, lexed_token);
            assert_eq!(token, lexed_token);
        }

        return Ok(());
    }

    #[test]
    fn simple_lexing_check() -> Result<()> {
        let input = "=+(){},;";
        let tokens = vec![
            Token::Assign,
            Token::Plus,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::Comma,
            Token::SemiColon,
            Token::EOF,
        ];
        iterate_through_tokens(input, tokens)
    }

    #[test]
    fn assignment_lexing_check() -> Result<()> {
        let input = "let five = 5;
let ten = 10;

let add = fn(x, y) {
  x + y;
};

let result = add(five, ten);

";
        let tokens = vec![
            Token::Let,
            Token::Identifier("five".to_string()),
            Token::Assign,
            Token::Int("5".to_string()),
            Token::SemiColon,
            Token::Let,
            Token::Identifier("ten".to_string()),
            Token::Assign,
            Token::Int("10".to_string()),
            Token::SemiColon,
            Token::Let,
            Token::Identifier("add".to_string()),
            Token::Assign,
            Token::Function,
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::Comma,
            Token::Identifier("y".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Identifier("x".to_string()),
            Token::Plus,
            Token::Identifier("y".to_string()),
            Token::SemiColon,
            Token::RBrace,
            Token::SemiColon,
            Token::Let,
            Token::Identifier("result".to_string()),
            Token::Assign,
            Token::Identifier("add".to_string()),
            Token::LParen,
            Token::Identifier("five".to_string()),
            Token::Comma,
            Token::Identifier("ten".to_string()),
            Token::RParen,
            Token::SemiColon,
        ];
        iterate_through_tokens(input, tokens)
    }
}
