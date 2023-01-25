use crate::structs::Edge;

impl Edge {
    pub fn to_dot(&self, indent: usize) -> String {
        let tabs = "\t".repeat(indent);
        let mut dot = String::from("");

        match self.attrs.get("headport") {
            Some(headport) => if !headport.is_empty() {
                dot.push_str(&format!("{}{} -> {}:{}", tabs, self.from, self.to, headport))
            } else {
                dot.push_str(&format!("{}{} -> {}", tabs, self.from, self.to))
            },
            None => dot.push_str(&format!("{}{} -> {}", tabs, self.from, self.to)),
        };

        dot
    }
}
