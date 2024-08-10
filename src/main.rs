use clap::Parser;
use std::process;
use bf_interpreter::*;

fn main() {
    let mut cnfg = Config::parse();

    let program = Program::tokenize(cnfg.get_program());

    if let Err(err) = program {
        eprintln!("Error parsing the code: {:?}", err);
        process::exit(1);
    }

    let program = program.unwrap();

    let mut machine = Machine::new(&cnfg);

    machine.run(&program);

    println!("\nFinal State: {}", machine);
}
