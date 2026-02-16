---
title: remove
description: Remove installed skills.
---

```bash
skil remove [skills...] [options]
```

## Options

- `-g, --global`: remove from global install location.
- `-a, --agent <agent...>`: target one or more agents.
- `-s, --skill <skill...>`: remove one or more named skills.
- `-y, --yes`: skip interactive prompts.
- `--all`: remove all skills across selected agents.

## Examples

```bash
skil remove gh-cli
skil remove --agent codex --skill gh-cli
skil remove --all --yes
```
