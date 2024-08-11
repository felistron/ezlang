mod lexer;

use lexer::Lexer;

fn main() {
    let mut lexer = Lexer::from_file("examples/main.ez");
    while let Some(token) = lexer.next() {
        println!("{:#?}", token);
    }

    println!("No more tokens found");
}
