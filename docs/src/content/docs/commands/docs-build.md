---
title: docs build
description: Build a static website from discovered repository skills.
---

```bash
skil docs build [options]
```

## Options

- `--source <path>`: source directory to scan for skills (default `.`).
- `--output <path>`: output directory for generated site (default `site`).
- `--full-depth`: keep full directory depth while discovering skills.

## Example

```bash
skil docs build --source . --output site
```
