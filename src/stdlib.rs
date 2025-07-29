use crate::error::{LuaError, LuaResult};
use crate::value::Value;
use std::collections::HashMap;

/// Built-in function signature
pub type BuiltinFunction = fn(&[Value]) -> LuaResult<Value>;

/// Registry of all built-in functions
pub struct StandardLibrary {
    functions: HashMap<String, (usize, BuiltinFunction)>, // (function_id, function)
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
        self.register_function("print", builtin_print);
        self.register_function("type", builtin_type);
        self.register_function("tostring", builtin_tostring);
        self.register_function("tonumber", builtin_tonumber);
        self.register_function("pairs", builtin_pairs);
        self.register_function("ipairs", builtin_ipairs);
        self.register_function("next", builtin_next);
        self.register_function("rawget", builtin_rawget);
        self.register_function("rawset", builtin_rawset);
        self.register_function("getmetatable", builtin_getmetatable);
        self.register_function("setmetatable", builtin_setmetatable);
        self.register_function("pcall", builtin_pcall);
        self.register_function("xpcall", builtin_xpcall);
        self.register_function("error", builtin_error);
        self.register_function("assert", builtin_assert);
    }

    // String functions
    fn register_string_functions(&mut self) {
        // String library would be a table in real Lua, but for simplicity
        // we'll register them as global functions with string. prefix
        self.register_function("string.len", string_len);
        self.register_function("string.sub", string_sub);
        self.register_function("string.upper", string_upper);
        self.register_function("string.lower", string_lower);
        self.register_function("string.char", string_char);
        self.register_function("string.byte", string_byte);
        self.register_function("string.find", string_find);
        self.register_function("string.gsub", string_gsub);
    }

    // Math functions
    fn register_math_functions(&mut self) {
        self.register_function("math.abs", math_abs);
        self.register_function("math.ceil", math_ceil);
        self.register_function("math.floor", math_floor);
        self.register_function("math.max", math_max);
        self.register_function("math.min", math_min);
        self.register_function("math.sqrt", math_sqrt);
        self.register_function("math.sin", math_sin);
        self.register_function("math.cos", math_cos);
        self.register_function("math.tan", math_tan);
        self.register_function("math.pi", math_pi);
        self.register_function("math.random", math_random);
    }

    // Table functions
    fn register_table_functions(&mut self) {
        self.register_function("table.insert", table_insert);
        self.register_function("table.remove", table_remove);
        self.register_function("table.concat", table_concat);
        self.register_function("table.sort", table_sort);
    }

    // IO functions (simplified)
    fn register_io_functions(&mut self) {
        self.register_function("io.write", io_write);
        self.register_function("io.read", io_read);
    }
}

// Core function implementations

pub fn builtin_print(args: &[Value]) -> LuaResult<Value> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!("\t");
        }
        print!("{}", arg);
    }
    println!();
    Ok(Value::Nil)
}

pub fn builtin_type(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "type"));
    }
    Ok(Value::String(args[0].type_name().to_string()))
}

pub fn builtin_tostring(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "tostring"));
    }
    Ok(Value::String(args[0].to_string()))
}

pub fn builtin_tonumber(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() {
        return Ok(Value::Nil);
    }

    match &args[0] {
        Value::Number(n) => Ok(Value::Number(*n)),
        Value::String(s) => {
            if let Ok(n) = s.parse::<f64>() {
                Ok(Value::Number(n))
            } else {
                Ok(Value::Nil)
            }
        }
        _ => Ok(Value::Nil),
    }
}

pub fn builtin_pairs(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - would return iterator in real implementation
    Err(LuaError::runtime_error("pairs not implemented"))
}

pub fn builtin_ipairs(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - would return iterator in real implementation
    Err(LuaError::runtime_error("ipairs not implemented"))
}

pub fn builtin_next(_args: &[Value]) -> LuaResult<Value> {
    Err(LuaError::runtime_error("next not implemented"))
}

pub fn builtin_rawget(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 2 {
        return Err(LuaError::argument_error(2, args.len(), "rawget"));
    }

    if let Value::Table(ref table) = args[0] {
        let key = args[1].to_string();
        Ok(table.get(&key).cloned().unwrap_or(Value::Nil))
    } else {
        Err(LuaError::type_error("table", args[0].type_name(), "rawget"))
    }
}

pub fn builtin_rawset(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 3 {
        return Err(LuaError::argument_error(3, args.len(), "rawset"));
    }

    if let Value::Table(mut table) = args[0].clone() {
        let key = args[1].to_string();
        table.insert(key, args[2].clone());
        Ok(Value::Table(table))
    } else {
        Err(LuaError::type_error("table", args[0].type_name(), "rawset"))
    }
}

pub fn builtin_getmetatable(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - metatables not implemented
    Ok(Value::Nil)
}

pub fn builtin_setmetatable(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - metatables not implemented
    Ok(Value::Nil)
}

pub fn builtin_pcall(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - would implement protected call
    Err(LuaError::runtime_error("pcall not implemented"))
}

