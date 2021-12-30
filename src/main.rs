mod engine;

use engine::engine::*;
use std::env;

fn print_result(res: Result<CalcResult, CalcError>) {
    match res {
        Ok(res) => println!("{}", res),
        Err(err) => println!("{}", err),
    }
}

fn print_version() {
    println!("0.0.1");
}

fn print_help() {
    println!("\n****************************************\nWELCOME TO THE SQL ENGINE\n\nHELP: -h, --help\nGET VERSION: -v, --version\n\n\n----------------------------------------\nSTATEMENTS: SELECT\nOPERATORS: +, -, *, >\nFUNCS: SQRT\n****************************************\n");
}

fn print_default() {
    println!("\n****************************************\nSQL ENGINE\nPLEASE, TEXT ME COMMAND\n****************************************\n");
}

fn main() {
    
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        return print_default(); 
    }

    match args[1].as_ref() {
        "-v" => return print_version(),
        "--version" => return print_version(),
        "-h" => return print_help(),
        "--help" => return print_help(),
        _ => (),
    }

    print_result(exec(args[1..].join(" ")));
}
