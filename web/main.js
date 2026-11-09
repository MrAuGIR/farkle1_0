// Interface navigateur du Farkle : pilote le moteur Rust compilé en WebAssembly.
// Construire d'abord le paquet : wasm-pack build crates/farkle-web --target web --out-dir ../../web/pkg

import init, { WebGame, rules_markdown } from "./pkg/farkle_web.js";
import { renderMarkdown } from "./markdown.js";
import { describeEvent, formatPercent, selectionScore } from "./format.js";

const HUMAN = 0;
const AI_STEP_DELAY = 900; // ms entre deux étapes de l'IA
const TURN_CHANGE_DELAY = 1400; // ms avant que l'IA commence son tour
const LOG_MAX = 60;
const ROLL_ANIMATION_MS = 550; // un peu plus que l'animation CSS « roll »

// Position des points sur une grille 3×3 (0 = haut gauche, 8 = bas droite).
const PIPS = {
  1: [4],
  2: [2, 6],
  3: [2, 4, 6],
  4: [0, 2, 6, 8],
  5: [0, 2, 4, 6, 8],
  6: [0, 2, 3, 5, 6, 8],
};

const $ = (id) => document.getElementById(id);

let game = null;
let state = null;
let target = 4000;
let selected = new Set();
let kept = []; // dés mis de côté pendant le tour en cours
let aiTimer = null;
let rollTimer = null;
let help = localStorage.getItem("farkle.help") === "on";

// ---------- Partie ----------

function randomSeed() {
  return crypto.getRandomValues(new Uint32Array(1))[0];
}

function startGame() {
  clearTimeout(aiTimer);
  game?.free();
  game = new WebGame(target, randomSeed());
  selected = new Set();
  kept = [];
  $("log").replaceChildren();
  showMessage("");
  $("config").hidden = true;
  $("table").hidden = false;
  setState(JSON.parse(game.state()));
}

/** Exécute une action du moteur ; affiche l'erreur éventuelle. */
function apply(action) {
  try {
    setState(JSON.parse(action()));
    return true;
  } catch (e) {
    showMessage(String(e), true);
    return false;
  }
}

function setState(next) {
  const previous = state;
  state = next;
  selected = new Set();
  const event = state.last_event;

  if (event && previous && event !== previous.last_event) {
    addLog(event);
    if (event.kind === "rolled" && state.turn_points === 0) kept = [];
    if (event.kind === "kept") kept = [...kept, ...event.dice];
    if (event.kind === "rolled" || event.kind === "farkle") animateRoll();
  }
  showMessage(state.phase === "game_over" ? "" : describeEvent(event, state.players));
  render();

  if (state.ai_turn) {
    const turnJustChanged = !previous || previous.current !== state.current;
    clearTimeout(aiTimer);
    aiTimer = setTimeout(aiStep, turnJustChanged ? TURN_CHANGE_DELAY : AI_STEP_DELAY);
  }
}

function aiStep() {
  if (state?.ai_turn) apply(() => game.ai_turn_step());
}

function humanTurn() {
  return !state.ai_turn && state.phase !== "game_over";
}

function selectedIndices() {
  return [...selected].sort((a, b) => a - b);
}

function currentSelectionScore() {
  return selectionScore(state.dice, selectedIndices(), state.combos);
}

// Lancer / Garder & relancer / Garder & encaisser.
function onRoll() {
  apply(() => game.roll());
}

function keepThen(next) {
  if (state.phase === "choosing") {
    const indices = selectedIndices();
    if (!indices.length) {
      showMessage("Cliquez sur les dés à garder.", true);
      return;
    }
    if (!apply(() => game.select(Uint32Array.from(indices)))) return;
  }
  apply(next);
}

// ---------- Affichage ----------

function dieElement(value, classes = []) {
  const die = document.createElement("div");
  die.className = ["die", ...classes].join(" ");
  die.setAttribute("aria-label", `dé ${value}`);
  for (let cell = 0; cell < 9; cell++) {
    const pip = document.createElement("span");
    if (PIPS[value].includes(cell)) pip.className = "pip";
    die.append(pip);
  }
  return die;
}

function render() {
  renderScores();
  renderTurn();
  renderDice();
  renderActions();
  renderHelp();
  renderGameOver();
}

