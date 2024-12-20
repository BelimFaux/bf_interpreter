use clap::Parser;
use std::process;
use bf_interpreter::*;

fn main() {
    let mut cnfg = Config::parse();
    let optimize = cnfg.optimize;

    let program_str = match cnfg.get_program() {
        Ok(str) => str,
        Err(err) => {
            eprintln!("Error while reading the Input file:\n{err}");
            process::exit(1);
        }
    };

    let program = match compiler::Program::from_str(program_str, optimize) {
        Ok(program) => program,
        Err(err) => {
            eprintln!("{}", err.get_error_msg(program_str));
            process::exit(1);
        }
    };

    let mut machine = vm::Machine::new(&cnfg);
    if let Err(err) = machine.run(&program) {
        eprintln!("{}", err);
        process::exit(1);
    }
}
