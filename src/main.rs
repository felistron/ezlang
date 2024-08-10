mod lexer;

use lexer::Lexer;

fn main() {
    let mut lexer = Lexer::from_file("examples/label.ez");
    while let Some(token) = lexer.next() {
        println!("{:#?}", token);
    }

    println!("No more tokens found");
}
