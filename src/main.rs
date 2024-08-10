mod lexer;

use clap::Parser;
use lexer::Lexer;

fn main() {
    let mut lexer = Lexer::from_file("examples/number.ez");
    while let Some(token) = lexer.next() {
        println!("{:#?}", token);
    }

    println!("No more tokens found");
}
