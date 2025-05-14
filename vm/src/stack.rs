/// Stack and Stack Value types for the Hades VM.
///
/// This module contains the implementation of the stack data structure and 
/// stack value types used by the VM, including stack frames for function calls.

/// Represents different types of values that can be stored on the stack.
///
/// This enum allows the VM to support multiple data types in the same stack,
/// enabling type-safe operations and rich data representation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackValue {
    /// 32-bit signed integer
    Integer(i32),
    /// 32-bit floating point number
    Float(f32),
    /// Boolean value (true/false)
    Boolean(bool),
    /// Memory reference/pointer (address)
    Reference(usize),
}

impl StackValue {
    /// Convert to i32, panicking if not an Integer
    pub fn as_int(&self) -> i32 {
        match *self {
            StackValue::Integer(i) => i,
            _ => panic!("Expected Integer, got {:?}", self),
        }
    }
    
    /// Try to get as i32
    pub fn try_as_int(&self) -> Option<i32> {
        match *self {
            StackValue::Integer(i) => Some(i),
            _ => None,
        }
    }
    
    /// Convert to f32, panicking if not a Float
    pub fn as_float(&self) -> f32 {
        match *self {
            StackValue::Float(f) => f,
            _ => panic!("Expected Float, got {:?}", self),
        }
    }
    
    /// Convert to bool, panicking if not a Boolean
    pub fn as_bool(&self) -> bool {
        match *self {
            StackValue::Boolean(b) => b,
            _ => panic!("Expected Boolean, got {:?}", self),
        }
    }
    
    /// Convert to reference address, panicking if not a Reference
    pub fn as_reference(&self) -> usize {
        match *self {
            StackValue::Reference(addr) => addr,
            _ => panic!("Expected Reference, got {:?}", self),
        }
    }
    
    /// Convenience method to check if value is zero (or equivalent)
    pub fn is_zero(&self) -> bool {
        match *self {
            StackValue::Integer(i) => i == 0,
            StackValue::Float(f) => f == 0.0,
            StackValue::Boolean(b) => !b,
            StackValue::Reference(addr) => addr == 0,
        }
    }
}

impl From<i32> for StackValue {
    fn from(value: i32) -> Self {
        StackValue::Integer(value)
    }
}

impl From<f32> for StackValue {
    fn from(value: f32) -> Self {
        StackValue::Float(value)
    }
}

impl From<bool> for StackValue {
    fn from(value: bool) -> Self {
        StackValue::Boolean(value)
    }
}

/// Represents a stack frame for function calls.
///
/// Stack frames maintain the execution context for function calls,
/// storing the return address, base pointer for accessing local variables,
/// and the number of local variables in the frame.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Return address in the code to jump back to after function completion
    pub return_address: usize,
    /// Base pointer for accessing local variables
    pub base_pointer: usize,
    /// Number of local variables in this frame
    pub local_count: usize,
}

/// Stack implementation with frames support for function calls.
///
/// The Stack manages both values and frames, providing an execution
/// context for the virtual machine and supporting function calls with
/// local variables.
#[derive(Clone)]
pub struct Stack {
    /// The actual stack values
    pub values: Vec<StackValue>,
    /// Stack frames for function calls
    pub frames: Vec<StackFrame>,
    /// Current frame pointer (index into frames)
    pub current_frame: Option<usize>,
}

