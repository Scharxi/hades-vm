/// Arithmetic Logic Unit for the Hades VM.
///
/// The ALU module provides functionality for performing mathematical operations
/// on different data types supported by the VM.

/// Arithmetic Logic Unit for performing mathematical operations.
///
/// The ALU provides methods for basic arithmetic operations
/// on different data types (integers, floats).
pub struct ALU; 

impl ALU {
    /// Add two integers.
    pub fn add_int(&self, a: i32, b: i32) -> i32 {
        a + b
    }
    
    /// Subtract second integer from first.
    pub fn sub_int(&self, a: i32, b: i32) -> i32 {
        a - b
    }
    
    /// Multiply two integers.
    pub fn multiply_int(&self, a: i32, b: i32) -> i32 {
        a * b
    }

    /// Divide first integer by second.
    pub fn divide_int(&self, a: i32, b: i32) -> i32 {
        a / b
    }
    
    /// Add two floats.
    pub fn add_float(&self, a: f32, b: f32) -> f32 {
        a + b
    }
    
    /// Subtract second float from first.
    pub fn sub_float(&self, a: f32, b: f32) -> f32 {
        a - b
    }
    
    /// Multiply two floats.
    pub fn multiply_float(&self, a: f32, b: f32) -> f32 {
        a * b
    }
    
    /// Divide first float by second.
    pub fn divide_float(&self, a: f32, b: f32) -> f32 {
        a / b
    }

    /// Performs mathematical modulo operation.
    /// Unlike Rust's remainder operator (%), this implements
    /// true mathematical modulo where the result is always
    /// in the range [0, |b|) for positive b, or (b, 0] for negative b.
    pub fn modulo(&self, a: i32, b: i32) -> i32 {
        let remainder = a % b;
        
        // If remainder is 0 or has the same sign as b, return it
        if remainder == 0 || (remainder > 0 && b > 0) || (remainder < 0 && b < 0) {
            remainder
        } else {
            // Otherwise, add b to get the correct mathematical modulo
            remainder + b
        }
    }
    
    /// Performs mathematical power operation.
    /// Returns b raised to the power of a.
    pub fn power(&self, b_val: i32, a_val: i32) -> i32 {
        b_val.wrapping_pow(a_val as u32)
    }

    /// Performs bitwise AND operation.
    pub fn bit_and(&self, a: i32, b: i32) -> i32 {
        a & b
    }

    /// Performs bitwise OR operation.
    pub fn bit_or(&self, a: i32, b: i32) -> i32 {
        a | b
    }

    /// Performs bitwise XOR operation.
    pub fn bit_xor(&self, a: i32, b: i32) -> i32 {
        a ^ b
    }

    /// Performs bitwise NOT operation (one's complement).
    pub fn bit_not(&self, a: i32) -> i32 {
        !a
    }

    /// Performs left shift operation.
    pub fn shift_left(&self, value: i32, shift: i32) -> i32 {
        if shift < 0 || shift >= 32 {
            panic!("Invalid shift amount");
        }
        value << shift
    }

    /// Performs right shift operation.
    pub fn shift_right(&self, value: i32, shift: i32) -> i32 {
        if shift < 0 || shift >= 32 {
            panic!("Invalid shift amount");
        }
        value >> shift
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_operations() {
        let alu = ALU;
        
        // Addition
        assert_eq!(alu.add_int(5, 7), 12);
        assert_eq!(alu.add_int(-5, 7), 2);
        
        // Subtraction
        assert_eq!(alu.sub_int(10, 5), 5);
        assert_eq!(alu.sub_int(5, 10), -5);
        
        // Multiplication
        assert_eq!(alu.multiply_int(5, 7), 35);
        assert_eq!(alu.multiply_int(-5, 7), -35);
        
        // Division
        assert_eq!(alu.divide_int(35, 5), 7);
        assert_eq!(alu.divide_int(10, 3), 3); // Integer division

        // Modulo
        assert_eq!(alu.modulo(10, 3), 1);
        assert_eq!(alu.modulo(-10, 3), 2);
        assert_eq!(alu.modulo(10, -3), -2);
        assert_eq!(alu.modulo(-10, -3), -1);

        // Power
        assert_eq!(alu.power(2, 3), 8);
        assert_eq!(alu.power(-2, 3), -8);
    }

    #[test]
    fn test_float_operations() {
        let alu = ALU;
        
        // Addition
        assert_eq!(alu.add_float(5.0, 7.0), 12.0);
        assert_eq!(alu.add_float(-5.0, 7.0), 2.0);
        
        // Subtraction
        assert_eq!(alu.sub_float(10.0, 5.0), 5.0);
        assert_eq!(alu.sub_float(5.0, 10.0), -5.0);
        
        // Multiplication
        assert_eq!(alu.multiply_float(5.0, 7.0), 35.0);
        assert_eq!(alu.multiply_float(-5.0, 7.0), -35.0);
        
        // Division
        assert_eq!(alu.divide_float(35.0, 5.0), 7.0);
        assert!((alu.divide_float(10.0, 3.0) - 3.3333333).abs() < 0.0001);
    }

    #[test]
    fn test_bitwise_operations() {
        let alu = ALU;
        
        // Test BitAnd
        assert_eq!(alu.bit_and(0b1100, 0b1010), 0b1000);
        assert_eq!(alu.bit_and(-1, 5), 5);
        
        // Test BitOr
        assert_eq!(alu.bit_or(0b1100, 0b1010), 0b1110);
        assert_eq!(alu.bit_or(0, 5), 5);
        
        // Test BitXor
        assert_eq!(alu.bit_xor(0b1100, 0b1010), 0b0110);
        assert_eq!(alu.bit_xor(5, 5), 0);
        
        // Test BitNot
        assert_eq!(alu.bit_not(0), -1);
        assert_eq!(alu.bit_not(-1), 0);
        
        // Test ShiftLeft
        assert_eq!(alu.shift_left(1, 2), 4);
        assert_eq!(alu.shift_left(0b1010, 1), 0b10100);
        
        // Test ShiftRight
        assert_eq!(alu.shift_right(4, 2), 1);
        assert_eq!(alu.shift_right(0b1010, 1), 0b0101);
    }

    #[test]
    #[should_panic(expected = "Invalid shift amount")]
    fn test_invalid_shift_left() {
        let alu = ALU;
        alu.shift_left(1, 32);
    }

    #[test]
    #[should_panic(expected = "Invalid shift amount")]
    fn test_invalid_shift_right() {
        let alu = ALU;
        alu.shift_right(1, -1);
    }
} 