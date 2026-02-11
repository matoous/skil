<div align="center">

# Skil

A fast, friendly CLI for managing agent skills.

[![Version](https://img.shields.io/github/v/release/matoous/skil)](https://github.com/matoous/skil/releases)
[![License](https://img.shields.io/github/license/matoous/skil)](https://github.com/matoous/skil/blob/main/LICENSE)
[![Stars](https://img.shields.io/github/stars/matoous/skil)](https://github.com/matoous/skil)

</div>

Skil is a CLI tool that makes it easy to install, update, and organize [Agent Skills](https://agentskills.io/home) from Git repositories or archives. It wraps the workflows around skill packages so developers and teams can share curated skill sets, keep them up to date, and bootstrap new agents quickly without manual copying or custom scripts.

This tool is inspired by [vercel-labs/skills](https://github.com/vercel-labs/skills) but meant for those, that don't want to rely on javascript.

## Why?

Existing tools recommend that you copy skill over into your repository or your tool's global configuration folder. `skil` aims to make the maintenance of skills easier by tracking their upstreams and allowing you to update skills just like you would with other dependencies.

You can still have your local skills, but if you decide to use skill from another repository, e.g.:

```sh
skil add https://github.com/github/awesome-copilot --skill gh-cli
```

`skil` will create a `.skil.toml` lock-file for you:

```toml
[source."https://github.com/github/awesome-copilot.git"]
checksum = "d99ba7198680e68f49d7e4cd2f7cc38209f3b232"
skills = ["gh-cli"]
```

You can then update installed skills to their latest version:

```sh
skil update
```

Or keep only `.skil.toml` in your VCS and allow anyone else to install the tracked skills for the tool of their choice using:

```sh
skil install
```

## Installation

Install with Cargo:

```bash
cargo install skil
```

Or build from source:

```bash
git clone https://github.com/matoous/skil
cd skil
cargo install --path src/skil
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

# Build static docs website from repository skills
skil docs build --source .

# Build and serve docs locally
skil docs serve --source . --port 4173
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

### docs build

Build a static website that renders discovered repository skills.

```bash
skil docs build [--source <path>] [--output <path>] [--full-depth]
```

### docs serve

Build docs and serve them over a local HTTP server.

```bash
skil docs serve [--source <path>] [--output <path>] [--host <host>] [--port <port>] [--full-depth]
```

## Build

```bash
cargo build
cargo run -p skil -- --help
```
