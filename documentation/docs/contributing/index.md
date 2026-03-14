# Contributing

Help improve sigc! We welcome contributions.

## Ways to Contribute

| Contribution | Description |
|--------------|-------------|
| [Bug Reports](#reporting-bugs) | Report issues you find |
| [Feature Requests](#feature-requests) | Suggest improvements |
| [Code](#code-contributions) | Submit pull requests |
| [Documentation](#documentation) | Improve docs |
| [Strategies](#strategy-examples) | Share example strategies |

## Getting Started

### 1. Fork the Repository

```bash
# Fork on GitHub, then clone
git clone https://github.com/YOUR_USERNAME/sigc.git
cd sigc
```

### 2. Set Up Development Environment

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build
cargo build

# Run tests
cargo test
```

See [Development Setup](development-setup.md) for details.

### 3. Create a Branch

```bash
git checkout -b feature/your-feature-name
```

### 4. Make Changes

Follow our [Code Style](code-style.md) guide.

### 5. Test

```bash
cargo test
cargo clippy
cargo fmt --check
```

See [Testing](testing.md) for details.

### 6. Submit PR

Push and open a pull request.

## Reporting Bugs

### Before Reporting

1. Check existing issues
2. Try latest version
3. Reproduce the bug

### Bug Report Template

```markdown
## Description
Brief description of the bug.

## Steps to Reproduce
1. Step one
2. Step two
3. Step three

## Expected Behavior
What should happen.

## Actual Behavior
What actually happens.

## Environment
- sigc version:
- OS:
- Rust version:

## Additional Context
Any other information.
```

## Feature Requests

### Request Template

```markdown
## Problem
What problem does this solve?

## Proposed Solution
How would this feature work?

## Alternatives Considered
Other approaches you considered.

## Additional Context
Examples, mockups, etc.
```

## Code Contributions

### What We're Looking For

- Bug fixes
- Performance improvements
- New operators
- Documentation improvements
- Test coverage

### PR Requirements

- [ ] Tests pass
- [ ] Code is formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Commit messages are clear

### Commit Messages

```
type(scope): description

Types: feat, fix, docs, style, refactor, test, chore
```

Examples:
```
feat(operators): add bollinger band operator
fix(parser): handle empty signal blocks
docs(tutorial): add momentum strategy example
```

## Documentation

### Improving Docs

1. Edit files in `documentation/docs/`
2. Preview with `mkdocs serve`
3. Submit PR

### Writing Guide

- Use clear, simple language
- Include code examples
- Add diagrams where helpful
- Link to related topics

See [Documentation Guide](documentation.md).

## Strategy Examples

Share your strategies:

1. Add to `strategies/` directory
2. Include comments explaining the approach
3. Add documentation in `docs/strategies/`

## Code of Conduct

We follow the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

### Summary

- Be welcoming and inclusive
- Be respectful and constructive
- Be collaborative

## Getting Help

- [GitHub Discussions](https://github.com/skelf-Research/sigc/discussions)
- [Issues](https://github.com/skelf-Research/sigc/issues)

## Documentation Index

- [Development Setup](development-setup.md) - Setting up your environment
- [Code Style](code-style.md) - Coding standards
- [Testing](testing.md) - Running tests
- [Documentation](documentation.md) - Writing docs

## Recognition

Contributors are recognized in:

- Release notes
- Contributors file
- Documentation credits

Thank you for contributing!
