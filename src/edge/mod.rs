use crate::node::NodeId;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

pub type EdgeId = (NodeId, NodeId);

pub type EdgeMap = HashMap<NodeId, HashSet<NodeId>>;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An (directed) `Edge` of a graph.
pub struct Edge {
    /// A tuple of start and end points
    pub id: EdgeId,
    /// Attributes of the edge in key, value mappings
    pub attrs: HashMap<String, String>,
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
    /// Write the edge to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let tabs = "\t".repeat(indent);
        let mut ports = Vec::with_capacity(2);

        write!(writer, "{}{}", tabs, self.id.0)?;

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
                    write!(writer, "{key}={value} ")?;
                }
            }
            write!(writer, "]")?;
        }

        Ok(())
    }
}
