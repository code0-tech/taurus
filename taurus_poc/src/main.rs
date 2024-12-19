use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::BufRead;
use std::time::Instant;

macro_rules! add_integers {
    ($first:expr, $second:expr) => {
        $first + $second
    };
}

macro_rules! subtract_integers {
    ($first:expr, $second:expr) => {
        $first - $second
    };
}

macro_rules! multiply_integers {
    ($first:expr, $second:expr) => {
        $first * $second
    };
}

type OperationFn = fn(i32, i32) -> i32;

fn add(a: i32, b: i32) -> i32 {
    add_integers!(a, b)
}
fn subtract(a: i32, b: i32) -> i32 {
    subtract_integers!(a, b)
}
fn multiply(a: i32, b: i32) -> i32 {
    multiply_integers!(a, b)
}

/*
       Interpreted POC 1.
       This is just interprets incoming flows
       What can be optimised?
           - flow till be typed so that no string processing is needed
           - compromise match statement into map
           - import macros from shared macro module

       Av. execution time: 800-600µs

       Interpreted POC 2.
       Moved Timer after string processing to consider typed request that are incoming
       Function call will be put into HashMap to simulate function registry

       Av. execution time: 190-160µs
*/
fn main() {
    // Simulate Standard Function Registry
    let mut operations: HashMap<&str, OperationFn> = HashMap::new();
    operations.insert("add", add);
    operations.insert("sub", subtract);
    operations.insert("multiply", multiply);

    // Simulate typed gRPC Messages.
    // Will be slower in reality but at least no string processing is needed
    let mut functions: Vec<(&str, i32, i32)> = Vec::new();

    let binding = read_to_string("taurus_poc/poc.txt").unwrap();
    for line in binding.lines() {
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() == 3 {
            let operator = parts[0];
            let first: i32 = parts[1].parse().unwrap_or(0);
            let second: i32 = parts[2].parse().unwrap_or(0);

            functions.push((operator, first, second));
        } else {
            println!("Invalid line format");
        }
    }

    // Here would start the main application
    let start = Instant::now();

    // Resolve incoming requests
    for func in functions {
        match operations.get(func.0) {
            Some(operation) => {
                let result = operation(func.1, func.2);
                println!("{} {} {} = {}", &func.0, func.1, func.2, result);
            }
            None => println!("Unsupported operator: {}", func.0),
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}
