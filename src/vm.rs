use core::fmt::Display;
use std::io::Read;

use crate::{Config, compiler::{Instruction, Program}};

pub enum RuntimeError {
    CellOverflow(String),
    CellUnderflow(String),
}

impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::CellOverflow(str) => write!(f, "CellOverflow Error: {}", str),
            RuntimeError::CellUnderflow(str) => write!(f, "CellUnderflow Error: {}", str),
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

    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        let mut instr_ptr = 0usize;
        let mut instr = program.get(0).expect("should always be inside vec");

        while *instr != Instruction::Exit {
            match instr {
                Instruction::MvLeft(times) => self.mv_left(*times)?,
                Instruction::MvRight(times) => self.mv_right(*times)?,
                Instruction::Inc(times) => self.inc(*times),
                Instruction::Dec(times) => self.dec(*times),
                Instruction::Get => self.get(),
                Instruction::Put => self.put(),
                Instruction::Jmp(addr) => {
                    instr_ptr = *addr;
                    instr = program.get(instr_ptr).expect("jump failed");
                    continue;
                },
                Instruction::JmpZ(addr) => {
                    if self.value() == 0 {
                        instr_ptr = *addr + 1;
                        instr = program.get(instr_ptr).expect("jump failed");
                        continue;
                    }
                },
                Instruction::Exit => continue,
            }
            instr_ptr += 1;
            instr = program.get(instr_ptr).expect("should be inside vec");
        }

        Ok(())
    }

    fn value(&self) -> u8 {
        *&self.cells[self.ptr]
    }

    fn mv_right(&mut self, times: usize) -> Result<(), RuntimeError> {
        // pointer can't move further than the cell size, so throw a runtime error
        if self.ptr + times >= self.cells.len() {
            return Err(
                RuntimeError::CellOverflow(
                    format!("Pointer can't move beyond {}. Try running again with a bigger cell size", self.cells.len())
                    )
                );
        }
        self.ptr += times;
        Ok(())
    }

    fn mv_left(&mut self, times: usize) -> Result<(), RuntimeError> {
        // pointer can't move below 0, so exit program
        if self.ptr.saturating_sub(times - 1) == 0 {
            return Err(
                RuntimeError::CellOverflow(
                    String::from("Pointer can't move below 0")
                    )
                );
        }
        self.ptr -= times;
        // println!("{}", self.ptr);
        Ok(())
    }

    fn inc(&mut self, times: usize) {
        self.cells[self.ptr] = self.cells[self.ptr].wrapping_add((times % u8::max_value() as usize) as u8);
    }

    fn dec(&mut self, times: usize) {
        self.cells[self.ptr] = self.cells[self.ptr].wrapping_sub((times % u8::max_value() as usize) as u8);
    }

    fn put(&self) {
        let ch = char::from(self.value());
        print!("{ch}");
    }

    fn get(&mut self) {
        let input = std::io::stdin()
            .bytes()
            .next()
            .and_then(|result| result.ok())
            .map(|byte| byte)
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
                cells.push_str(&format!(" [{cell}] "));
            }
        }
        write!(f, "{}", cells)
    }
}
