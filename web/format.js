// Textes et calculs d'affichage (sans DOM).

const HUMAN = 0;

// Verbes conjugués : [deuxième personne du pluriel (« Vous »), troisième personne].
const VERBS = {
  lancer: ["lancez", "lance"],
  garder: ["gardez", "garde"],
  perdre: ["perdez", "perd"],
  avoir: ["avez", "a"],
  encaisser: ["encaissez", "encaisse"],
  gagner: ["gagnez", "gagne"],
};

/** Phrase décrivant un événement de partie (`last_event` de l'état). */
export function describeEvent(event, players) {
  if (!event) return "";
  const name = players[event.player].name;
  const verb = (v) => VERBS[v][event.player === HUMAN ? 0 : 1];
  const dice = (event.dice ?? []).join(" ");
  switch (event.kind) {
    case "rolled":
      return `${name} ${verb("lancer")} : ${dice}`;
    case "kept":
      return `${name} ${verb("garder")} ${dice} (+${event.points})`;
    case "farkle":
      return event.lost_points > 0
        ? `Farkle ! ${name} ${verb("perdre")} ${event.lost_points} points.`
        : `Farkle ! ${name} n'${verb("avoir")} aucun dé qui marque.`;
    case "banked":
      return event.won
        ? `${name} ${verb("encaisser")} ${event.points} points et ${verb("gagner")} la partie !`
        : `${name} ${verb("encaisser")} ${event.points} points (total ${event.new_score}).`;
    default:
      return "";
  }
}

/** Probabilité (0..1) en pourcentage à la française : « 3,1 % ». */
export function formatPercent(p) {
  return `${(p * 100).toFixed(1).replace(".", ",")} %`;
}

/**
 * Score des dés sélectionnés (positions dans `dice`), ou `null` si la sélection
 * est vide ou ne correspond à aucune mise de côté valide.
 */
export function selectionScore(dice, selected, combos) {
  if (!selected.length) return null;
  const key = (values) => [...values].sort((a, b) => a - b).join(",");
  const wanted = key(selected.map((i) => dice[i]));
  const combo = combos.find((c) => key(c.dice) === wanted);
  return combo ? combo.score : null;
}

/** « 1 dé », « 5 dés » (en français, 0 et 1 sont au singulier). */
export function plural(n, word) {
  return `${n} ${word}${n > 1 ? "s" : ""}`;
}
