#![allow(warnings)]

mod parser;
mod lexer;

use std::{any::{Any, TypeId}, collections::HashMap, fs::File, path::{Path, PathBuf}, io::{self, Write}};
use serde::{Deserialize, Serialize};
use crate::parser::*;
use crate::lexer::*;

pub fn run() -> io::Result<()> {
    // Test code
    let mut database = Database::new(String::from("business"), DatabaseConfig::default());
    let customers = database.new_table(
        String::from("customers"),
        vec![Column::new(String::from("Name"), FieldType::Text),
            Column::new(String::from("ID"), FieldType::Number)]
        ).unwrap();
    customers.new_row(vec![FieldValue::Text(String::from("james")), FieldValue::Integer(1)]);
    customers.new_row(vec![FieldValue::Text(String::from("jim")), FieldValue::Integer(2)]);
    customers.new_row(vec![FieldValue::Text(String::from("jimmy")), FieldValue::Integer(3)]);
    // database.save();
    // let mut database = Database::from_file(Path::new("./business")).unwrap();
    // let mut database = Database::new(String::from("default"), DatabaseConfig::default());

    let mut lexer = Lexer::new();
    let mut parser = Parser::new();
    loop {
        // Input handling
        print!("coil> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        if input.starts_with("q") {
            break;
        }
        // Lexing, parsing, and interpreting
        let tokens = Lexer::lex(&mut lexer, input);
        let query = Parser::parse(&mut parser, tokens);
        // println!("{:#?}", query);
        let result = database.run_query(query);
        // println!("{:#?}", result);
        result.unwrap().print();
    }

    Ok(())
}

#[derive(Debug)]
pub struct QueryResult<'a> {
    pub operation: Operation,
    pub database: Option<&'a Database>,
    pub table: Option<&'a Table>,
    pub columns: Option<Vec<&'a Column>>,
    pub rows: Option<Vec<Row>>,
}

impl<'a> QueryResult<'a> {
    pub fn new(operation: Operation) -> Self {
        QueryResult{operation: operation, database: None, table: None, columns: None, rows: None}
    }

    pub fn print(&self) {
        if self.operation != Operation::Get {
            return;
        }
        let mut table = prettytable::Table::new();
        let mut names: Vec<&str> = Vec::new();
        let mut cells: Vec<prettytable::Cell> = Vec::new();
        // Header
        for column in &self.table.unwrap().columns {
            names.push(column.name.as_str());
            cells.push(prettytable::Cell::new(names[names.len() - 1]))
        }
        table.add_row(prettytable::Row::new(cells));
        // Rows
        for row in self.rows.as_ref().unwrap() {
            let mut values: Vec<prettytable::Cell> = Vec::new();
            for name in &names {
                values.push(prettytable::Cell::new(row.get(name).unwrap().to_string().as_str()));
            }
            table.add_row(prettytable::Row::new(values));
        }

        table.printstd();
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    // The path doesn't actually need to be mutated
    // after initialization, but `std::path::Path`'s
    // size is unknown at compile time, so we'll use
    // this because it's an owned buffer with a type
    // known at compile time :).
    path: PathBuf
}

impl DatabaseConfig {
    pub fn new(path: PathBuf) -> Self {
        DatabaseConfig{path: path}
    }

    pub fn default() -> Self {
        let mut config = DatabaseConfig{path: PathBuf::new()};
        config.path.push("./");
        config
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum CoilError {
    NotEnoughValues,
    TooManyValues,
    TableAlreadyExists,
    TableDoesntExist,
    DatabaseAlreadyExists,
    DatabaseDoesntExist,
    MismatchedTypes
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    name: String,
    config: DatabaseConfig,
    tables: Vec<Table>
}

impl Database {
    pub fn new(name: String, config: DatabaseConfig) -> Self {
        Database{name: name, config: config, tables: Vec::new()}
    }

    pub fn from_file(path: &Path) -> Result<Self, CoilError> {
        let file = File::open(path);
        if file.is_err() {
            return Err(CoilError::DatabaseDoesntExist);
        }
        let database = serde_json::from_reader(file.unwrap()).unwrap();
        Ok(database)
    }

    pub fn run_query(&mut self, query: Query) -> Option<QueryResult> {
        let mut result = QueryResult::new(query.operation);
        match result.operation {
            Operation::Get => {
                let table = self.get_table(query.table?)?;
                let mut rows;
                if query.condition.is_some() {
                    rows = table.get_rows(Some(*(query.condition?)));
                }
                else {
                    rows = table.get_rows(None);
                }
                result.table = Some(table);
                result.rows = rows;
            },
            Operation::Put => {
                let _ = self.get_table_mut(query.table?)?.new_row(query.values?);
            },
            Operation::Update => {
                todo!("updating");
            },
            Operation::Create => {
                if let Some(table) = query.table {
                    result.table = Some(self.new_table(table, query.columns?).unwrap());
                }
                todo!("creating databases");
            },
            Operation::Delete => {
                todo!("deletion");
            },
        }

        Some(result)
    }

    pub fn new_table(&mut self, name: String, columns: Vec<Column>) -> Result<&mut Table, CoilError> {
        for table in &self.tables {
            if table.name == name {
                return Err(CoilError::TableAlreadyExists);
            }
        }
        self.tables.push(Table::new(name, columns));

        let new_table_index = self.tables.len() - 1;
        Ok(&mut self.tables[new_table_index])
    }

    pub fn get_table<'a>(&'a self, name: String) -> Option<&'a Table> {
        for table in &self.tables {
            if table.name == name {
                return Some(&table);
            }
        }
        None
    }

