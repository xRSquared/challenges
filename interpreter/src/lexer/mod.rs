pub mod token;
use token::Token;
use anyhow::Result;

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
    pub fn read_char(&mut self) -> () {
        if self.read_position >= self.input.len() {
            self.char = 0 as char; // NOTE: should this be EOF?
        } else {
            self.char = self.input[self.read_position]
        }
        self.position = self.read_position;
        self.read_position += 1;
    }
    // TODO: make this an iterator
    fn next_token(&mut self) -> Result<Token> {
        let tok: token::Token;

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
            | _ => todo!(),
        }
        self.read_char();
        return Ok(tok);
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::{Token, Lexer};

    #[test]
    fn get_next_token() -> Result<()> {
        let input = "=+(){},;";
        let mut lexer = Lexer::new(input.chars().collect());

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

        for token in tokens {
            let lexed_token = lexer.next_token()?;
            println!("{:?},{:?}",token,lexed_token);
            assert_eq!(token, lexed_token);
        }


        return Ok(());
    }
}