function renderScores() {
  const container = $("scores");
  container.replaceChildren(
    ...state.players.map((p, i) => {
      const card = document.createElement("div");
      card.className = "player" + (i === state.current && state.phase !== "game_over" ? " active" : "");
      const pct = Math.min(100, (p.score / state.target) * 100);
      card.innerHTML = `
        <div class="player-name">${i === HUMAN ? "🧑" : "🤖"} ${p.name}</div>
        <div class="player-score">${p.score}</div>
        <div class="progress"><div style="width:${pct}%"></div></div>`;
      return card;
    }),
  );
  const goal = document.createElement("div");
  goal.className = "goal";
  goal.textContent = `Objectif : ${state.target}`;
  container.append(goal);
}

function renderTurn() {
  if (state.phase === "game_over") {
    $("turn").textContent = "Partie terminée";
    return;
  }
  const name = state.players[state.current].name;
  const who = state.current === HUMAN ? "Votre tour" : `Tour de ${name}`;
  const hot = state.hot_dice ? ' <span class="badge">Main chaude !</span>' : "";
  $("turn").innerHTML =
    `<strong>${who}</strong> · points du tour : <strong>${state.turn_points}</strong>` +
    ` · dés à lancer : ${state.dice_left}${hot}`;
}

function renderDice() {
  const event = state.last_event;
  let values = state.dice;
  let farkle = false;
  if (!values.length && event && (event.kind === "farkle" || event.kind === "rolled")) {
    values = event.dice;
    farkle = event.kind === "farkle";
  }

  const clickable = humanTurn() && state.phase === "choosing";
  const scoringValues = new Set(state.combos.flatMap((c) => c.dice));
  const container = $("dice");
  container.classList.toggle("clickable", clickable);
  container.replaceChildren(
    ...values.map((v, i) => {
      const classes = [];
      if (selected.has(i)) classes.push("selected");
      if (farkle) classes.push("dead");
      if (help && clickable && scoringValues.has(v)) classes.push("scoring");
      const die = dieElement(v, classes);
      if (clickable) {
        die.tabIndex = 0;
        die.setAttribute("role", "checkbox");
        die.setAttribute("aria-checked", String(selected.has(i)));
        die.addEventListener("click", () => toggleDie(i));
        die.addEventListener("keydown", (e) => {
          if (e.key === " " || e.key === "Enter") {
            e.preventDefault();
            toggleDie(i);
          }
        });
      }
      return die;
    }),
  );
  if (!values.length) {
    const hint = document.createElement("p");
    hint.className = "placeholder";
    hint.textContent = humanTurn() ? "Lancez les dés !" : "";
    container.append(hint);
  }

  $("kept").replaceChildren(...kept.map((v) => dieElement(v, ["small"])));

  const sel = $("selection");
  if (clickable && selected.size) {
    const score = currentSelectionScore();
    sel.textContent =
      score === null ? "Sélection invalide : chaque dé gardé doit marquer." : `Sélection : ${score} points`;
    sel.className = "selection" + (score === null ? " invalid" : "");
  } else {
    sel.textContent = clickable ? "Cliquez sur les dés à garder." : "";
    sel.className = "selection";
  }
}

function toggleDie(i) {
  if (selected.has(i)) selected.delete(i);
  else selected.add(i);
  renderDice();
  renderActions();
  $("dice").children[i]?.focus();
}

function renderActions() {
  const mine = humanTurn();
  const canKeep =
    mine &&
    ((state.phase === "choosing" && currentSelectionScore() !== null) || state.phase === "kept");
  $("roll-btn").disabled = !(mine && state.phase === "start");
  $("reroll-btn").disabled = !canKeep;
  $("bank-btn").disabled = !canKeep;
  $("reroll-btn").textContent = state.phase === "kept" ? "Relancer" : "Garder & relancer";
  $("bank-btn").textContent = state.phase === "kept" ? "Encaisser" : "Garder & encaisser";
}

