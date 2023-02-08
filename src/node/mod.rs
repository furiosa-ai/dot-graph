use std::collections::BTreeMap;
use std::io::{Result, Write};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub id: String,
    pub attrs: BTreeMap<String, String>,
}

impl Node {
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
