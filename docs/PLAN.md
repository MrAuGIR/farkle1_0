# Plan — Farkle (Rust) : console, navigateur, mode entraînement

> **État (2026-11-09)** : lots 0 à 8 terminés, version `v1.0`.

## Contexte
Le dépôt `farkle1_0` est vide : il contient seulement `cargo new` (édition 2024, `src/main.rs`), et la branche courante est `develop`. On veut un jeu de Farkle aux règles de Kingdom Come: Deliverance, où le joueur affronte une IA (PNJ). Il y aura deux modes de jeu, console et navigateur (Rust → WASM), et un mode « entraîner » en console où l'on saisit ses lancers à la main pour obtenir des conseils. Chaque mode a un interrupteur « aide » qui affiche les combinaisons possibles, et le score cible est réglable. Les règles s'affichent avec `--help` en console et dans un panneau superposé dans le navigateur.
**Avant de coder**, on écrit les règles (`docs/REGLES.md`) et ce plan découpé en lots (`docs/PLAN.md`).

Choix validés : règles style KCD · navigateur en Rust→WASM (wasm-bindgen + HTML/CSS/JS léger) · adversaire IA.

---

## Étape 0 — Corriger le remote git (demandé)
```
git remote remove origin
git remote add origin git@github.com:MrAuGIR/farkle1_0.git
git remote -v   # vérification
```
(On peut aussi faire `git remote set-url origin git@github.com:MrAuGIR/farkle1_0.git`, qui donne le même résultat.)

---

## Règles KCD retenues (résumé de `docs/REGLES.md`)
- 6 dés. À chaque lancer, on doit mettre de côté **au moins un dé qui marque**. Un dé mis de côté ne peut **pas** être combiné avec les dés d'un autre lancer.
- Valeurs : un 1 vaut 100, un 5 vaut 50.
- Brelan : la valeur du dé × 100, sauf trois 1 qui valent 1000. **Chaque dé identique en plus double le score** : 4 dés = ×2, 5 dés = ×4, 6 dés = ×8 (six 1 = 8000).
- Suites : 1-2-3-4-5 = 500, 2-3-4-5-6 = 750, 1-2-3-4-5-6 = 1500.
- Après chaque mise de côté, on choisit : **relancer** les dés restants, ou **encaisser** les points du tour.
- **Farkle** : si un lancer ne contient aucun dé qui marque, on perd tous les points du tour et la main passe.
- **Hot dice** : si les 6 dés ont marqué, on peut relancer les 6.
- Fin de partie : le premier joueur qui atteint le score cible gagne. La cible est réglable, 4000 par défaut.
- Hors périmètre : les dés spéciaux et les badges de KCD.

---

## Architecture (workspace Cargo)
```
farkle1_0/
├── Cargo.toml                 # [workspace] members = crates/*
├── docs/REGLES.md, docs/PLAN.md
├── crates/
│   ├── farkle-core/           # lib pure, sans I/O : dés, scoring, moteur, conseiller, IA
│   │   └── src/{dice.rs, scoring.rs, combos.rs, game.rs, advisor.rs, ai.rs, rules.rs}
│   ├── farkle-cli/            # bin `farkle` (clap) : play, train
│   └── farkle-web/            # cdylib wasm-bindgen : façade JS du moteur
├── web/                       # index.html, style.css, main.js (+ pkg/ généré)
├── .githooks/pre-push         # cargo fmt --check && clippy && test
└── .github/workflows/ci.yml
```
Principes :
- **Le cœur est 100 % testable.** Le hasard passe par un trait `DiceRoller` qui a deux implémentations : `RandomRoller` (rand) et `ScriptedRoller` (des lancers prédéfinis, pour les tests et le mode entraîner).
- **Une seule source pour les règles.** `docs/REGLES.md` est intégré avec `include_str!` dans le CLI (pour `--help`) et dans le WASM (pour le panneau du navigateur).
- **Le conseiller est partagé** par l'aide, l'IA et le mode entraîner. Il calcule l'espérance du choix « relancer » face à « encaisser », à partir de la probabilité de Farkle selon le nombre de dés restants (valeurs exactes pour 1 à 6 dés, calculées par énumération).

