pub mod edge;
pub mod error;
pub mod graphs;
mod graphviz;
pub mod node;
pub mod parser;

pub use edge::Edge;
pub use error::DotGraphError;
pub use graphs::{Graph, SubGraph};
pub use node::Node;

pub(crate) fn pretty_id(id: &str) -> String {
    if id.chars().all(char::is_alphanumeric) {
        id.to_string()
    } else {
        format!("\"{id}\"")
    }
}
