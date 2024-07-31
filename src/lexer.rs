use std::iter::Peekable;
use owned_chars::OwnedChars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Operations
    Get, Put, Update, Create, Delete,
    // Keywords
    In, From, Where,
    Table, Database,
    // Type Keywords
    NumberType, TextType,
    // Operators
    Equal, NotEqual,
    LessThan, LessThanOrEqual,
    GreaterThan, GreaterThanOrEqual,
    And, Or, Xor,
    // Misc
    Star, Comma, Period, Colon,
    LeftBracket, RightBracket,
    // Literals
    Integer(i64), Float(f64), String(String),
    None, Identifier(String)
}

pub struct Lexer {
    src: Peekable<OwnedChars>,
    cur: Option<char>
}

impl Lexer {
    pub fn new() -> Self {
        // Placeholder values.
        Lexer{src: OwnedChars::from_string(String::new()).peekable(), cur: None}
    }

    fn next(&mut self) -> Option<char> {
        self.cur = self.src.next();
        self.cur
    }

    fn peek(&mut self) -> Option<&char> {
        self.src.peek()
    }

    fn consume(&mut self, expected: char) -> bool {
        let c = self.next().unwrap();
        c == expected
    }

    // `stop_condition` closures can safely call `unwrap`,
    // on the current char, because this function ensures
    // that it isn't None before calling `stop_condition`.
    fn push_until<F>(&mut self, buffer: &mut String, stop_condition: F)
      where F: Fn(Option<&char>) -> bool {
        while self.peek() != None && !stop_condition(self.peek()) {
            buffer.push(self.next().unwrap());
        }
    }

    fn parse_string(&mut self) -> Option<Token> {
        let mut string = String::from(self.cur.unwrap());
        self.push_until(&mut string, |c: Option<&char>| *c.unwrap() == '"');
        if !self.consume('"') {
            return None;
        }

        Some(Token::String(string))
    }

    fn parse_number(&mut self) -> Token {
        let is_valid_number_char = |c: char| {
            // Support negative, floating
            // point and hexadecimal numbers.
            c.is_numeric()
            || c == '-'
            || c == '.'
            || c == 'x'
            || c == 'a' || c == 'b' || c == 'c'
            || c == 'd' || c == 'e' || c == 'f'
        };

        let mut number = String::from(self.cur.unwrap());
        let stop_condition = |c: Option<&char>| {
            // The Rust Programming Language.
            let value = (*c.unwrap()).to_lowercase().next().unwrap();
            !is_valid_number_char(value)
        };
        self.push_until(&mut number, stop_condition);

        number = number.to_lowercase();
        if number.contains('.') {
            return Token::Float(number.parse::<f64>().unwrap())
        }
        else if number.contains('x') {
            return Token::Integer(
                i64::from_str_radix(number.trim_start_matches("0x"), 16).unwrap());
        }

        Token::Integer(number.parse::<i64>().unwrap())
    }

    fn parse_keyword_or_identifier(&mut self) -> Token {
        let mut string = String::from(self.cur.unwrap());
        self.push_until(&mut string, |c: Option<&char>| !c.unwrap().is_alphanumeric());

        match string.to_lowercase().as_str() {
            "get" => Token::Get,
            "put" => Token::Put,
            "update" => Token::Update,
            "create" => Token::Create,
            "delete" => Token::Delete,
            "in" => Token::In,
            "from" => Token::From,
            "where" => Token::Where,
            "table" => Token::Table,
            "database" => Token::Database,
            "and" => Token::And,
            "or" => Token::Or,
            "xor" => Token::Xor,
            "number" => Token::NumberType,
            "text" => Token::TextType,
            "none" => Token::None,
            _ => Token::Identifier(string)
        }
    }

    // This function needs to be able to give out
    // mutable references to `lexer`, so it's a
    // static method that accepts a mutable Lexer
    // reference.
    pub fn lex(lexer: &mut Lexer, src: String) -> Vec<Token> {
        lexer.src = OwnedChars::from_string(src).peekable();
        lexer.cur = None;

        let mut tokens: Vec<Token> = Vec::new();

        while let Some(c) = lexer.next() {
            match c {
                ' ' | '\r' | '\n'  => continue,
                '*' => tokens.push(Token::Star),
                ',' => tokens.push(Token::Comma),
                '.' => tokens.push(Token::Period),
                '[' => tokens.push(Token::LeftBracket),
                ']' => tokens.push(Token::RightBracket),
                ':' => tokens.push(Token::Colon),
                '"' => {
                    let _ = lexer.next();
                    let string = lexer.parse_string().unwrap(); 
                    tokens.push(string);
                },
                '<' => {
                    if lexer.peek() == Some(&'=') {
                        tokens.push(Token::LessThanOrEqual);
                    }
                    else {
                        tokens.push(Token::LessThan);
                    }
                },
                '>' => {
                    if lexer.peek() == Some(&'=') {
                        tokens.push(Token::GreaterThanOrEqual);
                    }
                    else {
                        tokens.push(Token::GreaterThan);
                    }
                },
                '=' => tokens.push(Token::Equal),
                '!' => {
                    if *(lexer.peek().unwrap()) == '=' {
                        tokens.push(Token::NotEqual);
                    }
                    else {
                        // TODO: error handling...
                    }
                }
                '0'..='9' => tokens.push(lexer.parse_number()),
                _ => tokens.push(lexer.parse_keyword_or_identifier()),
            }
        }
        tokens
    }
}