mod token;
pub use token::{Token, TokenKind, Span};

use std::str::Chars;
use std::iter::Peekable;

pub struct Lexer<'a> {
    input: Peekable<Chars<'a>>,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.chars().peekable(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        let token = match self.peek() {
            None => self.create_token(TokenKind::EOF, start_pos, start_line, start_column),
            Some(c) => {
                match c {
                    '0'..='9' => self.read_number(),
                    'a'..='z' | 'A'..='Z' => self.read_identifier(),
                    '_' => {
                        self.advance();
                        if let Some(next) = self.peek() {
                            if next.is_alphanumeric() || next == '_' {
                                let mut identifier = String::from("_");
                                while let Some(c) = self.peek() {
                                    if c.is_alphanumeric() || c == '_' {
                                        identifier.push(c);
                                        self.advance();
                                    } else {
                                        break;
                                    }
                                }
                                return self.create_token(TokenKind::Identifier(identifier), start_pos, start_line, start_column);
                            }
                        }
                        self.create_token(TokenKind::Underscore, start_pos, start_line, start_column)
                    },
                    '"' => self.read_string(),
                    '+' => self.single_char_token(TokenKind::Plus),
                    '-' => self.read_minus_or_arrow(),
                    '*' => self.single_char_token(TokenKind::Star),
                    '/' => self.single_char_token(TokenKind::Slash),
                    '=' => self.read_equals(),
                    '<' => self.read_less_than(),
                    '>' => self.read_greater_than(),
                    '|' => {
                        self.advance();
                        match self.peek() {
                            Some('|') => {
                                self.advance();
                                self.create_token(TokenKind::Or, start_pos, start_line, start_column)
                            }
                            Some('!') => {
                                self.advance();
                                Token::error("unexpected '!' after '|'", Span {
                                    start: start_pos,
                                    end: self.position,
                                    line: start_line,
                                    column: start_column,
                                })
                            }
                            Some(c) => {
                                Token::error(format!("unexpected character after '|': {}", c), Span {
                                    start: start_pos,
                                    end: self.position + 1,
                                    line: start_line,
                                    column: start_column,
                                })
                            }
                            None => Token::error("Expected '|' after '|'", Span {
                                start: start_pos,
                                end: self.position,
                                line: start_line,
                                column: start_column,
                            })
                        }
                    },
                    '&' => {
                        self.advance();
                        match self.peek() {
                            Some('&') => {
                                self.advance();
                                self.create_token(TokenKind::And, start_pos, start_line, start_column)
                            }
                            Some('!') => {
                                self.advance();
                                Token::error("unexpected '!' after '&'", Span {
                                    start: start_pos,
                                    end: self.position,
                                    line: start_line,
                                    column: start_column,
                                })
                            }
                            Some(c) => {
                                Token::error(format!("unexpected character after '&': {}", c), Span {
                                    start: start_pos,
                                    end: self.position + 1,
                                    line: start_line,
                                    column: start_column,
                                })
                            }
                            None => Token::error("Expected '&' after '&'", Span {
                                start: start_pos,
                                end: self.position,
                                line: start_line,
                                column: start_column,
                            })
                        }
                    },
                    '!' => {
                        self.advance();
                        match self.peek() {
                            Some('=') => {
                                self.advance();
                                self.create_token(TokenKind::NotEq, start_pos, start_line, start_column)
                            }
                            Some(c) if !c.is_whitespace() => {
                                Token::error(format!("unexpected character after '!': {}", c), Span {
                                    start: start_pos,
                                    end: self.position + 1,
                                    line: start_line,
                                    column: start_column,
                                })
                            }
                            _ => self.create_token(TokenKind::Not, start_pos, start_line, start_column)
                        }
                    },
                    '(' => self.single_char_token(TokenKind::LParen),
                    ')' => self.single_char_token(TokenKind::RParen),
                    '{' => self.single_char_token(TokenKind::LBrace),
                    '}' => self.single_char_token(TokenKind::RBrace),
                    '[' => self.single_char_token(TokenKind::LBracket),
                    ']' => self.single_char_token(TokenKind::RBracket),
                    ',' => self.single_char_token(TokenKind::Comma),
                    '.' => self.single_char_token(TokenKind::Dot),
                    ':' => self.single_char_token(TokenKind::Colon),
                    ';' => self.single_char_token(TokenKind::Semicolon),
                    _ => {
                        let message = format!("unexpected character: {}", c);
                        self.advance();
                        Token::error(message, Span {
                            start: start_pos,
                            end: self.position,
                            line: start_line,
                            column: start_column,
                        })
                    }
                }
            }
        };
        
        token
    }
    
    fn read_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        let mut number = String::new();
        let mut is_float = false;
        
        while let Some(c) = self.peek() {
            match c {
                '0'..='9' => {
                    number.push(c);
                    self.advance();
                }
                '.' => {
                    if is_float {
                        break;
                    }
                    is_float = true;
                    number.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        
        let kind = if is_float {
            match number.parse() {
                Ok(n) => TokenKind::Float(n),
                Err(_) => return Token::error("invalid float literal", Span {
                    start: start_pos,
                    end: self.position,
                    line: start_line,
                    column: start_column,
                }),
            }
        } else {
            match number.parse() {
                Ok(n) => TokenKind::Integer(n),
                Err(_) => return Token::error("invalid integer literal", Span {
                    start: start_pos,
                    end: self.position,
                    line: start_line,
                    column: start_column,
                }),
            }
        };
        
        self.create_token(kind, start_pos, start_line, start_column)
    }
    
    fn read_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        let mut identifier = String::new();
        
        while let Some(c) = self.peek() {
            match c {
                'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => {
                    identifier.push(c);
                    self.advance();
                }
                _ => break,
            }
        }
        
        let kind = match identifier.as_str() {
            "let" => TokenKind::Let,
            "var" => TokenKind::Var,
            "mut" => TokenKind::Mut,
            "fn" => TokenKind::Fn,
            "return" => TokenKind::Return,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "when" => TokenKind::When,
            "is" => TokenKind::Is,
            "trait" => TokenKind::Trait,
            "impl" => TokenKind::Impl,
            "enum" => TokenKind::Enum,
            "struct" => TokenKind::Struct,
            "async" => TokenKind::Async,
            "await" => TokenKind::Await,
            "type" => TokenKind::Type,
            "const" => TokenKind::Const,
            "for" => TokenKind::For,
            "true" => TokenKind::Boolean(true),
            "false" => TokenKind::Boolean(false),
            _ => TokenKind::Identifier(identifier),
        };
        
        self.create_token(kind, start_pos, start_line, start_column)
    }
    
    fn read_string(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip opening quote
        
        let mut string = String::new();
        
        while let Some(c) = self.peek() {
            match c {
                '"' => {
                    self.advance(); // Skip closing quote
                    return self.create_token(
                        TokenKind::String(string),
                        start_pos,
                        start_line,
                        start_column,
                    );
                }
                '\\' => {
                    self.advance();
                    match self.peek() {
                        Some('n') => string.push('\n'),
                        Some('r') => string.push('\r'),
                        Some('t') => string.push('\t'),
                        Some('\\') => string.push('\\'),
                        Some('"') => string.push('"'),
                        Some(c) => {
                            return Token::error(
                                format!("invalid escape sequence: \\{}", c),
                                Span {
                                    start: start_pos,
                                    end: self.position,
                                    line: start_line,
                                    column: start_column,
                                },
                            );
                        }
                        None => {
                            return Token::error(
                                "unterminated string literal",
                                Span {
                                    start: start_pos,
                                    end: self.position,
                                    line: start_line,
                                    column: start_column,
                                },
                            );
                        }
                    }
                    self.advance();
                }
                '\n' => {
                    return Token::error(
                        "unterminated string literal",
                        Span {
                            start: start_pos,
                            end: self.position,
                            line: start_line,
                            column: start_column,
                        },
                    );
                }
                c => {
                    string.push(c);
                    self.advance();
                }
            }
        }
        
        Token::error(
            "unterminated string literal",
            Span {
                start: start_pos,
                end: self.position,
                line: start_line,
                column: start_column,
            },
        )
    }
    
    fn read_minus_or_arrow(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip '-'
        
        if let Some('>') = self.peek() {
            self.advance();
            self.create_token(TokenKind::Arrow, start_pos, start_line, start_column)
        } else {
            self.create_token(TokenKind::Minus, start_pos, start_line, start_column)
        }
    }
    
    fn read_equals(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip first '='
        
        if let Some('=') = self.peek() {
            self.advance();
            self.create_token(TokenKind::Eq, start_pos, start_line, start_column)
        } else if let Some('>') = self.peek() {
            self.advance();
            self.create_token(TokenKind::FatArrow, start_pos, start_line, start_column)
        } else {
            self.create_token(TokenKind::Assign, start_pos, start_line, start_column)
        }
    }
    
    fn read_less_than(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip '<'
        
        if let Some('=') = self.peek() {
            self.advance();
            self.create_token(TokenKind::LtEq, start_pos, start_line, start_column)
        } else {
            self.create_token(TokenKind::Lt, start_pos, start_line, start_column)
        }
    }
    
    fn read_greater_than(&mut self) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance(); // Skip '>'
        
        if let Some('=') = self.peek() {
            self.advance();
            self.create_token(TokenKind::GtEq, start_pos, start_line, start_column)
        } else {
            self.create_token(TokenKind::Gt, start_pos, start_line, start_column)
        }
    }
    
    fn single_char_token(&mut self, kind: TokenKind) -> Token {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        self.advance();
        self.create_token(kind, start_pos, start_line, start_column)
    }
    
    fn create_token(&self, kind: TokenKind, start: usize, line: usize, column: usize) -> Token {
        Token::new(
            kind,
            Span {
                start,
                end: self.position,
                line,
                column,
            },
        )
    }
    
    fn peek(&mut self) -> Option<char> {
        self.input.peek().copied()
    }
    
    fn advance(&mut self) -> Option<char> {
        let c = self.input.next();
        if let Some(c) = c {
            self.position += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }
    
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            self.advance();
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.next_token())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let input = "let var mut fn return if else when is trait impl enum struct async await";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Let);
        assert_eq!(lexer.next_token().kind, TokenKind::Var);
        assert_eq!(lexer.next_token().kind, TokenKind::Mut);
        assert_eq!(lexer.next_token().kind, TokenKind::Fn);
        assert_eq!(lexer.next_token().kind, TokenKind::Return);
        assert_eq!(lexer.next_token().kind, TokenKind::If);
        assert_eq!(lexer.next_token().kind, TokenKind::Else);
        assert_eq!(lexer.next_token().kind, TokenKind::When);
        assert_eq!(lexer.next_token().kind, TokenKind::Is);
        assert_eq!(lexer.next_token().kind, TokenKind::Trait);
        assert_eq!(lexer.next_token().kind, TokenKind::Impl);
        assert_eq!(lexer.next_token().kind, TokenKind::Enum);
        assert_eq!(lexer.next_token().kind, TokenKind::Struct);
        assert_eq!(lexer.next_token().kind, TokenKind::Async);
        assert_eq!(lexer.next_token().kind, TokenKind::Await);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_literals() {
        let input = r#"42 3.14 "hello" true false"#;
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(42));
        assert_eq!(lexer.next_token().kind, TokenKind::Float(3.14));
        assert_eq!(lexer.next_token().kind, TokenKind::String("hello".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Boolean(true));
        assert_eq!(lexer.next_token().kind, TokenKind::Boolean(false));
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_operators() {
        let input = "+ - * / = == => -> < > <= >= != && ||";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Plus);
        assert_eq!(lexer.next_token().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().kind, TokenKind::Star);
        assert_eq!(lexer.next_token().kind, TokenKind::Slash);
        assert_eq!(lexer.next_token().kind, TokenKind::Assign);
        assert_eq!(lexer.next_token().kind, TokenKind::Eq);
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::Arrow);
        assert_eq!(lexer.next_token().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().kind, TokenKind::Gt);
        assert_eq!(lexer.next_token().kind, TokenKind::LtEq);
        assert_eq!(lexer.next_token().kind, TokenKind::GtEq);
        assert_eq!(lexer.next_token().kind, TokenKind::NotEq);
        assert_eq!(lexer.next_token().kind, TokenKind::And);
        assert_eq!(lexer.next_token().kind, TokenKind::Or);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_string_escapes() {
        let input = r#""Hello\nWorld" "Tab\there" "Quote\"here" "Backslash\\""#;
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::String("Hello\nWorld".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::String("Tab\there".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::String("Quote\"here".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::String("Backslash\\".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_invalid_string() {
        let input = r#""unterminated"#;
        let mut lexer = Lexer::new(input);
        
        match lexer.next_token().kind {
            TokenKind::Error(_) => (),
            other => panic!("Expected error token, got {:?}", other),
        }
    }

    #[test]
    fn test_identifiers() {
        let input = "foo bar_baz _hidden camelCase UPPERCASE";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("foo".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("bar_baz".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("_hidden".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("camelCase".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("UPPERCASE".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_numbers() {
        let input = "42 3.14159 0.123 123.456 0 -42 -3.14";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(42));
        assert_eq!(lexer.next_token().kind, TokenKind::Float(3.14159));
        assert_eq!(lexer.next_token().kind, TokenKind::Float(0.123));
        assert_eq!(lexer.next_token().kind, TokenKind::Float(123.456));
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(0));
        assert_eq!(lexer.next_token().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(42));
        assert_eq!(lexer.next_token().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().kind, TokenKind::Float(3.14));
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_delimiters() {
        let input = "( ) { } [ ] , . : ;";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::LBracket);
        assert_eq!(lexer.next_token().kind, TokenKind::RBracket);
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().kind, TokenKind::Dot);
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_complex_code() {
        let input = r#"
            fn factorial(n: i32) -> i32 {
                when (n) {
                    0 => { return 1; }
                    n => { return n * factorial(n - 1); }
                }
            }
        "#;
        let mut lexer = Lexer::new(input);
        
        // Function declaration
        assert_eq!(lexer.next_token().kind, TokenKind::Fn);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("factorial".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("n".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("i32".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Arrow);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("i32".to_string()));
        
        // Function body
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::When);
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("n".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        
        // First when arm
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(0));
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::Return);
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(1));
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        
        // Second when arm
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("n".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::Return);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("n".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Star);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("factorial".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("n".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Minus);
        assert_eq!(lexer.next_token().kind, TokenKind::Integer(1));
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        
        // Closing braces
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_logical_operators() {
        let input = "! && || != true false";
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Not);
        assert_eq!(lexer.next_token().kind, TokenKind::And);
        assert_eq!(lexer.next_token().kind, TokenKind::Or);
        assert_eq!(lexer.next_token().kind, TokenKind::NotEq);
        assert_eq!(lexer.next_token().kind, TokenKind::Boolean(true));
        assert_eq!(lexer.next_token().kind, TokenKind::Boolean(false));
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_invalid_operators() {
        let input = "& | &! |!";
        let mut lexer = Lexer::new(input);
        
        match lexer.next_token().kind {
            TokenKind::Error(_) => (),
            other => panic!("Expected error token for single '&', got {:?}", other),
        }
        
        match lexer.next_token().kind {
            TokenKind::Error(_) => (),
            other => panic!("Expected error token for single '|', got {:?}", other),
        }
        
        match lexer.next_token().kind {
            TokenKind::Error(_) => (),
            other => panic!("Expected error token for '&!', got {:?}", other),
        }
        
        match lexer.next_token().kind {
            TokenKind::Error(_) => (),
            other => panic!("Expected error token for '|!', got {:?}", other),
        }
    }

    #[test]
    fn test_pattern_matching() {
        let input = r#"
            when (value) {
                is String => { "it's a string" }
                _ => { "wildcard" }
            }
        "#;
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::When);
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("value".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Is);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("String".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::String("it's a string".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Underscore);
        assert_eq!(lexer.next_token().kind, TokenKind::FatArrow);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::String("wildcard".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_async_function() {
        let input = r#"
            async fn fetch_data() -> Result<String, Error> {
                let response = http.get("https://api.example.com").await;
                return response;
            }
        "#;
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Async);
        assert_eq!(lexer.next_token().kind, TokenKind::Fn);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("fetch_data".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Arrow);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("Result".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("String".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("Error".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Gt);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Let);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("response".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Assign);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("http".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Dot);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("get".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::LParen);
        assert_eq!(lexer.next_token().kind, TokenKind::String("https://api.example.com".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::RParen);
        assert_eq!(lexer.next_token().kind, TokenKind::Dot);
        assert_eq!(lexer.next_token().kind, TokenKind::Await);
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Return);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("response".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Semicolon);
        
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }

    #[test]
    fn test_struct_definition() {
        let input = r#"
            struct User<T> {
                name: String,
                age: i32,
                data: T,
                mut settings: HashMap<String, bool>
            }
        "#;
        let mut lexer = Lexer::new(input);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Struct);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("User".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("T".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Gt);
        assert_eq!(lexer.next_token().kind, TokenKind::LBrace);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("name".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("String".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("age".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("i32".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("data".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("T".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        
        assert_eq!(lexer.next_token().kind, TokenKind::Mut);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("settings".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Colon);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("HashMap".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Lt);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("String".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Comma);
        assert_eq!(lexer.next_token().kind, TokenKind::Identifier("bool".to_string()));
        assert_eq!(lexer.next_token().kind, TokenKind::Gt);
        
        assert_eq!(lexer.next_token().kind, TokenKind::RBrace);
        assert_eq!(lexer.next_token().kind, TokenKind::EOF);
    }
} 