use clap::Parser;
use bf_interpreter::*;

fn main() {
    let cnfg = Config::parse();

    let mut machine = Machine::new(&cnfg);

    machine.run(&cnfg.program);

    println!("\nFinal State: {}", machine);
}
