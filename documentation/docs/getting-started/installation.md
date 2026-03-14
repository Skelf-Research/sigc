# Installation

This guide covers how to install sigc on your system.

## Prerequisites

### Rust Toolchain

sigc is written in Rust and requires the Rust toolchain to build from source.

=== "Linux / macOS"

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

    Follow the prompts and restart your terminal.

=== "Windows"

    Download and run [rustup-init.exe](https://rustup.rs/).

    !!! warning "Windows Support"
        sigc is primarily developed on Linux/macOS. Windows users should use WSL2 for the best experience.

Verify your installation:

```bash
rustc --version
# rustc 1.70.0 or later
```

## Installation Methods

### Build from Source (Recommended)

Building from source ensures you have the latest features and optimizations for your system.

```bash
# Clone the repository
git clone https://github.com/skelf-Research/sigc.git
cd sigc

# Build in release mode
cargo build --release

# The binary is at ./target/release/sigc
```

Verify the build:

```bash
./target/release/sigc --version
# sigc 0.10.0
```

### Add to PATH

To use `sigc` from anywhere, add it to your PATH:

=== "Linux / macOS (bash/zsh)"

    ```bash
    # Add to ~/.bashrc or ~/.zshrc
    export PATH="$PATH:/path/to/sigc/target/release"

    # Reload shell
    source ~/.bashrc  # or ~/.zshrc
    ```

=== "Fish"

    ```fish
    # Add to ~/.config/fish/config.fish
    set -gx PATH $PATH /path/to/sigc/target/release
    ```

### Pre-built Binaries

Pre-built binaries are available for common platforms on the [GitHub Releases](https://github.com/skelf-Research/sigc/releases) page.

```bash
# Download for Linux x86_64
curl -L https://github.com/skelf-Research/sigc/releases/latest/download/sigc-linux-x86_64.tar.gz | tar xz

# Download for macOS (Apple Silicon)
curl -L https://github.com/skelf-Research/sigc/releases/latest/download/sigc-darwin-arm64.tar.gz | tar xz
```

## Verify Installation

Run these commands to verify sigc is installed correctly:

```bash
# Check version
sigc --version

# Show help
sigc --help

# Compile a test file
echo 'data:
  prices: load csv from "test.csv"
signal test:
  emit prices
portfolio main:
  weights = test
  backtest from 2024-01-01 to 2024-12-31' > test.sig

sigc compile test.sig
```

Expected output:

```
INFO sigc: sigc v0.10.0
INFO sig_compiler: Parsing source
INFO sig_compiler: Parsed 1 data, 0 params, 1 signals, 1 portfolios
INFO sig_compiler: Lowered to 2 IR nodes
INFO sigc: Compilation complete: 2 nodes
```

## Optional: Python Integration

To use sigc from Python (Jupyter notebooks, scripts), install the `pysigc` package:

```bash
# From the sigc repository
cd crates/pysigc
pip install maturin
maturin develop --release
```

Verify Python installation:

```python
import pysigc
print(pysigc.__version__)
```

See [Python Integration](../integrations/python.md) for detailed usage.

## Optional: VS Code Extension

For the best development experience, install the VS Code extension:

```bash
cd editors/vscode
npm install
npm run compile
npx @vscode/vsce package
```

Then install the generated `sigc-0.1.0.vsix` in VS Code.

See [IDE Setup](ide-setup.md) for detailed instructions.

## Troubleshooting

### Build Fails with "linker not found"

Install build essentials:

=== "Ubuntu/Debian"

    ```bash
    sudo apt install build-essential
    ```

=== "Fedora"

    ```bash
    sudo dnf install gcc
    ```

=== "macOS"

    ```bash
    xcode-select --install
    ```

### Build Fails with OpenSSL Errors

Install OpenSSL development headers:

=== "Ubuntu/Debian"

    ```bash
    sudo apt install libssl-dev pkg-config
    ```

=== "Fedora"

    ```bash
    sudo dnf install openssl-devel
    ```

=== "macOS"

    ```bash
    brew install openssl
    export OPENSSL_DIR=$(brew --prefix openssl)
    ```

### "Command not found: sigc"

Ensure the binary is in your PATH:

```bash
# Check if sigc is in PATH
which sigc

# If not found, add it
export PATH="$PATH:/path/to/sigc/target/release"
```

### Slow Build Times

Enable incremental compilation and use `sccache`:

```bash
# Install sccache
cargo install sccache

# Set as compiler wrapper
export RUSTC_WRAPPER=sccache

# Rebuild
cargo build --release
```

## Next Steps

Now that sigc is installed, continue to:

- [5-Minute Quickstart](quickstart.md) - Run your first backtest
- [Your First Strategy](first-strategy.md) - Build a complete strategy
- [IDE Setup](ide-setup.md) - Configure VS Code
