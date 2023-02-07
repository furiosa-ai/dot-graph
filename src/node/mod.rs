use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node {
    pub id: String,
    pub attrs: BTreeMap<String, String>,
}

impl Node {
    pub fn to_dot(&self, indent: usize) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        dot.push_str(&format!("{}{}[\n", tabs, self.id));
        for (key, value) in &self.attrs {
            // TODO naive workaround to visualize HTML strings
            if value.contains("TABLE") {
                dot.push_str(&format!("{}{}=<{}>\n", tabs, key, value));
            } else {
                dot.push_str(&format!("{}{}=\"{}\"\n", tabs, key, value));
            }
        }
        dot.push_str(&format!("{}];", tabs));

        dot
    }
}