pub fn builtin_xpcall(_args: &[Value]) -> LuaResult<Value> {
    // Simplified - would implement extended protected call
    Err(LuaError::runtime_error("xpcall not implemented"))
}

pub fn builtin_error(args: &[Value]) -> LuaResult<Value> {
    let message = if args.is_empty() {
        "error"
    } else {
        &args[0].to_string()
    };
    Err(LuaError::runtime_error(message))
}

pub fn builtin_assert(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() {
        return Err(LuaError::runtime_error("assertion failed!"));
    }

    if !args[0].is_truthy() {
        let message = if args.len() > 1 {
            args[1].to_string()
        } else {
            "assertion failed!".to_string()
        };
        return Err(LuaError::runtime_error(&message));
    }

    Ok(args[0].clone())
}

// String function implementations
pub fn string_len(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "string.len"));
    }

    if let Value::String(s) = &args[0] {
        Ok(Value::Number(s.len() as f64))
    } else {
        Err(LuaError::type_error("string", args[0].type_name(), "string.len"))
    }
}

pub fn string_sub(args: &[Value]) -> LuaResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LuaError::argument_error(2, args.len(), "string.sub"));
    }

    let s = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LuaError::type_error("string", args[0].type_name(), "string.sub")),
    };

    let start = match args[1].to_number() {
        Some(n) => n as i32,
        None => return Err(LuaError::type_error("number", args[1].type_name(), "string.sub")),
    };

    let end = if args.len() == 3 {
        match args[2].to_number() {
            Some(n) => n as i32,
            None => return Err(LuaError::type_error("number", args[2].type_name(), "string.sub")),
        }
    } else {
        s.len() as i32
    };

    // Lua uses 1-based indexing
    let start_idx = if start > 0 { start - 1 } else { 0 }.max(0) as usize;
    let end_idx = if end > 0 { end } else { s.len() as i32 }.min(s.len() as i32) as usize;

    if start_idx >= s.len() || start_idx >= end_idx {
        Ok(Value::String(String::new()))
    } else {
        Ok(Value::String(s[start_idx..end_idx].to_string()))
    }
}

pub fn string_upper(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "string.upper"));
    }

    if let Value::String(s) = &args[0] {
        Ok(Value::String(s.to_uppercase()))
    } else {
        Err(LuaError::type_error("string", args[0].type_name(), "string.upper"))
    }
}

pub fn string_lower(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "string.lower"));
    }

    if let Value::String(s) = &args[0] {
        Ok(Value::String(s.to_lowercase()))
    } else {
        Err(LuaError::type_error("string", args[0].type_name(), "string.lower"))
    }
}

pub fn string_char(args: &[Value]) -> LuaResult<Value> {
    let mut result = String::new();

    for arg in args {
        if let Some(n) = arg.to_number() {
            if n >= 0.0 && n <= 255.0 {
                result.push(n as u8 as char);
            } else {
                return Err(LuaError::runtime_error("character code out of range"));
            }
        } else {
            return Err(LuaError::type_error("number", arg.type_name(), "string.char"));
        }
    }

    Ok(Value::String(result))
}

pub fn string_byte(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() || args.len() > 3 {
        return Err(LuaError::argument_error(1, args.len(), "string.byte"));
    }

    let s = match &args[0] {
        Value::String(s) => s,
        _ => return Err(LuaError::type_error("string", args[0].type_name(), "string.byte")),
    };

    let index = if args.len() > 1 {
        match args[1].to_number() {
            Some(n) => (n as i32 - 1).max(0) as usize,
            None => 0,
        }
    } else {
        0
    };

    if index < s.len() {
        Ok(Value::Number(s.bytes().nth(index).unwrap_or(0) as f64))
    } else {
        Ok(Value::Nil)
    }
}

pub fn string_find(_args: &[Value]) -> LuaResult<Value> {
    // Simplified implementation
    Err(LuaError::runtime_error("string.find not implemented"))
}

pub fn string_gsub(_args: &[Value]) -> LuaResult<Value> {
    // Simplified implementation
    Err(LuaError::runtime_error("string.gsub not implemented"))
}

// Math function implementations
pub fn math_abs(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.abs"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.abs()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.abs"))
    }
}

pub fn math_ceil(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.ceil"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.ceil()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.ceil"))
    }
}

pub fn math_floor(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.floor"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.floor()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.floor"))
    }
}

pub fn math_max(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() {
        return Err(LuaError::argument_error(1, args.len(), "math.max"));
    }

    let mut max = match args[0].to_number() {
        Some(n) => n,
        None => return Err(LuaError::type_error("number", args[0].type_name(), "math.max")),
    };

    for arg in &args[1..] {
        if let Some(n) = arg.to_number() {
            if n > max {
                max = n;
            }
        } else {
            return Err(LuaError::type_error("number", arg.type_name(), "math.max"));
        }
    }

    Ok(Value::Number(max))
}

