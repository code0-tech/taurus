use std::fs::File;
use std::io::{BufRead, BufReader, Write};

/*
        Compiled POC 1.
        This is just writing `rust` as string to a file
        What can be optimised?
            - marco to turn macros into a string | import from macro module

        Av. execution time: 2.1µs
 */
fn process_input(input_file: &str, output_file: &str) -> std::io::Result<()> {
    let input = File::open(input_file)?;
    let reader = BufReader::new(input);
    let mut output = File::create(output_file)?;

    writeln!(output, "\
    //This is a generated main file
macro_rules! add_integers {{
    ($first:expr, $second:expr) => {{
        $first + $second
    }};
}}

macro_rules! subtract_integers {{
    ($first:expr, $second:expr) => {{
        $first - $second
    }};
}}

macro_rules! multiply_integers {{
    ($first:expr, $second:expr) => {{
        $first * $second
    }};
}}
fn main() {{")?;

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split('|').collect();

        if parts.len() == 3 {
            match parts[0] {
                "add" => writeln!(output, "    add_integers!({}, {});", parts[1], parts[2])?,
                "sub" => writeln!(output, "    subtract_integers!({}, {});", parts[1], parts[2])?,
                "multiply" => writeln!(output, "    multiply_integers!({}, {});", parts[1], parts[2])?,
                _ => eprintln!("Unknown operation: {}", parts[0]),
            }
        }
    }

    writeln!(output, "}}")?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    process_input("./poc.txt", "main.rs")
}