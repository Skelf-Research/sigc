# sigc VS Code Extension

Language support for the sigc quantitative research DSL.

## Features

- **Syntax Highlighting**: Full syntax highlighting for .sig files
- **Snippets**: Quick templates for common patterns
- **Commands**: Compile, run, and explain directly from VS Code
- **LSP Support**: Connect to sigc language server for advanced features

## Installation

### From VSIX (Local)

1. Build the extension:
   ```bash
   cd editors/vscode
   npm install
   npm run compile
   npx vsce package
   ```

2. Install in VS Code:
   - Open VS Code
   - Press `Ctrl+Shift+P`
   - Run "Extensions: Install from VSIX..."
   - Select the generated `.vsix` file

### From Source (Development)

1. Open the `editors/vscode` folder in VS Code
2. Press `F5` to launch Extension Development Host

## Configuration

### Language Server

To enable advanced features (hover, completion, go-to-definition), configure the language server path:

```json
{
  "sigc.server.path": "/path/to/sigc-lsp"
}
```

## Snippets

| Prefix | Description |
|--------|-------------|
| `strategy` | Complete strategy template |
| `signal` | Signal block |
| `data` | Data section |
| `params` | Parameters section |
| `portfolio` | Portfolio block |
| `fn` | User-defined function |
| `momentum` | Momentum signal template |
| `meanrev` | Mean reversion signal template |
| `volatility` | Volatility signal template |

### Function Snippets

| Prefix | Description |
|--------|-------------|
| `rmean` | Rolling mean |
| `rstd` | Rolling std |
| `zs` | Z-score |
| `win` | Winsorize |
| `ret` | Return calculation |
| `lag` | Lag function |
| `ema` | Exponential moving average |
| `rsi` | RSI indicator |
| `macd` | MACD indicator |
| `clip` | Clip values |
| `fillna` | Fill NaN |
| `where` | Conditional |
| `longshort` | Long-short weights |

## Commands

| Command | Description |
|---------|-------------|
| `sigc: Compile Current File` | Compile the current .sig file |
| `sigc: Run Backtest` | Run backtest on current file |
| `sigc: Explain IR` | Show IR explanation |

## Syntax Highlighting

The extension provides highlighting for:

- **Keywords**: `data`, `params`, `signal`, `portfolio`, `fn`, `emit`
- **Functions**: All built-in operators (time-series, cross-sectional, technical)
- **Strings**: Double and single quoted strings
- **Numbers**: Integers, floats, and dates
- **Comments**: `//`, `#`, and `/* */`
- **Operators**: Arithmetic, comparison, and logical operators

## Development

### Building

```bash
npm install
npm run compile
```

### Testing

```bash
npm run lint
```

### Packaging

```bash
npx vsce package
```

## License

MIT
