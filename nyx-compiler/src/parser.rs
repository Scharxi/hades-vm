use crate::ast::*;
use crate::lexer::{tokenize, Token};
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at position {}: {}", self.position, self.message)
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(input: &str) -> Self {
        let tokens = tokenize(input);
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse_program(&mut self) -> ParseResult<Program> {
        let mut functions = Vec::new();
        
        while !self.is_at_end() {
            functions.push(self.parse_function()?);
        }
        
        Ok(Program { functions })
    }

    /// Parse a module-aware program that can include imports and modules
    pub fn parse_module_aware_program(&mut self) -> ParseResult<ModuleAwareProgram> {
        let mut imports = Vec::new();
        let mut functions = Vec::new();
        let mut modules = Vec::new();
        
        while !self.is_at_end() {
            match self.peek() {
                Token::Import => {
                    imports.push(self.parse_import()?);
                }
                Token::Mod | Token::Pub => {
                    modules.push(self.parse_module_declaration()?);
                }
                Token::Fun => {
                    functions.push(self.parse_function()?);
                }
                _ => {
                    return Err(self.error("Expected import, module, or function declaration"));
                }
            }
        }
        
        Ok(ModuleAwareProgram {
            imports,
            functions,
            modules,
        })
    }

    pub fn parse_function(&mut self) -> ParseResult<Function> {
        // Parse visibility first (could be at the start before 'fun')
        let visibility = if matches!(self.peek(), Token::Pub | Token::Internal | Token::Protected | Token::Package) {
            self.parse_visibility()?
        } else {
            Visibility::Private
        };

        self.consume(Token::Fun, "Expected 'fun'")?;
        
        let name = self.consume_identifier("Expected function name")?;
        
        self.consume(Token::LeftParen, "Expected '(' after function name")?;
        
        let mut parameters = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                let param_name = self.consume_identifier("Expected parameter name")?;
                self.consume(Token::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;
                
                parameters.push(Parameter {
                    name: param_name,
                    type_: param_type,
                });
                
                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        
        self.consume(Token::RightParen, "Expected ')' after parameters")?;
        
        self.consume(Token::Colon, "Expected ':' before return type")?;
        let return_type = self.parse_type()?;
        
        let body = self.parse_block()?;
        
        Ok(Function {
            name,
            visibility,
            parameters,
            return_type,
            body,
        })
    }

    fn parse_type(&mut self) -> ParseResult<Type> {
        match self.advance() {
            Token::Int => Ok(Type::Int),
            Token::Float => Ok(Type::Float),
            Token::Bool => Ok(Type::Bool),
            Token::String => Ok(Type::String),
            _ => Err(self.error("Expected type")),
        }
    }

    fn parse_block(&mut self) -> ParseResult<Block> {
        self.consume(Token::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        
        self.consume(Token::RightBrace, "Expected '}'")?;
        
        Ok(Block { statements })
    }

    fn parse_statement(&mut self) -> ParseResult<Statement> {
        match self.peek() {
            Token::Val => self.parse_variable_declaration(false),
            Token::Var => self.parse_variable_declaration(true),
            Token::Return => self.parse_return_statement(),
            Token::If => self.parse_if_statement(),
            Token::While => self.parse_while_statement(),
            _ => {
                // Check if it's an assignment
                if self.is_assignment() {
                    self.parse_assignment()
                } else {
                    // Expression statement
                    let expr = self.parse_expression()?;
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }

    fn parse_variable_declaration(&mut self, mutable: bool) -> ParseResult<Statement> {
        if mutable {
            self.advance(); // consume 'var'
        } else {
            self.advance(); // consume 'val'
        }
        
        let name = self.consume_identifier("Expected variable name")?;
        
        let type_ = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let initializer = if self.match_token(&Token::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        Ok(Statement::VariableDeclaration {
            name,
            mutable,
            type_,
            initializer,
        })
    }

    fn parse_assignment(&mut self) -> ParseResult<Statement> {
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(Token::Assign, "Expected '='")?;
        let value = self.parse_expression()?;
        
        Ok(Statement::Assignment { name, value })
    }

    fn parse_return_statement(&mut self) -> ParseResult<Statement> {
        self.advance(); // consume 'return'
        
        let value = if self.check(&Token::RightBrace) {
            None
        } else {
            Some(self.parse_expression()?)
        };
        
        Ok(Statement::Return(value))
    }

    fn parse_if_statement(&mut self) -> ParseResult<Statement> {
        self.advance(); // consume 'if'
        
        self.consume(Token::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after if condition")?;
        
        let then_block = self.parse_block()?;
        
        let else_block = if self.match_token(&Token::Else) {
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_block,
            else_block,
        })
    }

    fn parse_while_statement(&mut self) -> ParseResult<Statement> {
        self.advance(); // consume 'while'
        
        self.consume(Token::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::RightParen, "Expected ')' after while condition")?;
        
        let body = self.parse_block()?;
        
        Ok(Statement::While { condition, body })
    }

    fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_logical_and()?;
        
        while self.match_token(&Token::Or) {
            let right = self.parse_logical_and()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_equality()?;
        
        while self.match_token(&Token::And) {
            let right = self.parse_equality()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_comparison()?;
        
        while let Some(op) = self.match_equality_operator() {
            let right = self.parse_comparison()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_term()?;
        
        while let Some(op) = self.match_comparison_operator() {
            let right = self.parse_term()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_term(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_factor()?;
        
        while let Some(op) = self.match_term_operator() {
            let right = self.parse_factor()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_factor(&mut self) -> ParseResult<Expression> {
        let mut expr = self.parse_unary()?;
        
        while let Some(op) = self.match_factor_operator() {
            let right = self.parse_unary()?;
            expr = Expression::Binary {
                left: Box::new(expr),
                operator: op,
                right: Box::new(right),
            };
        }
        
        Ok(expr)
    }

    fn parse_unary(&mut self) -> ParseResult<Expression> {
        if let Some(op) = self.match_unary_operator() {
            let operand = self.parse_unary()?;
            Ok(Expression::Unary {
                operator: op,
                operand: Box::new(operand),
            })
        } else {
            self.parse_call()
        }
    }

    fn parse_call(&mut self) -> ParseResult<Expression> {
        let expr = self.parse_primary()?;
        
        if let Expression::Identifier(name) = expr {
            // Check for qualified call: module.function(args)
            if self.check(&Token::Dot) {
                self.advance(); // consume '.'
                let method_name = self.consume_identifier("Expected method name after '.'")?;
                let qualified_name = format!("{}.{}", name, method_name);
                
                if self.check(&Token::LeftParen) {
                    self.advance(); // consume '('
                    
                    let mut arguments = Vec::new();
                    if !self.check(&Token::RightParen) {
                        loop {
                            arguments.push(self.parse_expression()?);
                            if !self.match_token(&Token::Comma) {
                                break;
                            }
                        }
                    }
                    
                    self.consume(Token::RightParen, "Expected ')' after arguments")?;
                    
                    Ok(Expression::Call {
                        function: qualified_name,
                        arguments,
                    })
                } else {
                    // Just a qualified identifier, not a function call
                    Ok(Expression::Identifier(qualified_name))
                }
            } else if self.check(&Token::LeftParen) {
                // Regular function call
                self.advance(); // consume '('
                
                let mut arguments = Vec::new();
                if !self.check(&Token::RightParen) {
                    loop {
                        arguments.push(self.parse_expression()?);
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                
                self.consume(Token::RightParen, "Expected ')' after arguments")?;
                
                Ok(Expression::Call {
                    function: name,
                    arguments,
                })
            } else {
                Ok(Expression::Identifier(name))
            }
        } else {
            Ok(expr)
        }
    }

    fn parse_primary(&mut self) -> ParseResult<Expression> {
        match self.advance() {
            Token::True => Ok(Expression::Literal(Literal::Boolean(true))),
            Token::False => Ok(Expression::Literal(Literal::Boolean(false))),
            Token::IntegerLiteral(value) => Ok(Expression::Literal(Literal::Integer(value))),
            Token::FloatLiteral(value) => Ok(Expression::Literal(Literal::Float(value))),
            Token::StringLiteral(value) => Ok(Expression::Literal(Literal::String(value))),
            Token::Identifier(name) => Ok(Expression::Identifier(name)),
            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::If => {
                self.consume(Token::LeftParen, "Expected '(' after 'if'")?;
                let condition = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after if condition")?;
                
                self.consume(Token::LeftBrace, "Expected '{' after if condition")?;
                let then_expr = self.parse_expression()?;
                self.consume(Token::RightBrace, "Expected '}' after then expression")?;
                
                self.consume(Token::Else, "Expected 'else' after then block")?;
                self.consume(Token::LeftBrace, "Expected '{' after 'else'")?;
                let else_expr = self.parse_expression()?;
                self.consume(Token::RightBrace, "Expected '}' after else expression")?;
                
                Ok(Expression::If {
                    condition: Box::new(condition),
                    then_expr: Box::new(then_expr),
                    else_expr: Box::new(else_expr),
                })
            }
            _ => Err(self.error("Expected expression")),
        }
    }

    // Helper methods

    fn match_equality_operator(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Equal => {
                self.advance();
                Some(BinaryOperator::Equal)
            }
            Token::NotEqual => {
                self.advance();
                Some(BinaryOperator::NotEqual)
            }
            _ => None,
        }
    }

    fn match_comparison_operator(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Greater => {
                self.advance();
                Some(BinaryOperator::Greater)
            }
            Token::GreaterEqual => {
                self.advance();
                Some(BinaryOperator::GreaterEqual)
            }
            Token::Less => {
                self.advance();
                Some(BinaryOperator::Less)
            }
            Token::LessEqual => {
                self.advance();
                Some(BinaryOperator::LessEqual)
            }
            _ => None,
        }
    }

    fn match_term_operator(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Plus => {
                self.advance();
                Some(BinaryOperator::Add)
            }
            Token::Minus => {
                self.advance();
                Some(BinaryOperator::Subtract)
            }
            _ => None,
        }
    }

    fn match_factor_operator(&mut self) -> Option<BinaryOperator> {
        match self.peek() {
            Token::Multiply => {
                self.advance();
                Some(BinaryOperator::Multiply)
            }
            Token::Divide => {
                self.advance();
                Some(BinaryOperator::Divide)
            }
            _ => None,
        }
    }

    fn match_unary_operator(&mut self) -> Option<UnaryOperator> {
        match self.peek() {
            Token::Minus => {
                self.advance();
                Some(UnaryOperator::Minus)
            }
            Token::Not => {
                self.advance();
                Some(UnaryOperator::Not)
            }
            _ => None,
        }
    }

    fn is_assignment(&self) -> bool {
        if let Some(Token::Identifier(_)) = self.tokens.get(self.current) {
            if let Some(Token::Assign) = self.tokens.get(self.current + 1) {
                return true;
            }
        }
        false
    }

    pub fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(self.peek()) == std::mem::discriminant(token)
        }
    }

    pub fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous().clone()
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::Error)
    }

    pub fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    pub fn consume(&mut self, token: Token, message: &str) -> ParseResult<()> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    pub fn consume_identifier(&mut self, message: &str) -> ParseResult<String> {
        match self.advance() {
            Token::Identifier(name) => Ok(name),
            _ => Err(self.error(message)),
        }
    }

    pub fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            position: self.current,
        }
    }

    /// Parse a module path like "std.io.file"
    pub fn parse_module_path(&mut self) -> ParseResult<ModulePath> {
        let mut segments = Vec::new();
        
        // First segment must be an identifier
        segments.push(self.consume_identifier("Expected module name")?);
        
        // Parse additional segments separated by dots
        while self.match_token(&Token::Dot) {
            segments.push(self.consume_identifier("Expected module name after '.'")?);
        }
        
        Ok(ModulePath::new(segments))
    }
    
    /// Parse an import statement
    /// Syntax: import std.io.file
    /// Syntax: import std.io.file.{read, write}
    /// Syntax: import std.io.file as io
    pub fn parse_import(&mut self) -> ParseResult<Import> {
        self.consume(Token::Import, "Expected 'import'")?;
        
        let path = self.parse_module_path()?;
        
        // Check for specific items import: import mod.{item1, item2}
        let items = if self.match_token(&Token::LeftBrace) {
            let mut imported_items = Vec::new();
            
            if !self.check(&Token::RightBrace) {
                loop {
                    imported_items.push(self.consume_identifier("Expected item name")?);
                    if !self.match_token(&Token::Comma) {
                        break;
                    }
                }
            }
            
            self.consume(Token::RightBrace, "Expected '}' after import items")?;
            Some(imported_items)
        } else {
            None
        };
        
        // Check for alias: import mod as alias
        let alias = if self.match_token(&Token::As) {
            Some(self.consume_identifier("Expected alias name")?)
        } else {
            None
        };
        
        Ok(Import { path, items, alias })
    }
    
    /// Parse visibility modifier
    pub fn parse_visibility(&mut self) -> ParseResult<Visibility> {
        match self.peek() {
            Token::Pub => {
                self.advance();
                // Check for restricted visibility like pub(crate), pub(super), etc.
                if self.match_token(&Token::LeftParen) {
                    let restriction = self.parse_visibility_restriction()?;
                    self.consume(Token::RightParen, "Expected ')' after visibility restriction")?;
                    Ok(Visibility::Restricted { restriction })
                } else {
                    Ok(Visibility::Public)
                }
            }
            Token::Internal => {
                self.advance();
                Ok(Visibility::Internal)
            }
            Token::Protected => {
                self.advance();
                Ok(Visibility::Protected)
            }
            Token::Package => {
                self.advance();
                Ok(Visibility::Package)
            }
            _ => Ok(Visibility::Private)
        }
    }

    /// Parse visibility restriction for pub(restriction) syntax
    fn parse_visibility_restriction(&mut self) -> ParseResult<VisibilityRestriction> {
        match self.peek() {
            Token::Crate => {
                self.advance();
                Ok(VisibilityRestriction::Crate)
            }
            Token::Super => {
                self.advance();
                Ok(VisibilityRestriction::Super)
            }
            Token::SelfKeyword => {
                self.advance();
                Ok(VisibilityRestriction::Module)
            }
            Token::In => {
                self.advance();
                let path = self.parse_module_path()?;
                Ok(VisibilityRestriction::Path(path))
            }
            _ => Err(self.error("Expected 'crate', 'super', 'self', or 'in' after 'pub('"))
        }
    }
    
    /// Parse a module declaration
    /// Syntax: mod mymodule { ... }
    /// Syntax: pub mod mymodule { ... }
    pub fn parse_module_declaration(&mut self) -> ParseResult<ModuleDecl> {
        let visibility = self.parse_visibility()?;
        
        self.consume(Token::Mod, "Expected 'mod'")?;
        let name = self.consume_identifier("Expected module name")?;
        
        self.consume(Token::LeftBrace, "Expected '{' after module name")?;
        
        let mut items = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            // Parse items with their own visibility modifiers
            match self.peek() {
                Token::Fun | Token::Pub | Token::Internal | Token::Protected | Token::Package => {
                    let function = self.parse_function()?;
                    items.push(Item::Function(function));
                }
                _ => {
                    return Err(self.error("Only functions are currently supported in modules"));
                }
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after module items")?;
        
        Ok(ModuleDecl {
            name,
            visibility,
            items,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_function() {
        let input = r#"
            fun add(a: Int, b: Int): Int {
                return a + b
            }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        assert_eq!(program.functions.len(), 1);
        let func = &program.functions[0];
        assert_eq!(func.name, "add");
        assert_eq!(func.parameters.len(), 2);
        assert_eq!(func.return_type, Type::Int);
    }

    #[test]
    fn test_arithmetic_expression() {
        let input = r#"
            fun test(): Int {
                return 1 + 2 * 3
            }
        "#;
        
        let mut parser = Parser::new(input);
        let program = parser.parse_program().unwrap();
        
        assert_eq!(program.functions.len(), 1);
        // Additional assertions would verify the AST structure
    }
} 