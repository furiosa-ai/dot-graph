use crate::structs::Edge;

impl Edge {
    pub fn to_dot(&self, indent: usize) -> String {
        let indent = "\t".repeat(indent);
        let mut dot = String::from("");

        dot.push_str(&format!("{}{} -> {}", indent, self.from, self.to));

        dot
    }
}
