use luna::runtime::LuaJitRuntime;
use luna::value::Value;

#[test]
fn test_basic_arithmetic() {
    let mut runtime = LuaJitRuntime::new();

    let result = runtime.execute("return 2 + 3").unwrap();
    assert_eq!(result, Value::Number(5.0));

    let result = runtime.execute("return 10 - 4").unwrap();
    assert_eq!(result, Value::Number(6.0));

    let result = runtime.execute("return 3 * 4").unwrap();
    assert_eq!(result, Value::Number(12.0));

    let result = runtime.execute("return 8 / 2").unwrap();
    assert_eq!(result, Value::Number(4.0));
}

#[test]
fn test_variable_assignment() {
    let mut runtime = LuaJitRuntime::new();

    runtime.execute("x = 42").unwrap();
    let result = runtime.execute("return x").unwrap();
    assert_eq!(result, Value::Number(42.0));

    let result = runtime.execute("return x + 8").unwrap();
    assert_eq!(result, Value::Number(50.0));
}

#[test]
fn test_local_variables() {
    let mut runtime = LuaJitRuntime::new();

    let source = r#"
        local x = 10
        local y = 20
        return x + y
    "#;

    let result = runtime.execute(source).unwrap();
    assert_eq!(result, Value::Number(30.0));
}

#[test]
fn test_if_statement() {
    let mut runtime = LuaJitRuntime::new();

    let source = r#"
        local x = 10
        if x > 5 then
            return "greater"
        else
            return "lesser"
        end
    "#;

    let result = runtime.execute(source).unwrap();
    assert_eq!(result, Value::String("greater".to_string()));
}

#[test]
fn test_while_loop() {
    let mut runtime = LuaJitRuntime::new();

    let source = r#"
        local sum = 0
        local i = 1
        while i <= 5 do
            sum = sum + i
            i = i + 1
        end
        return sum
    "#;

    let result = runtime.execute(source).unwrap();
    assert_eq!(result, Value::Number(15.0)); // 1+2+3+4+5 = 15
}

#[test]
fn test_for_loop() {
    let mut runtime = LuaJitRuntime::new();

    let source = r#"
        local sum = 0
        for i = 1, 5 do
            sum = sum + i
        end
        return sum
    "#;

    let result = runtime.execute(source).unwrap();
    assert_eq!(result, Value::Number(15.0));
}

#[test]
fn test_complex_expression() {
    let mut runtime = LuaJitRuntime::new();

    let result = runtime.execute("return 2 + 3 * 4 - 1").unwrap();
    assert_eq!(result, Value::Number(13.0)); // 2 + 12 - 1 = 13

    let result = runtime.execute("return (2 + 3) * 4").unwrap();
    assert_eq!(result, Value::Number(20.0)); // 5 * 4 = 20
}

#[test]
fn test_boolean_operations() {
    let mut runtime = LuaJitRuntime::new();

    let result = runtime.execute("return true and false").unwrap();
    assert_eq!(result, Value::Boolean(false));

    let result = runtime.execute("return true or false").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = runtime.execute("return not true").unwrap();
    assert_eq!(result, Value::Boolean(false));
}

#[test]
fn test_comparison_operations() {
    let mut runtime = LuaJitRuntime::new();

    let result = runtime.execute("return 5 > 3").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = runtime.execute("return 5 < 3").unwrap();
    assert_eq!(result, Value::Boolean(false));

    let result = runtime.execute("return 5 == 5").unwrap();
    assert_eq!(result, Value::Boolean(true));

    let result = runtime.execute("return 5 ~= 3").unwrap();
    assert_eq!(result, Value::Boolean(true));
}

#[test]
fn test_jit_hot_path() {
    let mut runtime = LuaJitRuntime::new();

    // Execute the same code multiple times to trigger JIT compilation
    let source = "return 2 * 3 + 1";

    for i in 0..150 {  // Exceed JIT threshold
        let result = runtime.execute(source).unwrap();
        assert_eq!(result, Value::Number(7.0));

        if i == 149 {
            // Check that stats show compilation happened
            runtime.print_stats();
        }
    }
}

#[test]
fn test_error_handling() {
    let mut runtime = LuaJitRuntime::new();

    // Division by zero
    let result = runtime.execute("return 5 / 0");
    assert!(result.is_err());

    // Invalid syntax
    let result = runtime.execute("return 5 +");
    assert!(result.is_err());
}
