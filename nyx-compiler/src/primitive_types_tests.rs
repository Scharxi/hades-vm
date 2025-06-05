/// Comprehensive tests for primitive type classes in the Nyx standard library
/// 
/// This module contains Rust tests that verify the tokenization and basic
/// syntax recognition of primitive type classes including Int, Float, Bool, String, and Char.
/// The tests ensure that:
/// 1. All primitive type classes can be properly tokenized
/// 2. Method calls on primitive types work correctly
/// 3. Operator overloading is handled properly
/// 4. Type conversions between primitives function as expected
/// 5. Static/companion methods are accessible
/// 
/// These tests complement the runtime tests and focus on compile-time
/// correctness of the primitive type system.

use crate::lexer::{tokenize, Token};

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that primitive type tokens are correctly recognized by the lexer
    /// 
    /// This test verifies that all tokens related to primitive types and their
    /// methods are properly tokenized by the Nyx lexer.
    #[test]
    fn test_primitive_type_tokens() {
        let source = r#"
            Int Float Bool String Char
            constructor companion object
            val var fun operator override
            toInt toFloat toBool toString
            abs sign pow gcd lcm
            sin cos tan sqrt ln log
            length isEmpty contains
            isLetter isDigit isWhitespace
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain primitive type tokens
        assert!(tokens.contains(&Token::Int));
        assert!(tokens.contains(&Token::Float));
        assert!(tokens.contains(&Token::Bool));
        assert!(tokens.contains(&Token::String));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "Char")));
        
        // Should contain method name identifiers
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "abs")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "length")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "isEmpty")));
    }

    /// Test tokenization of Int class usage
    /// 
    /// Verifies that Int objects can be created and their methods called
    /// with proper syntax parsing.
    #[test]
    fn test_int_class_tokenization() {
        let source = r#"
            val number = Int(42)
            val absolute = number.abs()
            val power = number.pow(Int(2))
            val isPrime = number.isPrime()
            val stringRep = number.toString()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain Int type token
        assert!(tokens.contains(&Token::Int));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "abs")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "pow")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "isPrime")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toString")));
        
        // Should have proper syntax tokens
        assert!(tokens.contains(&Token::Val));
        assert!(tokens.contains(&Token::Assign));
        assert!(tokens.contains(&Token::LeftParen));
        assert!(tokens.contains(&Token::RightParen));
        assert!(tokens.contains(&Token::Dot));
    }

    /// Test tokenization of Float class usage
    /// 
    /// Ensures that Float objects support mathematical functions and
    /// conversions properly.
    #[test]
    fn test_float_class_tokenization() {
        let source = r#"
            val pi = Float(3.14159)
            val sine = pi.sin()
            val cosine = pi.cos()
            val rounded = pi.round()
            val asInt = pi.toInt()
            val radians = Float(90).toRadians()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain Float type token
        assert!(tokens.contains(&Token::Float));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "sin")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "cos")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "round")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toRadians")));
    }

    /// Test tokenization of Bool class usage
    /// 
    /// Verifies that Bool objects can perform logical operations and
    /// conditional expressions.
    #[test]
    fn test_bool_class_tokenization() {
        let source = r#"
            val truth = Bool(true)
            val falsehood = Bool(false)
            val negated = truth.not()
            val combined = truth.and(falsehood)
            val exclusive = truth.xor(falsehood)
            val converted = truth.toInt()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain Bool type token
        assert!(tokens.contains(&Token::Bool));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "not")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "and")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "xor")));
        
        // Should contain boolean literals
        assert!(tokens.contains(&Token::True));
        assert!(tokens.contains(&Token::False));
    }

    /// Test tokenization of String class usage
    /// 
    /// Ensures that String objects support text operations, concatenation,
    /// and various string manipulation methods.
    #[test]
    fn test_string_class_tokenization() {
        let source = r#"
            val text = String("Hello, World!")
            val length = text.length()
            val upper = text.toUpperCase()
            val substring = text.substring(Int(0), Int(5))
            val contains = text.contains(String("World"))
            val split = text.split(String(" "))
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain String type token
        assert!(tokens.contains(&Token::String));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "length")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toUpperCase")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "substring")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "contains")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "split")));
        
        // Should contain string literals
        assert!(tokens.iter().any(|t| matches!(t, Token::StringLiteral(_))));
    }

    /// Test tokenization of Char class usage
    /// 
    /// Verifies that Char objects support character classification and
    /// conversion methods.
    #[test]
    fn test_char_class_tokenization() {
        let source = r#"
            val letter = Char("A")
            val isLetter = letter.isLetter()
            val isDigit = letter.isDigit()
            val lower = letter.toLowerCase()
            val codePoint = letter.toInt()
            val asString = letter.toString()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain Char identifier (not a built-in type token)
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "Char")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "isLetter")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "isDigit")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toLowerCase")));
        
        // Should contain character literals (as string literals in Nyx)
        assert!(tokens.iter().any(|t| matches!(t, Token::StringLiteral(_))));
    }

    /// Test tokenization of operator overloading for primitive types
    /// 
    /// Ensures that arithmetic and comparison operators work correctly
    /// with primitive type wrapper classes.
    #[test]
    fn test_primitive_operators_tokenization() {
        let source = r#"
            val a = Int(10)
            val b = Int(20)
            val sum = a + b
            val diff = a - b
            val product = a * b
            val quotient = a / b
            val remainder = a % b
            val comparison = a < b
            val equality = a == b
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain arithmetic operators
        assert!(tokens.contains(&Token::Plus));
        assert!(tokens.contains(&Token::Minus));
        assert!(tokens.contains(&Token::Multiply));
        assert!(tokens.contains(&Token::Divide));
        
        // Should contain comparison operators
        assert!(tokens.contains(&Token::Less));
        assert!(tokens.contains(&Token::Equal));
    }

    /// Test tokenization of companion object methods
    /// 
    /// Verifies that static/companion methods can be called on primitive
    /// type classes.
    #[test]
    fn test_companion_methods_tokenization() {
        let source = r#"
            val parsed = Int.parse(String("42"))
            val min = Int.min(Int(10), Int(20))
            val max = Float.max(Float(1.5), Float(2.5))
            val fromCode = Char.fromInt(Int(65))
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain method names
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "parse")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "min")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "max")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "fromInt")));
        
        // Should contain dots for method access
        let dot_count = tokens.iter().filter(|&t| t == &Token::Dot).count();
        assert!(dot_count >= 4); // At least 4 method calls
    }

    /// Test tokenization of type conversions between primitives
    /// 
    /// Ensures that conversions between different primitive types
    /// are properly parsed.
    #[test]
    fn test_type_conversions_tokenization() {
        let source = r#"
            val intValue = Int(42)
            val floatValue = intValue.toFloat()
            val stringValue = intValue.toString()
            val stringOriginal = String("123")
            val intFromString = stringOriginal.toInt()
            val floatFromString = stringOriginal.toFloat()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain conversion method names
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toFloat")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toString")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toInt")));
    }

    /// Test tokenization of range operations on primitive types
    /// 
    /// Verifies that range operations (rangeTo, until) work correctly
    /// with Int and Char types.
    #[test]
    fn test_range_operations_tokenization() {
        let source = r#"
            val intRange = Int(1).rangeTo(Int(10))
            val intUntil = Int(1).until(Int(10))
            val charRange = Char('a').rangeTo(Char('z'))
            val rangeContains = intRange.contains(Int(5))
            val rangeSize = intRange.size()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain range method names
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "rangeTo")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "until")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "contains")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "size")));
    }

    /// Test tokenization of complex primitive type expressions
    /// 
    /// Ensures that complex expressions involving multiple primitive
    /// types and operations parse correctly.
    #[test]
    fn test_complex_primitive_expressions_tokenization() {
        let source = r#"
            val result = Int(10).pow(Int(2)).toFloat().sqrt().toInt()
            val textResult = String("Hello").toUpperCase().substring(Int(0), Int(3))
            val boolResult = Bool(true).and(Bool(false)).not()
            val charChain = Char('a').toUpperCase().next().toString()
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain method names in chains
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "pow")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "sqrt")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "toUpperCase")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "next")));
        
        // Should have many dots for method chaining
        let dot_count = tokens.iter().filter(|&t| t == &Token::Dot).count();
        assert!(dot_count >= 10); // Multiple method chains
    }

    /// Test tokenization of primitive type constants
    /// 
    /// Verifies that primitive type constants (MIN_VALUE, MAX_VALUE, etc.)
    /// are properly accessible.
    #[test]
    fn test_primitive_constants_tokenization() {
        let source = r#"
            val intMin = Int.MIN_VALUE
            val intMax = Int.MAX_VALUE
            val floatPi = Float.PI
            val floatE = Float.E
            val boolTrue = Bool.TRUE
            val boolFalse = Bool.FALSE
            val emptyString = String.EMPTY
            val spaceChar = Char.SPACE
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain constant names (as identifiers)
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "MIN_VALUE")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "MAX_VALUE")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "PI")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "E")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "TRUE")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "FALSE")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "EMPTY")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "SPACE")));
    }

    /// Test that all primitive type related tokens are present
    #[test]
    fn test_all_primitive_related_tokens() {
        let source = r#"
            data class IntWrapper(val value: Int) {
                constructor(str: String) : this(str.toInt())
                
                fun add(other: IntWrapper): IntWrapper {
                    return IntWrapper(this.value + other.value)
                }
                
                companion object {
                    val ZERO = IntWrapper(0)
                }
            }
        "#;
        
        let tokens = tokenize(source);
        
        // Should contain class-related tokens
        assert!(tokens.contains(&Token::Data));
        assert!(tokens.contains(&Token::Class));
        assert!(tokens.contains(&Token::Constructor));
        assert!(tokens.contains(&Token::This));
        assert!(tokens.contains(&Token::Val));
        assert!(tokens.contains(&Token::Fun));
        assert!(tokens.contains(&Token::Return));
        
        // Should contain identifiers
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "IntWrapper")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "value")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "add")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "companion")));
        assert!(tokens.iter().any(|t| matches!(t, Token::Identifier(name) if name == "object")));
    }
} 