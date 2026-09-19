# Contributing

Thanks for improving OrangeCNC Studio.

1. Keep domain logic out of the UI.
2. Add tests for parser and geometry changes.
3. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
4. Prefer small commits with a reason, not only a description of the changed line.
5. Do not add machine-control behavior without a failure-mode discussion.
