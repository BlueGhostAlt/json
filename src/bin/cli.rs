use std::error;

use json::{input_reader::MemoryReader, lexer::Lexer};

const SOURCE: &[u8] = r#"{
    "example": {
        "title": "Just an example",
		"content": {
            "id": 0,
			"list": [
                {
                    "idx": 0,
                    "entry": {
                        "id": "First Example Entry",
                        "states": [true, false, true, true]
                    }
                }
            ]
        }
    }
}"#
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
