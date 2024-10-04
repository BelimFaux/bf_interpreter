use core::ops::Deref;

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
