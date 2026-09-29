# MLS (My Life Storage)

Open-source, local-first, zero-knowledge storage for passwords, notes, and personal data.

---

## Philosophy

- **Zero-Knowledge**: Your master password never leaves your device. The server only sees encrypted blobs.
- **Local-First**: Works offline. Sync is background and conflict-free via CRDT.
- **One Codebase**: Single Rust core for desktop, mobile, and CLI.
- **No Technical Debt**: Built to last. No shortcuts.

---

## Stack

| Component       | Technology                          |
|-----------------|-------------------------------------|
| Core            | Rust (2021), `mls-core`             |
| Crypto          | Argon2id, HKDF, AES-256-GCM, X25519 |
| Storage         | SQLite (bundled)                    |
| CRDT            | [yrs](https://github.com/y-crdt/yrs) |
| Sync            | WebSocket, mDNS, tokio               |
| Server          | axum, sqlx (SQLite/PostgreSQL)      |
| CLI             | clap, ratatui                       |
| Desktop/Mobile  | Tauri v2 + React/TypeScript        |

---

## Quick Start

### Prerequisites
- Rust (stable, 2021 edition)
- Node.js (for Tauri frontend)

### Build & Run
```bash
# Build everything
cargo build --workspace

# Run CLI
cargo run -p mls-cli -- init
cargo run -p mls-cli -- unlock

# Run server
cargo run -p mls-server

# Run Tauri (GUI)
cd crates/mls-tauri && cargo tauri dev
```

---

## Project Structure
```
mls/
├── crates/
│   ├── mls-core/      # Data models, CRDT, traits
│   ├── mls-crypto/    # Encryption, key derivation
│   ├── mls-storage/   # SQLite backend
│   ├── mls-sync/      # P2P/LAN/Relay sync
│   ├── mls-cli/       # Terminal interface
│   ├── mls-server/    # Relay server
│   └── mls-tauri/     # GUI (Tauri + React)
└── README.md
```

---

## Security
- All secrets use `Secret<T>` and `Zeroizing`.
- No `unwrap()` or `expect()` in production code.
- No `println!` in libraries (use `tracing`).
- No global mutable state.
- `#[forbid(unsafe_code)]` everywhere.

---

## Contributing
See [CONTRIBUTING.md](CONTRIBUTING.md).

---

## License
[MPL-2.0](LICENSE)
