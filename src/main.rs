use luna::{execute, new_runtime, Value};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // Check for special flags
        if args.contains(&"--jit-stats".to_string()) {
            run_with_jit_stats(&args[1]);
        } else if args.contains(&"--benchmark".to_string()) {
            run_benchmark(&args[1]);
        } else {
            // Regular file execution
            let filename = &args[1];
            match fs::read_to_string(filename) {
                Ok(code) => {
                    println!("Running file: {}", filename);
                    let start = Instant::now();
                    match execute(&code) {
                        Ok(result) => {
                            let duration = start.elapsed();
                            println!("Result: {}", result);
                            println!("Execution time: {:?}", duration);
                        }
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                Err(e) => eprintln!("Could not read file '{}': {}", filename, e),
            }
        }
    } else {
        // Interactive REPL
        println!("Luna Lua JIT Interpreter v0.1.0");
        println!("Type 'exit' to quit, '.stats' to show JIT stats");

        let mut runtime = new_runtime();

        loop {
            print!("> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let input = input.trim();
                    if input == "exit" || input == "quit" {
                        break;
                    }

                    if input == ".stats" {
                        runtime.print_stats();
                        continue;
                    }

                    if !input.is_empty() {
                        let start = Instant::now();
                        match runtime.execute(input) {
                            Ok(result) => {
                                let duration = start.elapsed();
                                if !matches!(result, Value::Nil) {
                                    println!("{}", result);
                                }
                                if duration.as_millis() > 1 {
                                    println!("(took {:?})", duration);
                                }
                            }
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error reading input: {}", e);
                    break;
                }
            }
        }

        println!("\nFinal JIT Statistics:");
        runtime.print_stats();
    }
}

fn run_with_jit_stats(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(code) => {
            println!("Running file with JIT stats: {}", filename);
            let mut runtime = new_runtime();
            let start = Instant::now();

            match runtime.execute(&code) {
                Ok(result) => {
                    let duration = start.elapsed();
                    println!("Result: {}", result);
                    println!("Execution time: {:?}", duration);
                    println!("\nJIT Statistics:");
                    runtime.print_stats();
                }
                Err(e) => eprintln!("Error: {}", e),
            }
        }
        Err(e) => eprintln!("Could not read file '{}': {}", filename, e),
    }
}

fn run_benchmark(filename: &str) {
    match fs::read_to_string(filename) {
        Ok(code) => {
            println!("Benchmarking file: {}", filename);

            // Run multiple times to see JIT improvements
            let iterations = 5;
            let mut times = Vec::new();

            for i in 1..=iterations {
                let mut runtime = new_runtime();
                let start = Instant::now();

                match runtime.execute(&code) {
                    Ok(_) => {
                        let duration = start.elapsed();
                        times.push(duration);
                        println!("Run {}: {:?}", i, duration);

                        if i == 1 {
                            println!("JIT Stats after first run:");
                            runtime.print_stats();
                        }
                    }
                    Err(e) => {
                        eprintln!("Error on run {}: {}", i, e);
                        return;
                    }
                }
            }

            // Calculate statistics
            let avg_time = times.iter().sum::<std::time::Duration>() / times.len() as u32;
            let min_time = times.iter().min().unwrap();
            let max_time = times.iter().max().unwrap();

            println!("\nBenchmark Results:");
            println!("Average time: {:?}", avg_time);
            println!("Fastest run:  {:?}", min_time);
            println!("Slowest run:  {:?}", max_time);

            if times.len() > 1 {
                let improvement = (times[0].as_nanos() as f64 - min_time.as_nanos() as f64)
                    / times[0].as_nanos() as f64
                    * 100.0;
                if improvement > 0.0 {
                    println!("JIT improvement: {:.1}% faster", improvement);
                }
            }
        }
        Err(e) => eprintln!("Could not read file '{}': {}", filename, e),
    }
}
