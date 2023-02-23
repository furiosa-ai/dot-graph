use crate::{node::NodeId, utils};

use std::io::{Result, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeId {
    /// Start point's node id
    pub(crate) from: NodeId,
    /// Start point's port
    pub(crate) tailport: Option<String>,
    /// End point's node id
    pub(crate) to: NodeId,
    /// End point's port
    pub(crate) headport: Option<String>,
}

impl EdgeId {
    pub fn new(
        from: NodeId,
        tailport: Option<String>,
        to: NodeId,
        headport: Option<String>,
    ) -> EdgeId {
        EdgeId { from, tailport, to, headport }
    }

    pub fn from(&self) -> &NodeId {
        &self.from
    }

    pub fn tailport(&self) -> &Option<String> {
        &self.tailport
    }

    pub fn to(&self) -> &NodeId {
        &self.to
    }

    pub fn headport(&self) -> &Option<String> {
        &self.headport
    }

    /// Write the edge id to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        (0..indent).try_for_each(|_| write!(writer, "\t"))?;

        let from = utils::pretty_id(&self.from);
        write!(writer, "{from}")?;
        if let Some(tailport) = &self.tailport {
            write!(writer, ":{tailport}")?;
        }

        let to = utils::pretty_id(&self.to);
        write!(writer, " -> {to}")?;
        if let Some(headport) = &self.headport {
            write!(writer, ":{headport}")?;
        }

        Ok(())
    }
}
