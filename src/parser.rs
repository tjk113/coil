use crate::{FieldValue, FieldType, Column};
use crate::lexer::*;

#[derive(Debug, PartialEq)]
pub enum Operation {
    Get,
    Put,
    Update,
    Create,
    Delete
}

// This is largely a copy of Token,
// but only including the operators
// and literals.
#[derive(Debug)]
pub enum ExpressionType {
    // Unary
    Not, Negate, Positive,
    // Binary
    Equal, NotEqual,
    LessThan, LessThanOrEqual,
    GreaterThan, GreaterThanOrEqual,
    And, Or, Xor,
    // Literals
    Number(f64), String(String),
    None, Identifier(String)
}

#[derive(Debug)]
pub struct Expression<'a> {
    // Literal expressions only use `expression_type`.
    pub expression_type: ExpressionType,
    // Unary expressions only use `l_operand`.
    pub l_operand: Option<&'a Expression<'a>>,
    pub r_operand: Option<&'a Expression<'a>>
}

#[derive(Debug)]
pub struct Query<'a> {
    pub operation: Operation,
    pub database: Option<String>,
    pub table: Option<String>,
    pub values: Option<Vec<FieldValue>>,
    pub columns: Option<Vec<Column>>,
    pub condition: Option<Expression<'a>>,
}

impl<'a> Query<'a> {
    pub fn new(operation: Operation) -> Self {
        Query{operation: operation, database: None, table: None, values: None, columns: None, condition: None}
    }
}

// Just your good ol' fashioned recursive descent parser.
pub struct Parser {
    tokens: Vec<Token>
}

impl Parser {
    pub fn new() -> Self {
        // Placeholder value.
        Parser{tokens: Vec::new()}
    }

    pub fn parse(parser: &mut Parser, tokens: Vec<Token>) -> Query {
        parser.tokens = tokens;
        parser.tokens.reverse();
        parser.parse_query().unwrap()
    }

    fn next(&mut self) -> Option<Token> {
        self.tokens.pop()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.last()
    }

    fn consume(&mut self, expected: &[Token]) -> bool {
        if self.peek() == None {
            return false;
        }
        for token in expected {
            if *self.peek().unwrap() == *token {
                let _ = self.next();
                return true;
            }
        }
        false
    }

    fn check(&self, expected: &[Token]) -> bool {
        if self.peek() == None {
            return false;
        }
        for token in expected {
            if *self.peek().unwrap() == *token {
                return true;
            }
        }
        false
    }

    fn parse_query(&mut self) -> Option<Query> {
        match self.next()? {
            Token::Get => self.parse_get_query(),
            Token::Put => self.parse_put_query(),
            Token::Update => self.parse_update_query(),
            Token::Create => self.parse_create_query(),
            Token::Delete => self.parse_delete_query(),
            _ => None
        }
    }

    fn parse_create_query(&mut self) -> Option<Query> {
        let mut query = Query::new(Operation::Create);
        let keyword = self.next()?;
        let identifier = self.next()?;
        match identifier {
            Token::Identifier(name) => {
                match keyword {
                    Token::Database => {
                        query.database = Some(name);
                        return Some(query);
                    },
                    Token::Table => { query.table = Some(name); },
                    _ => { return None; }
                }
            }
            _ => { return None; }
        }

        let mut columns: Vec<Column> = Vec::new();

        if !self.consume(&[Token::LeftBracket]) {
            return None;
        }
        loop {
            let Token::Identifier(name) = self.next()? else { return None; };

            if !self.consume(&[Token::Colon]) {
                return None;
            }

            let field_type = match self.next()? {
                Token::NumberType => FieldType::Number,
                Token::TextType => FieldType::Text,
                _ => { return None; } 
            };

            columns.push(Column::new(name, field_type));

            if !self.consume(&[Token::Comma]) {
                if self.consume(&[Token::RightBracket]) {
                    break;
                }
                return None;
            }
        }
        query.columns = Some(columns);

        Some(query)
    }

    fn parse_get_query(&mut self) -> Option<Query> {
        let mut query = Query::new(Operation::Get);

        if !self.consume(&[Token::Star]) {
            return None;
        }
        if !self.consume(&[Token::From]) {
            return None;
        }
        let identifier = self.next()?;
        match identifier {
            Token::Identifier(name) => { query.table = Some(name); },
            _ => { return None; }
        }
                
        Some(query)
    }

    fn parse_put_query(&mut self) -> Option<Query> {
        let mut query = Query::new(Operation::Put);
        let mut values: Vec<FieldValue> = Vec::new();
        
        if !self.consume(&[Token::LeftBracket]) {
            return None;
        }
        loop {
            match self.next()? {
                Token::Comma => continue,
                Token::Float(number) => { values.push(FieldValue::Float(number)); },
                Token::Integer(number) => { values.push(FieldValue::Integer(number)); },
                Token::String(text) => { values.push(FieldValue::Text(String::from(text))); },
                Token::None => { values.push(FieldValue::None); },
                Token::RightBracket => { break; },
                _ => { return None; }
            }
        }
        query.values = Some(values);

        if !self.consume(&[Token::In]) {
            return None;
        }

        query.table = match self.next()? {
            Token::Identifier(name) => Some(name),
            _ => None
        };

        Some(query)
    }

    fn parse_update_query(&mut self) -> Option<Query> {
        let mut query = Query::new(Operation::Update);
        todo!("update queries");
        Some(query)
    }

    fn parse_delete_query(&mut self) -> Option<Query> {
        let mut query = Query::new(Operation::Delete);
        let keyword = self.next()?;
        let identifier = self.next()?;
        match identifier {
            Token::Identifier(name) => {
                match keyword {
                    Token::Database => { query.database = Some(name); },
                    Token::Table => { query.table = Some(name); },
                    _ => { return None; }
                }
            }
            _ => { return None; }
        }
        todo!("delete queries");
        Some(query)
    }

    fn parse_or(&mut self) {
        let mut expression = self.parse_and();
        todo!("parse_or");
    }

    fn parse_and(&mut self) {
        let mut expression = self.parse_equality();
        todo!("parse_and");
    }

    fn parse_equality(&mut self) {
        let mut expression = self.parse_comparison();
        todo!("parse_equality");
    }

    fn parse_comparison(&mut self) {
        let mut expression = self.parse_term();
        todo!("parse_comparison");
    }

    fn parse_term(&mut self) {
        let mut expression = self.parse_factor();
        todo!("parse_term");
    }

    fn parse_factor(&mut self) {
        let mut expression = self.parse_unary();
        todo!("parse_factor");
    }

    fn parse_unary(&mut self) {
        let mut expression = self.parse_primary();
        todo!("parse_unary");
    }

    fn parse_primary(&mut self) {
        let mut expression: Expression;
        todo!("parse_primary");
    }
}