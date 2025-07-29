use crate::value::Value;

#[derive(Debug, Clone)]
pub enum Instruction {
    LoadConst(Value),
    LoadGlobal(String),
    StoreGlobal(String),
    LoadLocal(usize),
    StoreLocal(usize),

    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Neg,

    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,

    And,
    Or,
    Not,

    Jump(usize),
    JumpIfFalse(usize),
    JumpIfTrue(usize),
    Call(usize),
    Return,

    Pop,
    Dup,

    MakeFunction(usize),

    NewTable,
    GetIndex,
    SetIndex,

    Concat,

    ForPrep(usize),  // Prepare for loop
    ForLoop(usize),  // Loop back if condition is true
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Value>,
    pub line_numbers: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            line_numbers: Vec::new(),
        }
    }

    pub fn emit(&mut self, instruction: Instruction, line: usize) {
        self.instructions.push(instruction);
        self.line_numbers.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn patch_jump(&mut self, offset: usize) {
        let jump_target = self.instructions.len();

        match &mut self.instructions[offset] {
            Instruction::Jump(ref mut target) |
            Instruction::JumpIfFalse(ref mut target) |
            Instruction::JumpIfTrue(ref mut target) => {
                *target = jump_target;
            }
            _ => panic!("Not a jump instruction"),
        }
    }
}

