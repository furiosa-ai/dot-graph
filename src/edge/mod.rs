use crate::node::NodeId;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type EdgeId = (NodeId, NodeId);

#[derive(Debug, Clone, Eq)]
/// An (directed) `Edge` of a graph.
pub struct Edge {
    /// A tuple of start and end points
    pub(crate) id: EdgeId,
    /// Attributes of the edge in key, value mappings
    pub(crate) attrs: HashMap<String, String>,
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
    pub fn id(&self) -> &EdgeId {
        &self.id
    }

    pub fn attrs(&self) -> &HashMap<String, String> {
        &self.attrs
    }

    /// Start point's node id
    pub fn from(&self) -> &NodeId {
        &self.id.0
    }

    /// End point's node id
    pub fn to(&self) -> &NodeId {
        &self.id.1
    }

    /// Write the edge to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let mut ports = Vec::with_capacity(2);

        (0..indent).try_for_each(|_| write!(writer, "\t"))?;

        write!(writer, "{}", self.id.0)?;

        let tailport = self.attrs.get("tailport");
        if let Some(tailport) = tailport {
            write!(writer, ":{}", tailport)?;
            ports.push("tailport");
        }

        write!(writer, " -> {}", self.id.1)?;

        let headport = self.attrs.get("headport");
        if let Some(headport) = headport {
            write!(writer, ":{}", headport)?;
            ports.push("headport");
        };

        if self.attrs.len() > ports.len() {
            write!(writer, " [ ")?;
            for (key, value) in &self.attrs {
                if !ports.contains(&&key[..]) {
                    write!(writer, "{}=\"{}\" ", key, value)?;
                }
            }
            writeln!(writer, "]")?;
        }

        Ok(())
    }
}
