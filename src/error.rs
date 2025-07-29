use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum LuaError {
    // Lexical errors
    LexError {
        message: String,
        line: usize,
        column: usize,
    },

    // Parse errors
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    // Runtime errors
    RuntimeError {
        message: String,
        line: Option<usize>,
    },

    // Type errors
    TypeError {
        expected: String,
        found: String,
        operation: String,
    },

    // Variable errors
    UndefinedVariable {
        name: String,
    },

    // Function errors
    ArgumentError {
        expected: usize,
        found: usize,
        function: String,
    },

    CallError {
        message: String,
        function: String,
    },

    // Arithmetic errors
    ArithmeticError {
        message: String,
        operation: String,
    },

    // Stack errors
    StackOverflow,
    StackUnderflow,

    // JIT compilation errors
    JitError {
        message: String,
        pc: usize,
    },

    // IO errors
    IoError {
        message: String,
    },

    // Custom errors for user-defined error handling
    CustomError {
        message: String,
        error_type: String,
    },
}

impl LuaError {
    pub fn lex_error(message: &str, line: usize, column: usize) -> Self {
        Self::LexError {
            message: message.to_string(),
            line,
            column,
        }
    }

    pub fn parse_error(message: &str, line: usize, column: usize) -> Self {
        Self::ParseError {
            message: message.to_string(),
            line,
            column,
        }
    }

    pub fn runtime_error(message: &str) -> Self {
        Self::RuntimeError {
            message: message.to_string(),
            line: None,
        }
    }

    pub fn runtime_error_with_line(message: &str, line: usize) -> Self {
        Self::RuntimeError {
            message: message.to_string(),
            line: Some(line),
        }
    }

    pub fn type_error(expected: &str, found: &str, operation: &str) -> Self {
        Self::TypeError {
            expected: expected.to_string(),
            found: found.to_string(),
            operation: operation.to_string(),
        }
    }

    pub fn undefined_variable(name: &str) -> Self {
        Self::UndefinedVariable {
            name: name.to_string(),
        }
    }

    pub fn argument_error(expected: usize, found: usize, function: &str) -> Self {
        Self::ArgumentError {
            expected,
            found,
            function: function.to_string(),
        }
    }

    pub fn call_error(message: &str, function: &str) -> Self {
        Self::CallError {
            message: message.to_string(),
            function: function.to_string(),
        }
    }

    pub fn arithmetic_error(message: &str, operation: &str) -> Self {
        Self::ArithmeticError {
            message: message.to_string(),
            operation: operation.to_string(),
        }
    }

    pub fn jit_error(message: &str, pc: usize) -> Self {
        Self::JitError {
            message: message.to_string(),
            pc,
        }
    }

    pub fn io_error(message: &str) -> Self {
        Self::IoError {
            message: message.to_string(),
        }
    }

    pub fn custom_error(message: &str, error_type: &str) -> Self {
        Self::CustomError {
            message: message.to_string(),
            error_type: error_type.to_string(),
        }
    }

