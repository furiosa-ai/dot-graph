use std::collections::BTreeMap;
use std::fmt::Write;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub id: String,
    pub attrs: BTreeMap<String, String>,
}

impl Node {
    pub fn to_dot(&self, indent: usize) -> String {
        let mut dot = String::new();
        let tabs = "\t".repeat(indent);

        write!(dot, "{}{}[\n", tabs, self.id).unwrap();

        for (key, value) in &self.attrs {
            // TODO naive workaround to visualize HTML strings
            if value.contains("TABLE") {
                write!(dot, "{}{}=<{}>\n", tabs, key, value).unwrap();
            } else {
                write!(dot, "{}{}=\"{}\"\n", tabs, key, value).unwrap();
            }
        }

        write!(dot, "{}];", tabs).unwrap();

        dot
    }
}
