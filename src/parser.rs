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
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum ExpressionType {
    // Unary
    Not, Negate, Positive,
    // Binary
    Equal, NotEqual,
    LessThan, LessThanOrEqual,
    GreaterThan, GreaterThanOrEqual,
    And, Or, Xor,
    // Arithmetic
    Add, Subtract, Multiply, Divide,
    Power, Modulus,
    // Literals
    Integer(i64), Float(f64), String(String),
    None, Identifier(String)
}

impl ExpressionType {
    pub fn is_literal(&self) -> bool {
        self == ExpressionType::Integer
        || self == ExpressionType::Float
        || self == ExpressionType::String
        || self == ExpressionType::None
        || self == ExpressionType::Identifier
    }
}

#[derive(Debug)]
pub struct Expression {
    // Literal expressions only use `expression_type`.
    pub expression_type: ExpressionType,
    // Unary expressions only use `l_operand`.
    pub l_operand: Option<Box<Expression>>,
    pub r_operand: Option<Box<Expression>>
}

#[derive(Debug)]
pub struct Query {
    pub operation: Operation,
    pub database: Option<String>,
    pub table: Option<String>,
    pub values: Option<Vec<FieldValue>>,
    pub columns: Option<Vec<Column>>,
    pub condition: Option<Box<Expression>>,
}

impl Query {
    pub fn new(operation: Operation) -> Self {
        Query{operation: operation, database: None, table: None, values: None, columns: None, condition: None}
    }
}

// Just your good ol' fashioned recursive descent parser.
pub struct Parser {
    tokens: Vec<Token>,
    previous: Option<Token>
}

impl Parser {
    pub fn new() -> Self {
        // Placeholder value.
        Parser{tokens: Vec::new(), previous: None}
    }

    pub fn parse(parser: &mut Parser, tokens: Vec<Token>) -> Query {
        parser.tokens = tokens;
        parser.tokens.reverse();
        parser.parse_query().unwrap()
    }

    fn next(&mut self) -> Option<Token> {
        self.previous = self.tokens.pop();
        self.previous.clone()
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.last()
    }

    fn peek_back(&self) -> Option<&Token> {
        self.previous.as_ref()
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

        if self.consume(&[Token::Where]) {
            query.condition = self.parse_or();
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

    fn parse_or(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_and();

        while self.consume(&[Token::Or]) {
            let expression_type = match *self.peek_back()? {
                Token::Or => ExpressionType::Or,
                _ => { return None; }
            };
            let r_expression = self.parse_and();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_and(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_equality();

        while self.consume(&[Token::And]) {
            let expression_type = match *self.peek_back()? {
                Token::And => ExpressionType::And,
                _ => { return None; }
            };
            let r_expression = self.parse_equality();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_equality(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_comparison();
        
        while self.consume(&[Token::Equal, Token::NotEqual]) {
            let expression_type = match *self.peek_back()? {
                Token::Equal => ExpressionType::Equal,
                Token::NotEqual => ExpressionType::NotEqual,
                _ => { return None; }
            };
            let r_expression = self.parse_comparison();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                    l_operand: expression,
                    r_operand: r_expression}));
                }
        expression
    }

    fn parse_comparison(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_term();

        while self.consume(&[Token::GreaterThan, Token::GreaterThanOrEqual,
                             Token::LessThan, Token::LessThanOrEqual]) {
            let expression_type = match *self.peek_back()? {
                Token::GreaterThan => ExpressionType::GreaterThan,
                Token::GreaterThanOrEqual => ExpressionType::GreaterThanOrEqual,
                Token::LessThan => ExpressionType::LessThan,
                Token::LessThanOrEqual => ExpressionType::LessThanOrEqual,
                _ => { return None; }
            };
            let r_expression = self.parse_term();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_term(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_factor();

        while self.consume(&[Token::Add, Token::Subtract]) {
            let expression_type = match *self.peek_back()? {
                Token::Add => ExpressionType::Add,
                Token::Subtract => ExpressionType::Subtract,
                _ => { return None; }
            };
            let r_expression = self.parse_factor();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_factor(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_unary();

        while self.consume(&[Token::Star, Token::Divide,
                             Token::Power, Token::Modulus]) {
            let expression_type = match *self.peek_back()? {
                Token::Star => ExpressionType::Multiply,
                Token::Divide => ExpressionType::Divide,
                Token::Power => ExpressionType::Power,
                Token::Modulus => ExpressionType::Modulus,
                _ => { return None; }
            };
            let r_expression = self.parse_unary();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_unary(&mut self) -> Option<Box<Expression>> {
        let mut expression = self.parse_primary();

        while self.consume(&[Token::Not, Token::Add, Token::Subtract]) {
            let expression_type = match *self.peek_back()? {
                Token::Not => ExpressionType::Not,
                Token::Add => ExpressionType::Positive,
                Token::Subtract => ExpressionType::Negate,
                _ => { return None; }
            };
            let r_expression = self.parse_primary();
            expression = Some(Box::new(
                Expression{expression_type: expression_type,
                           l_operand: expression,
                           r_operand: r_expression}));
        }

        expression
    }

    fn parse_primary(&mut self) -> Option<Box<Expression>> {
        let mut expression: Option<Box<Expression>> = None;

        let is_primary_type = |token: &Token| {
            match *token {
                Token::None
                | Token::Integer(_)
                | Token::Float(_)
                | Token::String(_)
                | Token::Identifier(_) => true,
                _ => false
            }
        };

        while is_primary_type(self.peek()?) {
            let next = self.next();
            let expression_type = match next? {
                Token::None => Some(ExpressionType::None),
                Token::Integer(number) => Some(ExpressionType::Integer(number)),
                Token::Float(number) => Some(ExpressionType::Float(number)),
                Token::String(string) => Some(ExpressionType::String(string)),
                Token::Identifier(identifier) => Some(ExpressionType::Identifier(identifier)),
                _ => None
            };

            if expression_type.is_none() && *self.peek()? == Token::LeftParenthesis {
                let grouped_expression = self.parse_or();
                if !self.consume(&[Token::RightParenthesis]) {
                    return None;
                }
                expression = grouped_expression;
                return expression;
            }
            else {
                expression = Some(Box::new(
                    Expression{expression_type: expression_type.unwrap(),
                        l_operand: None, r_operand: None}));
                return expression;
            }
        }
        None
    }
}
