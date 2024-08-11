mod lexer;
mod parser;

use parser::Parser;

fn main() {
    let mut parser = Parser::from_file("examples/basic.ez");

    parser.generate_tokens();

    let program = parser.generate_program();

    println!("{:#?}", program);
}
