# envcheck

[![Crates.io](https://img.shields.io/crates/v/envcheck.svg)](https://crates.io/crates/envcheck)
[![CI](https://github.com/envcheck/envcheck/actions/workflows/ci.yml/badge.svg)](https://github.com/envcheck/envcheck/actions)
[![npm](https://img.shields.io/npm/v/envcheck)](https://www.npmjs.com/package/envcheck)

A fast, modern Rust CLI for linting `.env` files and ensuring environment synchronization across your entire DevSecOps stack.

## ✨ Features

### Core
- **Lint** - Detects duplicate keys, invalid syntax, empty values, trailing whitespace, unsorted keys
- **Compare** - Identifies missing keys across multiple environment files
- **Fix** - Auto-fix issues with `--commit` and `--pr` flags for CI integration
- **TUI** - Interactive terminal UI for comparing and merging `.env` files

### DevSecOps Integrations
| Integration | Command | What it checks |
|-------------|---------|----------------|
| **Kubernetes** | `envcheck k8s-sync` | SecretKeyRef/ConfigMapKeyRef vs `.env` |
| **Terraform** | `envcheck terraform` | `TF_VAR_*` variable usage |
| **Ansible** | `envcheck ansible` | `lookup('env', 'VAR')` calls |
| **GitHub Actions** | `envcheck actions` | `env:` blocks in workflows |
| **Helm** | `envcheck helm` | `SCREAMING_SNAKE_CASE` in `values.yaml` |
| **ArgoCD** | `envcheck argo` | `plugin.env` and `kustomize.commonEnv` |

### Output Formats
- **Text** - Colored terminal output (default)
- **JSON** - Machine-readable for pipelines
- **GitHub** - Native GitHub Actions annotations
- **SARIF** - GitHub Security tab integration
- **PR Comment** - Markdown for PR/MR comments

## 🚀 Installation

### Cargo (Recommended)
```bash
cargo install envcheck
```

### npm
```bash
npm install -g envcheck
# or use without installing
npx envcheck lint .env
```

### Homebrew
```bash
brew tap envcheck/tap
brew install envcheck
```

### Binary Releases
Download pre-built binaries from [GitHub Releases](https://github.com/envcheck/envcheck/releases).

## 📖 Usage

### Lint `.env` files
```bash
envcheck lint .env
envcheck lint .env .env.local .env.prod
envcheck lint .env --format json
envcheck lint .env --format sarif > results.sarif
```

### Compare environments
```bash
envcheck compare .env.example .env.prod
```

### Fix issues automatically
```bash
envcheck fix .env                    # Sort keys, remove whitespace
envcheck fix .env --commit           # Auto-commit changes
envcheck fix .env --pr               # Create a PR with fixes
```

### Interactive TUI
```bash
envcheck tui .env.example .env .env.prod
```

### K8s Sync
```bash
envcheck k8s-sync k8s/*.yaml --env .env.example
```

### Terraform
```bash
envcheck terraform infra/ --env .env
```

### Ansible
```bash
envcheck ansible playbooks/ --env .env
```

### GitHub Actions
```bash
envcheck actions .github/workflows --env .env
```

### Helm
```bash
envcheck helm charts/myapp --env .env
```

### ArgoCD
```bash
envcheck argo argocd/apps --env .env
```

### Shell Completions
```bash
envcheck completions bash > /etc/bash_completion.d/envcheck
envcheck completions zsh > ~/.zsh/completions/_envcheck
envcheck completions fish > ~/.config/fish/completions/envcheck.fish
```

## 📋 Lint Rules

| ID | Rule | Severity | Description |
|----|------|----------|-------------|
| `E001` | Duplicate Key | Error | Key defined multiple times |
| `E002` | Invalid Syntax | Error | Line is not `KEY=VALUE` |
| `W001` | Empty Value | Warning | Key has no value |
| `W002` | Trailing Whitespace | Warning | Line ends with whitespace |
| `W003` | Unsorted Keys | Warning | Keys are not alphabetically sorted |
| `W004` | Missing Key | Warning | Key missing in comparison file |
| `W005` | K8s Missing Env | Warning | Key in K8s not in `.env` |
| `W006` | Unused Env | Info | Key in `.env` not in K8s |

## ⚙️ Configuration

Create `.envcheckrc.yaml` or `.envcheckrc.toml` in your project root:

```yaml
# .envcheckrc.yaml
rules:
  disable:
    - W003  # Don't warn about unsorted keys
  warnings_as_errors: false

ignore:
  - "*.local"
  - ".env.development"

format: text

files:
  - .env
  - .env.example
```

### `.envcheckignore`

```
# Ignore patterns (like .gitignore)
*.local
.env.development
tests/fixtures/**
```

JSON Schema for IDE autocompletion: `https://envcheck.github.io/schema/envcheckrc.json`

## 🔄 CI/CD Integration

### GitHub Actions
```yaml
- uses: envcheck/action-envcheck@v1
  with:
    command: lint
    args: .env.example .env
    format: github
```

### Pre-commit
```yaml
repos:
  - repo: https://github.com/envcheck/envcheck
    rev: v0.1.0
    hooks:
      - id: envcheck-lint
        args: [".env.example", ".env"]
      - id: envcheck-k8s
        args: ["k8s/*.yaml", "--env", ".env.example"]
```

### GitLab CI
```yaml
envcheck:
  image: rust:latest
  script:
    - cargo install envcheck
    - envcheck lint .env --format json > envcheck-report.json
  artifacts:
    reports:
      codequality: envcheck-report.json
```

## 🏗️ Architecture

```
envcheck/
├── envcheck/           # Core Rust CLI (this repo)
├── envcheck-npm/       # npm wrapper package
├── action-envcheck/    # GitHub Action
└── envcheck.github.io/ # Documentation website
```

## 🔧 Performance

- **Parallel processing** with Rayon
- **Zero-copy parsing** with `Cow<str>` for reduced allocations
- **Benchmarks** available via `cargo bench`

```
parse_env_file:  ~3.3 µs
lint_rules:      ~2.3 µs
```

## 🤝 Comparison

| Feature | envcheck | dotenv-linter |
|---------|----------|---------------|
| Linting | ✅ | ✅ |
| Compare | ✅ | ✅ |
| Auto-fix | ✅ + commit/PR | ✅ |
| K8s Sync | ✅ | ❌ |
| Terraform | ✅ | ❌ |
| Ansible | ✅ | ❌ |
| GitHub Actions | ✅ | ❌ |
| Helm | ✅ | ❌ |
| ArgoCD | ✅ | ❌ |
| TUI | ✅ | ❌ |
| SARIF | ✅ | ❌ |
| Config files | ✅ | ❌ |
| Shell completions | ✅ | ❌ |

## 📦 Related Packages

- [envcheck-npm](https://github.com/envcheck/envcheck-npm) - npm wrapper
- [action-envcheck](https://github.com/envcheck/action-envcheck) - GitHub Action
- [envcheck.github.io](https://github.com/envcheck/envcheck.github.io) - Documentation

## 📄 License

MIT
