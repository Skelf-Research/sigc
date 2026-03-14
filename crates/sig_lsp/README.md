# sig_lsp

Language Server Protocol implementation for the sigc DSL.

## Overview

`sig_lsp` provides IDE support for sigc with:

- **Real-time diagnostics** - Syntax and type errors as you type
- **Hover documentation** - Operator signatures and descriptions
- **Code completion** - Operators, keywords, and snippets
- **Go to definition** - Navigate to signal, function, and macro definitions
- **Document symbols** - Outline view of strategy structure

## Installation

```bash
cargo install sig_lsp
```

The binary is named `sigc-lsp`.

## VS Code Integration

Install the sigc VS Code extension which automatically uses this language server:

```bash
code --install-extension skelf-Research.sigc-vscode
```

## Configuration

Configure in your editor's LSP settings:

```json
{
  "sigc.binaryPath": "/path/to/sigc-lsp"
}
```

## Part of sigc

This crate is part of the [sigc](https://github.com/skelf-Research/sigc) quantitative finance platform.

## License

MIT License - see [LICENSE](../../LICENSE) for details.
