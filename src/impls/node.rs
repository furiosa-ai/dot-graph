use std::fmt;
use crate::structs::Node;

impl fmt::Display for Node {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str("id = ")?;
        fmt.write_str(self.id.as_str())?;
        fmt.write_str("\n")?;

        for (key, value) in &self.attrs {
            fmt.write_str(key.as_str())?;
            fmt.write_str(" : ")?;
            fmt.write_str(value.as_str())?;
            fmt.write_str("\n")?;
        }

        Ok(())
    }
}
