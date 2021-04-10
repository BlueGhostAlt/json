use json::input_reader::{BufferedReader, MemoryReader};

fn main() {
    let source = "aăâbcdefghiîjklmnopqrsștțuvwxyz";

    let buf_reader = BufferedReader::new(source.as_bytes());
    let mem_reader =
        MemoryReader::new(source.as_bytes()).expect("Failed to create a memory reader");

    assert!(buf_reader.eq(mem_reader));
}
