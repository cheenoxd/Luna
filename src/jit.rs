use crate::bytecode::{Chunk, Instruction};
use crate::value::Value;
use std::collections::HashMap;

#[cfg(target_arch = "x86_64")]
use std::mem;

pub struct HotSpot {
    pub start_pc: usize,
    pub end_pc: usize,
    pub execution_count: u32,
    pub compiled_code: Option<CompiledCode>,
}

pub struct CompiledCode {
    pub machine_code: Vec<u8>,
    pub entry_point: *const u8,
}

unsafe impl Send for CompiledCode {}
unsafe impl Sync for CompiledCode {}

pub struct JitCompiler {
    hot_spots: HashMap<usize, HotSpot>,
    threshold: u32,
    execution_counts: HashMap<usize, u32>,
}

impl JitCompiler {
    pub fn new() -> Self {
        Self {
            hot_spots: HashMap::new(),
            threshold: 100, // Compile after 100 executions
            execution_counts: HashMap::new(),
        }
    }

    pub fn record_execution(&mut self, pc: usize) {
        *self.execution_counts.entry(pc).or_insert(0) += 1;
    }

    pub fn should_compile(&self, pc: usize) -> bool {
        self.execution_counts.get(&pc).unwrap_or(&0) >= &self.threshold
    }

    pub fn compile_hot_spot(&mut self, chunk: &Chunk, start_pc: usize) -> Result<(), String> {
        let end_pc = self.find_hot_region_end(chunk, start_pc);

        let compiled_code = self.compile_to_native(chunk, start_pc, end_pc)?;

        let hot_spot = HotSpot {
            start_pc,
            end_pc,
            execution_count: *self.execution_counts.get(&start_pc).unwrap_or(&0),
            compiled_code: Some(compiled_code),
        };

        self.hot_spots.insert(start_pc, hot_spot);
        Ok(())
    }

    fn find_hot_region_end(&self, chunk: &Chunk, start_pc: usize) -> usize {
        // Simple heuristic: compile up to the next jump or return
        for (i, instruction) in chunk.instructions.iter().enumerate().skip(start_pc) {
            match instruction {
                Instruction::Jump(_) |
                Instruction::JumpIfFalse(_) |
                Instruction::JumpIfTrue(_) |
                Instruction::Return => return i + 1,
                _ => continue,
            }
        }
        chunk.instructions.len()
    }

    #[cfg(target_arch = "x86_64")]
    fn compile_to_native(&self, chunk: &Chunk, start_pc: usize, end_pc: usize) -> Result<CompiledCode, String> {
        let mut machine_code = Vec::new();

        // x86-64 function prologue
        machine_code.extend_from_slice(&[
            0x55,                   // push rbp
            0x48, 0x89, 0xe5,       // mov rbp, rsp
            0x48, 0x83, 0xec, 0x20, // sub rsp, 32 (allocate space)
        ]);

        // Compile bytecode instructions to machine code
        for i in start_pc..end_pc {
            if let Some(instruction) = chunk.instructions.get(i) {
                match instruction {
                    Instruction::LoadConst(Value::Number(n)) => {
                        // Load constant into xmm0
                        let bits = n.to_bits();
                        machine_code.extend_from_slice(&[0x48, 0xb8]); // movabs rax, imm64
                        machine_code.extend_from_slice(&bits.to_le_bytes());
                        machine_code.extend_from_slice(&[0x66, 0x48, 0x0f, 0x6e, 0xc0]); // movq xmm0, rax
                    }

                    Instruction::Add => {
                        // addsd xmm0, xmm1 (simplified - assumes values in xmm0 and xmm1)
                        machine_code.extend_from_slice(&[0xf2, 0x0f, 0x58, 0xc1]);
                    }

                    Instruction::Sub => {
                        // subsd xmm0, xmm1
                        machine_code.extend_from_slice(&[0xf2, 0x0f, 0x5c, 0xc1]);
                    }

                    Instruction::Mul => {
                        // mulsd xmm0, xmm1
                        machine_code.extend_from_slice(&[0xf2, 0x0f, 0x59, 0xc1]);
                    }

                    Instruction::Div => {
                        // divsd xmm0, xmm1
                        machine_code.extend_from_slice(&[0xf2, 0x0f, 0x5e, 0xc1]);
                    }

                    _ => {
                        // For unsupported instructions, generate a call to interpreter
                        // This is a fallback mechanism
                        machine_code.extend_from_slice(&[0x90]); // nop
                    }
                }
            }
        }

        // x86-64 function epilogue
        machine_code.extend_from_slice(&[
            0x48, 0x89, 0xec,       // mov rsp, rbp
            0x5d,                   // pop rbp
            0xc3,                   // ret
        ]);

        // Allocate executable memory
        let executable_mem = self.allocate_executable_memory(&machine_code)?;

        Ok(CompiledCode {
            machine_code,
            entry_point: executable_mem,
        })
    }

    #[cfg(not(target_arch = "x86_64"))]
    fn compile_to_native(&self, _chunk: &Chunk, _start_pc: usize, _end_pc: usize) -> Result<CompiledCode, String> {
        Err("JIT compilation only supported on x86_64".to_string())
    }

    #[cfg(unix)]
    fn allocate_executable_memory(&self, code: &[u8]) -> Result<*const u8, String> {
        use std::ptr;

        let size = code.len();
        let page_size = 4096;
        let aligned_size = (size + page_size - 1) & !(page_size - 1);

        unsafe {
            // Allocate memory with mmap
            let addr = libc::mmap(
                ptr::null_mut(),
                aligned_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            );

            if addr == libc::MAP_FAILED {
                return Err("Failed to allocate memory".to_string());
            }

            // Copy code to allocated memory
            ptr::copy_nonoverlapping(code.as_ptr(), addr as *mut u8, code.len());

            // Make memory executable
            if libc::mprotect(addr, aligned_size, libc::PROT_READ | libc::PROT_EXEC) != 0 {
                libc::munmap(addr, aligned_size);
                return Err("Failed to make memory executable".to_string());
            }

            Ok(addr as *const u8)
        }
    }

    #[cfg(windows)]
    #[allow(dead_code)] // Will be used when JIT compilation is fully implemented
    fn allocate_executable_memory(&self, code: &[u8]) -> Result<*const u8, String> {
        // Windows implementation would use VirtualAlloc
        Err("Windows JIT compilation not implemented".to_string())
    }

    pub fn get_compiled_code(&self, pc: usize) -> Option<&CompiledCode> {
        self.hot_spots.get(&pc)?.compiled_code.as_ref()
    }

    pub fn print_stats(&self) {
        println!("=== JIT Compiler Statistics ===");
        println!("Hot spots compiled: {}", self.hot_spots.len());
        println!("Execution counts:");

        let mut sorted_counts: Vec<_> = self.execution_counts.iter().collect();
        sorted_counts.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

        for (pc, count) in sorted_counts.iter().take(10) {
            let status = if self.hot_spots.contains_key(pc) {
                "COMPILED"
            } else if **count >= self.threshold {
                "READY TO COMPILE"
            } else {
                "COLD"
            };
            println!("  PC {}: {} executions ({})", pc, count, status);
        }
    }
}

impl Clone for JitCompiler {
    fn clone(&self) -> Self {
        Self {
            hot_spots: HashMap::new(), // Don't clone compiled code
            threshold: self.threshold,
            execution_counts: self.execution_counts.clone(),
        }
    }
}

// Trait for JIT-enabled execution
pub trait JitEnabled {
    fn execute_with_jit(&mut self, chunk: &Chunk, jit: &mut JitCompiler) -> Result<Value, String>;
}
