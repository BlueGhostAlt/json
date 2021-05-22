use std::error;

use json::{input_reader::MemoryReader, lexer::Lexer};

const SOURCE: &[u8] = r#"[
    null,
    true,
    false,
    0,
    -0,
    19,
    -23,
    0.1,
    -2.5
    42e-1
    18E1
]"#
.as_bytes();

fn try_main() -> Result<(), Box<dyn error::Error>> {
    let reader = MemoryReader::new(SOURCE)?;
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
