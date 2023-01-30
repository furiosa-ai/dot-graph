use thiserror::Error;

#[derive(Error, Debug)]
pub enum DotGraphError {
    #[error("`{0}` is not a valid dot graph")]
    Invalid(String),
    #[error("`{0}` is not a digraph")]
    UnDiGraph(String),
}
