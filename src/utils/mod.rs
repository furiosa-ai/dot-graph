pub(crate) fn pretty_id(id: &str) -> String {
    if id.chars().all(char::is_alphanumeric) {
        id.to_string()
    } else {
        format!("\"{id}\"")
    }
}
