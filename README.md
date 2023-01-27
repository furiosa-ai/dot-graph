# dot-graph
Dot parser in Rust implemented with FFI to Graphviz cgraph library.

## Prerequisites

It is required that [Graphviz is installed (compiled)](https://graphviz.org/download/source/) beforehand such that the followings can be included.
```C
#include <graphviz/gvc.h>
#include <graphviz/cgraph.h>
```

## Usage

```rust
use dot_graph::parser::parse;

fn main() {
  let graph = parse(path);
  println!("{}", graph.to_dot());
}
```
