use std::collections::BTreeMap;
use std::io::{Result, Write};

pub type NodeId = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
/// A `Node` of a graph.
pub struct Node {
    /// Name of the node
    pub id: NodeId,
    /// Attributes of the node in key, value mappings
    pub attrs: BTreeMap<String, String>,
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
