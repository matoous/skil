# skillz

Rust clone of `vercel-labs/skills` using `clap` and `gix`.

## Usage

```bash
skills add vercel-labs/agent-skills
skills add vercel-labs/agent-skills --list
skills add vercel-labs/agent-skills --skill frontend-design
skills list
skills find typescript
skills check
skills update
skills init my-skill
```

## Build

```bash
cargo build
cargo run -- --help
```

## Notes

- Interactive selection for skills/agents is supported when you omit `--skill` or `--agent`.
- Agent list is a trimmed, common subset; expand in `src/agent.rs` if you need more.
