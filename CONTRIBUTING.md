# Contributing to envcheck

Thank you for your interest in contributing to `envcheck`! We welcome contributions from the community to make this tool better.

## Getting Started

1.  **Fork** the repository on GitHub.
2.  **Clone** your fork locally:
    ```bash
    git clone https://github.com/skew202/envcheck.git
    cd envcheck
    ```
3.  **Install dependencies**: Ensure you have Rust and Cargo installed (latest stable version).

## Development Workflow

We use a standard Rust development workflow.

### Running Tests

Run the full test suite, including unit and integration tests:

```bash
cargo test
```

### Formatting and Linting

Ensure your code is formatted and passes clippy checks before submitting:

```bash
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
```

### Project Structure

- `src/parser/`: Modules for parsing `.env` and K8s YAML files.
- `src/rules/`: Implementation of individual lint rules.
- `src/commands/`: CLI command implementations.
- `tests/`: Integration tests and fixtures.

## Pull Requests

1.  Create a new branch for your feature or fix.
2.  Write tests for your changes.
3.  Ensure all tests and lints pass.
4.  Commit your changes using **Conventional Commits** (see below).
5.  Push to your fork and submit a Pull Request.

## Commit Messages

This project uses [Conventional Commits](https://www.conventionalcommits.org/) to automate versioning and changelog generation. Please format your commit messages as follows:

- `feat: add new lint rule for X` (Minor update)
- `fix: crash when parsing invalid YAML` (Patch update)
- `docs: update README usage examples` (No release trigger)
- `chore: update dependencies` (No release trigger)
- `refactor: simplify parser logic` (No release trigger)

**Breaking Changes**: Add a footer `BREAKING CHANGE:` or append `!` to the type/scope (e.g., `feat!: change API`).

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
