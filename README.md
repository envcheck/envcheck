# envcheck

[![Crates.io](https://img.shields.io/crates/v/envcheck.svg)](https://crates.io/crates/envcheck)
[![CI](https://github.com/envcheck/envcheck/actions/workflows/ci.yml/badge.svg)](https://github.com/envcheck/envcheck/actions)

A fast, modern CLI tool for linting `.env` files, comparing environments, and ensuring synchronization between Kubernetes manifests and environment variables.

## Features

- **Lint**: Detects duplicate keys, invalid syntax, empty values, and trailing whitespace.
- **Compare**: Identifies missing keys across multiple environment files (e.g., `.env.example` vs `.env.prod`).
- **K8s Sync**: Detects mismatches between K8s manifests (Deployments, Secrets, ConfigMaps) and your `.env` files.
- **Formats**: Supports colored text, JSON, and GitHub Actions annotation output.
- **Fast**: Built in Rust for speed and reliability.

## Why envcheck?

| Feature | envcheck 🦀 | dotenv-linter |
|---------|-------------|---------------|
| **Linting** | ✅ (Duplicate, Syntax, Empty, Whitespace, Sort) | ✅ |
| **Compare** | ✅ (Missing keys across files) | ✅ |
| **K8s Sync** | ✅ (Mismatches between .env & YAML) | ❌ |
| **Output** | Text, JSON, GitHub | Text |
| **Performance**| 🚀 Native Rust | 🚀 Native Rust |

## Installation

```bash
cargo install envcheck
```

## Usage

### Lint .env files

```bash
# Lint one or more files
envcheck lint .env
envcheck lint .env .env.local

# JSON output
envcheck lint .env --format=json
```

### Compare environments

Check if `.env.prod` has all the keys defined in `.env.example`:

```bash
envcheck compare .env.example .env.prod
```

### K8s Synchronization

Ensure all secrets referenced in your K8s manifests are Present in your `.env.example`, and vice-versa.

```bash
envcheck k8s-sync k8s/base/*.yaml --env .env.example
```

Detects:
- Keys used in K8s (SecretKeyRef) but missing in `.env` (Warning W005)
- Keys in `.env` but never used in K8s (Info W006)

### Doctor

Lint all `.env*` files in the current directory:

```bash
envcheck doctor
```

## Lint Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `E001` | Duplicate Key | Error | Key defined multiple times |
| `E002` | Invalid Syntax | Error | Line is not KEY=VALUE |
| `W001` | Empty Value | Warning | Key has no value |
| `W002` | Trailing Whitespace | Warning | Line ends with whitespace |
| `W004` | Missing Key | Warning | Key missing in comparison file |
| `W005` | K8s Missing Env | Warning | Key in K8s not in .env |
| `W006` | Unused Env | Info | Key in .env not in K8s |

## Integration

### GitHub Actions

```yaml
- name: Install envcheck
  run: cargo install envcheck

- name: Check .env files
  run: envcheck lint .env.example .env.prod --format=github

### Pre-commit

Add this to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/envcheck/envcheck
    rev: v0.1.0
    hooks:
      # Lint .env files
      - id: envcheck
        args: ["lint", ".env.example", ".env"]

      # Ensure K8s manifests match .env
      - id: envcheck-k8s
        args: ["k8s-sync", "k8s/**/*.yaml", "--env", ".env.example"]
```
```

## Roadmap

- [x] **Auto-fix**: Automatically sort keys (`fix`).
- [ ] **Interactive Mode**: Guided fixes for mismatches.
- [ ] **Shell Completions**: Native shell autocompletion.

## License

MIT
