use crate::value::Value;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Scope>,
    global_scope: HashMap<String, Value>,
}

#[derive(Debug, Clone)]
pub struct Scope {
    variables: HashMap<String, Value>,
    #[allow(dead_code)]
    parent: Option<usize>, // Index into scopes vector
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: Vec::new(),
            global_scope: HashMap::new(),
        }
    }

    pub fn with_builtins() -> Self {
        let mut env = Self::new();
        env.add_builtins();
        env
    }

    fn add_builtins(&mut self) {
        // Add built-in functions
        self.global_scope.insert("print".to_string(), Value::Function(0));
        self.global_scope.insert("type".to_string(), Value::Function(1));
        self.global_scope.insert("tostring".to_string(), Value::Function(2));
        self.global_scope.insert("tonumber".to_string(), Value::Function(3));

        // Add constants
        self.global_scope.insert("_VERSION".to_string(), Value::String("Luna 1.0".to_string()));
    }

    pub fn push_scope(&mut self) -> usize {
        let parent = if self.scopes.is_empty() {
            None
        } else {
            Some(self.scopes.len() - 1)
        };

        let scope = Scope {
            variables: HashMap::new(),
            parent,
        };

        self.scopes.push(scope);
        self.scopes.len() - 1
    }

    pub fn pop_scope(&mut self) -> Option<Scope> {
        self.scopes.pop()
    }

    pub fn define_local(&mut self, name: String, value: Value) -> Result<(), String> {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.variables.insert(name, value);
            Ok(())
        } else {
            // No local scope, define in global
            self.global_scope.insert(name, value);
            Ok(())
        }
    }

    pub fn define_global(&mut self, name: String, value: Value) {
        self.global_scope.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        // Search from innermost scope outward
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.variables.get(name) {
                return Some(value.clone());
            }
        }

        // Check global scope
        self.global_scope.get(name).cloned()
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        // Search from innermost scope outward
        for scope in self.scopes.iter_mut().rev() {
            if scope.variables.contains_key(name) {
                scope.variables.insert(name.to_string(), value);
                return Ok(());
            }
        }

        // Check if it exists in global scope
        if self.global_scope.contains_key(name) {
            self.global_scope.insert(name.to_string(), value);
            Ok(())
        } else {
            // Create new global variable
            self.global_scope.insert(name.to_string(), value);
            Ok(())
        }
    }

    pub fn get_local(&self, name: &str) -> Option<Value> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.variables.get(name).cloned()
        } else {
            None
        }
    }

    pub fn set_local(&mut self, name: &str, value: Value) -> Result<(), String> {
        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.variables.insert(name.to_string(), value);
            Ok(())
        } else {
            Err("No local scope available".to_string())
        }
    }

    pub fn scope_depth(&self) -> usize {
        self.scopes.len()
    }

    pub fn is_global_scope(&self) -> bool {
        self.scopes.is_empty()
    }

    pub fn get_all_locals(&self) -> HashMap<String, Value> {
        if let Some(current_scope) = self.scopes.last() {
            current_scope.variables.clone()
        } else {
            HashMap::new()
        }
    }

    pub fn get_globals(&self) -> &HashMap<String, Value> {
        &self.global_scope
    }

    pub fn get_globals_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.global_scope
    }

    // For debugging
    pub fn print_environment(&self) {
        println!("=== Environment Debug ===");
        println!("Global scope ({} variables):", self.global_scope.len());
        for (name, value) in &self.global_scope {
            println!("  {}: {}", name, value);
        }

        for (i, scope) in self.scopes.iter().enumerate() {
            println!("Local scope {} ({} variables):", i, scope.variables.len());
            for (name, value) in &scope.variables {
                println!("  {}: {}", name, value);
            }
        }
        println!("========================");
    }
}

impl Scope {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            parent: None,
        }
    }
}

// Helper for managing function environments
#[derive(Debug, Clone)]
pub struct FunctionEnvironment {
    pub captures: HashMap<String, Value>, // Captured variables (closures)
    pub parameters: Vec<String>,
    pub local_count: usize,
}

impl FunctionEnvironment {
    pub fn new(parameters: Vec<String>) -> Self {
        Self {
            captures: HashMap::new(),
            parameters,
            local_count: 0,
        }
    }

    pub fn capture_variable(&mut self, name: String, value: Value) {
        self.captures.insert(name, value);
    }

    pub fn get_parameter_index(&self, name: &str) -> Option<usize> {
        self.parameters.iter().position(|p| p == name)
    }

    pub fn is_parameter(&self, name: &str) -> bool {
        self.parameters.contains(&name.to_string())
    }
}

// Environment manager for the VM
#[derive(Debug)]
pub struct EnvironmentManager {
    environments: Vec<Environment>,
    current: usize,
}

impl EnvironmentManager {
    pub fn new() -> Self {
        let manager = Self {
            environments: vec![Environment::new()],
            current: 0,
        };
        manager
    }

    pub fn current_env(&self) -> &Environment {
        &self.environments[self.current]
    }

    pub fn current_env_mut(&mut self) -> &mut Environment {
        &mut self.environments[self.current]
    }

    pub fn push_environment(&mut self) -> usize {
        let new_env = self.environments[self.current].clone();
        self.environments.push(new_env);
        self.current = self.environments.len() - 1;
        self.current
    }

    pub fn pop_environment(&mut self) -> Option<Environment> {
        if self.current > 0 {
            let env = self.environments.pop();
            self.current = self.environments.len() - 1;
            env
        } else {
            None
        }
    }

    pub fn with_new_scope<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Environment) -> R,
    {
        let scope_id = self.current_env_mut().push_scope();
        let result = f(self.current_env_mut());
        self.current_env_mut().pop_scope();
        result
    }

    pub fn push_scope(&mut self) {
        let _scope_id = self.current_env_mut().push_scope();
    }
}

#[derive(Clone, Debug)]
pub struct Scope {
  variables: HashMap<String, Value>,
  #[allow(dead_code)]
  parent: Option<usize>,
}
