# Development Setup

Set up your development environment for contributing to sigc.

## Prerequisites

### Required

- **Rust** (latest stable): [Install Rust](https://rustup.rs/)
- **Git**: Version control
- **Python 3.8+**: For pysigc development

### Recommended

- **VSCode** with rust-analyzer
- **Docker**: For testing in containers

## Clone the Repository

```bash
git clone https://github.com/skelf-Research/sigc.git
cd sigc
```

## Build

### Debug Build

```bash
cargo build
```

### Release Build

```bash
cargo build --release
```

### Build All Crates

```bash
cargo build --workspace
```

## Run Tests

### All Tests

```bash
cargo test --workspace
```

### Specific Crate

```bash
cargo test -p sig_parser
cargo test -p sig_runtime
```

### Integration Tests

```bash
cargo test --test integration
```

## Code Quality

### Linting

```bash
cargo clippy --workspace
```

### Formatting

```bash
# Check
cargo fmt --check

# Auto-format
cargo fmt
```

### Type Checking

```bash
cargo check --workspace
```

## Project Structure

```
sigc/
├── crates/
│   ├── sigc/          # Main CLI
│   ├── sig_parser/    # DSL parser
│   ├── sig_runtime/   # Execution engine
│   └── sig_types/     # Type definitions
├── pysigc/            # Python bindings
├── docs/              # Legacy documentation
├── documentation/     # New MkDocs documentation
├── strategies/        # Example strategies
└── tests/             # Integration tests
```

## Development Workflow

### 1. Create Branch

```bash
git checkout -b feature/my-feature
```

### 2. Make Changes

Edit code, add tests.

### 3. Test Locally

```bash
cargo test
cargo clippy
cargo fmt
```

### 4. Commit

```bash
git add .
git commit -m "feat: add new feature"
```

### 5. Push

```bash
git push -u origin feature/my-feature
```

### 6. Open PR

Go to GitHub and create a Pull Request.

## Python Development

### Set Up Python Environment

```bash
cd pysigc

# Create virtual environment
python -m venv venv
source venv/bin/activate

# Install development dependencies
pip install -e ".[dev]"
```

### Build Python Bindings

```bash
maturin develop
```

### Test Python

```bash
pytest
```

## Documentation Development

### Set Up MkDocs

```bash
cd documentation
pip install -r requirements.txt
```

### Preview Locally

```bash
mkdocs serve
# Open http://localhost:8000
```

### Build

```bash
mkdocs build
```

## IDE Setup

### VSCode

Install extensions:
- rust-analyzer
- Even Better TOML
- CodeLLDB (for debugging)

Settings (`.vscode/settings.json`):
```json
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true
}
```

### IntelliJ/CLion

Install the Rust plugin.

## Debugging

### Debug Build

```bash
cargo build
./target/debug/sigc run strategy.sig
```

### With LLDB

```bash
lldb ./target/debug/sigc -- run strategy.sig
```

### Verbose Logging

```bash
RUST_LOG=debug cargo run -- run strategy.sig
```

## Common Issues

### Build Failures

```bash
# Update dependencies
cargo update

# Clean build
cargo clean && cargo build
```

### Test Failures

```bash
# Run specific test with output
cargo test test_name -- --nocapture
```

## See Also

- [Code Style](code-style.md)
- [Testing](testing.md)
