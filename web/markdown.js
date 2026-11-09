// Mini-rendu Markdown pour le panneau des règles (titres, paragraphes,
// listes imbriquées, tableaux, gras, code). Le texte est échappé.

const HEADING = /^(#{1,6})\s+(.*)$/;
const LIST_ITEM = /^(\s*)(\d+\.|[-*])\s+(.*)$/;
const TABLE_ROW = /^\s*\|/;
const TABLE_SEPARATOR = /^\s*\|?\s*:?-{3,}/;

function escapeHtml(text) {
  return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function inline(text) {
  return escapeHtml(text.trim())
    .replace(/`([^`]+)`/g, "<code>$1</code>")
    .replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
}

function cells(row) {
  return row.trim().replace(/^\|/, "").replace(/\|$/, "").split("|").map(inline);
}

function renderTable(rows) {
  const [head, , ...body] = rows;
  const th = cells(head).map((c) => `<th>${c}</th>`).join("");
  const trs = body
    .map((r) => `<tr>${cells(r).map((c) => `<td>${c}</td>`).join("")}</tr>`)
    .join("");
  return `<table><thead><tr>${th}</tr></thead><tbody>${trs}</tbody></table>`;
}

// Construit l'arbre des éléments de liste d'après leur indentation.
function listTree(items) {
  const root = { children: [] };
  const stack = [{ indent: -1, node: root }];
  for (const item of items) {
    while (stack.at(-1).indent >= item.indent) stack.pop();
    stack.at(-1).node.children.push(item);
    stack.push({ indent: item.indent, node: item });
  }
  return root.children;
}

function renderList(nodes) {
  const tag = nodes[0].ordered ? "ol" : "ul";
  const lis = nodes
    .map((n) => `<li>${inline(n.text)}${n.children.length ? renderList(n.children) : ""}</li>`)
    .join("");
  return `<${tag}>${lis}</${tag}>`;
}

export function renderMarkdown(md) {
  const lines = md.replace(/\r\n?/g, "\n").split("\n");
  const out = [];
  let i = 0;
  while (i < lines.length) {
    const line = lines[i];
    if (!line.trim()) {
      i++;
      continue;
    }

    const heading = line.match(HEADING);
    if (heading) {
      const level = heading[1].length;
      out.push(`<h${level}>${inline(heading[2])}</h${level}>`);
      i++;
      continue;
    }

    if (TABLE_ROW.test(line) && TABLE_SEPARATOR.test(lines[i + 1] ?? "")) {
      const rows = [];
      while (i < lines.length && TABLE_ROW.test(lines[i])) rows.push(lines[i++]);
      out.push(renderTable(rows));
      continue;
    }

    if (LIST_ITEM.test(line)) {
      const items = [];
      while (i < lines.length && lines[i].trim()) {
        const m = lines[i].match(LIST_ITEM);
        if (m) {
          items.push({ indent: m[1].length, ordered: /\d/.test(m[2]), text: m[3], children: [] });
        } else if (/^\s/.test(lines[i])) {
          items.at(-1).text += " " + lines[i].trim(); // ligne de continuation
        } else {
          break;
        }
        i++;
      }
      out.push(renderList(listTree(items)));
      continue;
    }

    const para = [];
    while (
      i < lines.length &&
      lines[i].trim() &&
      !HEADING.test(lines[i]) &&
      !LIST_ITEM.test(lines[i]) &&
      !TABLE_ROW.test(lines[i])
    ) {
      para.push(lines[i++].trim());
    }
    out.push(`<p>${inline(para.join(" "))}</p>`);
  }
  return out.join("");
}
