# sig_types

Core type definitions for the sigc quantitative finance DSL.

## Overview

`sig_types` provides the foundational types used throughout the sigc ecosystem:

- **AST types** - Abstract syntax tree nodes for the sigc DSL
- **IR types** - Intermediate representation for compiled strategies
- **Value types** - Runtime value representations (Numeric, Date, Symbol, etc.)
- **Error types** - Structured error definitions

## Usage

```rust
use sig_types::{Ast, Value, DataType};

// Work with sigc types
let value = Value::Numeric(42.0);
let dtype = DataType::Numeric;
```

## Part of sigc

This crate is part of the [sigc](https://github.com/skelf-Research/sigc) quantitative finance platform.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
