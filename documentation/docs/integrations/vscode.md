# VSCode Extension

Full IDE support for sigc development.

## Installation

### From Marketplace

1. Open VSCode
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "sigc"
4. Click Install

### From Command Line

```bash
code --install-extension skelf-Research.sigc-vscode
```

## Features

### Syntax Highlighting

Full syntax highlighting for `.sig` files:

- Keywords (`data`, `signal`, `portfolio`, `emit`)
- Operators (`zscore`, `rank`, `rolling_mean`)
- Numbers, strings, comments
- Type annotations

### Code Completion

IntelliSense for:

- **Operators**: All 120+ operators with signatures
- **Keywords**: Language keywords
- **Parameters**: Function parameters
- **Variables**: Defined variables in scope

Press `Ctrl+Space` to trigger completion.

### Hover Documentation

Hover over any operator to see:

- Function signature
- Description
- Parameters
- Return type
- Example usage

### Error Diagnostics

Real-time error checking:

- Syntax errors
- Type mismatches
- Undefined variables
- Invalid operator usage

Errors appear with red underlines and in the Problems panel.

### Go to Definition

`Ctrl+Click` or `F12` to jump to:

- Signal definitions
- Variable declarations
- Function definitions

### Find References

`Shift+F12` to find all usages of:

- Signals
- Variables
- Functions

### Format Document

`Shift+Alt+F` to format your strategy:

- Consistent indentation
- Proper spacing
- Aligned operators

### Code Folding

Collapse sections:

- `data:` blocks
- `signal` blocks
- `portfolio` blocks

## Configuration

### Extension Settings

Open Settings (`Ctrl+,`) and search for "sigc":

```json
{
  // Path to sigc binary
  "sigc.binaryPath": "/usr/local/bin/sigc",

  // Enable real-time diagnostics
  "sigc.enableDiagnostics": true,

  // Diagnostic delay (ms)
  "sigc.diagnosticDelay": 500,

  // Format on save
  "sigc.formatOnSave": true,

  // Show inline hints
  "sigc.inlayHints.enabled": true,

  // LSP log level
  "sigc.logLevel": "info"
}
```

### Workspace Settings

Create `.vscode/settings.json`:

```json
{
  "sigc.binaryPath": "./target/release/sigc",
  "sigc.formatOnSave": true,
  "[sig]": {
    "editor.tabSize": 2,
    "editor.insertSpaces": true
  }
}
```

## Snippets

Built-in snippets for common patterns:

### `data` - Data Section

```sig
data:
  source = "$1"
  format = ${2|csv,parquet|}
  columns:
    date: Date
    ticker: Symbol
    $3: Numeric as $4
```

### `signal` - Signal Block

```sig
signal ${1:name}:
  $2
  emit $3
```

### `portfolio` - Portfolio Block

```sig
portfolio ${1:name}:
  weights = rank(${2:signal}).long_short(top=${3:0.2}, bottom=${4:0.2})
  backtest from ${5:2020-01-01} to ${6:2024-12-31}
```

### `momentum` - Momentum Signal

```sig
signal momentum:
  mom = zscore(ret(prices, ${1:60}))
  emit mom
```

### `longshort` - Long-Short Portfolio

```sig
portfolio long_short:
  weights = rank(${1:signal}).long_short(top=${2:0.2}, bottom=${3:0.2}, cap=${4:0.05})
  backtest rebal=${5:21} from ${6:2020-01-01} to ${7:2024-12-31}
```

Type the snippet prefix and press `Tab` to expand.

## Commands

Access via Command Palette (`Ctrl+Shift+P`):

| Command | Description |
|---------|-------------|
| `sigc: Run Strategy` | Run current file |
| `sigc: Run with Parameters` | Run with param overrides |
| `sigc: Validate` | Check for errors |
| `sigc: Format Document` | Format current file |
| `sigc: Show Metrics` | Display performance metrics |
| `sigc: Export Results` | Export to CSV/JSON |
| `sigc: Open Documentation` | Open docs in browser |

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+R` | Run strategy |
| `Ctrl+Shift+V` | Validate |
| `Ctrl+Shift+D` | Open diagnostics |
| `F5` | Run with debugger |

### Customize Shortcuts

Add to `keybindings.json`:

```json
[
  {
    "key": "ctrl+shift+r",
    "command": "sigc.run",
    "when": "editorLangId == sig"
  }
]
```

## Output Panel

Results appear in the Output panel:

1. Open Output panel (`Ctrl+Shift+U`)
2. Select "sigc" from dropdown

Shows:

- Backtest results
- Performance metrics
- Errors and warnings
- Debug output

## Problems Panel

Errors and warnings appear in Problems panel:

- Click error to jump to line
- Hover for details
- Quick fix suggestions

## Debugging

### Basic Debugging

1. Set breakpoints (click line number)
2. Press `F5` to start debugging
3. Step through execution

### Debug Configuration

Create `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "sigc",
      "request": "launch",
      "name": "Debug Strategy",
      "program": "${file}",
      "args": ["--verbose"]
    }
  ]
}
```

### Watch Variables

Add variables to watch panel:

- Signal values
- Weight calculations
- Intermediate results

## Tasks

Define tasks in `.vscode/tasks.json`:

```json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Run Backtest",
      "type": "shell",
      "command": "sigc run ${file}",
      "group": {
        "kind": "build",
        "isDefault": true
      }
    },
    {
      "label": "Validate All",
      "type": "shell",
      "command": "sigc validate strategies/*.sig"
    }
  ]
}
```

Run tasks: `Ctrl+Shift+B`

## Multi-Root Workspaces

Support for multiple strategy folders:

```json
// workspace.code-workspace
{
  "folders": [
    { "path": "production" },
    { "path": "research" },
    { "path": "backtest" }
  ],
  "settings": {
    "sigc.binaryPath": "/usr/local/bin/sigc"
  }
}
```

## File Associations

Ensure `.sig` files are recognized:

```json
{
  "files.associations": {
    "*.sig": "sig"
  }
}
```

## Recommended Extensions

Enhance your sigc development:

| Extension | Purpose |
|-----------|---------|
| [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) | Inline error display |
| [GitLens](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens) | Git history |
| [Jupyter](https://marketplace.visualstudio.com/items?itemName=ms-toolsai.jupyter) | Notebook support |

## Troubleshooting

### Extension Not Working

1. Check sigc is installed: `sigc --version`
2. Verify path in settings
3. Restart VSCode

### No Syntax Highlighting

1. Check file extension is `.sig`
2. Reload window: `Ctrl+Shift+P` → "Reload Window"

### LSP Errors

1. Check Output panel for errors
2. Increase log level:
   ```json
   { "sigc.logLevel": "debug" }
   ```
3. Restart LSP: Command Palette → "sigc: Restart Language Server"

### Slow Diagnostics

1. Increase delay:
   ```json
   { "sigc.diagnosticDelay": 1000 }
   ```
2. Disable for large files:
   ```json
   { "sigc.diagnostics.maxFileSize": 100000 }
   ```

## Updating

### Auto Update

Extensions update automatically by default.

### Manual Update

1. Open Extensions view
2. Click update button on sigc extension

### Check Version

```bash
code --list-extensions --show-versions | grep sigc
```

## Contributing

Report issues or contribute:

- [GitHub Issues](https://github.com/skelf-Research/sigc-vscode/issues)
- [Extension Source](https://github.com/skelf-Research/sigc-vscode)

## Next Steps

- [Quickstart](../getting-started/quickstart.md) - Start using sigc
- [Syntax Reference](../language/syntax.md) - Language details
- [Python](python.md) - Python integration
