# Luna - Lua JIT Interpreter

A Lua interpreter with Just-In-Time (JIT) compilation written in Rust. Learning, so AI was used.

## Current Features ✅

- **Lexical Analysis**: Full tokenization of Lua source code
- **Parsing**: AST generation for supported language constructs
- **Bytecode Compilation**: Converts AST to efficient bytecode instructions
- **JIT Compilation**: Framework for just-in-time optimization
- **Basic Arithmetic**: `+`, `-`, `*`, `/`, `%`, `^` (power)
- **Comparison Operations**: `<`, `<=`, `>`, `>=`, `==`, `~=`
- **Logical Operations**: `and`, `or`, `not`
- **Print Statements**: `print("Hello World")`git add src/main.rs examples/
git commit -m "feat: add CLI interface and example programs

- Interactive REPL mode
- File execution with timing
- JIT statistics and benchmark modes
- Example Lua programs for testing"
- **Math Library**: `math.abs()`, `math.sqrt()`, `math.max()`, `math.min()`
- **String Library**: `string.upper()`, `string.lower()`, `string.len()`
- **For Loops**: `for i = 1, 10 do ... end`
- **Global Variables**: Variable assignment and access
- **Function Calls**: Built-in function invocation
- **Field Access**: Table field access with dot notation (`math.abs`)

## Missing Features

### Control Flow
- [ ] **If-Then-Else Statements**: `if condition then ... else ... end`
- [ ] **While Loops**: `while condition do ... end`
- [ ] **Repeat-Until Loops**: `repeat ... until condition`
- [ ] **Break/Continue**: Loop control statements

### Functions
- [ ] **Function Definitions**: `function name(params) ... end`
- [ ] **Local Functions**: `local function name() ... end`
- [ ] **Anonymous Functions**: `function() ... end`
- [ ] **Function Parameters**: Parameter passing and return values
- [ ] **Closures**: Lexical scoping and upvalues
- [ ] **Variadic Functions**: `...` parameter handling

### Variables and Scope
- [ ] **Local Variables**: `local var = value` (partially implemented)
- [ ] **Multiple Assignment**: `a, b = 1, 2`
- [ ] **Upvalues**: Access to outer scope variables
- [ ] **Global Environment**: `_G` table

### Data Types and Structures
- [ ] **Tables**: `{}`, table construction and manipulation
- [ ] **Table Indexing**: `table[key]` syntax
- [ ] **Metatables**: `setmetatable()`, `getmetatable()`
- [ ] **Table Library**: `table.insert()`, `table.remove()`, etc.
- [ ] **Arrays**: Numeric indexing and `ipairs()`
- [ ] **Associative Arrays**: String keys and `pairs()`

### String Operations
- [ ] **String Concatenation**: `..` operator
- [ ] **String Literals**: Multi-line strings, escape sequences
- [ ] **String Interpolation**: Template strings
- [ ] **Pattern Matching**: `string.find()`, `string.match()`, etc.
- [ ] **String Substitution**: `string.gsub()`

### Advanced Features
- [ ] **Coroutines**: `coroutine.create()`, `coroutine.resume()`
- [ ] **Error Handling**: `pcall()`, `xpcall()`, `error()`
- [ ] **Modules**: `require()`, `module()` system
- [ ] **File I/O**: `io.open()`, `io.read()`, `io.write()`
- [ ] **OS Interface**: `os.time()`, `os.execute()`
- [ ] **Debug Library**: `debug.getinfo()`, stack traces

### Return Statements
- [ ] **Return Values**: `return value` (partially working)
- [ ] **Multiple Returns**: `return a, b, c`
- [ ] **Early Returns**: Return from middle of function

### Operators
- [ ] **Length Operator**: `#table`, `#string`
- [ ] **Bitwise Operations**: `&`, `|`, `~`, `<<`, `>>`
- [ ] **Ternary-like**: `condition and value1 or value2`
