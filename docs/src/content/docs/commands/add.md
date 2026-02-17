---
title: add
description: Install skills from a repository or archive source.
---

```bash
skil add <source> [options]
```

## Options

- `-g, --global`: install for all agents (home-level) instead of project-local.
- `--copy`: copy files instead of symlinking.
- `-a, --agent <agent...>`: target one or more agents.
- `-s, --skill <skill...>`: install one or more specific skills.
- `-l, --list`: list available skills in the source without installing.
- `-y, --yes`: skip interactive prompts.
- `--all`: install all skills and target all agents.
- `--full-depth`: keep full directory depth while discovering skills.

## Examples

```bash
skil add vercel-labs/agent-skills
skil add vercel-labs/agent-skills --list
skil add vercel-labs/agent-skills --skill frontend-design
skil add owner/repo
skil add https://github.com/github/awesome-copilot --skill gh-cli
```