pub fn math_min(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() {
        return Err(LuaError::argument_error(1, args.len(), "math.min"));
    }

    let mut min = match args[0].to_number() {
        Some(n) => n,
        None => return Err(LuaError::type_error("number", args[0].type_name(), "math.min")),
    };

    for arg in &args[1..] {
        if let Some(n) = arg.to_number() {
            if n < min {
                min = n;
            }
        } else {
            return Err(LuaError::type_error("number", arg.type_name(), "math.min"));
        }
    }

    Ok(Value::Number(min))
}

pub fn math_sqrt(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.sqrt"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.sqrt()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.sqrt"))
    }
}

pub fn math_sin(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.sin"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.sin()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.sin"))
    }
}

pub fn math_cos(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.cos"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.cos()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.cos"))
    }
}

pub fn math_tan(args: &[Value]) -> LuaResult<Value> {
    if args.len() != 1 {
        return Err(LuaError::argument_error(1, args.len(), "math.tan"));
    }

    if let Some(n) = args[0].to_number() {
        Ok(Value::Number(n.tan()))
    } else {
        Err(LuaError::type_error("number", args[0].type_name(), "math.tan"))
    }
}

pub fn math_pi(_args: &[Value]) -> LuaResult<Value> {
    Ok(Value::Number(std::f64::consts::PI))
}

pub fn math_random(args: &[Value]) -> LuaResult<Value> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Simple deterministic "random" for now
    let mut hasher = DefaultHasher::new();
    std::ptr::addr_of!(args).hash(&mut hasher);
    let random_val = (hasher.finish() % 10000) as f64 / 10000.0;

    match args.len() {
        0 => Ok(Value::Number(random_val)),
        1 => {
            if let Some(max) = args[0].to_number() {
                Ok(Value::Number((random_val * max).floor() + 1.0))
            } else {
                Err(LuaError::type_error("number", args[0].type_name(), "math.random"))
            }
        }
        2 => {
            if let (Some(min), Some(max)) = (args[0].to_number(), args[1].to_number()) {
                let range = max - min + 1.0;
                Ok(Value::Number((random_val * range).floor() + min))
            } else {
                Err(LuaError::runtime_error("math.random expects numbers"))
            }
        }
        _ => Err(LuaError::argument_error(2, args.len(), "math.random")),
    }
}

// Table function implementations
pub fn table_insert(args: &[Value]) -> LuaResult<Value> {
    if args.len() < 2 || args.len() > 3 {
        return Err(LuaError::argument_error(2, args.len(), "table.insert"));
    }

    if let Value::Table(mut table) = args[0].clone() {
        if args.len() == 2 {
            // Insert at end
            let len = table.len();
            table.insert((len + 1).to_string(), args[1].clone());
        } else {
            // Insert at position
            if let Some(pos) = args[1].to_number() {
                table.insert(pos.to_string(), args[2].clone());
            } else {
                return Err(LuaError::type_error("number", args[1].type_name(), "table.insert"));
            }
        }
        Ok(Value::Table(table))
    } else {
        Err(LuaError::type_error("table", args[0].type_name(), "table.insert"))
    }
}

pub fn table_remove(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() || args.len() > 2 {
        return Err(LuaError::argument_error(1, args.len(), "table.remove"));
    }

    if let Value::Table(mut table) = args[0].clone() {
        let pos = if args.len() == 2 {
            match args[1].to_number() {
                Some(n) => n.to_string(),
                None => return Err(LuaError::type_error("number", args[1].type_name(), "table.remove")),
            }
        } else {
            table.len().to_string()
        };

        Ok(table.remove(&pos).unwrap_or(Value::Nil))
    } else {
        Err(LuaError::type_error("table", args[0].type_name(), "table.remove"))
    }
}

pub fn table_concat(args: &[Value]) -> LuaResult<Value> {
    if args.is_empty() || args.len() > 4 {
        return Err(LuaError::argument_error(1, args.len(), "table.concat"));
    }

    if let Value::Table(table) = &args[0] {
        let sep = if args.len() > 1 {
            match &args[1] {
                Value::String(s) => s.clone(),
                _ => return Err(LuaError::type_error("string", args[1].type_name(), "table.concat")),
            }
        } else {
            String::new()
        };

        let mut result = String::new();
        let values: Vec<_> = table.values().collect();
        for (i, value) in values.iter().enumerate() {
            if i > 0 {
                result.push_str(&sep);
            }
            result.push_str(&value.to_string());
        }

        Ok(Value::String(result))
    } else {
        Err(LuaError::type_error("table", args[0].type_name(), "table.concat"))
    }
}

pub fn table_sort(_args: &[Value]) -> LuaResult<Value> {
    // Simplified implementation
    Err(LuaError::runtime_error("table.sort not implemented"))
}

// IO function implementations
pub fn io_write(args: &[Value]) -> LuaResult<Value> {
    for arg in args {
        print!("{}", arg);
    }
    Ok(Value::Nil)
}

pub fn io_read(_args: &[Value]) -> LuaResult<Value> {
    // Simplified implementation
    Err(LuaError::runtime_error("io.read not implemented"))
}
