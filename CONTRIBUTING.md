# Contributing to PM8150B-SMB5-driver

[Русская версия](CONTRIBUTING.ru.md)

Follow the [code quality agreement](CODE_QUALITY.md) and use English [Conventional Commits](COMMIT_CONVENTIONS.md). Keep board-specific wiring and configuration in the board project.

Run these checks from the workspace root before submitting a pull request.

```sh
cargo fmt --all --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

Describe what changed and how you verified it. For changes that affect hardware behavior, include the Linux environment and hardware used for verification when available.