function renderHelp() {
  const panel = $("help");
  const mine = humanTurn();
  panel.hidden = !(help && mine && (state.phase === "choosing" || state.phase === "kept"));
  if (panel.hidden) return;

  const decisionText = (d) => (d === "reroll" ? "relancer" : "encaisser");
  if (state.phase === "kept") {
    const r = state.recommendation;
    panel.innerHTML = `<h3>Conseil</h3><p>Plutôt <strong>${decisionText(r.decision)}</strong>
      (risque de Farkle ${formatPercent(r.farkle_probability)},
      espérance en relançant ${Math.round(r.expected_reroll)} points).</p>`;
    return;
  }

  const a = state.advice;
  const best = a.indices.map((i) => state.dice[i]).join(" ");
  panel.innerHTML = `
    <h3>Conseil</h3>
    <p>Garder <strong>${best}</strong> (${a.score} points), puis
      <strong>${decisionText(a.decision)}</strong> : ${a.turn_points_after} points en jeu,
      ${a.dice_left_after} dés à relancer, risque de Farkle ${formatPercent(a.farkle_probability)}.</p>
    <h3>Combinaisons possibles</h3>`;
  const list = document.createElement("ul");
  list.className = "combos";
  for (const c of state.combos) {
    const li = document.createElement("li");
    const btn = document.createElement("button");
    btn.type = "button";
    btn.innerHTML = `<span class="combo-dice">${c.dice.join(" ")}</span>
      <span class="combo-patterns">${c.patterns.join(" + ")}</span>
      <span class="combo-score">${c.score}</span>`;
    btn.title = "Sélectionner ces dés";
    btn.addEventListener("click", () => {
      selected = new Set(c.indices);
      renderDice();
      renderActions();
    });
    li.append(btn);
    list.append(li);
  }
  panel.append(list);
}

function renderGameOver() {
  const over = state.phase === "game_over";
  $("gameover").hidden = !over;
  $("table").classList.toggle("over", over);
  if (over) {
    const text =
      state.winner === HUMAN
        ? `🏆 Vous gagnez ${state.players[0].score} à ${state.players[1].score} !`
        : `${state.players[state.winner].name} gagne ${state.players[1].score} à ${state.players[0].score}.`;
    $("gameover-text").textContent = text;
  }
}

function animateRoll() {
  const dice = $("dice");
  dice.classList.remove("rolling");
  void dice.offsetWidth; // relance l'animation CSS
  dice.classList.add("rolling");
  // Retire la classe, sinon les dés recréés (clic, aide) seraient animés à nouveau.
  clearTimeout(rollTimer);
  rollTimer = setTimeout(() => dice.classList.remove("rolling"), ROLL_ANIMATION_MS);
}

function addLog(event) {
  const li = document.createElement("li");
  li.textContent = describeEvent(event, state.players);
  li.className = `log-${event.kind} player-${event.player}`;
  const log = $("log");
  log.prepend(li);
  while (log.children.length > LOG_MAX) log.lastChild.remove();
}

function showMessage(text, isError = false) {
  const msg = $("message");
  msg.textContent = text;
  msg.classList.toggle("error", isError);
}

// ---------- Règles ----------

function openRules() {
  $("rules").hidden = false;
  $("rules-close").focus();
}

function closeRules() {
  $("rules").hidden = true;
}

// ---------- Démarrage ----------

function wire() {
  $("config-form").addEventListener("submit", (e) => {
    e.preventDefault();
    const value = Number($("target").value);
    if (!Number.isInteger(value) || value <= 0) return;
    target = value;
    localStorage.setItem("farkle.target", String(target));
    startGame();
  });
  for (const btn of document.querySelectorAll("[data-target]")) {
    btn.addEventListener("click", () => ($("target").value = btn.dataset.target));
  }

  $("roll-btn").addEventListener("click", onRoll);
  $("reroll-btn").addEventListener("click", () => keepThen(() => game.reroll()));
  $("bank-btn").addEventListener("click", () => keepThen(() => game.bank()));
  $("replay-btn").addEventListener("click", startGame);
  $("config-btn").addEventListener("click", () => {
    clearTimeout(aiTimer);
    $("table").hidden = true;
    $("config").hidden = false;
  });

  const toggle = $("help-toggle");
  toggle.checked = help;
  toggle.addEventListener("change", () => {
    help = toggle.checked;
    localStorage.setItem("farkle.help", help ? "on" : "off");
    if (state) render();
  });

  $("rules-btn").addEventListener("click", openRules);
  $("rules-close").addEventListener("click", closeRules);
  $("rules").addEventListener("click", (e) => {
    if (e.target === $("rules")) closeRules();
  });
  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && !$("rules").hidden) closeRules();
  });

  const saved = Number(localStorage.getItem("farkle.target"));
  if (saved > 0) $("target").value = saved;
}

async function main() {
  wire();
  try {
    await init();
  } catch (e) {
    const err = $("load-error");
    err.hidden = false;
    err.textContent =
      "Impossible de charger le moteur WebAssembly. Construisez-le avec " +
      "« wasm-pack build crates/farkle-web --target web --out-dir ../../web/pkg ».";
    document.querySelector("#config-form button[type=submit]").disabled = true;
    console.error(e);
    return;
  }
  $("rules-body").innerHTML = renderMarkdown(rules_markdown());
}

main();