pub struct Compiler {
    chunk: Chunk,
    #[allow(dead_code)]
    scope_depth: usize,
    locals: Vec<String>,
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            scope_depth: 0,
            locals: Vec::new(),
        }
    }

    pub fn compile(&mut self, program: &crate::ast::Program) -> Result<Chunk, String> {
        for stmt in &program.statements {
            self.compile_statement(stmt)?;
        }

        self.chunk.emit(Instruction::LoadConst(Value::Nil), 0);
        self.chunk.emit(Instruction::Return, 0);

        Ok(self.chunk.clone())
    }

    fn compile_statement(&mut self, stmt: &crate::ast::Stmt) -> Result<(), String> {
        match stmt {
            crate::ast::Stmt::Expression(expr) => {
                self.compile_expression(expr)?;
                self.chunk.emit(Instruction::Pop, 0);
            }

            crate::ast::Stmt::Assignment { target, value } => {
                self.compile_expression(value)?;

                if let Some(local_index) = self.resolve_local(target) {
                    self.chunk.emit(Instruction::StoreLocal(local_index), 0);
                } else {
                    self.chunk.emit(Instruction::StoreGlobal(target.clone()), 0);
                }
            }

            crate::ast::Stmt::LocalAssignment { names, values } => {
                // Compile values first
                for value in values {
                    self.compile_expression(value)?;
                }

                // Pad with nils if fewer values than names
                for _ in values.len()..names.len() {
                    self.chunk.emit(Instruction::LoadConst(Value::Nil), 0);
                }

                // Store in locals
                for name in names.iter().rev() {
                    self.add_local(name.clone());
                    let local_index = self.locals.len() - 1;
                    self.chunk.emit(Instruction::StoreLocal(local_index), 0);
                }
            }

            crate::ast::Stmt::If { condition, then_branch, else_branch } => {
                self.compile_expression(condition)?;
                let else_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                for stmt in then_branch {
                    self.compile_statement(stmt)?;
                }

                let end_jump = self.emit_jump(Instruction::Jump(0));
                self.patch_jump(else_jump);

                if let Some(else_stmts) = else_branch {
                    for stmt in else_stmts {
                        self.compile_statement(stmt)?;
                    }
                }

                self.patch_jump(end_jump);
            }

            crate::ast::Stmt::While { condition, body } => {
                let loop_start = self.chunk.instructions.len();

                self.compile_expression(condition)?;
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                for stmt in body {
                    self.compile_statement(stmt)?;
                }

                self.chunk.emit(Instruction::Jump(loop_start), 0);
                self.patch_jump(exit_jump);
            }

            crate::ast::Stmt::Return(expr) => {
                if let Some(expr) = expr {
                    self.compile_expression(expr)?;
                } else {
                    self.chunk.emit(Instruction::LoadConst(Value::Nil), 0);
                }
                self.chunk.emit(Instruction::Return, 0);
            }

            crate::ast::Stmt::For { var, start, end, step, body } => {
                // Compile start value and store in loop variable
                self.compile_expression(start)?;
                self.add_local(var.clone());
                let var_index = self.locals.len() - 1;
                self.chunk.emit(Instruction::StoreLocal(var_index), 0);

                // Compile and store end value
                self.compile_expression(end)?;
                self.add_local(format!("{}_end", var));
                let end_index = self.locals.len() - 1;
                self.chunk.emit(Instruction::StoreLocal(end_index), 0);

                // Compile and store step value
                if let Some(step_expr) = step {
                    self.compile_expression(step_expr)?;
                } else {
                    self.chunk.emit(Instruction::LoadConst(Value::Number(1.0)), 0);
                }
                self.add_local(format!("{}_step", var));
                let step_index = self.locals.len() - 1;
                self.chunk.emit(Instruction::StoreLocal(step_index), 0);

                // Mark the start of the loop
                let loop_start = self.chunk.instructions.len();

                // Load loop variable and end value for comparison
                self.chunk.emit(Instruction::LoadLocal(var_index), 0);
                self.chunk.emit(Instruction::LoadLocal(end_index), 0);
                self.chunk.emit(Instruction::LessEqual, 0);

                // Jump out of loop if condition is false
                let exit_jump = self.emit_jump(Instruction::JumpIfFalse(0));

                // Compile loop body
                for stmt in body {
                    self.compile_statement(stmt)?;
                }

                // Increment loop variable
                self.chunk.emit(Instruction::LoadLocal(var_index), 0);     // current value
                self.chunk.emit(Instruction::LoadLocal(step_index), 0);    // step
                self.chunk.emit(Instruction::Add, 0);
                self.chunk.emit(Instruction::StoreLocal(var_index), 0);

                // Jump back to loop start
                self.chunk.emit(Instruction::Jump(loop_start), 0);

                // Patch the exit jump
                self.patch_jump(exit_jump);
            }

            _ => return Err("Statement not implemented".to_string()),
        }

        Ok(())
    }

    fn compile_expression(&mut self, expr: &crate::ast::Expr) -> Result<(), String> {
        match expr {
            crate::ast::Expr::Literal(value) => {
                self.chunk.emit(Instruction::LoadConst(value.clone()), 0);
            }

            crate::ast::Expr::Identifier(name) => {
                if let Some(local_index) = self.resolve_local(name) {
                    self.chunk.emit(Instruction::LoadLocal(local_index), 0);
                } else {
                    self.chunk.emit(Instruction::LoadGlobal(name.clone()), 0);
                }
            }

            crate::ast::Expr::Binary { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;

                let instruction = match operator {
                    crate::ast::BinaryOp::Add => Instruction::Add,
                    crate::ast::BinaryOp::Sub => Instruction::Sub,
                    crate::ast::BinaryOp::Mul => Instruction::Mul,
                    crate::ast::BinaryOp::Div => Instruction::Div,
                    crate::ast::BinaryOp::Mod => Instruction::Mod,
                    crate::ast::BinaryOp::Pow => Instruction::Pow,
                    crate::ast::BinaryOp::Equal => Instruction::Equal,
                    crate::ast::BinaryOp::NotEqual => Instruction::NotEqual,
                    crate::ast::BinaryOp::Less => Instruction::Less,
                    crate::ast::BinaryOp::LessEqual => Instruction::LessEqual,
                    crate::ast::BinaryOp::Greater => Instruction::Greater,
                    crate::ast::BinaryOp::GreaterEqual => Instruction::GreaterEqual,
                    crate::ast::BinaryOp::And => Instruction::And,
                    crate::ast::BinaryOp::Or => Instruction::Or,
                    crate::ast::BinaryOp::Concat => Instruction::Concat,
                };

                self.chunk.emit(instruction, 0);
            }

            crate::ast::Expr::Unary { operator, operand } => {
                self.compile_expression(operand)?;

                let instruction = match operator {
                    crate::ast::UnaryOp::Minus => Instruction::Neg,
                    crate::ast::UnaryOp::Not => Instruction::Not,
                };

                self.chunk.emit(instruction, 0);
            }

            crate::ast::Expr::Call { callee, args } => {
                // First compile all arguments
                for arg in args {
                    self.compile_expression(arg)?;
                }

                // Then compile the function (so it's on top of stack)
                self.compile_expression(callee)?;

                self.chunk.emit(Instruction::Call(args.len()), 0);
            }

            crate::ast::Expr::FieldAccess { object, field } => {
                self.compile_expression(object)?;
                self.chunk.emit(Instruction::LoadConst(Value::String(field.clone())), 0);
                self.chunk.emit(Instruction::GetIndex, 0);
            }

            _ => return Err("Expression not implemented".to_string()),
        }

        Ok(())
    }

    fn emit_jump(&mut self, instruction: Instruction) -> usize {
        self.chunk.emit(instruction, 0);
        self.chunk.instructions.len() - 1
    }

    fn patch_jump(&mut self, offset: usize) {
        self.chunk.patch_jump(offset);
    }

    fn add_local(&mut self, name: String) {
        self.locals.push(name);
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local == name {
                return Some(i);
            }
        }
        None
    }
}
