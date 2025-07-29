use crate::bytecode::{Chunk, Compiler};
use crate::jit::{JitCompiler, JitEnabled};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::value::Value;
use std::collections::HashMap;

pub struct LuaJitRuntime {
    globals: HashMap<String, Value>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    jit_compiler: JitCompiler,
    stdlib: crate::vm::StandardLibrary,
}

#[derive(Debug)]
struct CallFrame {
    chunk: Chunk,
    pc: usize,
    locals: Vec<Value>,
}

impl LuaJitRuntime {
    pub fn new() -> Self {
        let stdlib = crate::vm::StandardLibrary::new();
        let mut runtime = Self {
            globals: HashMap::new(),
            stack: Vec::new(),
            call_stack: Vec::new(),
            jit_compiler: JitCompiler::new(),
            stdlib,
        };

        runtime.add_builtins();
        runtime
    }

    pub fn with_config(_config: crate::LunaConfig) -> Self {
        let runtime = Self::new();
        runtime
    }

    fn add_builtins(&mut self) {
        self.globals.insert("print".to_string(), Value::Function(0));

        let mut math_table = std::collections::HashMap::new();
        if let Some((id, _)) = self.stdlib.get_function("math.abs") {
            math_table.insert("abs".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("math.sqrt") {
            math_table.insert("sqrt".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("math.max") {
            math_table.insert("max".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("math.min") {
            math_table.insert("min".to_string(), Value::Function(id));
        }
        self.globals.insert("math".to_string(), Value::Table(math_table));

        let mut string_table = std::collections::HashMap::new();
        if let Some((id, _)) = self.stdlib.get_function("string.len") {
            string_table.insert("len".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("string.sub") {
            string_table.insert("sub".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("string.upper") {
            string_table.insert("upper".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("string.lower") {
            string_table.insert("lower".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("string.char") {
            string_table.insert("char".to_string(), Value::Function(id));
        }
        if let Some((id, _)) = self.stdlib.get_function("string.byte") {
            string_table.insert("byte".to_string(), Value::Function(id));
        }
        self.globals.insert("string".to_string(), Value::Table(string_table));
    }

    pub fn execute(&mut self, code: &str) -> Result<Value, crate::error::LuaError> {
        match self.execute_internal(code) {
            Ok(value) => Ok(value),
            Err(msg) => Err(crate::error::LuaError::runtime_error(&msg)),
        }
    }

    fn execute_internal(&mut self, source: &str) -> Result<Value, String> {
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize()?;

        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;

        let mut compiler = Compiler::new();
        let chunk = compiler.compile(&program)?;

        self.execute_with_jit(&chunk, &mut self.jit_compiler.clone())
    }

    fn execute_instruction(&mut self, instruction: &crate::bytecode::Instruction) -> Result<(), String> {
        use crate::bytecode::Instruction;

        match instruction {
            Instruction::LoadConst(value) => {
                self.stack.push(value.clone());
            }
            Instruction::LoadGlobal(name) => {
                let value = self.globals.get(name).cloned().unwrap_or(Value::Nil);
                self.stack.push(value);
            }
            Instruction::StoreGlobal(name) => {
                if let Some(value) = self.stack.pop() {
                    self.globals.insert(name.clone(), value);
                } else {
                    return Err("Stack underflow".to_string());
                }
            }
            Instruction::Call(arg_count) => {
                if self.stack.len() < arg_count + 1 {
                    return Err("Not enough arguments for call".to_string());
                }

                let func = self.stack.pop().unwrap();
                let mut args = Vec::new();
                for _ in 0..*arg_count {
                    if let Some(arg) = self.stack.pop() {
                        args.push(arg);
                    }
                }
                args.reverse();

                match func {
                    Value::Function(id) => {
                        if id == 0 {
                            for (i, arg) in args.iter().enumerate() {
                                if i > 0 {
                                    print!("\t");
                                }
                                print!("{}", arg);
                            }
                            println!();
                            self.stack.push(Value::Nil);
                        } else if let Some(builtin_func) = self.stdlib.get_function_by_id(id) {
                            match builtin_func(&args) {
                                Ok(result) => self.stack.push(result),
                                Err(e) => return Err(format!("Function error: {}", e)),
                            }
                        } else {
                            return Err(format!("Unknown function ID: {}", id));
                        }
                    }
                    _ => {
                        return Err(format!("Cannot call non-function value: {:?}", func));
                    }
                }
            }
            Instruction::GetIndex => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for index access".to_string());
                }
                let key = self.stack.pop().unwrap();
                let table = self.stack.pop().unwrap();

                match table {
                    Value::Table(ref t) => {
                        let key_str = key.to_string();
                        let value = t.get(&key_str).cloned().unwrap_or(Value::Nil);
                        self.stack.push(value);
                    }
                    _ => {
                        let table_name = table.to_string();
                        let field_name = key.to_string();
                        let full_name = format!("{}.{}", table_name, field_name);

                        if let Some((id, _)) = self.stdlib.get_function(&full_name) {
                            self.stack.push(Value::Function(id));
                        } else {
                            self.stack.push(Value::Nil);
                        }
                    }
                }
            }
            Instruction::Pop => {
                self.stack.pop();
            }
            Instruction::Return => {
            }
            Instruction::Add => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for addition".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Number(a_num + b_num));
                    }
                    _ => return Err("Cannot add non-numeric values".to_string()),
                }
            }
            Instruction::Sub => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for subtraction".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Number(a_num - b_num));
                    }
                    _ => return Err("Cannot subtract non-numeric values".to_string()),
                }
            }
            Instruction::Mul => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for multiplication".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Number(a_num * b_num));
                    }
                    _ => return Err("Cannot multiply non-numeric values".to_string()),
                }
            }
            Instruction::Div => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for division".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        if b_num == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        self.stack.push(Value::Number(a_num / b_num));
                    }
                    _ => return Err("Cannot divide non-numeric values".to_string()),
                }
            }
            Instruction::LoadLocal(index) => {
                if let Some(frame) = self.call_stack.last() {
                    if *index < frame.locals.len() {
                        self.stack.push(frame.locals[*index].clone());
                    } else {
                        self.stack.push(Value::Nil);
                    }
                } else {
                    return Err("No call frame for local variable".to_string());
                }
            }
            Instruction::StoreLocal(index) => {
                if let Some(value) = self.stack.pop() {
                    if let Some(frame) = self.call_stack.last_mut() {
                        while frame.locals.len() <= *index {
                            frame.locals.push(Value::Nil);
                        }
                        frame.locals[*index] = value;
                    } else {
                        return Err("No call frame for local variable".to_string());
                    }
                } else {
                    return Err("Stack underflow".to_string());
                }
            }
            Instruction::Neg => {
                if self.stack.is_empty() {
                    return Err("Not enough operands for negation".to_string());
                }
                let operand = self.stack.pop().unwrap();

                match operand.to_number() {
                    Some(n) => {
                        self.stack.push(Value::Number(-n));
                    }
                    None => return Err("Cannot negate non-numeric value".to_string()),
                }
            }
            Instruction::Mod => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for modulo".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        if b_num == 0.0 {
                            return Err("Modulo by zero".to_string());
                        }
                        self.stack.push(Value::Number(a_num % b_num));
                    }
                    _ => return Err("Cannot take modulo of non-numeric values".to_string()),
                }
            }
            Instruction::Pow => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for power".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Number(a_num.powf(b_num)));
                    }
                    _ => return Err("Cannot raise non-numeric values to power".to_string()),
                }
            }
            Instruction::Equal => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for equality".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(Value::Boolean(a == b));
            }
            Instruction::NotEqual => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for inequality".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(Value::Boolean(a != b));
            }
            Instruction::Less => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for less than".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Boolean(a_num < b_num));
                    }
                    _ => return Err("Cannot compare non-numeric values".to_string()),
                }
            }
            Instruction::LessEqual => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for less than or equal".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Boolean(a_num <= b_num));
                    }
                    _ => return Err("Cannot compare non-numeric values".to_string()),
                }
            }
            Instruction::Greater => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for greater than".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Boolean(a_num > b_num));
                    }
                    _ => return Err("Cannot compare non-numeric values".to_string()),
                }
            }
            Instruction::GreaterEqual => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for greater than or equal".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                match (a.to_number(), b.to_number()) {
                    (Some(a_num), Some(b_num)) => {
                        self.stack.push(Value::Boolean(a_num >= b_num));
                    }
                    _ => return Err("Cannot compare non-numeric values".to_string()),
                }
            }
            Instruction::And => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for logical and".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(Value::Boolean(a.is_truthy() && b.is_truthy()));
            }
            Instruction::Or => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for logical or".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();
                self.stack.push(Value::Boolean(a.is_truthy() || b.is_truthy()));
            }
            Instruction::Not => {
                if self.stack.is_empty() {
                    return Err("Not enough operands for logical not".to_string());
                }
                let operand = self.stack.pop().unwrap();
                self.stack.push(Value::Boolean(!operand.is_truthy()));
            }
            Instruction::Concat => {
                if self.stack.len() < 2 {
                    return Err("Not enough operands for concatenation".to_string());
                }
                let b = self.stack.pop().unwrap();
                let a = self.stack.pop().unwrap();

                let result = format!("{}{}", a.to_string(), b.to_string());
                self.stack.push(Value::String(result));
            }
            Instruction::Jump(target) => {
                if let Some(frame) = self.call_stack.last_mut() {
                    frame.pc = *target;
                }
            }
            Instruction::JumpIfFalse(target) => {
                if let Some(condition) = self.stack.pop() {
                    if !condition.is_truthy() {
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.pc = *target;
                        }
                    }
                } else {
                    return Err("Stack underflow for jump condition".to_string());
                }
            }
            Instruction::JumpIfTrue(target) => {
                if let Some(condition) = self.stack.pop() {
                    if condition.is_truthy() {
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.pc = *target;
                        }
                    }
                } else {
                    return Err("Stack underflow for jump condition".to_string());
                }
            }
            _ => {
                return Err(format!("Unimplemented instruction: {:?}", instruction));
            }
        }

        Ok(())
    }

    pub fn print_stats(&self) {
        self.jit_compiler.print_stats();
    }
}