    pub fn get_table_mut(&mut self, name: String) -> Option<&mut Table> {
        for table in &mut self.tables {
            if table.name == name {
                return Some(table);
            }
        }
        None
    }

    pub fn save(&self) -> Result<usize, std::io::Error> {
        let mut file = File::create((*self.config.path).with_file_name(self.name.as_str()))?;
        file.write(serde_json::to_string(self).unwrap().as_bytes())
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Table {
    name: String,
    columns: Vec<Column>
}

impl Table {
    pub fn new(name: String, columns: Vec<Column>) -> Self {
        Table{name: name, columns: columns}
    }

    pub fn new_row(&mut self, values: Vec<FieldValue>) -> Option<CoilError> {
        if values.len() > self.columns.len() {
            return Some(CoilError::TooManyValues);
        }
        else if values.len() < self.columns.len() {
            return Some(CoilError::NotEnoughValues);
        }

        for i in 0..values.len() {
            let _ = self.columns[i].push(values[i].clone());
        }

        None
    }

    pub fn get_rows(&self, condition: Option<Expression>) -> Option<Vec<Row>> {
        let mut rows: Vec<Row> = Vec::new();
        // I figured it's better to branch once before
        // the loop than to branch and unwrap on every
        // iteration. Unfortunately, this does end up
        // looking very ugly!
        if let Some(row_condition) = condition {
            for i in 0..self.columns[0].rows.len() {
                let row = Row::from_columns(&self.columns, i);
                if row.check_condition(&row_condition) {
                    rows.push(row);
                }
            }
        }
        else {
            for i in 0..self.columns[0].rows.len() {
                let row = Row::from_columns(&self.columns, i);
                rows.push(row);
            }
        }

        Some(rows)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Row {
    columns: HashMap<String, FieldValue>
}

impl Row {
    pub fn from_columns(columns: &Vec<Column>, index: usize) -> Self {
        let mut row = Row{columns: HashMap::new()};
        for column in columns {
            row.columns.insert(column.name.clone(), column.rows[index].clone());
        }
        row
    }

    pub fn get(&self, field: &str) -> Option<&FieldValue> {
        self.columns.get(field)
    }

    // TODO: this function cannot handle nested expressions...
    pub fn check_condition(&self, condition: &Expression) -> bool {
        let l_operand = condition.l_operand.as_ref().unwrap();
        let r_operand = condition.r_operand.as_ref().unwrap();
        let mut l_value;
        let mut r_value;
        // Resolve identifier values and convert
        // ExpressionTypes into FieldValues
        if let ExpressionType::Identifier(identifier) = &l_operand.expression_type {
            // TODO: this function should actually
            // return a Result<bool> to handle errors
            l_value = self.get(identifier.as_str()).unwrap().clone();
        }
        else {
            l_value = FieldValue::from_expression_type(l_operand.expression_type.clone());
        }
        if let ExpressionType::Identifier(identifier) = &r_operand.expression_type {
            r_value = self.get(identifier.as_str()).unwrap().clone();
        }
        else {
            r_value = FieldValue::from_expression_type(r_operand.expression_type.clone());
        }

        match condition.expression_type {
            ExpressionType::Equal => l_value == r_value,
            ExpressionType::NotEqual => l_value != r_value,
            ExpressionType::LessThan => l_value < r_value,
            ExpressionType::LessThanOrEqual => l_value <= r_value,
            ExpressionType::GreaterThan => l_value > r_value,
            ExpressionType::GreaterThanOrEqual => l_value >= r_value,
            // ExpressionType::And => l_value && r_value,
            // ExpressionType::Or => l_value || r_value,
            // ExpressionType::Xor => l_value != r_value,
            _ => false
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Column {
    pub name: String,
    pub rows: Vec<FieldValue>,
    pub field_type: FieldType
}

impl Column {
    pub fn new(name: String, field_type: FieldType) -> Self {
        Column{name: name, rows: Vec::new(), field_type: field_type}
    }

    pub fn push(&mut self, value: FieldValue) -> Result<(), CoilError> {
        if self.field_type.check_field_value_type(&value) {
            self.rows.push(value);
            return Ok(());
        }
        Err(CoilError::MismatchedTypes)
    }
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub enum FieldType {
    Text,
    Number
}

impl FieldType {
    pub fn check_field_value_type(&self, field_value: &FieldValue) -> bool {
        match *field_value {
            FieldValue::None => true,
            FieldValue::Text(_) => self == &FieldType::Text,
            FieldValue::Integer(_)
            | FieldValue::Float(_) => self == &FieldType::Number
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Deserialize, Serialize)]
pub enum FieldValue {
    None,
    Text(String),
    Integer(i64),
    Float(f64)
}

impl FieldValue {
    pub fn from_expression_type(expression_type: ExpressionType) -> Self {
        match expression_type {
            ExpressionType::None => FieldValue::None,
            ExpressionType::String(string) => FieldValue::Text(string),
            ExpressionType::Integer(number) => FieldValue::Integer(number),
            ExpressionType::Float(number) => FieldValue::Float(number),
            // Hmm... this constructor could
            // return an Option<Self> maybe...
            _ => FieldValue::None
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            FieldValue::None => String::from("None"),
            FieldValue::Text(string) => string.to_string(),
            FieldValue::Integer(number) => number.to_string(),
            FieldValue::Float(number) => number.to_string()
        }
    }
}
