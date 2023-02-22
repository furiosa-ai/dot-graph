use crate::{attr::Attr, node::NodeId, utils};

use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type EdgeId = (NodeId, NodeId);

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

        let from = utils::pretty_id(&self.id.0);
        write!(writer, "{from}")?;

        let tailport = self.attrs.get("tailport");
        if let Some(tailport) = tailport {
            let tailport = &tailport.value;
            write!(writer, ":{tailport}")?;
            ports.push("tailport");
        }

        let to = utils::pretty_id(&self.id.1);
        write!(writer, " -> {to}")?;

        let headport = self.attrs.get("headport");
        if let Some(headport) = headport {
            let headport = &headport.value;
            write!(writer, ":{headport}")?;
            ports.push("headport");
        };

        if self.attrs.len() > ports.len() {
            writeln!(writer, " [")?;
            for attr in &self.attrs {
                let key = &attr.key;
                if !ports.contains(&&key[..]) {
                    attr.to_dot(indent + 1, writer)?;
                }
            }
            writeln!(writer, "]")?;
        }

        Ok(())
    }
}
