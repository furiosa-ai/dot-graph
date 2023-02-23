# dot-graph

dot parser in Rust implemented with FFI to Graphviz cgraph library.

## Prerequisites

`dot-graph` parses a dot format file using C bindings to [Graphviz (v7.0.6)](https://gitlab.com/graphviz/graphviz/-/tree/7.0.6/lib).

The system environment should be able to find and include the following header files.

```C
#include <gvc.h>
#include <cgraph.h>
```

#### Option 1. Installing Graphviz from Package Manager

Coming from Linux,
```console
$ sudo apt install graphviz-dev
```

And coming from vanilla Ubuntu, you may want to install these too.
```console
$ sudo apt install build-essentials cmake
$ sudo apt install clang
```

Coming from Mac,
```console
$ brew install graphviz
```

And coming from Apple Silicon Mac, and [add an environment variable](https://apple.stackexchange.com/questions/414622/installing-a-c-c-library-with-homebrew-on-m1-macs),
```shell
export CPATH=/opt/homebrew/include
```

#### Option 2. Building Graphviz from Source

Or, try building from the source code yourself following the [guide](https://graphviz.org/download/source/).


## Usage

```rust
use dot_graph::prelude::*;

fn main() -> Result<(), DotGraphError> {
  let graph = parser::parse(/* path */)?;
  let mut stdout = std::io::stdout();
  graph.to_dot(&mut stdout)?;
  
  Ok(())
}
```
