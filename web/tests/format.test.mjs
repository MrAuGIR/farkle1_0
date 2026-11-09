import { test } from "node:test";
import assert from "node:assert/strict";
import { describeEvent, formatPercent, selectionScore } from "../format.js";

const players = [{ name: "Vous", score: 0 }, { name: "Hans", score: 0 }];

test("événements du joueur (deuxième personne)", () => {
  assert.equal(
    describeEvent({ kind: "rolled", player: 0, dice: [1, 5, 2] }, players),
    "Vous lancez : 1 5 2",
  );
  assert.equal(
    describeEvent({ kind: "kept", player: 0, dice: [1, 1, 1], points: 1000 }, players),
    "Vous gardez 1 1 1 (+1000)",
  );
  assert.equal(
    describeEvent({ kind: "farkle", player: 0, dice: [2, 3], lost_points: 350 }, players),
    "Farkle ! Vous perdez 350 points.",
  );
  assert.equal(
    describeEvent(
      { kind: "banked", player: 0, points: 1000, new_score: 2500, won: false },
      players,
    ),
    "Vous encaissez 1000 points (total 2500).",
  );
  assert.equal(
    describeEvent(
      { kind: "banked", player: 0, points: 1000, new_score: 4000, won: true },
      players,
    ),
    "Vous encaissez 1000 points et gagnez la partie !",
  );
});

test("événements de l'IA (troisième personne)", () => {
  assert.equal(
    describeEvent({ kind: "rolled", player: 1, dice: [6, 6] }, players),
    "Hans lance : 6 6",
  );
  assert.equal(
    describeEvent({ kind: "kept", player: 1, dice: [5], points: 50 }, players),
    "Hans garde 5 (+50)",
  );
  assert.equal(
    describeEvent({ kind: "farkle", player: 1, dice: [2], lost_points: 0 }, players),
    "Farkle ! Hans n'a aucun dé qui marque.",
  );
  assert.equal(
    describeEvent({ kind: "farkle", player: 1, dice: [2], lost_points: 400 }, players),
    "Farkle ! Hans perd 400 points.",
  );
  assert.equal(
    describeEvent(
      { kind: "banked", player: 1, points: 300, new_score: 300, won: false },
      players,
    ),
    "Hans encaisse 300 points (total 300).",
  );
  assert.equal(
    describeEvent({ kind: "banked", player: 1, points: 300, new_score: 4100, won: true }, players),
    "Hans encaisse 300 points et gagne la partie !",
  );
});

test("farkle du joueur sans points en jeu", () => {
  assert.equal(
    describeEvent({ kind: "farkle", player: 0, dice: [2], lost_points: 0 }, players),
    "Farkle ! Vous n'avez aucun dé qui marque.",
  );
});

test("pas d'événement", () => {
  assert.equal(describeEvent(null, players), "");
});

test("pourcentages à la française", () => {
  assert.equal(formatPercent(0.0309), "3,1 %");
  assert.equal(formatPercent(2 / 3), "66,7 %");
  assert.equal(formatPercent(0), "0,0 %");
});

test("score de la sélection en cours", () => {
  const dice = [1, 5, 2, 1, 1, 6];
  const combos = [
    { dice: [1, 1, 1, 5], indices: [0, 1, 3, 4], score: 1050, patterns: [] },
    { dice: [1, 1, 1], indices: [0, 3, 4], score: 1000, patterns: [] },
    { dice: [5], indices: [1], score: 50, patterns: [] },
    { dice: [1, 5], indices: [0, 1], score: 150, patterns: [] },
    { dice: [1], indices: [0], score: 100, patterns: [] },
  ];
  assert.equal(selectionScore(dice, [0, 1, 3, 4], combos), 1050);
  // Même valeurs à d'autres positions : même score.
  assert.equal(selectionScore(dice, [4], combos), 100);
  assert.equal(selectionScore(dice, [1, 4], combos), 150);
  // Sélection invalide ou vide.
  assert.equal(selectionScore(dice, [2], combos), null);
  assert.equal(selectionScore(dice, [], combos), null);
});
