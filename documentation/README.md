# sigc Documentation

This directory contains the user-facing documentation for sigc, built with [MkDocs](https://www.mkdocs.org/) and the [Material theme](https://squidfunk.github.io/mkdocs-material/).

## Quick Start

### Prerequisites

- Python 3.9+
- pip

### Installation

```bash
# Install dependencies
pip install -r requirements.txt
```

### Development

```bash
# Serve locally with hot reload
mkdocs serve

# Build static site
mkdocs build

# Build with strict mode (fails on warnings)
mkdocs build --strict
```

The development server will be available at `http://localhost:8000`.

## Structure

```
documentation/
├── docs/                    # Markdown source files
│   ├── index.md            # Landing page
│   ├── getting-started/    # Installation & quickstart
│   ├── concepts/           # Core concepts
│   ├── language/           # DSL language reference
│   ├── operators/          # Operator documentation
│   ├── data/               # Data loading guides
│   ├── backtesting/        # Backtesting documentation
│   ├── production/         # Production deployment
│   ├── integrations/       # External integrations
│   ├── advanced/           # Advanced topics
│   ├── tutorials/          # Step-by-step tutorials
│   ├── quant-guide/        # Comprehensive quant education
│   ├── strategies/         # Strategy library (23 strategies)
│   ├── reference/          # CLI & configuration reference
│   ├── api/                # Rust & Python API docs
│   └── contributing/       # Contribution guidelines
├── mkdocs.yml              # MkDocs configuration
└── requirements.txt        # Python dependencies
```

## Deployment

Documentation is automatically deployed to https://docs.skelfresearch.com/sigc when changes are merged to the main branch.

### Manual Deployment

```bash
# Deploy to GitHub Pages
mkdocs gh-deploy
```

## Writing Documentation

### Style Guidelines

- Use clear, concise language
- Include code examples for all features
- Use admonitions for notes, warnings, and tips
- Keep line length under 100 characters
- Use relative links for internal navigation

### Admonitions

```markdown
!!! note
    This is a note.

!!! warning
    This is a warning.

!!! tip
    This is a tip.

!!! example
    This is an example.
```

### Code Blocks

```markdown
```sig
signal momentum:
  emit zscore(ret(prices, 20))
```

```bash
sigc run strategy.sig
```

```rust
let compiler = Compiler::new();
```
```

### Tabs

```markdown
=== "sigc DSL"

    ```sig
    signal example:
      emit zscore(ret(prices, 20))
    ```

=== "Python"

    ```python
    result = pysigc.backtest(source)
    ```
```

## License

Documentation is licensed under CC BY 4.0.
