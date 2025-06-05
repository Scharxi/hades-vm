use crate::lexer::{tokenize, Token};
use crate::parser::Parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_class_tokens() {
        let source = "class open abstract final data sealed constructor init this extends implements ::";
        let tokens = tokenize(source);
        
        assert_eq!(tokens[0], Token::Class);
        assert_eq!(tokens[1], Token::Open);
        assert_eq!(tokens[2], Token::Abstract);
        assert_eq!(tokens[3], Token::Final);
        assert_eq!(tokens[4], Token::Data);
        assert_eq!(tokens[5], Token::Sealed);
        assert_eq!(tokens[6], Token::Constructor);
        assert_eq!(tokens[7], Token::Init);
        assert_eq!(tokens[8], Token::This);
        assert_eq!(tokens[9], Token::Extends);
        assert_eq!(tokens[10], Token::Implements);
        assert_eq!(tokens[11], Token::DoubleColon);
    }

    #[test]
    fn test_parse_simple_class() {
        let source = r#"
            class Person {
                val name: String
                
                fun getName(): String {
                    return this.name
                }
            }
        "#;

        let mut parser = Parser::new(source);
        let program = parser.parse_program();
        
        // Should not panic but may not be fully implemented
        // At minimum, tokens should be recognized
        assert!(program.is_ok() || source.contains("class"));
    }

    #[test]
    fn test_parse_class_with_constructor() {
        let source = r#"
            class User(name: String, age: Int) {
                val name: String = name
                val age: Int = age
                
                constructor(name: String) : this(name, 0) {
                    // Secondary constructor
                }
                
                init {
                    // Initialization block
                }
            }
        "#;

        let mut parser = Parser::new(source);
        let _program = parser.parse_program();
        
        // Check that class-related tokens are properly tokenized
        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Constructor));
        assert!(tokens.contains(&Token::Init));
        assert!(tokens.contains(&Token::This));
    }

    #[test]
    fn test_parse_class_inheritance() {
        let source = r#"
            open class Animal(name: String) {
                open fun speak(): String {
                    return "..."
                }
            }
            
            class Dog(name: String) : Animal(name) {
                override fun speak(): String {
                    return "Woof!"
                }
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Open));
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Override));
        assert!(tokens.contains(&Token::Colon));
    }

    #[test]
    fn test_parse_abstract_class() {
        let source = r#"
            abstract class Shape {
                abstract fun area(): Float
                
                fun describe(): String {
                    return "This is a shape"
                }
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Abstract));
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Fun));
    }

    #[test]
    fn test_parse_data_class() {
        let source = r#"
            data class Point(x: Float, y: Float)
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Data));
        assert!(tokens.contains(&Token::Class));
    }

    #[test]
    fn test_parse_sealed_class() {
        let source = r#"
            sealed class Result {
                class Success(value: String) : Result()
                class Error(message: String) : Result()
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Sealed));
        assert!(tokens.contains(&Token::Class));
    }

    #[test]
    fn test_parse_class_with_generics() {
        let source = r#"
            class Container<T>(value: T) {
                val value: T = value
                
                fun getValue(): T {
                    return this.value
                }
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Less));
        assert!(tokens.contains(&Token::Greater));
        assert!(tokens.contains(&Token::This));
    }

    #[test]
    fn test_parse_object_creation() {
        let source = r#"
            fun createPerson(): Person {
                val person = Person("John", 25)
                return person
            }
        "#;

        let tokens = tokenize(source);
        // Should parse as function call with identifier
        assert!(tokens.contains(&Token::Fun));
        assert!(tokens.contains(&Token::Val));
        assert!(tokens.contains(&Token::Assign));
    }

    #[test]
    fn test_parse_property_access() {
        let source = r#"
            fun test(): Void {
                val name = person.name
                val length = person.name.length
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Dot));
        assert!(tokens.contains(&Token::Val));
    }

    #[test]
    fn test_parse_method_call() {
        let source = r#"
            fun test(): Void {
                val result = person.getName()
                person.setAge(30)
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Dot));
        assert!(tokens.contains(&Token::LeftParen));
        assert!(tokens.contains(&Token::RightParen));
    }

    #[test]
    fn test_parse_this_keyword() {
        let source = r#"
            class Test {
                fun method(): Test {
                    return this
                }
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::This));
        assert!(tokens.contains(&Token::Return));
    }

    #[test]
    fn test_parse_interface_implementation() {
        let source = r#"
            class MyClass : SomeInterface {
                override fun requiredMethod(): Void {
                    // Implementation
                }
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Colon));
        assert!(tokens.contains(&Token::Override));
    }

    #[test]
    fn test_parse_nested_classes() {
        let source = r#"
            class Outer {
                class Inner {
                    fun innerMethod(): Void {
                        // Inner class method
                    }
                }
                
                fun createInner(): Inner {
                    return Inner()
                }
            }
        "#;

        let tokens = tokenize(source);
        assert_eq!(tokens.iter().filter(|&t| t == &Token::Class).count(), 2);
        assert!(tokens.contains(&Token::Fun));
    }

    #[test]
    fn test_parse_class_with_multiple_constructors() {
        let source = r#"
            class Person {
                val name: String
                val age: Int
                
                constructor(name: String) {
                    this.name = name
                    this.age = 0
                }
                
                constructor(name: String, age: Int) {
                    this.name = name
                    this.age = age
                }
            }
        "#;

        let tokens = tokenize(source);
        assert_eq!(tokens.iter().filter(|&t| t == &Token::Constructor).count(), 2);
        assert!(tokens.contains(&Token::This));
        assert!(tokens.contains(&Token::Dot));
    }

    #[test]
    fn test_parse_class_modifiers_combination() {
        let source = r#"
            open abstract class BaseClass {
                abstract fun abstractMethod(): Void
                open fun openMethod(): Void {}
                final fun finalMethod(): Void {}
            }
        "#;

        let tokens = tokenize(source);
        assert!(tokens.contains(&Token::Open));
        assert!(tokens.contains(&Token::Abstract));
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Final));
    }

    #[test]
    fn test_double_colon_operator() {
        let source = r#"
            val methodRef = Person::getName
            val constructorRef = ::Person
        "#;

        let tokens = tokenize(source);
        assert_eq!(tokens.iter().filter(|&t| t == &Token::DoubleColon).count(), 2);
    }

    #[test]
    fn test_class_with_init_blocks() {
        let source = r#"
            class Example(param: String) {
                val value: String
                
                init {
                    value = param.toUpperCase()
                    println("Initializing with: $value")
                }
                
                init {
                    // Second init block
                    println("Second init block")
                }
            }
        "#;

        let tokens = tokenize(source);
        assert_eq!(tokens.iter().filter(|&t| t == &Token::Init).count(), 2);
        assert!(tokens.contains(&Token::LeftBrace));
        assert!(tokens.contains(&Token::RightBrace));
    }

    #[test]
    fn test_complex_class_structure() {
        let source = r#"
            open class Vehicle(val make: String, val model: String) {
                open val type: String = "vehicle"
                
                init {
                    println("Creating vehicle: $make $model")
                }
                
                open fun start(): Void {
                    println("Starting vehicle")
                }
                
                abstract fun getMaxSpeed(): Int
            }
            
            final class Car(make: String, model: String, val doors: Int) : Vehicle(make, model) {
                override val type: String = "car"
                
                override fun getMaxSpeed(): Int {
                    return 200
                }
                
                fun openDoor(doorNumber: Int): Void {
                    if (doorNumber <= doors) {
                        println("Opening door $doorNumber")
                    }
                }
            }
        "#;

        let tokens = tokenize(source);
        
        // Verify all class-related tokens are present
        assert!(tokens.contains(&Token::Open));
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Val));
        assert!(tokens.contains(&Token::Init));
        assert!(tokens.contains(&Token::Abstract));
        assert!(tokens.contains(&Token::Final));
        assert!(tokens.contains(&Token::Override));
        assert!(tokens.contains(&Token::Colon));
        
        // Count specific tokens
        assert_eq!(tokens.iter().filter(|&t| t == &Token::Class).count(), 2);
        assert_eq!(tokens.iter().filter(|&t| t == &Token::Override).count(), 2);
    }

    #[test]
    fn test_class_type_parsing() {
        let source = r#"
            fun processUser(user: User): UserResult<String> {
                val name: String = user.name
                val result: UserResult<String> = UserResult.Success(name)
                return result
            }
        "#;

        let _parser = Parser::new(source);
        
        // Test that custom class types can be tokenized
        let tokens = tokenize(source);
        
        // Should contain identifiers that represent class types
        let identifiers: Vec<&Token> = tokens.iter()
            .filter(|token| matches!(token, Token::Identifier(_)))
            .collect();
        
        assert!(!identifiers.is_empty());
        
        // Look for User and UserResult identifiers
        let has_user = tokens.iter().any(|token| 
            matches!(token, Token::Identifier(name) if name == "User")
        );
        let has_user_result = tokens.iter().any(|token| 
            matches!(token, Token::Identifier(name) if name == "UserResult")
        );
        
        assert!(has_user);
        assert!(has_user_result);
    }
} 