impl JitEnabled for LuaJitRuntime {
    fn execute_with_jit(&mut self, chunk: &Chunk, _jit: &mut JitCompiler) -> Result<Value, String> {
        let frame = CallFrame {
            chunk: chunk.clone(),
            pc: 0,
            locals: Vec::new(),
        };

        self.call_stack.push(frame);
        self.stack.clear();

        loop {
            let (instruction, should_increment_pc) = {
                let frame = match self.call_stack.last() {
                    Some(frame) => frame,
                    None => break,
                };

                if frame.pc >= frame.chunk.instructions.len() {
                    self.call_stack.pop();
                    if self.call_stack.is_empty() {
                        break;
                    }
                    continue;
                }

                let instruction = frame.chunk.instructions[frame.pc].clone();
                let should_increment = !matches!(instruction,
                    crate::bytecode::Instruction::Jump(_) |
                    crate::bytecode::Instruction::JumpIfFalse(_) |
                    crate::bytecode::Instruction::JumpIfTrue(_)
                );

                (instruction, should_increment)
            };

            match self.execute_instruction(&instruction) {
                Ok(_) => {
                    if should_increment_pc {
                        if let Some(frame) = self.call_stack.last_mut() {
                            frame.pc += 1;
                        }
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(self.stack.pop().unwrap_or(Value::Nil))
    }
}
