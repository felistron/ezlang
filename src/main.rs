use std::process::exit;

use clap::Parser;

mod ezlang;

#[derive(Debug, Parser)]
struct Args {
    /// Filename of the source code
    #[arg()]
    input_filename: String,
}

fn main() {
    let args = Args::parse();
    
    let mut compiler: ezlang::Compiler = ezlang::Compiler::new(&args.input_filename);

    match compiler.compile() {
        Ok(_) => {
            println!("Compilation succesful!");
        },
        Err(error) => {
            println!("{}", error);
            exit(1);
        },
    };    
}
