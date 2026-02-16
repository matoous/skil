---
title: install
description: Install tracked skills from .skil.toml using pinned checksums or versions.
---

```bash
skil install [options]
```

## Options

- `-g, --global`: install from the global config.
- `--copy`: copy files instead of symlinking.
- `-a, --agent <agent...>`: target one or more agents.
- `-y, --yes`: skip interactive prompts.
- `--full-depth`: keep full directory depth while discovering skills.

## Example

```bash
skil install
```
