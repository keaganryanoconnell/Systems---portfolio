pub mod ast;
mod lexer;

use crate::catalog::ColumnType;
pub use ast::*;
use lexer::{Token, TokenKind};

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.pos]
    }

    fn advance(&mut self) -> &Token {
        let t = &self.tokens[self.pos];
        self.pos += 1;
        t
    }

    fn expect_token(&mut self, kind: TokenKind) -> Result<&Token, String> {
        if self.peek().kind == kind {
            Ok(self.advance())
        } else {
            Err(format!("Expected {:?}, got {:?}", kind, self.peek().kind))
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.peek().kind {
            TokenKind::Select => self.parse_select().map(Statement::Select),
            TokenKind::Insert => self.parse_insert().map(Statement::Insert),
            TokenKind::Create => self.parse_create().map(Statement::Create),
            TokenKind::Drop => self.parse_drop().map(Statement::Drop),
            TokenKind::Delete => self.parse_delete().map(Statement::Delete),
            TokenKind::Update => self.parse_update().map(Statement::Update),
            _ => Err(format!("Unexpected token: {:?}", self.peek().kind)),
        }
    }

    fn parse_select(&mut self) -> Result<SelectStmt, String> {
        self.expect_token(TokenKind::Select)?;

        let columns = if self.peek().kind == TokenKind::Star {
            self.advance();
            vec![SelectColumn::All]
        } else {
            let mut cols = Vec::new();
            loop {
                let name = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
                cols.push(SelectColumn::Named(name));
                if self.peek().kind != TokenKind::Comma {
                    break;
                }
                self.advance();
            }
            cols
        };

        self.expect_token(TokenKind::From)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();

        let where_clause = if self.peek().kind == TokenKind::Where {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(SelectStmt {
            columns,
            table,
            where_clause,
        })
    }

    fn parse_insert(&mut self) -> Result<InsertStmt, String> {
        self.expect_token(TokenKind::Insert)?;
        self.expect_token(TokenKind::Into)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
        self.expect_token(TokenKind::Values)?;
        self.expect_token(TokenKind::LParen)?;

        let mut values = Vec::new();
        loop {
            values.push(self.parse_expression()?);
            if self.peek().kind != TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect_token(TokenKind::RParen)?;

        Ok(InsertStmt {
            table,
            columns: None,
            values,
        })
    }

    fn parse_create(&mut self) -> Result<CreateStmt, String> {
        self.expect_token(TokenKind::Create)?;
        self.expect_token(TokenKind::Table)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
        self.expect_token(TokenKind::LParen)?;

        let mut columns = Vec::new();
        loop {
            let name = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
            let type_name = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
            let col_type = match type_name.as_str() {
                "int" | "integer" => ColumnType::Int,
                "text" | "varchar" | "string" => ColumnType::Text,
                "float" | "real" | "double" => ColumnType::Float,
                "bool" | "boolean" => ColumnType::Bool,
                _ => return Err(format!("Unknown type: {}", type_name)),
            };
            columns.push(ColumnDef { name, col_type });
            if self.peek().kind != TokenKind::Comma {
                break;
            }
            self.advance();
        }
        self.expect_token(TokenKind::RParen)?;

        Ok(CreateStmt { table, columns })
    }

    fn parse_drop(&mut self) -> Result<DropStmt, String> {
        self.expect_token(TokenKind::Drop)?;
        self.expect_token(TokenKind::Table)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
        Ok(DropStmt { table })
    }

    fn parse_delete(&mut self) -> Result<DeleteStmt, String> {
        self.expect_token(TokenKind::Delete)?;
        self.expect_token(TokenKind::From)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();

        let where_clause = if self.peek().kind == TokenKind::Where {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(DeleteStmt {
            table,
            where_clause,
        })
    }

    fn parse_update(&mut self) -> Result<UpdateStmt, String> {
        self.expect_token(TokenKind::Update)?;
        let table = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
        self.expect_token(TokenKind::Set)?;

        let mut assignments = Vec::new();
        loop {
            let col = self.expect_token(TokenKind::Identifier)?.lexeme.clone();
            self.expect_token(TokenKind::Eq)?;
            let val = self.parse_expression()?;
            assignments.push((col, val));
            if self.peek().kind != TokenKind::Comma {
                break;
            }
            self.advance();
        }

        let where_clause = if self.peek().kind == TokenKind::Where {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(UpdateStmt {
            table,
            assignments,
            where_clause,
        })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_and()
    }

    fn parse_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;
        while self.peek().kind == TokenKind::And {
            self.advance();
            let right = self.parse_comparison()?;
            left = Expression::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_primary()?;
        let op = self.peek().kind.clone();
        match op {
            TokenKind::Eq
            | TokenKind::Neq
            | TokenKind::Lt
            | TokenKind::Gt
            | TokenKind::Lte
            | TokenKind::Gte => {
                self.advance();
                let right = self.parse_primary()?;
                left = match op {
                    TokenKind::Eq => Expression::Eq(Box::new(left), Box::new(right)),
                    TokenKind::Neq => Expression::Neq(Box::new(left), Box::new(right)),
                    TokenKind::Lt => Expression::Lt(Box::new(left), Box::new(right)),
                    TokenKind::Gt => Expression::Gt(Box::new(left), Box::new(right)),
                    TokenKind::Lte => Expression::Lte(Box::new(left), Box::new(right)),
                    TokenKind::Gte => Expression::Gte(Box::new(left), Box::new(right)),
                    _ => unreachable!(),
                };
            }
            _ => {}
        }
        Ok(left)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match self.peek().kind {
            TokenKind::IntLiteral => {
                let val = self
                    .advance()
                    .lexeme
                    .parse()
                    .map_err(|e| format!("Invalid integer literal: {}", e))?;
                Ok(Expression::IntLiteral(val))
            }
            TokenKind::StringLiteral => {
                let val = self.advance().lexeme.clone();
                Ok(Expression::StringLiteral(val))
            }
            TokenKind::FloatLiteral => {
                let val = self
                    .advance()
                    .lexeme
                    .parse()
                    .map_err(|e| format!("Invalid float literal: {}", e))?;
                Ok(Expression::FloatLiteral(val))
            }
            TokenKind::BoolLiteral => {
                let val = self.advance().lexeme == "true";
                Ok(Expression::BoolLiteral(val))
            }
            TokenKind::Identifier => {
                let name = self.advance().lexeme.clone();
                Ok(Expression::Column(name))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect_token(TokenKind::RParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token: {:?}", self.peek().kind)),
        }
    }
}

pub fn parse_sql(input: &str) -> Result<Statement, String> {
    let tokens = lexer::tokenize(input);
    let mut parser = Parser::new(tokens);
    parser.parse_statement()
}
