---
name: skil
description: Use the Skil CLI to discover, install, update, remove, and document agent skills from local paths, git repositories, or tracked .skil.toml sources.
---

# skil

Use this skill when you need to manage agent skills with the `skil` CLI in this repository.

## When to Use

- You need to install skills from a local directory or a remote git repository.
- You need to install pinned skills from `.skil.toml`.
- You need to check or apply skill updates.
- You need to find, list, or remove installed skills.
- You need to scaffold a new skill or generate docs/completions.

## Instructions

1. Confirm `skil` is available.
   - Run: `skil --help`
   - If not installed, use the project binary instead: `cargo run -p skil -- --help`

2. Prefer non-interactive commands in automation.
   - Add `--yes` to skip prompts.
   - Use explicit selectors like `--skill <name>` and `--agent <agent>`.
   - Use `--copy` only when symlinks are undesirable.

3. Install skills from a source.
   - List skills in a package first:
     - `skil add <source> --list`
   - Install selected skills:
     - `skil add <source> --skill <skill-name> --agent <agent> --yes`
   - Install all skills from a package:
     - `skil add <source> --all`
   - Example sources:
     - `vercel-labs/agent-skills`
     - `https://github.com/matoous/skil`
     - `./local/path/to/skills`

4. Use lockfile-driven installs for reproducibility.
   - Commit `.skil.toml` to version control.
   - Install tracked, pinned skills with:
     - `skil install --yes`

5. Maintain installed skills.
   - Check available updates:
     - `skil check`
   - Apply updates:
     - `skil update`
   - Review what is installed:
     - `skil list`
   - Search registry by keyword:
     - `skil find <query>`

6. Remove skills safely.
   - Remove specific skills:
     - `skil remove --skill <skill-name> --agent <agent> --yes`
   - Remove all installed skills:
     - `skil remove --all --yes`

7. Create and document skills.
   - Scaffold a new skill:
     - `skil init <skill-name>`
   - Build docs:
     - `skil docs build --source .`
   - Serve docs locally:
     - `skil docs serve --source . --port 4173`
   - Generate shell completions:
     - `skil completions zsh`

## Notes

- `.skil.toml` is the source lockfile used by `skil install`, `skil check`, and `skil update`.
- Use `--global` only when you explicitly want skills installed into global agent directories.
- If command behavior differs across environments, run `skil <command> --help` for authoritative flags.
