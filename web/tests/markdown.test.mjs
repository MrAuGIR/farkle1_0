import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { renderMarkdown } from "../markdown.js";

test("titres", () => {
  assert.equal(renderMarkdown("# Règles"), "<h1>Règles</h1>");
  assert.equal(renderMarkdown("## But"), "<h2>But</h2>");
  assert.equal(renderMarkdown("### Farkle"), "<h3>Farkle</h3>");
});

test("paragraphes sur plusieurs lignes, séparés par une ligne vide", () => {
  assert.equal(
    renderMarkdown("Une ligne\nla suite.\n\nAutre paragraphe."),
    "<p>Une ligne la suite.</p><p>Autre paragraphe.</p>",
  );
});

test("gras et code en ligne", () => {
  assert.equal(
    renderMarkdown("Un **Farkle** avec `1 5 2`."),
    "<p>Un <strong>Farkle</strong> avec <code>1 5 2</code>.</p>",
  );
});

test("le HTML est échappé", () => {
  assert.equal(
    renderMarkdown("a < b & <script>"),
    "<p>a &lt; b &amp; &lt;script&gt;</p>",
  );
});

test("liste numérotée avec sous-liste et ligne de continuation", () => {
  const md = [
    "1. Lancer.",
    "2. Garder un dé",
    "   qui marque.",
    "3. Choisir :",
    "   - **Relancer** ;",
    "   - Encaisser.",
    "4. Fin.",
  ].join("\n");
  assert.equal(
    renderMarkdown(md),
    "<ol><li>Lancer.</li><li>Garder un dé qui marque.</li>" +
      "<li>Choisir :<ul><li><strong>Relancer</strong> ;</li><li>Encaisser.</li></ul></li>" +
      "<li>Fin.</li></ol>",
  );
});

test("tableau", () => {
  const md = "| A | B |\n|---|---|\n| Un 1 | 100 |\n| Un 5 | 50 |";
  assert.equal(
    renderMarkdown(md),
    "<table><thead><tr><th>A</th><th>B</th></tr></thead>" +
      "<tbody><tr><td>Un 1</td><td>100</td></tr><tr><td>Un 5</td><td>50</td></tr></tbody></table>",
  );
});

test("blocs enchaînés sans ligne vide", () => {
  assert.equal(
    renderMarkdown("## Titre\nTexte.\n- a\n- b"),
    "<h2>Titre</h2><p>Texte.</p><ul><li>a</li><li>b</li></ul>",
  );
});

test("rend docs/REGLES.md sans laisser de syntaxe markdown", () => {
  const md = readFileSync(new URL("../../docs/REGLES.md", import.meta.url), "utf8");
  const html = renderMarkdown(md);
  assert.match(html, /^<h1>Règles du Farkle/);
  assert.match(html, /<table>/);
  assert.match(html, /<ol>/);
  assert.doesNotMatch(html, /\*\*|\|---|^#|>#/);
});
