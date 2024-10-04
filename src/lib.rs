use clap::Parser;
use core::ops::Deref;
use std::{io::{self, Read}, fmt::Display, fs};

#[derive(Parser)]
#[command(version)]
pub struct Config {
    /// File OR programcode [default: File]
    program: String,

    /// Amount of cells available
    #[arg(default_value_t = 100, short = 'c', long = "cells")]
    pub cell_sz: usize,

    /// Type of input. If set, instead of a file the programcode is expected
    #[arg(short = 'i', long = "input", action)]
    inp_type: bool,
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
enum Token {
    RBrac { line: u32, col: u32 },  // Brackets store position information, because they are the only Tokens, that can produce ParseErrors
    LBrac { line: u32, col: u32 },
    Plus,
    Minus,
    Less,
    Greater,
    Dot,
    Comma,
    EOF,
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    MvLeft(usize),
    MvRight(usize),
    Inc(usize),
    Dec(usize),
    Jmp(usize),
    JmpZ(usize),
    Get,
    Put,
    Exit,
}

pub struct ParseError {
    errors: Vec<Token>,
}

impl ParseError {
    fn new() -> Self {
        ParseError { errors: Vec::new() }
    }

    fn report_error(&mut self, token: Token) {
        self.errors.push(token)
    }

    fn had_error(&self) -> bool {
        self.errors.len() != 0
    }

    pub fn get_error_msg(&self, _program: &str) -> String {
        let mut msg = format!("{} errors occured during parsing:\n", self.errors.len());

        for err in &self.errors {
            let str = match err {
                Token::RBrac { line, col } => format!("Unexpected closing bracket found (l.{line}:{col}).\n"),
                Token::LBrac { line, col } => format!("Opening bracket wasn't closed (l.{line}:{col}).\n"),
                _ => format!("Unexpected Error at {:?}\n", err),
            };
            msg.push_str(&str);
        }

        msg
    }
}

/// Wrapper for a Token vector to avoid manipulation
#[derive(Debug)]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl Deref for Program {
    type Target = Vec<Instruction>;

    fn deref(&self) -> &Self::Target {
        &self.instructions
    }
}

impl Program {
    /// parse a bf program to a series of Tokens
    fn tokenize(program: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut line = 1;
        let mut col = 0;

        for char in program.chars() {
            col += 1;
            let token = match char {
                '+' => Token::Plus,
                '-' => Token::Minus,
                '<' => Token::Less,
                '>' => Token::Greater,
                ']' => Token::RBrac { line, col },
                '[' => Token::LBrac { line, col },
                '.' => Token::Dot,
                ',' => Token::Comma,
                '\n' => {
                    line += 1;
                    col = 0;
                    continue;
                },
                _ => continue,
            };
            tokens.push(token);
        }

        tokens.push(Token::EOF);
        tokens
    }

    fn parse(program: Vec<Token>) -> Result<Program, ParseError> {
        let mut instructions = Vec::new();
        let mut jmp_addresses = Vec::new();
        let mut errors = ParseError::new();

        for token in program {
            let instr = match token {
                Token::Plus => Instruction::Inc(1),
                Token::Minus => Instruction::Dec(1),
                Token::Greater => Instruction::MvRight(1),
                Token::Less => Instruction::MvLeft(1),
                Token::Dot => Instruction::Put,
                Token::Comma => Instruction::Get,
                Token::RBrac { .. } => {
                    if let Some((token, address)) = jmp_addresses.pop() {
                        let jmp_addr = instructions.len() + 1;  // jump past this instr
                        match instructions.get_mut(address).expect("jmp address should always exist") {
                            Instruction::JmpZ(addr) => *addr = jmp_addr,
                            _ => errors.report_error(token),
                        }
                        Instruction::Jmp(address)
                    } else {    // if no address is on top of the stack, no open bracket is remaining
                        errors.report_error(token);
                        continue;
                    }
                },
                Token::LBrac { .. } => {
                    jmp_addresses.push((token, instructions.len()));
                    Instruction::JmpZ(0)
                }
                Token::EOF => Instruction::Exit,
            };
            instructions.push(instr)
        }

        while let Some((token, _address)) = jmp_addresses.pop() {
            errors.report_error(token);
        }

        if errors.had_error() {
            Err(errors)
        } else {
            Ok(Program { instructions })
        }
    }

    pub fn from_str(program: &str) -> Result<Program, ParseError> {
        Program::parse(Program::tokenize(&program))
    }
}

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
                        instr_ptr = *addr;
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

#[cfg(test)]
mod test {
    use super::*;

    fn setup_machine(cell_sz: usize) -> Machine {
        let cnfg = Config { program: "".to_owned(), cell_sz, inp_type: false };
        Machine::new(&cnfg)
    }
}
