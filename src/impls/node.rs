use std::fmt;
use crate::structs::Node;

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

impl fmt::Display for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("id : ")?;
        fmt.write_str(self.id.as_str())?;
        fmt.write_str("\n\n")?;

        for (key, value) in &self.attrs {
            fmt.write_str(key.as_str())?;
            fmt.write_str(" : ")?;
            let values = value.split("\\l");
            for (idx, value) in values.enumerate() {
                if value == "" {
                    break;
                }

                // TODO tabsation workaround (https://github.com/fdehau/tui-rs/issues/98)
                if idx > 0 {
                    let tabs = "\u{2800}";
                    fmt.write_str(format!("{}{}", tabs, tabs).as_str())?;
                }
                fmt.write_str(value)?;
                fmt.write_str("\n")?;
            } 
        }

        Ok(())
    }
}
