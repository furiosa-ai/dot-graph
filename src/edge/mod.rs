use std::fmt::Write;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: String,
    pub to: String,
    pub attrs: BTreeMap<String, String>,
}

pub type EdgeMap = HashMap<usize, HashSet<usize>>;

impl Edge {
    pub fn to_dot(&self, indent: usize) -> String {
        let mut dot = String::new();
        let mut ports = Vec::with_capacity(2);

        (0..indent).for_each(|_| dot.push('\t'));

        dot.push_str(&self.from);

        let tailport = self.attrs.get("tailport");
        if let Some(tailport) = tailport {
            write!(dot, ":{}", tailport).unwrap();
            ports.push("tailport");
        }

        dot.push_str(" -> ");

        dot.push_str(&self.to);

        let headport = self.attrs.get("headport");
        if let Some(headport) = headport {
            write!(dot, ":{}", headport).unwrap();
            ports.push("headport");
        };

        if self.attrs.len() > ports.len() {
            dot.push_str(" [ ");
            for (key, value) in &self.attrs {
                if !ports.contains(&&key[..]) {
                    write!(dot, "{key}={value} ").unwrap();
                }
            }
            dot.push(']');
        }

        dot
    }
}
