use core::ops::Deref;
use std::collections::hash_map::HashMap;

#[derive(Debug)]
enum Token {
    RBrac { line: usize, col: usize },  // Brackets store position information, because they are the only Tokens, that can produce ParseErrors
    LBrac { line: usize, col: usize },
    Plus,
    Minus,
    Less,
    Greater,
    Dot,
    Comma,
    EOF,
}

#[derive(Debug, PartialEq, Clone)]
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

impl Instruction {
    fn increment(&mut self) -> bool {
        match self {
            Instruction::MvLeft(amount) => *amount += 1,
            Instruction::MvRight(amount) => *amount += 1,
            Instruction::Inc(amount) => *amount += 1,
            Instruction::Dec(amount) => *amount += 1,
            _ => return false,
        }
        true
    }
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

    fn format_error(line: usize, col: usize, line_str: &str) -> String {
        let mut error_str = format!("{line} {line_str}");
        let ln_len = line.to_string().len();
        let arrow = col + ln_len;
        error_str.push_str("\n ");
        error_str.push_str(&" ".repeat(arrow));
        error_str.push('^');
        error_str
    }

    pub fn get_error_msg(mut self, program: &str) -> String {
        let ending = if self.errors.len() == 1 { '\0' } else { 's' };
        let mut msg = format!("{} error{} occured during parsing:\n", self.errors.len(), ending);

        self.errors.reverse();
        for err in self.errors {
            let str = match err {
                Token::RBrac { line, col } => {
                    let line_str = program.lines().nth(line-1).expect("line should always exist");
                    format!("Unexpected closing bracket found at {line}:{col}: \n {}\n", ParseError::format_error(line, col, line_str))
                },
                Token::LBrac { line, col } => {
                   let line_str = program.lines().nth(line-1).expect("line should always exist");
                   format!("Opening bracket at {line}:{col} wasn't closed: \n {}\n", ParseError::format_error(line, col, line_str))
                },
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
                        let jmp_addr = instructions.len();
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

    pub fn from_str(program: &str, optimize: bool) -> Result<Program, ParseError> {
        let mut program = Program::parse(Program::tokenize(&program))?;
        if optimize {
            program.optimize();
        }
        Ok(program)
    }

    fn optimize(&mut self) {
        if self.instructions.is_empty() { return; }

        let mut optimized_instructions = Vec::with_capacity(self.instructions.len());
        let instr = self.instructions.first().expect("").clone();
        let mut removed = 0usize;
        let mut new_jmp_addrs = HashMap::new();
        optimized_instructions.push(instr);

        for (i, instr) in self.instructions.iter().skip(1).enumerate() {
            let last_added = optimized_instructions.last_mut().expect("vec shouldnt be empty");

            // increment count, if type is the same
            if std::mem::discriminant(instr) == std::mem::discriminant(last_added) {
                if last_added.increment() { removed += 1; continue; }
            }
            // save new jmp addresses if necessary
            match instr {
                Instruction::Jmp(_) | Instruction::JmpZ(_) => {
                    new_jmp_addrs.insert(i + 1, removed);
                },
                _ => {},
            };
            optimized_instructions.push(instr.clone());
        }

        // patch jmp addresses
        for instr in &mut optimized_instructions {
            match instr {
                Instruction::Jmp(addr) | Instruction::JmpZ(addr) => {
                    *addr -= new_jmp_addrs.get(addr).expect("addr shoulb be in vec");
                },
                _ => {},
            }
        }

        optimized_instructions.shrink_to_fit();
        self.instructions = optimized_instructions;
    }
}
