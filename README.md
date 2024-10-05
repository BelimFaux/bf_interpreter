# Brainfuck Interpreter
simple Brainfuck interpreter written in Rust, to learn more about the language, and building CLI-Applications

## Brainfuck
Brainfuck is an esoteric programming language created in 1993 [...]. Designed to be extremely minimalistic, the language consists of only eight simple commands [...].  
[see Wikipedia](https://en.wikipedia.org/wiki/Brainfuck)

## Usage
The Program can be compiled with cargo:
```bash
git clone https://github.com/BelimFaux/bf_interpreter.git
cd bf_interpreter
cargo run --release -- examples/hello_world.bf
```
It takes atleast one parameter for the Input-file or optionally the bf-code.
It's also possible to set the size of the cell band, which is by default set to 10.

For all options run `./target/release/bf-interpreter -h`.

Examples are taken from the Wikipedia page of Brainfuck (see link above)