impl Stack {
    /// Creates a new stack with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            frames: Vec::with_capacity(32), // Reasonable default for call stack depth
            current_frame: None,
        }
    }

    /// Moves a value to the top of the stack.
    /// 
    /// This method removes the value from its current position and pushes it to the top of the stack.
    pub fn move_to_top(&mut self, value: StackValue) {
        let index = self.values.iter().position(|v| v == &value).expect("Value not found in stack");
        self.values.remove(index);
        self.values.push(value);
    }

    /// Copies a value to the top of the stack.
    /// 
    /// This method inserts a copy of the value at the top of the stack.
    pub fn copy_to_top(&mut self, value: StackValue) {
        let value = self.values.iter().find(|v| **v == value).expect("Value not found in stack");
        self.values.push(value.clone());           
    }

    /// Copies the nth value from the top of the stack to the top of the stack.
    /// 
    /// This method inserts a copy of the nth value from the top of the stack to the top of the stack.
    /// The index is 0-based from the end of the stack, so n=0 refers to the top element,
    /// n=1 refers to the element below the top, and so on.
    /// 
    /// # Arguments
    /// 
    /// * `n` - The 0-based index from the top of the stack (n=0 is the top element).
    /// 
    /// # Panics
    /// 
    /// This method will panic if the index is out of bounds.
    /// 
    /// # Examples
    /// 
    /// ```
    /// # use vm::stack::{Stack, StackValue};
    /// # let mut stack = Stack::new(10);
    /// # stack.push(StackValue::Integer(1));
    /// # stack.push(StackValue::Integer(2));
    /// # stack.push(StackValue::Integer(3));
    /// // Stack is now [1, 2, 3] with 3 at the top
    /// stack.copy_nth_to_top(1); // Copies value at index 1 (which is 2) to the top
    /// // Stack is now [1, 2, 3, 2]
    /// ```
    pub fn copy_nth_to_top(&mut self, n: usize) {
        if n >= self.len() {
            panic!("Stack index out of bounds");
        }
        
        let index = self.len() - 1 - n;
        let value = self.values[index].clone();
        self.values.push(value);
    }
    
    /// Pushes a value onto the stack.
    pub fn push(&mut self, value: StackValue) {
        self.values.push(value);
    }
    
    /// Pops a value from the stack.
    pub fn pop(&mut self) -> Option<StackValue> {
        self.values.pop()
    }
    
    /// Gets the top value without removing it.
    pub fn peek(&self) -> Option<&StackValue> {
        self.values.last()
    }
    
    /// Creates a new stack frame.
    ///
    /// This method sets up a new execution context for function calls,
    /// with the specified return address and local variable count.
    pub fn push_frame(&mut self, return_address: usize, local_count: usize) {
        // Die Base-Pointer sollte auf die aktuelle Position im Stack gesetzt werden,
        // vor dem Reservieren von Plätze für lokale Variablen
        let base_pointer = self.values.len();
        
        // Wir müssen keine zusätzlichen Plätze für lokale Variablen reservieren,
        // da die Parameter bereits auf dem Stack sind
        // Die eigentlichen lokalen Variablen werden bei Bedarf mit StoreLocal erstellt
        
        let frame = StackFrame {
            return_address,
            base_pointer,
            local_count,
        };
        
        self.frames.push(frame);
        self.current_frame = Some(self.frames.len() - 1);
    }
    
    /// Pops the current stack frame and returns to the previous one.
    ///
    /// This method is used when returning from a function call. It will:
    /// 1. Save any return value that might be on top of the stack
    /// 2. Remove all local variables and parameters
    /// 3. Pop the frame
    /// 4. Push the return value back (if any)
    /// 5. Return the address to jump back to
    pub fn pop_frame(&mut self) -> Option<usize> {
        if let Some(frame_idx) = self.current_frame {
            let frame = &self.frames[frame_idx];
            let return_address = frame.return_address;
            
            // Get the return value if there is one on top of the stack
            let return_value = if self.values.len() > frame.base_pointer {
                // Get the top value as the return value (we don't pop it yet)
                Some(self.values[self.values.len() - 1].clone())
            } else {
                None
            };
            
            // Clear all values from the stack
            // This removes all local variables and parameters
            self.values.clear();
            
            // Push the return value back if there was one
            if let Some(value) = return_value {
                self.values.push(value);
            }
            
            // Remove the frame
            self.frames.pop();
            
            // Update the current frame pointer
            self.current_frame = if self.frames.is_empty() {
                None
            } else {
                Some(self.frames.len() - 1)
            };
            
            Some(return_address)
        } else {
            None
        }
    }
    
    /// Gets a local variable from the current frame.
    pub fn get_local(&self, index: usize) -> Option<&StackValue> {
        if let Some(frame_idx) = self.current_frame {
            let frame = &self.frames[frame_idx];
            
            // The base_pointer is set to the stack length when the frame is created,
            // so we need to access elements BEFORE the base_pointer
            if index < frame.local_count {
                // Access local variables BEFORE the base_pointer
                let local_idx = if frame.base_pointer >= index + 1 {
                    frame.base_pointer - index - 1
                } else {
                    return None; // Invalid index
                };
                
                if local_idx < self.values.len() {
                    return Some(&self.values[local_idx]);
                }
            }
            None
        } else {
            None
        }
    }
    
    /// Sets a local variable in the current frame.
    pub fn set_local(&mut self, index: usize, value: StackValue) -> Result<(), &'static str> {
        if let Some(frame_idx) = self.current_frame {
            let frame = &self.frames[frame_idx];
            
            // The base_pointer is set to the stack length when the frame is created,
            // so we need to access elements BEFORE the base_pointer
            if index < frame.local_count {
                // Access local variables BEFORE the base_pointer
                let local_idx = if frame.base_pointer >= index + 1 {
                    frame.base_pointer - index - 1
                } else {
                    return Err("Local variable index out of bounds"); // Invalid index
                };
                
                if local_idx < self.values.len() {
                    self.values[local_idx] = value;
                    return Ok(());
                }
            }
            Err("Local variable index out of bounds")
        } else {
            Err("No active stack frame")
        }
    }
    
    /// Gets the number of values on the stack.
    pub fn len(&self) -> usize {
        self.values.len()
    }
    
    /// Checks if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    
    /// Gets the current frame depth.
    pub fn frame_depth(&self) -> usize {
        self.frames.len()
    }
    
    /// Clears the stack.
    pub fn clear(&mut self) {
        self.values.clear();
        self.frames.clear();
        self.current_frame = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stack_basics() {
        let mut stack = Stack::new(10);
        assert!(stack.is_empty());
        
        stack.push(StackValue::Integer(42));
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.peek(), Some(&StackValue::Integer(42)));
        
        let value = stack.pop();
        assert_eq!(value, Some(StackValue::Integer(42)));
        assert!(stack.is_empty());
    }

    #[test]
    fn test_stack_frame_basics() {
        let mut stack = Stack::new(10);
        
        // Push parameters that will become local variables
        // Local 0 will be the value on top of the stack when frame is created
        // Local 1 will be the value below that
        stack.push(StackValue::Integer(1)); // This will be local 1
        stack.push(StackValue::Integer(2)); // This will be local 0
        
        // Stack has 2 values now
        assert_eq!(stack.len(), 2);
        
        // Create a frame with 2 local variables
        // The base_pointer will be set to 2 (current stack length)
        stack.push_frame(100, 2);
        assert_eq!(stack.frame_depth(), 1);
        
        // In the corrected implementation, local variables are accessed in reverse order
        // from the top of the stack at the time the frame was created
        assert_eq!(stack.get_local(0), Some(&StackValue::Integer(2))); // Top value
        assert_eq!(stack.get_local(1), Some(&StackValue::Integer(1))); // Value below top
        
        // Test setting a local variable
        stack.set_local(0, StackValue::Integer(42)).unwrap();
        assert_eq!(stack.get_local(0), Some(&StackValue::Integer(42)));
        
        // Push a return value on top of the stack
        stack.push(StackValue::Integer(100));
        assert_eq!(stack.len(), 3); // 2 locals + 1 return value
        
        // Pop the frame - this should leave only the return value
        let return_addr = stack.pop_frame();
        assert_eq!(return_addr, Some(100));
        assert_eq!(stack.frame_depth(), 0);
        
        // The stack should now only have our return value
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.peek(), Some(&StackValue::Integer(100)));
    }

    #[test]
    fn test_simple_stack_frame() {
        // Simple test for stack frames without complex operations
        let mut stack = Stack::new(10);
        
        // Stack starts with no frames
        assert_eq!(stack.frame_depth(), 0);
        
        // Create a frame with no local variables
        stack.push_frame(100, 0);
        assert_eq!(stack.frame_depth(), 1);
        
        // Pop the frame and check return address
        let return_addr = stack.pop_frame();
        assert_eq!(return_addr, Some(100));
        assert_eq!(stack.frame_depth(), 0);
    }

    #[test]
    fn test_move_to_top() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(1));
        stack.push(StackValue::Integer(2));
        
        stack.move_to_top(StackValue::Integer(2));
        assert_eq!(stack.peek(), Some(&StackValue::Integer(2)));

        stack.move_to_top(StackValue::Integer(1));
        assert_eq!(stack.peek(), Some(&StackValue::Integer(1)));
    }

    #[test]
    fn test_copy_to_top() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(1));
        stack.push(StackValue::Integer(2));
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.peek(), Some(&StackValue::Integer(2)));

        stack.copy_to_top(StackValue::Integer(1));
        assert_eq!(stack.peek(), Some(&StackValue::Integer(1)));
        assert_eq!(stack.len(), 3);

        stack.copy_to_top(StackValue::Integer(2));
        assert_eq!(stack.peek(), Some(&StackValue::Integer(2)));
        assert_eq!(stack.len(), 4);
    }

    #[test]
    fn test_copy_nth_to_top() {
        let mut stack = Stack::new(10);
        stack.push(StackValue::Integer(1));
        stack.push(StackValue::Integer(2));
        stack.push(StackValue::Integer(3));
        // Stack now has [1, 2, 3] with 3 at the top

        // Copy the value at position 1 (which is 2) to the top
        stack.copy_nth_to_top(1);
        assert_eq!(stack.peek(), Some(&StackValue::Integer(2)));
        assert_eq!(stack.len(), 4);
        // Stack now has [1, 2, 3, 2]

        // When the stack is [1, 2, 3, 2], position 2 is element 1
        stack.copy_nth_to_top(2);
        assert_eq!(stack.peek(), Some(&StackValue::Integer(2)));
        assert_eq!(stack.len(), 5);
        // Stack now has [1, 2, 3, 2, 2]
    }
} 