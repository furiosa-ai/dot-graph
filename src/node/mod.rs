use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type NodeId = String;

#[derive(Debug, Clone, Eq)]
/// A `Node` of a graph.
pub struct Node {
    /// Name of the node
    pub id: NodeId,
    /// Attributes of the node in key, value mappings
    pub attrs: HashMap<String, String>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Node) -> bool {
        self.id == other.id
    }
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
        (0..indent).try_for_each(|_| write!(writer, "\t"))?;
        writeln!(writer, "{}[", self.id)?;

        for (key, value) in &self.attrs {
            (0..indent).try_for_each(|_| write!(writer, "\t"))?;

            // TODO naive workaround to visualize HTML strings
            if value.contains("TABLE") {
                writeln!(writer, "{}=<{}>", key, value)?;
            } else {
                writeln!(writer, "{}=\"{}\"", key, value)?;
            }
        }

        (0..indent).try_for_each(|_| write!(writer, "\t"))?;
        write!(writer, "];")?;

        Ok(())
    }
}
