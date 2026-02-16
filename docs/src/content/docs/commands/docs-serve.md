---
title: docs serve
description: Build docs and serve them locally over HTTP.
---

```bash
skil docs serve [options]
```

## Options

- `--source <path>`: source directory to scan for skills (default `.`).
- `--output <path>`: output directory for generated site (default `site`).
- `--host <host>`: bind host (default `127.0.0.1`).
- `--port <port>`: bind port (default `4173`).
- `--full-depth`: keep full directory depth while discovering skills.

## Example

```bash
skil docs serve --source . --port 4173
```
