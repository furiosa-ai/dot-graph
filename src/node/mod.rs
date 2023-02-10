use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type NodeId = String;

#[derive(Debug, Clone, PartialEq, Eq)]
/// A `Node` of a graph.
pub struct Node {
    /// Name of the node
    pub id: NodeId,
    /// Attributes of the node in key, value mappings
    pub attrs: HashMap<String, String>,
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<NodeId> for Node {
    fn borrow(&self) -> &NodeId {
        &self.id
    }
}

impl Node {
    /// Write the node to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let tabs = "\t".repeat(indent);

        writeln!(writer, "{}{}[", tabs, self.id)?;

        for (key, value) in &self.attrs {
            // TODO naive workaround to visualize HTML strings
            if value.contains("TABLE") {
                writeln!(writer, "{}{}=<{}>", tabs, key, value)?;
            } else {
                writeln!(writer, "{}{}=\"{}\"", tabs, key, value)?;
            }
        }

        write!(writer, "{}];", tabs)?;

        Ok(())
    }
}
