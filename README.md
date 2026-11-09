# Farkle 1.0

Jeu de dés Farkle (variante *Kingdom Come: Deliverance*) en Rust : console, navigateur (WASM) et mode entraînement.

- Règles : [docs/REGLES.md](docs/REGLES.md)
- Plan de développement par lots : [docs/PLAN.md](docs/PLAN.md)

## Structure
- `crates/farkle-core` : moteur (scoring, partie, conseiller, IA)
- `crates/farkle-cli` : binaire console `farkle`
- `crates/farkle-web` : bindings WebAssembly
- `web/` : interface navigateur

## Jouer en console
```bash
cargo run -p farkle-cli -- --help                          # usage + règles complètes
cargo run -p farkle-cli -- play --target 2000 --aide on    # partie contre l'IA
cargo run -p farkle-cli -- play --ia audacieuse            # IA prudente | normale | audacieuse
```
En jeu : numéros des dés à garder (`1 3 5` ou `135`), `r` relancer, `b` encaisser,
`s` suivre le conseil, `a` aide on/off, `?` règles, `q` quitter.

## Mode entraîner
```bash
cargo run -p farkle-cli -- train                            # objectif 4000, score 0
cargo run -p farkle-cli -- train --target 4000 --score 3500 # s'entraîner sur une fin de partie
```
Saisissez chaque lancer (`1 5 5 3 2 6` ou `155326`) : le programme affiche les combinaisons possibles,
la meilleure mise de côté, puis conseille de relancer ou d'encaisser (risque de Farkle, espérance).
Indiquez ensuite les dés réellement gardés, puis `r` (je relance) ou `b` (j'encaisse).

## WebAssembly
Prérequis : `rustup target add wasm32-unknown-unknown` et `cargo install wasm-pack --locked`
(avec rustc 1.90, prendre `--version 0.13.1`).
```bash
wasm-pack build crates/farkle-web --target web --out-dir ../../web/pkg   # génère web/pkg
wasm-pack test --node crates/farkle-web                                  # tests dans Node
```
API exposée à JavaScript (chaque action renvoie l'état de la partie en JSON, ou lève
une exception avec un message en français) :
- `new WebGame(target, seed)` : partie « Vous » (joueur 0) contre « Hans » (IA, joueur 1) ;
- `roll()`, `select([indices])`, `reroll()`, `bank()` : actions du joueur ;
- `ai_turn_step()` : une étape du tour de l'IA (à appeler tant que `state.ai_turn` est vrai) ;
- `state()` : phase (`start`, `choosing`, `kept`, `game_over`), scores, dés, combinaisons,
  conseil (`advice`, `recommendation`) et dernier événement (`last_event`) ;
- `combos([dés])` et `rules_markdown()`.

## Développement
```bash
git config core.hooksPath .githooks   # active le hook pre-push (fmt + clippy + tests)
cargo test --workspace
cargo run -p farkle-cli
```

Workflow : une branche `lot-N/<nom>` par lot, depuis `develop`, puis merge `--no-ff` dans `develop`.
La CI GitHub Actions (`.github/workflows/ci.yml`) lance fmt, clippy, tests, build WASM et tests WASM (Node) sur chaque push/PR.
