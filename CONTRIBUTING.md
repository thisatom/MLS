# Contributing to MLS

## Setup
1. Clone the repo:
   ```bash
   git clone https://github.com/your-org/mls.git
   cd mls
   ```
2. Install Rust:
   ```bash
   rustup default stable
   rustup component add rustfmt clippy
   ```
3. Install dependencies:
   ```bash
   cargo install cargo-deny cargo-audit cargo-nextest
   ```

---

## Architecture Rules
- **No `unwrap()`/`expect()`**: Use `?` with meaningful errors (`thiserror`/`anyhow`).
- **No `panic!` in libraries**: Only in tests.
- **No secrets in logs**: Use `Secret<T>` and `[REDACTED]` in `Debug`.
- **No `println!` in libraries**: Use `tracing` with levels.
- **No global mutable state**: Use `Arc<Mutex<T>>` or actors.
- **All public items must have `#[doc]` comments**.
- **Every module must have tests**: Happy path + edge cases + errors.
- **No `TODO` without an issue**: Link to GitHub issue if deferring work.
- **No dependencies without justification**: Audit before adding.

---

## Testing
```bash
# Run all tests
cargo nextest run --workspace --all-features

# Run tests for a specific crate
cargo nextest run -p mls-crypto

# Property-based tests (mls-crypto)
cargo test --features proptest
```

---

## Adding a New Crate
1. Add to workspace `Cargo.toml`:
   ```toml
   members = ["crates/new-crate"]
   ```
2. Add dependencies to `new-crate/Cargo.toml`.
3. Ensure `clippy` and `fmt` pass.
4. Add tests in `src/lib.rs` or `tests/`.

---

## Pull Requests
- Follow [Conventional Commits](https://www.conventionalcommits.org/).
- Atomic commits: One logical change per commit.
- All CI checks must pass.
- Add tests for new functionality.
- Update `CHANGELOG.md` (if applicable).
