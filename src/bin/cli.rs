use std::error;

use json::{input_reader::MemoryReader, lexer::Lexer};

fn try_main() -> Result<(), Box<dyn error::Error>> {
    let source = "{}?{{?}}";

    let reader = MemoryReader::new(source.as_bytes())?;
    let lexer = Lexer::new(reader)?;

    for t in lexer {
        println!("{:?}", t);
    }

    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        panic!("{:?}", error);
    }
}
