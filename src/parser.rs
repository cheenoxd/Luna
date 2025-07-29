use crate::ast::*;
use crate::lexer::{Token, TokenType};
use crate::value::Value;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            if self.match_types(&[TokenType::Newline]) {
                continue;
            }
            statements.push(self.statement()?);
        }

        Ok(Program { statements })
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_types(&[TokenType::Local]) {
            self.local_assignment()
        } else if self.match_types(&[TokenType::Function]) {
            self.function_declaration()
        } else if self.match_types(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_types(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_types(&[TokenType::For]) {
            self.for_statement()
        } else if self.match_types(&[TokenType::Return]) {
            self.return_statement()
        } else if self.match_types(&[TokenType::Break]) {
            Ok(Stmt::Break)
        } else {
            // Check for assignment
            if self.check_assignment() {
                self.assignment()
            } else {
                Ok(Stmt::Expression(self.expression()?))
            }
        }
    }

    fn local_assignment(&mut self) -> Result<Stmt, String> {
        let mut names = Vec::new();

        names.push(self.consume_identifier("Expected variable name")?);

        while self.match_types(&[TokenType::Comma]) {
            names.push(self.consume_identifier("Expected variable name")?);
        }

        let mut values = Vec::new();
        if self.match_types(&[TokenType::Assign]) {
            values.push(self.expression()?);

            while self.match_types(&[TokenType::Comma]) {
                values.push(self.expression()?);
            }
        }

        Ok(Stmt::LocalAssignment { names, values })
    }

    fn function_declaration(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Expected function name")?;

        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            params.push(self.consume_identifier("Expected parameter name")?);

            while self.match_types(&[TokenType::Comma]) {
                params.push(self.consume_identifier("Expected parameter name")?);
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        let mut body = Vec::new();
        while !self.check(&TokenType::End) && !self.is_at_end() {
            if self.match_types(&[TokenType::Newline]) {
                continue;
            }
            body.push(self.statement()?);
        }

        self.consume(&TokenType::End, "Expected 'end' after function body")?;

        Ok(Stmt::Function { name, params, body })
    }

    fn if_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;
        self.consume(&TokenType::Then, "Expected 'then' after if condition")?;

        let mut then_branch = Vec::new();
        while !self.check(&TokenType::Else) && !self.check(&TokenType::End) && !self.is_at_end() {
            if self.match_types(&[TokenType::Newline]) {
                continue;
            }
            then_branch.push(self.statement()?);
        }

        let else_branch = if self.match_types(&[TokenType::Else]) {
            let mut else_stmts = Vec::new();
            while !self.check(&TokenType::End) && !self.is_at_end() {
                if self.match_types(&[TokenType::Newline]) {
                    continue;
                }
                else_stmts.push(self.statement()?);
            }
            Some(else_stmts)
        } else {
            None
        };

        self.consume(&TokenType::End, "Expected 'end' after if statement")?;

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, String> {
        let condition = self.expression()?;
        self.consume(&TokenType::Do, "Expected 'do' after while condition")?;

        let mut body = Vec::new();
        while !self.check(&TokenType::End) && !self.is_at_end() {
            if self.match_types(&[TokenType::Newline]) {
                continue;
            }
            body.push(self.statement()?);
        }

        self.consume(&TokenType::End, "Expected 'end' after while body")?;

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, String> {
        let var = self.consume_identifier("Expected variable name in for loop")?;
        self.consume(&TokenType::Assign, "Expected '=' in for loop")?;

        let start = self.expression()?;
        self.consume(&TokenType::Comma, "Expected ',' after for loop start")?;

        let end = self.expression()?;

        let step = if self.match_types(&[TokenType::Comma]) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(&TokenType::Do, "Expected 'do' after for loop header")?;

        let mut body = Vec::new();
        while !self.check(&TokenType::End) && !self.is_at_end() {
            if self.match_types(&[TokenType::Newline]) {
                continue;
            }
            body.push(self.statement()?);
        }

        self.consume(&TokenType::End, "Expected 'end' after for loop body")?;

        Ok(Stmt::For {
            var,
            start,
            end,
            step,
            body,
        })
    }

    fn return_statement(&mut self) -> Result<Stmt, String> {
        let value = if self.check(&TokenType::Newline) || self.is_at_end() {
            None
        } else {
            Some(self.expression()?)
        };

        Ok(Stmt::Return(value))
    }

    fn check_assignment(&mut self) -> bool {
        if let Some(TokenType::Identifier(_)) = self.peek() {
            let saved = self.current;
            self.advance();
            let is_assignment = self.check(&TokenType::Assign);
            self.current = saved;
            is_assignment
        } else {
            false
        }
    }

    fn assignment(&mut self) -> Result<Stmt, String> {
        let target = self.consume_identifier("Expected variable name")?;
        self.consume(&TokenType::Assign, "Expected '=' in assignment")?;
        let value = self.expression()?;

        Ok(Stmt::Assignment { target, value })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_types(&[TokenType::Or]) {
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Or,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_types(&[TokenType::And]) {
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::And,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while let Some(op) = self.match_binary_op(&[TokenType::Equal, TokenType::NotEqual]) {
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while let Some(op) = self.match_binary_op(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.concat()?;

        while let Some(op) = self.match_binary_op(&[TokenType::Plus, TokenType::Minus]) {
            let right = self.concat()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn concat(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_types(&[TokenType::DotDot]) {
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Concat,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while let Some(op) = self.match_binary_op(&[
            TokenType::Star,
            TokenType::Slash,
            TokenType::Percent,
        ]) {
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if let Some(op) = self.match_unary_op(&[TokenType::Not, TokenType::Minus]) {
            let operand = self.unary()?;
            return Ok(Expr::Unary {
                operator: op,
                operand: Box::new(operand),
            });
        }

        self.power()
    }

    fn power(&mut self) -> Result<Expr, String> {
        let mut expr = self.call()?;

        while self.match_types(&[TokenType::Caret]) {
            let right = self.unary()?; // Right associative
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: BinaryOp::Pow,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_types(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_types(&[TokenType::Dot]) {
                let name = self.consume_identifier("Expected field name after '.'")?;
                expr = Expr::FieldAccess {
                    object: Box::new(expr),
                    field: name,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            args.push(self.expression()?);
            while self.match_types(&[TokenType::Comma]) {
                args.push(self.expression()?);
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            args,
        })
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if let Some(token_type) = self.advance() {
            match token_type {
                TokenType::True => Ok(Expr::Literal(Value::Boolean(true))),
                TokenType::False => Ok(Expr::Literal(Value::Boolean(false))),
                TokenType::Nil => Ok(Expr::Literal(Value::Nil)),
                TokenType::Number(n) => Ok(Expr::Literal(Value::Number(n))),
                TokenType::String(s) => Ok(Expr::Literal(Value::String(s))),
                TokenType::Identifier(name) => Ok(Expr::Identifier(name)),
                TokenType::LeftParen => {
                    let expr = self.expression()?;
                    self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                    Ok(expr)
                }
                _ => Err(format!("Unexpected token: {:?}", token_type)),
            }
        } else {
            Err("Unexpected end of input".to_string())
        }
    }

    fn match_binary_op(&mut self, types: &[TokenType]) -> Option<BinaryOp> {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return Some(match token_type {
                    TokenType::Plus => BinaryOp::Add,
                    TokenType::Minus => BinaryOp::Sub,
                    TokenType::Star => BinaryOp::Mul,
                    TokenType::Slash => BinaryOp::Div,
                    TokenType::Percent => BinaryOp::Mod,
                    TokenType::Caret => BinaryOp::Pow,
                    TokenType::Equal => BinaryOp::Equal,
                    TokenType::NotEqual => BinaryOp::NotEqual,
                    TokenType::Less => BinaryOp::Less,
                    TokenType::LessEqual => BinaryOp::LessEqual,
                    TokenType::Greater => BinaryOp::Greater,
                    TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                    TokenType::And => BinaryOp::And,
                    TokenType::Or => BinaryOp::Or,
                    _ => unreachable!(),
                });
            }
        }
        None
    }

    fn match_unary_op(&mut self, types: &[TokenType]) -> Option<UnaryOp> {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return Some(match token_type {
                    TokenType::Not => UnaryOp::Not,
                    TokenType::Minus => UnaryOp::Minus,
                    _ => unreachable!(),
                });
            }
        }
        None
    }

    fn match_types(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek().unwrap()) == std::mem::discriminant(token_type)
        }
    }

    fn advance(&mut self) -> Option<TokenType> {
        if !self.is_at_end() {
            self.current += 1;
            Some(self.previous().token_type.clone())
        } else {
            None
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Some(TokenType::Eof) | None)
    }

    fn peek(&self) -> Option<&TokenType> {
        self.tokens.get(self.current).map(|t| &t.token_type)
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<(), String> {
        if self.check(token_type) {
            self.advance();
            Ok(())
        } else {
            Err(message.to_string())
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let Some(token) = self.tokens.get(self.current) {
            if let TokenType::Identifier(name) = &token.token_type {
                let name = name.clone();
                self.advance();
                Ok(name)
            } else {
                Err(message.to_string())
            }
        } else {
            Err(message.to_string())
        }
    }
}
