use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::io::{Result, Write};

#[derive(Debug, Clone, Eq)]
/// An attribute of a graph, node, or edge.
pub struct Attr {
    /// Key of an attribute
    pub(crate) key: String,
    /// Value of an attribute
    pub(crate) value: String,
    /// Whether the value is a html-like string
    pub(crate) is_html: bool,
}

impl PartialEq for Attr {
    fn eq(&self, other: &Attr) -> bool {
        self.key == other.key
    }
}

impl Hash for Attr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.key.hash(state);
    }
}

impl Borrow<String> for Attr {
    fn borrow(&self) -> &String {
        &self.key
    }
}

impl Borrow<str> for Attr {
    fn borrow(&self) -> &str {
        &self.key
    }
}

impl Attr {
    pub(crate) fn new(key: String, value: String, is_html: bool) -> Attr {
        Attr { key, value, is_html }
    }

    pub fn key(&self) -> &String {
        &self.key
    }

    pub fn value(&self) -> &String {
        &self.value
    }

    pub fn is_html(&self) -> bool {
        self.is_html
    }

    /// Write the attribute to dot format
    pub fn to_dot<W: ?Sized>(&self, indent: usize, writer: &mut W) -> Result<()>
    where
        W: Write,
    {
        let key = &self.key;
        let value = &self.value;

        (0..=indent).try_for_each(|_| write!(writer, "\t"))?;
        if self.is_html {
            writeln!(writer, "{key}=<{value}>")?;
        } else {
            writeln!(writer, "{key}=\"{value}\"")?;
        }

        Ok(())
    }
}
