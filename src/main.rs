use clap::Parser;
use bf_interpreter::*;

fn main() {
    let mut cnfg = Config::parse();

    let mut machine = Machine::new(&cnfg);

    machine.run(cnfg.get_program());

    println!("\nFinal State: {}", machine);
}
