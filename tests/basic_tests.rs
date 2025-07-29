use luna::{execute, Value};

#[test]
fn test_print_hello() {
    let result = execute("print('Hello, World!')");
    assert!(result.is_ok());
}

#[test]
fn test_basic_math() {
    let result = execute("return 2 + 3").unwrap();
    assert_eq!(result, Value::Number(5.0));
}

#[test]
fn test_math_functions() {
    let result = execute("return math.abs(-42)").unwrap();
    assert_eq!(result, Value::Number(42.0));
}
