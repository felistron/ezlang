mod lexer;
mod parser;

use parser::Parser;

fn main() {
    let mut parser = Parser::from_file("examples/function_call.ez");

    parser.generate_tokens();

    let program = parser.generate_program();

    println!("{:#?}", program);
}
