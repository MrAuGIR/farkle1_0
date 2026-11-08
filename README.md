# Farkle 1.0

Jeu de dés Farkle (variante *Kingdom Come: Deliverance*) en Rust : console, navigateur (WASM) et mode entraînement.

- Règles : [docs/REGLES.md](docs/REGLES.md)
- Plan de développement par lots : [docs/PLAN.md](docs/PLAN.md)

## Structure
- `crates/farkle-core` : moteur (scoring, partie, conseiller, IA)
- `crates/farkle-cli` : binaire console `farkle`
- `crates/farkle-web` : bindings WebAssembly
- `web/` : interface navigateur

## Développement
```bash
git config core.hooksPath .githooks   # active le hook pre-push (fmt + clippy + tests)
cargo test --workspace
cargo run -p farkle-cli
```

Workflow : une branche `lot-N/<nom>` par lot, depuis `develop`, puis merge `--no-ff` dans `develop`.
La CI GitHub Actions (`.github/workflows/ci.yml`) lance fmt, clippy, tests et build WASM sur chaque push/PR.
