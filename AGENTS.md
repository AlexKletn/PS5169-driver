# Project instructions

Follow the [code quality agreement](CODE_QUALITY.md) when writing, editing, or reviewing code in this repository.

Follow the [commit message convention](COMMIT_CONVENTIONS.md) when creating commits in this repository.

Keep board-specific wiring and configuration in the board project.

Implement driver logic in Rust. Use thin C shims where the Linux kernel requires C macros, callbacks, or APIs without usable Rust bindings.

Use `NOTES.local.md` in the repository root for machine-specific paths and development notes, such as links to local driver sources. Add the file to `.gitignore` so it stays local.
