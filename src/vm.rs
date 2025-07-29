use crate::error::LuaResult;
use crate::value::Value;
use std::collections::HashMap;

pub type BuiltinFunction = fn(&[Value]) -> LuaResult<Value>;

pub struct StandardLibrary {
    functions: HashMap<String, (usize, BuiltinFunction)>,
    function_names: Vec<String>,
}

impl StandardLibrary {
    pub fn new() -> Self {
        let mut stdlib = Self {
            functions: HashMap::new(),
            function_names: Vec::new(),
        };

        stdlib.register_core_functions();
        stdlib.register_string_functions();
        stdlib.register_math_functions();
        stdlib.register_table_functions();
        stdlib.register_io_functions();

        stdlib
    }

    fn register_function(&mut self, name: &str, func: BuiltinFunction) -> usize {
        let id = self.function_names.len();
        self.function_names.push(name.to_string());
        self.functions.insert(name.to_string(), (id, func));
        id
    }

    pub fn get_function(&self, name: &str) -> Option<(usize, BuiltinFunction)> {
        self.functions.get(name).copied()
    }

    pub fn get_function_by_id(&self, id: usize) -> Option<BuiltinFunction> {
        if id < self.function_names.len() {
            let name = &self.function_names[id];
            self.functions.get(name).map(|(_, func)| *func)
        } else {
            None
        }
    }

    pub fn get_function_name(&self, id: usize) -> Option<&str> {
        self.function_names.get(id).map(|s| s.as_str())
    }

    pub fn get_all_functions(&self) -> HashMap<String, Value> {
        self.functions
            .iter()
            .map(|(name, (id, _))| (name.clone(), Value::Function(*id)))
            .collect()
    }

    // Core functions
    fn register_core_functions(&mut self) {
        self.register_function("print", crate::stdlib::builtin_print);
        self.register_function("type", crate::stdlib::builtin_type);
        self.register_function("tostring", crate::stdlib::builtin_tostring);
        self.register_function("tonumber", crate::stdlib::builtin_tonumber);
        self.register_function("pairs", crate::stdlib::builtin_pairs);
        self.register_function("ipairs", crate::stdlib::builtin_ipairs);
        self.register_function("next", crate::stdlib::builtin_next);
        self.register_function("rawget", crate::stdlib::builtin_rawget);
        self.register_function("rawset", crate::stdlib::builtin_rawset);
        self.register_function("getmetatable", crate::stdlib::builtin_getmetatable);
        self.register_function("setmetatable", crate::stdlib::builtin_setmetatable);
        self.register_function("pcall", crate::stdlib::builtin_pcall);
        self.register_function("xpcall", crate::stdlib::builtin_xpcall);
        self.register_function("error", crate::stdlib::builtin_error);
        self.register_function("assert", crate::stdlib::builtin_assert);
    }

    // String functions
    fn register_string_functions(&mut self) {
        // String library would be a table in real Lua, but for simplicity
        // we'll register them as global functions with string. prefix
        self.register_function("string.len", crate::stdlib::string_len);
        self.register_function("string.sub", crate::stdlib::string_sub);
        self.register_function("string.upper", crate::stdlib::string_upper);
        self.register_function("string.lower", crate::stdlib::string_lower);
        self.register_function("string.char", crate::stdlib::string_char);
        self.register_function("string.byte", crate::stdlib::string_byte);
        self.register_function("string.find", crate::stdlib::string_find);
        self.register_function("string.gsub", crate::stdlib::string_gsub);
    }

    // Math functions
    fn register_math_functions(&mut self) {
        self.register_function("math.abs", crate::stdlib::math_abs);
        self.register_function("math.ceil", crate::stdlib::math_ceil);
        self.register_function("math.floor", crate::stdlib::math_floor);
        self.register_function("math.max", crate::stdlib::math_max);
        self.register_function("math.min", crate::stdlib::math_min);
        self.register_function("math.sqrt", crate::stdlib::math_sqrt);
        self.register_function("math.sin", crate::stdlib::math_sin);
        self.register_function("math.cos", crate::stdlib::math_cos);
        self.register_function("math.tan", crate::stdlib::math_tan);
        self.register_function("math.pi", crate::stdlib::math_pi);
        self.register_function("math.random", crate::stdlib::math_random);
    }

    // Table functions
    fn register_table_functions(&mut self) {
        self.register_function("table.insert", crate::stdlib::table_insert);
        self.register_function("table.remove", crate::stdlib::table_remove);
        self.register_function("table.concat", crate::stdlib::table_concat);
        self.register_function("table.sort", crate::stdlib::table_sort);
    }

    // IO functions (simplified)
    fn register_io_functions(&mut self) {
        self.register_function("io.write", crate::stdlib::io_write);
        self.register_function("io.read", crate::stdlib::io_read);
    }
}
