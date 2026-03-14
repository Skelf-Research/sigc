# IDE Setup

Set up VS Code for the best sigc development experience.

## VS Code Extension

The sigc VS Code extension provides:

- Syntax highlighting for `.sig` files
- 25+ code snippets for common patterns
- Compile, Run, and Explain commands
- Language Server Protocol (LSP) integration
- Real-time error diagnostics
- Hover documentation for operators
- Code completion

## Installation

### Step 1: Build the Extension

```bash
cd /path/to/sigc/editors/vscode

# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Package as VSIX
npx @vscode/vsce package
```

This creates `sigc-0.1.0.vsix`.

### Step 2: Install in VS Code

1. Open VS Code
2. Press `Cmd+Shift+P` (macOS) or `Ctrl+Shift+P` (Windows/Linux)
3. Type "Install from VSIX"
4. Select `sigc-0.1.0.vsix`
5. Reload VS Code

### Step 3: Verify Installation

1. Create a new file with `.sig` extension
2. You should see syntax highlighting
3. Type `signal` and press Tab - a snippet should expand

## Features

### Syntax Highlighting

The extension highlights:

- Keywords: `data`, `params`, `signal`, `portfolio`, `fn`, `macro`, `emit`, `let`
- Operators: `ret`, `zscore`, `rank`, `rolling_mean`, etc.
- Comments: `//` and `#` style
- Strings and numbers

<!-- Screenshot placeholder: Add vscode-screenshot.png to assets/images/ -->

### Code Snippets

Type a prefix and press Tab to expand:

| Prefix | Expands To |
|--------|------------|
| `strategy` | Complete strategy template |
| `signal` | Signal block |
| `data` | Data section |
| `params` | Parameters section |
| `portfolio` | Portfolio block |
| `fn` | Function definition |
| `macro` | Macro definition |
| `momentum` | Momentum signal template |
| `meanrev` | Mean reversion template |
| `rmean` | `rolling_mean(${1:x}, ${2:window})` |
| `rstd` | `rolling_std(${1:x}, ${2:window})` |
| `zs` | `zscore(${1:x})` |
| `win` | `winsor(${1:x}, p=${2:0.01})` |
| `ret` | `ret(${1:prices}, ${2:lookback})` |
| `lag` | `lag(${1:x}, ${2:n})` |
| `ema` | `ema(${1:x}, ${2:span})` |
| `rsi` | `rsi(${1:x}, ${2:period})` |
| `macd` | `macd(${1:x}, ${2:fast}, ${3:slow}, ${4:signal})` |
| `longshort` | `long_short(top=${1:0.2}, bottom=${2:0.2})` |
| `where` | `where(${1:cond}, ${2:true_val}, ${3:false_val})` |

### Commands

Access via Command Palette (`Cmd/Ctrl+Shift+P`):

| Command | Description |
|---------|-------------|
| `sigc: Compile` | Compile current file |
| `sigc: Run` | Run backtest |
| `sigc: Explain` | Show IR structure |

You can also right-click in the editor for context menu options.

### Language Server (LSP)

The LSP provides real-time feedback:

#### Error Diagnostics

Errors appear as you type with red underlines:

```sig
signal test:
  x = ret(prices, lookback)
  emit unknownfunction(x)  // Error: Unknown function 'unknownfunction'
```

#### Hover Documentation

Hover over any operator to see its documentation:

```
zscore(x)

Cross-sectional z-score normalization.
Subtracts mean and divides by standard deviation.

Input: Series (cross-sectional)
Output: Series with mean=0, std=1

Example:
  normalized = zscore(returns)
```

#### Code Completion

Press `Ctrl+Space` for suggestions:

- Built-in operators
- Variables in scope
- Parameters
- Signals
- Functions and macros

#### Go to Definition

`Cmd/Ctrl+Click` or `F12` on:

- Signal references → jumps to signal definition
- Function calls → jumps to function definition
- Macro invocations → jumps to macro definition

#### Document Outline

View all signals, functions, and macros in the Outline panel (`Cmd/Ctrl+Shift+O`).

## Configuration

### Extension Settings

Open VS Code settings (`Cmd+,`) and search for "sigc":

| Setting | Default | Description |
|---------|---------|-------------|
| `sigc.binaryPath` | `sigc` | Path to sigc binary |
| `sigc.autoCompile` | `true` | Compile on save |
| `sigc.showInlineHints` | `true` | Show type hints |

### Recommended VS Code Settings

Add to your `settings.json`:

```json
{
  "[sig]": {
    "editor.tabSize": 2,
    "editor.insertSpaces": true,
    "editor.formatOnSave": false,
    "editor.wordWrap": "on"
  },
  "sigc.binaryPath": "/path/to/sigc/target/release/sigc",
  "sigc.autoCompile": true
}
```

## Keyboard Shortcuts

Add custom keybindings in `keybindings.json`:

```json
[
  {
    "key": "cmd+shift+r",
    "command": "sigc.run",
    "when": "editorLangId == sig"
  },
  {
    "key": "cmd+shift+c",
    "command": "sigc.compile",
    "when": "editorLangId == sig"
  }
]
```

## Troubleshooting

### Extension Not Loading

1. Check VS Code's Output panel (`View > Output`) and select "sigc"
2. Ensure the `sigc` binary is in your PATH or configured in settings

### No Syntax Highlighting

1. Verify the file has `.sig` extension
2. Try reloading VS Code (`Cmd+Shift+P` → "Reload Window")
3. Check that the extension is enabled

### LSP Not Working

The LSP requires `sigc-lsp` binary:

```bash
# Build the LSP
cd /path/to/sigc
cargo build --release -p sig_lsp

# Verify
./target/release/sig-lsp --version
```

Configure the path in settings:

```json
{
  "sigc.lspPath": "/path/to/sigc/target/release/sig-lsp"
}
```

### Slow Diagnostics

For large files, try:

1. Increase the diagnostic delay in settings
2. Disable `autoCompile` and compile manually

## Alternative Editors

### Vim/Neovim

Basic syntax highlighting:

```vim
" ~/.vim/syntax/sig.vim
syntax match sigKeyword /\<\(data\|params\|signal\|portfolio\|fn\|macro\|emit\|let\)\>/
syntax match sigOperator /\<\(ret\|lag\|zscore\|rank\|rolling_mean\|rolling_std\)\>/
syntax match sigComment /\/\/.*/
syntax match sigComment /#.*/

highlight link sigKeyword Keyword
highlight link sigOperator Function
highlight link sigComment Comment
```

### Sublime Text

Create a `.sublime-syntax` file in your Packages/User directory.

### JetBrains IDEs

No official plugin yet. Use the TextMate bundle for basic highlighting.

## Next Steps

- [Sample Data](sample-data.md) - Work with included datasets
- [DSL Basics](../language/syntax.md) - Learn the language
- [Quickstart](quickstart.md) - Run your first backtest
