use std::collections::HashMap;

#[derive(Debug)]
pub struct Node {
    pub id: String,
    pub attrs: HashMap<String, String>,
}
