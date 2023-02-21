use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotGraphError {
    #[error("`{0}` is not a valid dot graph")]
    InvalidGraph(String),
    #[error("`{0}` is not a digraph")]
    UndirectedGraph(String),
    #[error("`{0}` contains a cycle")]
    Cycle(String),
    #[error("`{0}` is not a node of graph `{1}`")]
    NoSuchNode(String, String),
    #[error("`{0}` is not a subgraph of graph `{1}`")]
    NoSuchSubGraph(String, String),
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
