use std::fs::{read_to_string};
use std::io::BufRead;
use std::time::Instant;

macro_rules! add_integers {
    ($first:expr, $second:expr) => {
        $first+$second
    };
}

macro_rules! subtract_integers {
    ($first:expr, $second:expr) => {
        $first-$second
    };
}

macro_rules! multiply_integers {
    ($first:expr, $second:expr) => {
        $first*$second
    };
}

/*
        Interpreted POC 1.
        This is just interprets incoming flows
        What can be optimised?
            - flow till be typed so that no string processing is needed
            - compromise match statement into map
            - import macros from shared macro module

        Av. execution time: 627.2µs
 */
fn main() {
    let start = Instant::now();

    for line in read_to_string("taurus_poc/poc.txt").unwrap().lines() {

        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() == 3 {
            let operator = parts[0];
            let first: i32 = parts[1].parse().unwrap_or(0);
            let second: i32 = parts[2].parse().unwrap_or(0);

            let result = match operator {
                "add" => add_integers!(first, second),
                "sub" => subtract_integers!(first, second),
                "multiply" => multiply_integers!(first, second),
                _ => {
                    println!("Unsupported operator: {}", operator);
                    continue;
                }
            };

            println!("{} {} {} = {}", operator, first, second, result);
        } else {
            println!("Invalid line format");
        }
    }

    let duration = start.elapsed();
    println!("Time elapsed in expensive_function() is: {:?}", duration);
}