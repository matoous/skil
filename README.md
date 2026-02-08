<div align="center">

# Skil

A fast, friendly CLI for managing agent skills.

[![Version](https://img.shields.io/github/v/release/matoous/skil)](https://github.com/matoous/skil/releases)
[![License](https://img.shields.io/github/license/matoous/skil)](https://github.com/matoous/skil/blob/main/LICENSE)
[![Stars](https://img.shields.io/github/stars/matoous/skil)](https://github.com/matoous/skil)

</div>

Skil is a CLI tool that makes it easy to install, update, and organize agent skills from Git repositories or archives. It wraps the workflows around skill packages so developers and teams can share curated skill sets, keep them up to date, and bootstrap new agents quickly without manual copying or custom scripts.

This tool is inspired by [vercel-labs/skills](https://github.com/vercel-labs/skills) but meant for those, that don't want to rely on javascript.

## Installation

Install with Cargo:

```bash
cargo install skil
```

Or build from source:

```bash
git clone https://github.com/matoous/skil
cd skil
cargo install --path .
```

## Usage

```bash
# Install a skill package
skil add vercel-labs/agent-skills

# Browse available skills in a package
skil add vercel-labs/agent-skills --list

# Install a specific skill
skil add vercel-labs/agent-skills --skill frontend-design

# See what is installed
skil list

# Search for skills
skil find typescript

# Check for updates and apply them
skil check
skil update

# Create a new SKILL.md template
skil init my-skill

# Generate shell completions
skil completions zsh
```

## Commands

### add

Add a skill package from a repository or archive.

```bash
skil add <source> [options]
```

Options:
- `-g, --global` Install for all agents (default is current agent only).
- `--copy` Copy files instead of symlinking.
- `-a, --agent <agent...>` Target one or more agents.
- `-s, --skill <skill...>` Install one or more skills from the package.
- `-l, --list` List skills found in the package.
- `-y, --yes` Skip confirmation prompts.
- `--all` Install all skills in the package.
- `--full-depth` Keep full directory depth when installing.

### remove

Remove installed skills.

```bash
skil remove [skills...] [options]
```

Options:
- `-g, --global` Remove from all agents (default is current agent only).
- `-a, --agent <agent...>` Target one or more agents.
- `-s, --skill <skill...>` Remove one or more specific skills.
- `-y, --yes` Skip confirmation prompts.
- `--all` Remove all installed skills.

### list

List installed skills.

```bash
skil list [options]
```

Options:
- `-g, --global` List global installs.
- `-a, --agent <agent...>` Filter by one or more agents.

### find

Search for skills by keyword.

```bash
skil find [query]
```

### check

Check for available skill updates.

```bash
skil check
```

### update

Update all installed skills to the latest versions.

```bash
skil update
```

### init

Initialize a new skill template.

```bash
skil init [name]
```

### completions

Generate shell completion scripts.

```bash
skil completions <shell>
```

Supported shells include: `bash`, `zsh`, `fish`, `elvish`, `powershell`.

## Build

```bash
cargo build
cargo run -- --help
```
