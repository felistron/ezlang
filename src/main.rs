mod compiler;
mod lexer;
mod parser;

use compiler::Compiler;

fn main() {
    let filename = "examples/square.ez";
    let mut program = Compiler::from_file(filename);
    program.compile();
}
