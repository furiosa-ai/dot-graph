pub mod id;

use crate::attr::Attr;
pub use id::EdgeId;

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

#[derive(Debug, Clone, Eq)]
/// An (directed) `Edge` of a graph.
pub struct Edge {
    /// A tuple of start and end points
    pub(crate) id: EdgeId,
    /// Attributes of the edge in key, value mappings
    pub(crate) attrs: HashSet<Attr>,
}

impl PartialEq for Edge {
    fn eq(&self, other: &Edge) -> bool {
        self.id == other.id
    }
}

impl Hash for Edge {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Borrow<EdgeId> for Edge {
    fn borrow(&self) -> &EdgeId {
        &self.id
    }
}

impl Edge {
    pub(crate) fn new(id: EdgeId, attrs: HashSet<Attr>) -> Edge {
        Edge { id, attrs }
    }

    pub fn id(&self) -> &EdgeId {
        &self.id
    }

    pub fn attrs(&self) -> &HashSet<Attr> {
        &self.attrs
    }

    /// Write the edge to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        self.id.to_dot(indent, writer)?;

        writeln!(writer, " [")?;
        for attr in &self.attrs {
            attr.to_dot(indent + 1, writer)?;
        }
        (0..indent).try_for_each(|_| write!(writer, "\t"))?;
        writeln!(writer, "]")?;

        Ok(())
    }
}