    pub fn error_type(&self) -> &'static str {
        match self {
            LuaError::LexError { .. } => "LexError",
            LuaError::ParseError { .. } => "ParseError",
            LuaError::RuntimeError { .. } => "RuntimeError",
            LuaError::TypeError { .. } => "TypeError",
            LuaError::UndefinedVariable { .. } => "UndefinedVariable",
            LuaError::ArgumentError { .. } => "ArgumentError",
            LuaError::CallError { .. } => "CallError",
            LuaError::ArithmeticError { .. } => "ArithmeticError",
            LuaError::StackOverflow => "StackOverflow",
            LuaError::StackUnderflow => "StackUnderflow",
            LuaError::JitError { .. } => "JitError",
            LuaError::IoError { .. } => "IoError",
            LuaError::CustomError { .. } => "CustomError",
        }
    }

    pub fn message(&self) -> &str {
        match self {
            LuaError::LexError { message, .. } |
            LuaError::ParseError { message, .. } |
            LuaError::RuntimeError { message, .. } |
            LuaError::CallError { message, .. } |
            LuaError::ArithmeticError { message, .. } |
            LuaError::JitError { message, .. } |
            LuaError::IoError { message, .. } |
            LuaError::CustomError { message, .. } => message,

            LuaError::TypeError { expected, found, operation } => {
                // This is a bit hacky, but we need to return a &str
                // In a real implementation, you might want to use Cow<str>
                "Type error (see Display implementation for details)"
            }

            LuaError::UndefinedVariable { name } => "Undefined variable",
            LuaError::ArgumentError { .. } => "Wrong number of arguments",
            LuaError::StackOverflow => "Stack overflow",
            LuaError::StackUnderflow => "Stack underflow",
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        match &mut self {
            LuaError::RuntimeError { line: line_ref, .. } => {
                *line_ref = Some(line);
            }
            _ => {}
        }
        self
    }
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaError::LexError { message, line, column } => {
                write!(f, "Lexical error at {}:{}: {}", line, column, message)
            }

            LuaError::ParseError { message, line, column } => {
                write!(f, "Parse error at {}:{}: {}", line, column, message)
            }

            LuaError::RuntimeError { message, line } => {
                match line {
                    Some(line) => write!(f, "Runtime error at line {}: {}", line, message),
                    None => write!(f, "Runtime error: {}", message),
                }
            }

            LuaError::TypeError { expected, found, operation } => {
                write!(f, "Type error in {}: expected {}, found {}", operation, expected, found)
            }

            LuaError::UndefinedVariable { name } => {
                write!(f, "Undefined variable: {}", name)
            }

            LuaError::ArgumentError { expected, found, function } => {
                write!(f, "Wrong number of arguments to {}: expected {}, got {}", function, expected, found)
            }

            LuaError::CallError { message, function } => {
                write!(f, "Error calling {}: {}", function, message)
            }

            LuaError::ArithmeticError { message, operation } => {
                write!(f, "Arithmetic error in {}: {}", operation, message)
            }

            LuaError::StackOverflow => {
                write!(f, "Stack overflow")
            }

            LuaError::StackUnderflow => {
                write!(f, "Stack underflow")
            }

            LuaError::JitError { message, pc } => {
                write!(f, "JIT compilation error at PC {}: {}", pc, message)
            }

            LuaError::IoError { message } => {
                write!(f, "IO error: {}", message)
            }

            LuaError::CustomError { message, error_type } => {
                write!(f, "{}: {}", error_type, message)
            }
        }
    }
}

impl std::error::Error for LuaError {}

// Conversion from std::io::Error
impl From<std::io::Error> for LuaError {
    fn from(error: std::io::Error) -> Self {
        LuaError::io_error(&error.to_string())
    }
}

// Result type alias for convenience
pub type LuaResult<T> = Result<T, LuaError>;

// Helper functions for creating common errors
pub fn division_by_zero() -> LuaError {
    LuaError::arithmetic_error("Division by zero", "division")
}

pub fn invalid_operation(op: &str, left_type: &str, right_type: &str) -> LuaError {
    LuaError::type_error(
        &format!("number or string"),
        &format!("{} and {}", left_type, right_type),
        op
    )
}

pub fn not_callable(value_type: &str) -> LuaError {
    LuaError::type_error("function", value_type, "call")
}

pub fn not_indexable(value_type: &str) -> LuaError {
    LuaError::type_error("table", value_type, "index")
}

// Error context for better error reporting
#[derive(Debug, Clone)]
pub struct ErrorContext {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub function_name: Option<String>,
    pub call_stack: Vec<String>,
}

impl ErrorContext {
    pub fn new() -> Self {
        Self {
            file: None,
            line: None,
            column: None,
            function_name: None,
            call_stack: Vec::new(),
        }
    }

    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    pub fn with_function(mut self, function_name: String) -> Self {
        self.function_name = Some(function_name);
        self
    }

    pub fn push_call(mut self, function_name: String) -> Self {
        self.call_stack.push(function_name);
        self
    }

    pub fn format_error(&self, error: &LuaError) -> String {
        let mut result = error.to_string();

        if let Some(function) = &self.function_name {
            result = format!("{} in function '{}'", result, function);
        }

        if let Some(file) = &self.file {
            result = format!("{} in file '{}'", result, file);
        }

        if !self.call_stack.is_empty() {
            result.push_str("\nCall stack:");
            for (i, call) in self.call_stack.iter().rev().enumerate() {
                result.push_str(&format!("\n  {}: {}", i + 1, call));
            }
        }

        result
    }
}
