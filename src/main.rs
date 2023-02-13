use dot_graph::parser;
use dot_graph::DotGraphError;

fn main() -> Result<(), DotGraphError> {
    let graph = parser::parse("ocr.dot")?;

    println!("{:?}", graph.neighbors(&"hello".to_string(), 5));

    Ok(())
}
