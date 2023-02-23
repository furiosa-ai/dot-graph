use crate::{node::NodeId, utils};

use std::io::{Result, Write};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EdgeId {
    pub(crate) from: NodeId,
    pub(crate) tailport: Option<String>,

    pub(crate) to: NodeId,
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

    /// Start point's node id
    pub fn from(&self) -> &NodeId {
        &self.from
    }

    /// Start point's port
    pub fn tailport(&self) -> &Option<String> {
        &self.tailport
    }

    /// End point's node id
    pub fn to(&self) -> &NodeId {
        &self.to
    }

    /// End point's port
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
