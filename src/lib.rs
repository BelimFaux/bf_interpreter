use clap::Parser;
use std::io::Read;

#[derive(Parser)]
#[command(version)]
pub struct Config {
    pub program: String,
    #[arg(default_value_t = 10, short = 's', long = "size")]
    pub cell_sz: usize,
}

#[derive(Debug)]
pub struct Machine {
    cells: Vec<u8>,
    ptr: usize,
}

impl Machine {
    pub fn new(cnfg: &Config) -> Machine {
        let cells = vec![0; cnfg.cell_sz];
        let ptr = 0;
        Machine { cells, ptr }
    }

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
                    let skip = find_close(&program[index..]) - 1;
                    it.nth(skip);
                },
                ']' => return,
                _ => continue,
            }
        }
    }

    fn mv_left(&mut self) {
        if self.ptr > self.cells.len() - 1 {
            panic!("Stack Overflow! Try running again with a bigger cell size");
        }
        self.ptr += 1;
    }

    fn mv_right(&mut self) {
       if self.ptr < 1 {
           panic!("Stack Underflow!")
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
            .unwrap();
        self.cells[self.ptr] = input;
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
    panic!("no corresponding bracket found.");
}
