use clap::Parser;
use std::{io::Read, fmt::Display, process, fs};

#[derive(Parser)]
#[command(version)]
pub struct Config {
    /// File OR programcode [default: File]
    program: String,

    /// Length of band
    #[arg(default_value_t = 10, short = 's', long = "size")]
    pub cell_sz: usize,

    /// Type of input. If set, instead of a file the programcode is expected
    #[arg(short = 'i', long = "input", action)]
    inp_type: bool,
}

impl Config {
    /// return the correct bf program as a string slice
    /// if inp_type isnt set, the file will be read and placed into the program field
    pub fn get_program(&mut self) -> &str {
        if self.inp_type {
            &self.program
        } else {
            let contents = fs::read_to_string(self.program.clone());
            if let Ok(contents) = contents {
                self.program = contents;
                self.inp_type = false;
                &self.program
            } else {
                eprintln!("Error: File couldn't be read: {}", contents.unwrap_err());
                process::exit(1);
            }
        }
    }
}

/// Machine struct, to emulate a kind of Turingmachine, that can be operated via Brainfuck code
pub struct Machine {
    cells: Vec<u8>,
    ptr: usize,
}

impl Machine {
    /// Create a new Machine from a Config struct
    /// The machine will contain a vec of cells with value 0, and a ptr starting at cell 0
    pub fn new(cnfg: &Config) -> Machine {
        let cells = vec![0; cnfg.cell_sz];
        let ptr = 0;
        Machine { cells, ptr }
    }

    /// run a bf program on the Machine
    /// input the program as a string slice, all invalid characters will be ignored as comments
    pub fn run(&mut self, program: &str) {
        let mut it = program.chars().enumerate();
        while let Some((index, char)) = it.next() {
            match char {
                '>' => self.mv_left(),
                '<' => self.mv_right(),
                '+' => self.inc(),
                '-' => self.dec(),
                '.' => self.put(),
                ',' => self.get(),
                '[' => {
                    while self.cells[self.ptr] != 0 {
                        self.run(&program[index+1..]);
                    }
                    // skip to closing bracket
                    let skip = find_close(&program[index..]) - 1;
                    it.nth(skip);
                },
                ']' => return,
                _ => continue,
            }
        }
    }

    fn mv_left(&mut self) {
        // pointer can't move further than the cell size, so exit program
        if self.ptr > self.cells.len() - 1 {
            eprintln!("Error: Stack Overflow. Pointer can't move beyond {}. Try running again with a bigger cell size", self.cells.len());
            process::exit(1);
        }
        self.ptr += 1;
    }

    fn mv_right(&mut self) {
        // pointer can't move below 0, so exit program
        if self.ptr < 1 {
           eprintln!("Error: Stack Underflow. Pointer can't move below 0");
           process::exit(1);
        }
        self.ptr -= 1;
    }

    fn inc(&mut self) {
        self.cells[self.ptr] = self.cells[self.ptr].wrapping_add(1);
    }

    fn dec(&mut self) {
        self.cells[self.ptr] = self.cells[self.ptr].wrapping_sub(1);
    }

    fn put(& self) {
        let ch = char::from(self.cells[self.ptr]);
        print!("{ch}");
    }

    fn get(&mut self) {
        let input = std::io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte as u8)
            .unwrap_or(0);

        self.cells[self.ptr] = input;
    }
}

impl Display for Machine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut cells = String::new();
        for (index, cell) in self.cells.iter().enumerate() {
            if index == self.ptr {
                cells.push_str(&format!(">[{cell}]<"));
            } else {
                cells.push_str(&format!("[{cell}]"));
            }
        }
        write!(f, "{}", cells)
    }
}

fn find_close(program: &str) -> usize {
    let mut stack = 0;
    for (index, char) in program.chars().enumerate() {
        match char {
            '[' => stack += 1,
            ']' => {
                stack -= 1;
                if stack == 0 {
                    return index;
                }
            },
            _ => continue,
        }
    }
    eprintln!("no corresponding bracket found.");
    process::exit(1)
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_machine(cell_sz: usize) -> Machine {
        let cnfg = Config { program: "".to_owned(), cell_sz, inp_type: false };
        Machine::new(&cnfg)
    }

    #[test]
    fn closing_bracket() {
        let program = "[+++-<]";
        assert_eq!(find_close(program), 6);
    }

    #[test]
    fn nested_closing_bracket() {
        let program = "[+[]+[]+-<]";
        assert_eq!(find_close(program), 10);

        let program = "[+[[]+++]++-<]";
        assert_eq!(find_close(program), 13);
    }

    #[test]
    fn incr_curr_cell() {
        let mut machine = setup_machine(1);
        machine.run("+++");
        assert_eq!(format!("{machine}"), ">[3]<");

        // wrapping
        let mut machine = setup_machine(1);
        for _ in 0..256 {
            machine.run("+");
        }
        assert_eq!(format!("{machine}"), ">[0]<");
    }

    #[test]
    fn decr_curr_cell() {
        let mut machine = setup_machine(1);
        machine.run("+++---");
        assert_eq!(format!("{machine}"), ">[0]<");

        // wrapping
        machine.run("-");
        assert_eq!(format!("{machine}"), ">[255]<");
    }

    #[test]
    fn loop_incr() {
        let mut machine = setup_machine(2);
        machine.run("++[>++<-]");
        assert_eq!(format!("{machine}"), ">[0]<[4]");
    }

    #[test]
    fn loop_nested() {
        let mut machine = setup_machine(3);
        machine.run("++[>++[>++<-]<-]");
        assert_eq!(format!("{machine}"), ">[0]<[0][8]");
    }

}
