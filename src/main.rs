use clap::Parser;
use std::process;
use bf_interpreter::*;

fn main() {
    let mut cnfg = Config::parse();

    let program_str = match cnfg.get_program() {
        Ok(str) => str,
        Err(err) => {
            eprintln!("Error while reading the Input file:\n{err}");
            process::exit(1);
        }
    };

    let program = match Program::tokenize(program_str) {
        Ok(prg) => prg,
        Err(err) => {
            eprintln!("Error parsing Code:\n{}", err.get_msg(program_str));
            process::exit(1);
        },
    };

    let mut machine = Machine::new(&cnfg);
    machine.run(&program);

    if cnfg.state {
        println!("\nFinal State: {}", machine);
    }
}
