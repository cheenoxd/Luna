pub mod ast;
pub mod bytecode;
pub mod environment;
pub mod error;
pub mod jit;
pub mod lexer;
pub mod parser;
pub mod runtime;
pub mod stdlib;
pub mod value;
pub mod vm;

use runtime::LuaJitRuntime;

pub use crate::value::Value;
pub use crate::error::{LuaError, LuaResult};

#[derive(Debug, Clone)]
pub struct LunaConfig {
    pub jit_enabled: bool,
    pub optimization_level: u8,
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self {
            jit_enabled: true,
            optimization_level: 2,
        }
    }
}

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn new_runtime() -> LuaJitRuntime {
    LuaJitRuntime::new()
}

pub fn new_runtime_with_config(config: LunaConfig) -> LuaJitRuntime {
    LuaJitRuntime::with_config(config)
}

pub fn execute(code: &str) -> LuaResult<Value> {
    let mut runtime = new_runtime();
    runtime.execute(code)
}

pub fn execute_with_config(code: &str, config: LunaConfig) -> LuaResult<Value> {
    let mut runtime = new_runtime_with_config(config);
    runtime.execute(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }

    #[test]
    fn test_default_config() {
        let config = LunaConfig::default();
        assert_eq!(config.jit_enabled, true);
        assert_eq!(config.optimization_level, 2);
    }

    #[test]
    fn test_execute_print() {
        let result = execute("print('Hello World')");
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_math_function() {
        let result = execute("math.abs(-42)");
        assert!(result.is_ok());
    }
}
