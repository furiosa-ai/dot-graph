# dot-graph
Dot parser in Rust implemented with FFI to Graphviz cgraph library.

## Prerequisites

dot-viewer parses a dot format file using C bindings to Graphviz.

#### 1. Installing Graphviz

Coming from Linux,
```console
$ sudo apt install graphviz-dev
```

Coming from Mac,
```console
$ brew install graphviz
```

#### 2. Graphviz Library

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
