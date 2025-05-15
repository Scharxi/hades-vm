use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenKind, Span};
use std::iter::Peekable;

pub struct Parser<'a> {
    lexer: Peekable<Lexer<'a>>,
    current: Token,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken {
        expected: &'static str,
        found: TokenKind,
        span: Span,
    },
    InvalidExpression(Span),
    InvalidStatement(Span),
    InvalidType(Span),
    UnexpectedEOF,
}

pub type Result<T> = std::result::Result<T, ParseError>;

impl<'a> Parser<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut lexer = Lexer::new(input).peekable();
        let current = lexer.next().unwrap(); // Safe because Lexer always returns at least EOF
        Self { lexer, current }
    }
    
    pub fn parse_program(&mut self) -> Result<Program> {
        let mut items = Vec::new();
        
        while self.current.kind != TokenKind::EOF {
            items.push(self.parse_item()?);
        }
        
        Ok(Program { items })
    }
    
    fn parse_item(&mut self) -> Result<Item> {
        match self.current.kind {
            TokenKind::Fn => self.parse_function().map(Item::Function),
            TokenKind::Struct => self.parse_struct().map(Item::Struct),
            TokenKind::Enum => self.parse_enum().map(Item::Enum),
            TokenKind::Trait => self.parse_trait().map(Item::Trait),
            TokenKind::Impl => self.parse_implementation().map(Item::Implementation),
            _ => Err(ParseError::UnexpectedToken {
                expected: "item declaration",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            }),
        }
    }
    
    fn parse_function(&mut self) -> Result<Function> {
        self.expect(TokenKind::Fn)?;
        
        let is_async = if self.check(TokenKind::Async) {
            self.advance();
            true
        } else {
            false
        };
        
        let name = self.parse_identifier()?;
        
        self.expect(TokenKind::LParen)?;
        let params = self.parse_separated_list(TokenKind::RParen, |p| p.parse_parameter())?;
        
        let return_type = if self.check(TokenKind::Arrow) {
            self.advance();
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            params,
            return_type,
            body,
            is_async,
        })
    }
    
    fn parse_parameter(&mut self) -> Result<Parameter> {
        let is_mutable = if self.check(TokenKind::Mut) {
            self.advance();
            true
        } else {
            false
        };
        
        let name = self.parse_identifier()?;
        
        self.expect(TokenKind::Colon)?;
        let type_ = self.parse_type()?;
        
        Ok(Parameter {
            name,
            type_,
            is_mutable,
        })
    }
    
    fn parse_block(&mut self) -> Result<Block> {
        self.expect(TokenKind::LBrace)?;
        
        let mut statements = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }
        
        self.expect(TokenKind::RBrace)?;
        
        Ok(Block { statements })
    }
    
    fn parse_statement(&mut self) -> Result<Statement> {
        match self.current.kind {
            TokenKind::Let => {
                self.advance();
                
                let is_mutable = if self.check(TokenKind::Mut) {
                    self.advance();
                    true
                } else {
                    false
                };
                
                let name = self.parse_identifier()?;
                
                let type_ = if self.check(TokenKind::Colon) {
                    self.advance();
                    Some(self.parse_type()?)
                } else {
                    None
                };
                
                let initializer = if self.check(TokenKind::Assign) {
                    self.advance();
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                self.expect(TokenKind::Semicolon)?;
                
                Ok(Statement::Let {
                    name,
                    type_,
                    initializer,
                    is_mutable,
                })
            }
            TokenKind::Return => {
                self.advance();
                
                let value = if !self.check(TokenKind::Semicolon) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                
                self.expect(TokenKind::Semicolon)?;
                
                Ok(Statement::Return(value))
            }
            TokenKind::If => self.parse_if_statement(),
            TokenKind::When => self.parse_when_statement(),
            _ => {
                let expr = self.parse_expression()?;
                self.expect(TokenKind::Semicolon)?;
                Ok(Statement::Expression(expr))
            }
        }
    }
    
    fn parse_if_statement(&mut self) -> Result<Statement> {
        self.advance(); // Skip 'if'
        
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        
        let else_branch = if self.check(TokenKind::Else) {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }
    
    fn parse_when_statement(&mut self) -> Result<Statement> {
        self.advance(); // Skip 'when'
        
        let subject = self.parse_expression()?;
        
        self.expect(TokenKind::LBrace)?;
        
        let mut arms = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.check(TokenKind::EOF) {
            arms.push(self.parse_when_arm()?);
        }
        
        self.expect(TokenKind::RBrace)?;
        
        Ok(Statement::When { subject, arms })
    }
    
    fn parse_when_arm(&mut self) -> Result<WhenArm> {
        let pattern = self.parse_pattern()?;
        
        let guard = if self.check(TokenKind::If) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.expect(TokenKind::FatArrow)?;
        
        let body = self.parse_block()?;
        
        Ok(WhenArm {
            pattern,
            guard,
            body,
        })
    }
    
    fn parse_pattern(&mut self) -> Result<Pattern> {
        match self.current.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Integer(n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Float(f)))
            }
            TokenKind::String(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(s)))
            }
            TokenKind::Boolean(b) => {
                self.advance();
                Ok(Pattern::Literal(Literal::Boolean(b)))
            }
            TokenKind::Identifier(_) => {
                let name = self.parse_identifier()?;
                if self.check(TokenKind::LParen) {
                    self.advance();
                    let fields = self.parse_separated_list(TokenKind::RParen, |p| p.parse_pattern())?;
                    Ok(Pattern::Constructor { name, fields })
                } else {
                    Ok(Pattern::Identifier(name))
                }
            }
            TokenKind::Is => {
                self.advance();
                let type_ = self.parse_type()?;
                Ok(Pattern::Is(type_))
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            }),
        }
    }
    
    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_binary_expression(0)
    }
    
    fn parse_binary_expression(&mut self, min_precedence: u8) -> Result<Expression> {
        let mut lhs = self.parse_unary_expression()?;
        
        while let Some((precedence, op)) = self.get_binary_operator() {
            if precedence < min_precedence {
                break;
            }
            
            self.advance();
            
            let rhs = self.parse_binary_expression(precedence + 1)?;
            
            lhs = Expression::Binary {
                left: Box::new(lhs),
                operator: op,
                right: Box::new(rhs),
            };
        }
        
        Ok(lhs)
    }
    
    fn parse_unary_expression(&mut self) -> Result<Expression> {
        if let Some(op) = self.get_unary_operator() {
            self.advance();
            let operand = self.parse_unary_expression()?;
            Ok(Expression::Unary {
                operator: op,
                operand: Box::new(operand),
            })
        } else {
            self.parse_postfix_expression()
        }
    }
    
    fn parse_postfix_expression(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary_expression()?;
        
        loop {
            match self.current.kind {
                TokenKind::LParen => {
                    self.advance();
                    let arguments = self.parse_separated_list(TokenKind::RParen, |p| p.parse_expression())?;
                    expr = Expression::Call {
                        function: Box::new(expr),
                        arguments,
                    };
                }
                TokenKind::Dot => {
                    self.advance();
                    let name = self.parse_identifier()?;
                    if self.check(TokenKind::LParen) {
                        self.advance();
                        let arguments = self.parse_separated_list(TokenKind::RParen, |p| p.parse_expression())?;
                        expr = Expression::MethodCall {
                            receiver: Box::new(expr),
                            method: name,
                            arguments,
                        };
                    } else {
                        expr = Expression::Field {
                            object: Box::new(expr),
                            field: name,
                        };
                    }
                }
                TokenKind::Await => {
                    self.advance();
                    expr = Expression::Await(Box::new(expr));
                }
                _ => break,
            }
        }
        
        Ok(expr)
    }
    
    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match self.current.kind {
            TokenKind::Integer(n) => {
                self.advance();
                Ok(Expression::Literal(Literal::Integer(n)))
            }
            TokenKind::Float(f) => {
                self.advance();
                Ok(Expression::Literal(Literal::Float(f)))
            }
            TokenKind::String(ref s) => {
                let s = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(s)))
            }
            TokenKind::Boolean(b) => {
                self.advance();
                Ok(Expression::Literal(Literal::Boolean(b)))
            }
            TokenKind::Identifier(_) => {
                let name = self.parse_identifier()?;
                Ok(Expression::Variable(name))
            }
            TokenKind::LBrace => {
                let block = self.parse_block()?;
                Ok(Expression::Block(block))
            }
            TokenKind::If => {
                self.advance();
                let condition = Box::new(self.parse_expression()?);
                let then_branch = self.parse_block()?;
                let else_branch = if self.check(TokenKind::Else) {
                    self.advance();
                    Some(self.parse_block()?)
                } else {
                    None
                };
                Ok(Expression::If {
                    condition,
                    then_branch,
                    else_branch,
                })
            }
            TokenKind::When => {
                self.advance();
                let subject = Box::new(self.parse_expression()?);
                self.expect(TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !self.check(TokenKind::RBrace) && !self.check(TokenKind::EOF) {
                    arms.push(self.parse_when_arm()?);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Expression::When { subject, arms })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            }),
        }
    }
    
    fn parse_type(&mut self) -> Result<Type> {
        let base = match self.current.kind {
            TokenKind::Identifier(_) => {
                let name = self.parse_identifier()?;
                if self.check(TokenKind::Lt) {
                    self.advance();
                    let args = self.parse_separated_list(TokenKind::Gt, |p| p.parse_type())?;
                    Type::Named { name, args }
                } else {
                    Type::Named {
                        name,
                        args: Vec::new(),
                    }
                }
            }
            TokenKind::LParen => {
                self.advance();
                let types = self.parse_separated_list(TokenKind::RParen, |p| p.parse_type())?;
                Type::Tuple(types)
            }
            _ => return Err(ParseError::UnexpectedToken {
                expected: "type",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            }),
        };
        
        if self.check(TokenKind::Arrow) {
            self.advance();
            let return_type = self.parse_type()?;
            Ok(Type::Function {
                params: vec![base],
                return_type: Box::new(return_type),
            })
        } else {
            Ok(base)
        }
    }
    
    fn parse_identifier(&mut self) -> Result<String> {
        match self.current.kind {
            TokenKind::Identifier(ref name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "identifier",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            }),
        }
    }
    
    fn parse_separated_list<T>(
        &mut self,
        end_token: TokenKind,
        mut parse_element: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut elements = Vec::new();
        
        if !self.check(end_token.clone()) {
            elements.push(parse_element(self)?);
            
            while self.check(TokenKind::Comma) {
                self.advance();
                if self.check(end_token.clone()) {
                    break;
                }
                elements.push(parse_element(self)?);
            }
        }
        
        self.expect(end_token)?;
        
        Ok(elements)
    }
    
    fn get_binary_operator(&self) -> Option<(u8, BinaryOp)> {
        let result = match self.current.kind {
            TokenKind::Plus => (1, BinaryOp::Add),
            TokenKind::Minus => (1, BinaryOp::Sub),
            TokenKind::Star => (2, BinaryOp::Mul),
            TokenKind::Slash => (2, BinaryOp::Div),
            TokenKind::Eq => (0, BinaryOp::Eq),
            TokenKind::NotEq => (0, BinaryOp::NotEq),
            TokenKind::Lt => (0, BinaryOp::Lt),
            TokenKind::LtEq => (0, BinaryOp::LtEq),
            TokenKind::Gt => (0, BinaryOp::Gt),
            TokenKind::GtEq => (0, BinaryOp::GtEq),
            _ => return None,
        };
        Some(result)
    }
    
    fn get_unary_operator(&self) -> Option<UnaryOp> {
        match self.current.kind {
            TokenKind::Minus => Some(UnaryOp::Neg),
            TokenKind::Not => Some(UnaryOp::Not),
            _ => None,
        }
    }
    
    fn check(&self, kind: TokenKind) -> bool {
        self.current.kind == kind
    }
    
    fn expect(&mut self, kind: TokenKind) -> Result<()> {
        if self.check(kind.clone()) {
            self.advance();
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "expected token",
                found: self.current.kind.clone(),
                span: self.current.span.clone(),
            })
        }
    }
    
    fn advance(&mut self) {
        self.current = self.lexer.next().unwrap(); // Safe because Lexer always returns at least EOF
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let input = r#"
            fn greet(name: String) -> String {
                return "Hello, " + name;
            }
        "#;
        
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert_eq!(function.name, "greet");
        assert_eq!(function.params.len(), 1);
        assert_eq!(function.params[0].name, "name");
        assert!(matches!(
            function.params[0].type_,
            Type::Named { name, args } if name == "String" && args.is_empty()
        ));
    }

    #[test]
    fn test_parse_when_expression() {
        let input = r#"
            when (x) {
                0 => { print("zero"); }
                1 => { print("one"); }
                _ => { print("other"); }
            }
        "#;
        
        let mut parser = Parser::new(input);
        let expr = parser.parse_expression().unwrap();
        
        assert!(matches!(expr, Expression::When { .. }));
    }

    #[test]
    fn test_parse_async_function() {
        let input = r#"
            async fn fetch_data() -> Result<String, Error> {
                let response = http.get("https://example.com").await;
                return response.text();
            }
        "#;
        
        let mut parser = Parser::new(input);
        let function = parser.parse_function().unwrap();
        
        assert!(function.is_async);
        assert_eq!(function.name, "fetch_data");
    }
} 