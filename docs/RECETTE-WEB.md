# Recette manuelle de l'interface web

Préparation :
```bash
wasm-pack build crates/farkle-web --target web --out-dir ../../web/pkg
python3 -m http.server -d web 8000   # puis ouvrir http://localhost:8000
```

## Configuration
- [ ] L'écran « Nouvelle partie » s'affiche, avec le score cible à 4000 par défaut.
- [ ] Les boutons 2000 / 4000 / 8000 remplissent le champ, et une valeur libre (ex. 1500) est acceptée.
- [ ] Le score cible choisi est conservé après un rechargement de la page.
- [ ] Sans `web/pkg`, un message explique comment construire le moteur et le bouton « Commencer » est désactivé.

## Table de jeu
- [ ] Les scores, les barres de progression et l'objectif sont affichés ; le joueur dont c'est le tour est encadré.
- [ ] « Lancer » lance 6 dés avec une animation (aucune animation si « réduire les animations » est activé).
- [ ] Un clic (ou Espace / Entrée) sélectionne ou désélectionne un dé ; le score de la sélection s'affiche.
- [ ] Une sélection invalide affiche un message et désactive « Garder & relancer » et « Garder & encaisser ».
- [ ] « Garder & relancer » ajoute les dés à « Mis de côté » et relance les dés restants.
- [ ] « Garder & encaisser » ajoute les points au score et passe la main à Hans.
- [ ] Un Farkle grise les dés, affiche « Farkle ! … » et passe la main.
- [ ] Une main chaude (6 dés gardés) affiche le badge « Main chaude ! » et permet de relancer les 6 dés.
- [ ] Le journal liste les actions des deux joueurs (Hans en italique, Farkle en rouge, encaissements en or).

## Tours de l'IA
- [ ] Hans joue pas à pas, avec une pause visible entre chaque étape.
- [ ] Pendant le tour de Hans, les dés ne sont pas cliquables et les boutons sont désactivés.

## Aide
- [ ] L'interrupteur « Aide » est conservé après un rechargement de la page.
- [ ] Avec l'aide, les dés qui peuvent marquer sont surlignés en vert, et le conseil (dés à garder, relancer ou encaisser, risque de Farkle) ainsi que la liste des combinaisons s'affichent.
- [ ] Un clic sur une combinaison sélectionne les dés correspondants.
- [ ] Sans l'aide, il n'y a ni surlignage ni panneau.

## Règles
- [ ] Le bouton « ? » ouvre le panneau des règles : titres, listes imbriquées et tableaux sont rendus.
- [ ] Le panneau se ferme avec ×, avec Échap ou par un clic en dehors.

## Fin de partie
- [ ] Le vainqueur et le score final s'affichent.
- [ ] « Rejouer » relance une partie avec le même score cible ; « Changer le score cible » revient à la configuration.
- [ ] Sur mobile (environ 400 px de large), les 6 dés tiennent sur une ligne.
