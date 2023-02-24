use crate::{attr::Attr, utils};

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type NodeId = String;

#[derive(Debug, Clone, Eq)]
/// A `Node` of a graph.
pub struct Node {
    /// Name of the node
    pub(crate) id: NodeId,
    /// Attributes of the node in key, value mappings
    pub(crate) attrs: HashSet<Attr>,
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
    pub(crate) fn new(id: NodeId, attrs: HashSet<Attr>) -> Node {
        Node { id, attrs }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn attrs(&self) -> &HashSet<Attr> {
        &self.attrs
    }

    /// Write the node to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let id = utils::pretty_id(&self.id);
        (0..indent).try_for_each(|_| write!(writer, "\t"))?;
        writeln!(writer, "{id} [")?;

        for attr in &self.attrs {
            attr.to_dot(indent + 1, writer)?;
        }

        (0..indent).try_for_each(|_| write!(writer, "\t"))?;
        writeln!(writer, "];")?;

        Ok(())
    }
}
