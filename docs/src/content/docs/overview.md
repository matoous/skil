---
title: Overview
description: What skil is, how it works, and how to get started quickly.
---

`skil` is a CLI for installing and maintaining Agent Skills from git repositories or archives.

## Install

```bash
curl -fsSL https://useskil.dev/install.sh | sh
```

Or with Cargo:

```bash
cargo install skil
```

## Basic Usage

```bash
# Add all skills from a package
skil add vercel-labs/agent-skills

# Add only specific skills
skil add vercel-labs/agent-skills --skill frontend-design

# See installed skills
skil list

# Check and apply updates
skil check
skil update
```

## Command Groups

- `add`, `install`, `remove`, `list`: install and manage skills.
- `find`, `check`, `update`: discover and update skill packages.
- `init`, `completions`: authoring and shell integration.
- `docs build`, `docs serve`: generate and preview static docs from repository skills.
