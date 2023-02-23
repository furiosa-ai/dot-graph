pub mod attr;
pub mod edge;
pub mod error;
pub mod graphs;
mod graphviz;
pub mod node;
pub mod parser;
mod utils;

pub mod prelude {
    pub use crate::attr::Attr;
    pub use crate::edge::{Edge, EdgeId};
    pub use crate::error::DotGraphError;
    pub use crate::graphs::{Graph, GraphId, SubGraph};
    pub use crate::node::{Node, NodeId};
    pub use crate::parser;
}
