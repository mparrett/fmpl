// Shared client-side FMPL execution helpers, used by repl.html (deep links)
// and fmpl-guide.html (run-in-place). One copy of the REPL line-handling
// logic — keep this file dependency-free.

// Net `{`/`}` delta of a line, ignoring string literals and -- comments.
export function braceDelta(line) {
  let d = 0, inStr = false, esc = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (inStr) {
      if (esc) esc = false;
      else if (c === "\\") esc = true;
      else if (c === '"') inStr = false;
    } else if (c === '"') inStr = true;
    else if (c === "-" && line[i + 1] === "-") break;
    else if (c === "{") d++;
    else if (c === "}") d--;
  }
  return d;
}

// Multi-line `@ { ... }` match blocks are a REPL gotcha: typed line-by-line
// they route to the grammar engine. The documented idiom is one line with
// `;` between arms — the arms already carry the `;`, so join them.
// Returns units of { text, endIdx } where endIdx is the index of the last
// source line the unit consumed (for interleaving results back into a block).
export function joinMatchBlocks(lines) {
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    let line = lines[i];
    if (/@\s*\{\s*$/.test(line.trim())) {
      let depth = braceDelta(line);
      while (depth > 0 && i + 1 < lines.length) {
        i++;
        line += " " + lines[i].trim();
        depth += braceDelta(lines[i]);
      }
    }
    out.push({ text: line, endIdx: i });
  }
  return out;
}

// Multi-line if/then/else has the same line-at-a-time trouble: the branch
// evaluates early and the trailing `else` dangles. Join the chain.
export function joinIfChains(units) {
  const out = [];
  for (let i = 0; i < units.length; i++) {
    let { text, endIdx } = units[i];
    while (i + 1 < units.length &&
           (/\b(then|else)\s*$/.test(text.trim()) || /^\s*else\b/.test(units[i + 1].text))) {
      i++;
      text += " " + units[i].text.trim();
      endIdx = units[i].endIdx;
    }
    out.push({ text, endIdx });
  }
  return out;
}

export function normalizeUnits(code) {
  return joinIfChains(joinMatchBlocks(code.split("\n")));
}

// ---- doctest-style annotations -------------------------------------------
// Same conventions as fmpl-core/tests/doc_examples.rs: `-- Returns: v`,
// `-- => v`, `// => v`, trailing an expression or on their own line right
// after one. A trailing parenthetical is prose, not value.

const ANNOT = /^\s*(?:--|\/\/)\s*(?:Returns:|=>)\s*(.*)$/;

export function annotationValue(line) {
  const m = line.match(ANNOT);
  if (!m) return null;
  return m[1].replace(/\s+\([^()]*\)\s*$/, "").trim();
}

// Byte index where a `--` comment starts outside string literals, or -1.
function commentStart(line) {
  let inStr = false, esc = false;
  for (let i = 0; i < line.length; i++) {
    const c = line[i];
    if (inStr) {
      if (esc) esc = false;
      else if (c === "\\") esc = true;
      else if (c === '"') inStr = false;
    } else if (c === '"') inStr = true;
    else if (c === "-" && line[i + 1] === "-") return i;
  }
  return -1;
}

// Trailing annotation on a code line: `expr  -- => v` / `expr -- Returns: v`.
export function trailingAnnotation(line) {
  const i = commentStart(line);
  if (i < 0) return null;
  return annotationValue(line.slice(i));
}

// Top-level-order-insensitive compare for `%{...}` display forms — Value::Map
// is a HashMap, so key order is nondeterministic (same rule as the harness).
function valuesMatch(actual, expected) {
  if (actual === expected) return true;
  if (actual.startsWith("%{") && expected.startsWith("%{") &&
      actual.endsWith("}") && expected.endsWith("}")) {
    const entries = (s) => s.slice(2, -1).split(", ").sort().join(", ");
    return entries(actual) === entries(expected);
  }
  return false;
}

// Run one documented block against `vm` ({ eval, is_complete }). Returns
// events for rendering: { kind: "result", afterIdx, text, ok } where ok is
// true/false when the statement had a documented value, null otherwise.
// Annotation-only lines are skipped as code but claimed as expectations.
export function runBlock(vm, source) {
  const rawLines = source.split("\n");
  const codeLines = rawLines.map((l) => (annotationValue(l) !== null ? "" : l));
  const units = normalizeUnits(codeLines.join("\n"));

  const events = [];
  let buffer = "", bufferEnd = -1, bufferExpected = null;

  const flush = () => {
    if (buffer.trim() === "") { buffer = ""; return; }
    // An annotation-only line right after the statement is its expectation
    // (trailing annotations on the statement's own lines take precedence).
    // expectedIdx marks that line so a renderer can replace the documented
    // claim with the live result.
    let expected = bufferExpected, expectedIdx = null;
    if (expected === null) {
      for (let j = bufferEnd + 1; j < rawLines.length; j++) {
        const v = annotationValue(rawLines[j]);
        if (v !== null) { expected = v; expectedIdx = j; break; }
        if (rawLines[j].trim() !== "") break;
      }
    }
    const out = vm.eval(buffer);
    let ok = null;
    if (expected !== null && expected !== "") {
      const actual = out.replace(/^=>\s*/, "");
      ok = valuesMatch(actual, expected);
    }
    events.push({ kind: "result", afterIdx: bufferEnd, text: out, ok, expected, expectedIdx });
    buffer = ""; bufferExpected = null;
  };

  for (const { text, endIdx } of units) {
    if (buffer === "" && text.trim() === "") continue;
    buffer = buffer === "" ? text : buffer + "\n" + text;
    bufferEnd = endIdx;
    const t = trailingAnnotation(text);
    if (t !== null) bufferExpected = t;
    if (text.trim() === "" || vm.is_complete(buffer)) flush();
  }
  if (buffer !== "") flush();
  return events;
}
