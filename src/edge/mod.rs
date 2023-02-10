use crate::graphs::graph::NodeIndex;
use crate::node::NodeId;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::io::{Result, Write};

pub type EdgeId = (NodeId, NodeId);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// An (directed) `Edge` of a graph.
pub struct Edge {
    /// Id of the start point node
    pub from: NodeId,
    /// Id of the end point node
    pub to: NodeId,
    /// Attributes of the edge in key, value mappings
    pub attrs: BTreeMap<String, String>,
}

pub type EdgeMap = HashMap<NodeIndex, HashSet<NodeIndex>>;

impl Edge {
    /// Write the edge to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let tabs = "\t".repeat(indent);
        let mut ports = Vec::with_capacity(2);

        write!(writer, "{}{}", tabs, self.from)?;

        let tailport = self.attrs.get("tailport");
        if let Some(tailport) = tailport {
            write!(writer, ":{}", tailport)?;
            ports.push("tailport");
        }

        write!(writer, " -> {}", self.to)?;

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
