# Documentation Contribution Guide

How to contribute to sigc documentation.

## Documentation Structure

```
documentation/
├── docs/
│   ├── index.md              # Landing page
│   ├── getting-started/      # Getting started guides
│   ├── concepts/             # Core concepts
│   ├── language/             # DSL reference
│   ├── operators/            # Operator reference
│   ├── tutorials/            # Step-by-step guides
│   └── ...
├── mkdocs.yml                # Configuration
└── requirements.txt          # Dependencies
```

## Local Development

### Setup

```bash
cd documentation
pip install -r requirements.txt
```

### Preview

```bash
mkdocs serve
```

Open http://localhost:8000

### Build

```bash
mkdocs build
```

## Writing Documentation

### Page Structure

````markdown
# Page Title

Brief introduction paragraph.

## Overview

What this page covers.

## Main Content

### Subsection

Details...

## Examples

```sig
// Code examples
```

## See Also

- [Related Page](other-page.md)
````

### Code Examples

Always use fenced code blocks with language:

````markdown
```sig
signal momentum:
  emit zscore(ret(prices, 60))
```

```python
import pysigc
results = pysigc.run("momentum.sig")
```

```bash
sigc run momentum.sig
```
````

### Admonitions

Use admonitions for callouts:

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

### Tables

```markdown
| Column 1 | Column 2 |
|----------|----------|
| Value 1  | Value 2  |
```

### Links

````markdown
[Internal Link](other-page.md)
[Section Link](page.md#section)
[External Link](https://example.com)
````

## Style Guidelines

### Tone

- Clear and concise
- Direct address ("You can...")
- Present tense
- Active voice

### Headings

- Title Case for H1
- Sentence case for H2+
- Don't skip levels

### Code

- Use realistic examples
- Include expected output
- Explain non-obvious code
- Keep examples focused

### Lists

- Use bullets for unordered items
- Use numbers for sequential steps
- Keep items parallel in structure

## Types of Documentation

### Tutorials

Step-by-step guides for beginners:
- Clear learning objective
- Sequential steps
- Complete examples
- Expected results

### How-To Guides

Solve specific problems:
- Start with the goal
- Focused steps
- Assume some knowledge

### Reference

Complete technical details:
- Comprehensive
- Organized for lookup
- Consistent format

### Explanations

Background and concepts:
- Discuss "why"
- Provide context
- Connect concepts

## Common Improvements

### Missing Documentation

- Undocumented features
- Missing examples
- Incomplete sections

### Fixes

- Typos and grammar
- Broken links
- Outdated information
- Unclear explanations

### Enhancements

- Better examples
- More context
- Visual diagrams
- Performance tips

## Submitting Changes

### 1. Fork and Clone

```bash
git clone https://github.com/YOUR_USERNAME/sigc.git
```

### 2. Create Branch

```bash
git checkout -b docs/improve-momentum-tutorial
```

### 3. Make Changes

Edit files in `documentation/docs/`.

### 4. Preview

```bash
cd documentation
mkdocs serve
```

### 5. Commit

```bash
git add .
git commit -m "docs: improve momentum tutorial examples"
```

### 6. Submit PR

Push and create a Pull Request.

## Review Checklist

Before submitting:

- [ ] Builds without errors (`mkdocs build`)
- [ ] Links work
- [ ] Code examples are correct
- [ ] Spelling/grammar checked
- [ ] Follows style guide
- [ ] Table of contents makes sense

## Getting Help

- Ask questions in GitHub Discussions
- Tag documentation maintainers in PRs
- Check existing docs for style examples

## See Also

- [MkDocs Documentation](https://www.mkdocs.org/)
- [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/)