Dépendances prévues : `rand`, `clap` (derive), `wasm-bindgen`, `getrandom` (feature `js`), `serde`/`serde-wasm-bindgen` ; pour les tests : `assert_cmd`, `predicates`, `wasm-bindgen-test`.

---

## Workflow git et tests
- Pour chaque lot : `git switch -c lot-N/<nom>` depuis `develop`, puis le développement **en TDD** (on écrit les tests rouges, puis le code, puis on refactorise), puis `git merge --no-ff` dans `develop`.
- **Tests unitaires** dans chaque module (`#[cfg(test)]`). **Tests d'intégration** dans `crates/*/tests/` : scénarios de partie complets avec `ScriptedRoller`, et le binaire CLI piloté par stdin avec `assert_cmd`.
- **Forcer les tests au push :**
  1. En local, un hook `.githooks/pre-push` lance fmt, clippy et les tests, et bloque le push en cas d'échec. Il s'active avec `git config core.hooksPath .githooks`, et on documente cette commande dans le README.
  2. À distance, une GitHub Action `ci.yml` se déclenche sur chaque push et chaque PR. Elle lance `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace` et le build `wasm32-unknown-unknown`.
  3. Une fois le repo connecté, on protège les branches `main` et `develop` sur GitHub : PR obligatoire et CI verte requise. GitHub ne peut pas bloquer un push brut qui ne respecte pas ces règles, c'est donc la combinaison hook + protection qui fait office de garde-fou. Attention : sur un repo privé avec un compte gratuit, la protection de branche n'est pas disponible.

---

## Lots

### Lot 0 — `lot-0/setup` : documentation et squelette
- `docs/REGLES.md` contient les règles complètes en français, avec des exemples de scoring.
- `docs/PLAN.md` reprend ce plan.
- Conversion en workspace (les 3 crates sont vides mais compilent), plus `.gitignore` (target, web/pkg).
- `.githooks/pre-push`, `.github/workflows/ci.yml` et un README avec les commandes.
- Test : un test bidon par crate pour valider la CI.

### Lot 1 — `lot-1/scoring` : dés et scoring (cœur, TDD)
- Types `Die(u8 1..=6)`, `Roll(Vec<Die>)` et `DiceRoller` (Random et Scripted).
- `scoring::score_selection(&[Die]) -> Option<u32>` renvoie `None` si un dé sélectionné ne marque pas.
- `combos::find_combos(&Roll) -> Vec<Combo>` liste toutes les mises de côté valides avec leur score. C'est la base de l'aide.
- `is_farkle(&Roll)`.
- Tests : toutes les règles, cas limites (six 1, suite + 1 ou 5 en plus, 4 identiques + 5, sélection invalide) et tests de propriété simples (le score est toujours positif ou nul).

### Lot 2 — `lot-2/engine` : moteur de partie
- `Game { players, target, current, turn_state }` avec la machine à états suivante :
  `Rolling → Choosing(roll) → (Reroll | Bank) / Farkle → NextPlayer` ; hot dice ; détection du vainqueur.
- Les erreurs sont typées (`GameError`) : sélection invalide, action hors tour, etc.
- Tests d'intégration : parties complètes avec `ScriptedRoller` (farkle, hot dice, victoire à la cible exacte ou au-delà).

### Lot 3 — `lot-3/advisor-ai` : conseiller et IA
- `advisor::farkle_probability(n_dice)`, valeurs exactes calculées par énumération des 6^n lancers.
- `advisor::suggest(roll, turn_points, scores, target) -> Advice { best_keep, all_combos, should_bank, expected_reroll }`.
- `ai::AiPlayer` (niveau normal) s'appuie sur `suggest`. Option : un niveau « prudent » ou « audacieux » selon un seuil.
- Tests : probabilités connues (1 dé ≈ 66,7 %, 6 dés ≈ 3,09 %) et décisions attendues sur des cas typiques.

