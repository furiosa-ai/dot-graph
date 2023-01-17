use std::fmt;
use crate::structs::Node;

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

                // TODO indentation workaround (https://github.com/fdehau/tui-rs/issues/98)
                if idx > 0 {
                    let indent = "\u{2800}";
                    fmt.write_str(format!("{}{}", indent, indent).as_str())?;
                }
                fmt.write_str(value)?;
                fmt.write_str("\n")?;
            } 
        }

        Ok(())
    }
}
