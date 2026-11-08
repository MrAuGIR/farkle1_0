# Règles du Farkle (variante Kingdom Come: Deliverance)

## But du jeu
Être le premier joueur à atteindre le **score cible** (4000 points par défaut, réglable).

## Matériel
6 dés à six faces.

## Déroulement d'un tour
1. Le joueur lance les **6 dés**.
2. Il doit mettre de côté **au moins un dé qui marque** (voir le tableau des scores).
   Tous les dés mis de côté doivent marquer : on ne peut pas garder un dé qui ne rapporte rien.
3. Il choisit ensuite :
   - **Relancer** les dés restants pour tenter d'augmenter les points du tour ;
   - **Encaisser** : les points du tour s'ajoutent à son score total et la main passe.
4. Les combinaisons ne se forment **qu'à l'intérieur d'un même lancer** : un dé mis de côté
   ne peut pas être combiné avec les dés d'un lancer suivant.

### Farkle
Si un lancer ne contient **aucun dé qui marque**, c'est un **Farkle** : le joueur perd
**tous les points accumulés pendant ce tour** et la main passe.

### Main chaude (hot dice)
Si les **6 dés** ont été mis de côté et qu'ils marquent tous, le joueur peut **relancer les 6 dés**
et continuer à accumuler des points dans le même tour.

## Tableau des scores

| Combinaison                    | Points                          |
|--------------------------------|---------------------------------|
| Un 1                           | 100                             |
| Un 5                           | 50                              |
| Trois 1                        | 1000                            |
| Trois 2 / 3 / 4 / 5 / 6        | 200 / 300 / 400 / 500 / 600     |
| Quatre identiques              | score du brelan × 2             |
| Cinq identiques                | score du brelan × 4             |
| Six identiques                 | score du brelan × 8             |
| Suite 1-2-3-4-5                | 500                             |
| Suite 2-3-4-5-6                | 750                             |
| Suite 1-2-3-4-5-6              | 1500                            |

Règle des dés identiques : **chaque dé identique au-delà du troisième double le score**.

Plusieurs combinaisons peuvent être mises de côté en même temps : leurs points s'additionnent.

## Exemples
| Lancer          | Meilleure mise de côté      | Points                 |
|-----------------|-----------------------------|------------------------|
| 1 5 2 3 4 6     | 1 2 3 4 5 6 (suite)         | 1500                   |
| 1 1 1 5 2 3     | 1 1 1 + 5                   | 1000 + 50 = 1050       |
| 4 4 4 4 2 3     | 4 4 4 4                     | 400 × 2 = 800          |
| 5 5 5 5 5 2     | 5 5 5 5 5                   | 500 × 4 = 2000         |
| 1 1 1 1 1 1     | 1 1 1 1 1 1                 | 1000 × 8 = 8000        |
| 2 2 2 3 3 3     | 2 2 2 + 3 3 3               | 200 + 300 = 500        |
| 1 2 3 4 5 5     | 1 2 3 4 5 + 5               | 500 + 50 = 550         |
| 2 3 4 6 6 3     | aucune                      | **Farkle !**           |

Il n'est pas obligatoire de garder tous les dés qui marquent : avec `1 5 2 3 4 6`, on peut aussi
ne garder que le `1` (100 points) pour relancer 5 dés.

## Fin de partie
Dès qu'un joueur atteint ou dépasse le score cible en encaissant, il gagne la partie.

## Hors périmètre
Les dés spéciaux (pipés, porte-bonheur…) et les badges du jeu Kingdom Come: Deliverance
ne sont pas implémentés.
