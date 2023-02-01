# dot-graph
Dot parser in Rust implemented with FFI to Graphviz cgraph library.

## Prerequisites

dot-graph parses a dot format file using C bindings to Graphviz (v.7.0.6).

#### Option 1. Installing Graphviz from Package Manager

Coming from Linux,
```console
$ sudo apt install graphviz-dev
```

Coming from Mac,
```console
$ brew install graphviz
```

And coming from Apple Silicon Mac,
```console
$ brew install graphviz
```

and [add an environment variable](https://apple.stackexchange.com/questions/414622/installing-a-c-c-library-with-homebrew-on-m1-macs),
```shell
export CPATH=/opt/homebrew/include
```

#### Option 2. Building Graphviz from Source

It is required that [Graphviz is installed (compiled)](https://graphviz.org/download/source/) beforehand such that the followings can be included.
```C
#include <graphviz/gvc.h>
#include <graphviz/cgraph.h>
```

## Usage

```rust
use dot_graph::parser::parse;

fn main() {
  let graph = parse(/* path */);
  println!("{}", graph.to_dot());
}
```
