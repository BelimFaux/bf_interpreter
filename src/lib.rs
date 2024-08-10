use clap::Parser;
use core::ops::Deref;
use std::{io::{self, Read}, fmt::Display, process, fs};

#[derive(Parser)]
#[command(version)]
pub struct Config {
    /// File OR programcode [default: File]
    program: String,

    /// Amount of cells available
    #[arg(default_value_t = 10, short = 'c', long = "cells")]
    pub cell_sz: usize,

    /// Type of input. If set, instead of a file the programcode is expected
    #[arg(short = 'i', long = "input", action)]
    inp_type: bool,
    /// Show final state after successfully running the programm
    #[arg(short = 's', long = "state", action)]
    pub state: bool,
}

impl Config {
    /// return the correct bf program as a string slice
    /// if inp_type isnt set, the file will be read and placed into the program field
    pub fn get_program(&mut self) -> Result<&str, io::Error> {
        if self.inp_type {
            Ok(&self.program)
        } else {
            let contents = fs::read_to_string(self.program.clone())?;
            self.program = contents;
            self.inp_type = false;
            Ok(&self.program)
        }
    }
}

#[derive(Debug)]
pub enum Token {
    MvRight,
    MvLeft,
    Inc,
    Dec,
    GetChar,
    PutChar,
    Loop(usize),
}

#[derive(Debug)]
pub enum ParseError {
    NoCloseBracket(usize, usize),
    NoOpenBracket(usize, usize),
}

impl ParseError {
    pub fn get_msg(self, program: &str) -> String {
        let (mut e_str, idx, ln) = match self {
            ParseError::NoCloseBracket(idx, ln) => (format!("Missing closing bracket for '[' at {ln};{idx}:\n{ln} "), idx, ln),
            ParseError::NoOpenBracket(idx, ln) => (format!("Missing opening bracket for ']' at {ln};{idx}:\n{ln} "), idx, ln),
        };
        let line = program.lines().nth(ln-1).expect("there should always be atleast ln-1 lines");
        let from = if idx > 5 { idx - 5 } else { 0 };
        let to = if idx < line.len().checked_sub(5).unwrap_or(0) { idx + 6 } else { line.len() };
        if from != 0 {
            e_str.push_str("...");
        }
        e_str.push_str(&line[from..to]);
        if to != line.len() {
            e_str.push_str("...");
        }
        let arrow = if from == 0 { idx } else { 8 };
        e_str.push_str("\n  ");
        e_str.push_str(&" ".repeat(arrow));
        e_str.push('^');
        e_str
    }
}

/// Wrapper for a Token vector to avoid manipulation
pub struct Program {
    tokens: Vec<Token>,
}

impl Deref for Program {
    type Target = Vec<Token>;

    fn deref(&self) -> &Self::Target {
        &self.tokens
    }
}

impl Program {
    /// parse a bf program to a series of Tokens
    /// if there are any errors in the program e.g. non matching bracket a ParseError is returned
    pub fn tokenize(program: &str) -> Result<Program, ParseError> {
        let mut tokens = Vec::new();
        let mut brackets = 0;
        let mut lines = 1;
        let mut chars = 0;
        for (index, instr) in program.chars().enumerate() {
            match instr {
                '>' => tokens.push(Token::MvRight),
                '<' => tokens.push(Token::MvLeft),
                '+' => tokens.push(Token::Inc),
                '-' => tokens.push(Token::Dec),
                '.' => tokens.push(Token::PutChar),
                ',' => tokens.push(Token::GetChar),
                '[' => {
                    brackets += 1;
                    let end_idx = count_tokens_cbrack(&program[index..], chars, lines)?;
                    tokens.push(Token::Loop(end_idx));
                },
                ']' => {
                    brackets -= 1;
                    if brackets < 0 {
                        return Err(ParseError::NoOpenBracket(chars, lines))
                    }
                },
                '\n' => {
                    lines += 1;
                    chars = 0;
                    continue
                },
                _ => {},
           }
           chars += 1;
        }
        Ok(Program { tokens })
    }
}

/// count the number of Tokens (< > + - . , [) up to the matching close bracket
fn count_tokens_cbrack(program: &str, index: usize, lines: usize) -> Result<usize, ParseError> {
    let mut stack = 0;
    let mut counter = 0usize;
    for instr in program.chars() {
        match instr {
            '<' | '>' | '+' | '-' | '.' | ',' => counter += 1,
            '[' => {
                stack += 1;
                counter += 1;
            },
            // ] don't count as tokens by themselves so the counter doesn't get increased
            ']' => {
                stack -= 1;
                if stack == 0 {
                    return Ok(counter - 1);
                }
            },
            _ => continue,
        }
    }
    Err(ParseError::NoCloseBracket(index, lines))
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

    /// Run a Program on the Machine
    pub fn run(&mut self, program: &Program) {
        self.run_slice(program);
    }

    /// recursive helper for run, to handle nested loops
    fn run_slice(&mut self, program: &[Token]) {
        let mut it = program.iter().enumerate();
        while let Some((index, token)) = it.next() {
            match token {
                Token::MvLeft => self.mv_right(),
                Token::MvRight => self.mv_left(),
                Token::Inc => self.inc(),
                Token::Dec => self.dec(),
                Token::GetChar => self.get(),
                Token::PutChar => self.put(),
                Token::Loop(idx) => {
                    let end = index + 1 + *idx;
                    while *self.value() != 0 {
                        self.run_slice(&program[index+1..end]);
                    }
                    it.nth(*idx - 1);
                },
            }
        }
    }

    fn value(&self) -> &u8 {
        &self.cells[self.ptr]
    }

    fn mv_left(&mut self) {
        // pointer can't move further than the cell size, so exit program
        if self.ptr > self.cells.len() - 1 {
            eprintln!("Runtime Error: Pointer can't move beyond {}. Try running again with a bigger cell size", self.cells.len());
            process::exit(1);
        }
        self.ptr += 1;
    }

    fn mv_right(&mut self) {
        // pointer can't move below 0, so exit program
        if self.ptr < 1 {
           eprintln!("Runtime Error: Pointer can't move below 0");
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

    fn put(&self) {
        let ch = char::from(*self.value());
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
                cells.push_str(&format!(" [{cell}] "));
            }
        }
        write!(f, "{}", cells)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn setup_machine(cell_sz: usize) -> Machine {
        let cnfg = Config { program: "".to_owned(), cell_sz, inp_type: false };
        Machine::new(&cnfg)
    }

}