### Lot 4 — `lot-4/cli-play` : console, jeu contre l'IA
- `farkle play --target 4000 --aide on|off` : la partie se joue contre l'IA.
- `farkle --help` affiche l'usage puis les règles complètes (`after_long_help` = REGLES.md). Il y a aussi une sous-commande `farkle rules`.
- En jeu : affichage des dés indexés, saisie des dés à garder (ex. `1 3 5`), puis `r` pour relancer, `b` pour encaisser, `a` pour basculer l'aide, `q` pour quitter. Quand l'aide est active, on affiche les combinaisons possibles et la suggestion.
- L'I/O est abstraite (traits sur `BufRead` et `Write`) pour être testable.
- Tests : parsing des entrées, et intégration avec `assert_cmd` (stdin scripté et graine RNG fixe via une option cachée `--seed`).

### Lot 5 — `lot-5/cli-train` : mode entraîner
- `farkle train [--target N]` : on saisit un lancer (`1 5 5 3 2 6`) et le programme répond :
  - la liste des combinaisons possibles avec leur score ;
  - la meilleure mise de côté, la probabilité de Farkle sur le prochain lancer, et la recommandation **garder la main / passer** avec l'espérance de gain.
- Le programme suit les points du tour et les dés restants. On saisit ensuite le lancer suivant pour les dés restants. La commande `b` encaisse, Farkle remet le tour à zéro, et le score total est suivi.
- Validation : le nombre de dés saisis doit correspondre aux dés restants, et chaque valeur doit être entre 1 et 6.
- Tests : des sessions stdin complètes avec `assert_cmd`.

### Lot 6 — `lot-6/wasm` : bindings
- `farkle-web` expose `WebGame` (new(target), roll, select, bank, reroll, state() en JSON, ai_turn_step()), ainsi que `combos(dice)` et `rules_markdown()`.
- Build avec `wasm-pack build crates/farkle-web --target web --out-dir ../../web/pkg`.
- Tests : `wasm-bindgen-test` (node ou headless) et des tests Rust natifs sur la façade.

### Lot 7 — `lot-7/web-ui` : interface navigateur
- Contenu de `web/index.html`, `style.css` et `main.js` :
  - **écran de config** avec le choix du score cible ;
  - **table de jeu** : dés cliquables à garder, boutons Lancer / Garder & relancer / Encaisser, scores et animation simple ;
  - **tours de l'IA** joués pas à pas avec une temporisation ;
  - **interrupteur Aide** : quand il est actif, les combinaisons possibles sont surlignées et listées ;
  - **panneau Règles** (bouton « ? ») qui rend `rules_markdown()` avec un mini-rendu markdown.
- Lancement local : `python3 -m http.server -d web` (documenté).
- Tests : vérification manuelle avec une checklist. On pourra ajouter un test e2e Playwright plus tard.

### Lot 8 — `lot-8/release` (optionnel)
- README final, déploiement GitHub Pages via une Action, tag `v1.0`, puis merge de `develop` dans `main`.

---

## Vérification
- `cargo test --workspace` passe au vert à chaque lot. `cargo clippy --workspace -- -D warnings` et `cargo fmt --check` passent aussi.
- `cargo run -p farkle-cli -- --help` affiche les règles.
- `cargo run -p farkle-cli -- play --target 2000 --aide on` permet une partie complète contre l'IA.
- `cargo run -p farkle-cli -- train`, avec la saisie `1 1 1 5 2 3`, propose des combinaisons dont le brelan de 1 à 1000, puis un conseil.
- `wasm-pack build ...` suivi de `python3 -m http.server -d web` : on peut jouer une partie dans le navigateur, avec l'aide et le panneau des règles.
- Sur GitHub, la CI est verte et le hook pre-push bloque un push si un test échoue.

## Premier pas après validation
Étape 0 (remote), puis le lot 0 : `git switch -c lot-0/setup`, écriture de `docs/REGLES.md` et `docs/PLAN.md`, squelette et CI, puis merge dans `develop`.
