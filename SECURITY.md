# Security Policy

SoulBind is early-stage desktop software. Please do not publish exploit details in public issues.

## Reporting

Report suspected vulnerabilities privately to the repository owner.

Include:

- Affected SoulBind version or commit
- Operating system
- Reproduction steps
- Expected impact
- Any relevant logs or screenshots

## Public Repository Hygiene

Before releases, run:

```sh
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
cargo audit --file src-tauri/Cargo.lock
node --check src/main.js
cargo tauri build --debug
```

If a secret is ever committed, remove it from Git history and rotate the credential. Removing it from history is not enough once it has been pushed to a public remote.

