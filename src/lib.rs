use clap::Parser;
use std::{io, fs};

pub mod compiler;
pub mod vm;

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

    /// If program should be optimized
    #[arg(short = 'o', long = "optimize", action)]
    pub optimize: bool,
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
