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
    const startIdx = i;
    let line = lines[i];
    if (/@\s*\{\s*$/.test(line.trim())) {
      let depth = braceDelta(line);
      while (depth > 0 && i + 1 < lines.length) {
        i++;
        line += " " + lines[i].trim();
        depth += braceDelta(lines[i]);
      }
    }
    out.push({ text: line, startIdx, endIdx: i });
  }
  return out;
}

// Multi-line if/then/else has the same line-at-a-time trouble: the branch
// evaluates early and the trailing `else` dangles. Join the chain.
export function joinIfChains(units) {
  const out = [];
  for (let i = 0; i < units.length; i++) {
    let { text, startIdx, endIdx } = units[i];
    while (i + 1 < units.length &&
           (/\b(then|else)\s*$/.test(text.trim()) || /^\s*else\b/.test(units[i + 1].text))) {
      i++;
      text += " " + units[i].text.trim();
      endIdx = units[i].endIdx;
    }
    out.push({ text, startIdx, endIdx });
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

// Where a trailing annotation begins, for renderers that transform it in
// place: { at, value } or null when the line has no trailing annotation.
export function trailingAnnotationSplit(line) {
  const i = commentStart(line);
  if (i < 0) return null;
  const v = annotationValue(line.slice(i));
  return v === null ? null : { at: i, value: v };
}

// First line at/after fromIdx that starts the next statement group — skips
// blanks and annotation-only lines. Returns null when nothing remains.
// Used to place the cursor without involving the VM.
export function findNextStatement(lines, fromIdx) {
  for (let i = fromIdx; i < lines.length; i++) {
    if (lines[i].trim() === "") continue;
    if (annotationValue(lines[i]) !== null) continue;
    return i;
  }
  return null;
}

// True while src ends inside an unterminated /* */ block comment. The VM's
// is_complete says "complete" for a lone `/*` (lex error counts as complete
// so evaluation surfaces the error), so runners must track this themselves
// to avoid splitting a block comment across statements.
export function hasOpenBlockComment(src) {
  let inStr = false, esc = false, inLine = false, inBlock = false;
  for (let i = 0; i < src.length; i++) {
    const c = src[i];
    if (inBlock) {
      if (c === "*" && src[i + 1] === "/") { inBlock = false; i++; }
    } else if (inLine) {
      if (c === "\n") inLine = false;
    } else if (inStr) {
      if (esc) esc = false;
      else if (c === "\\") esc = true;
      else if (c === '"') inStr = false;
    } else if (c === '"') inStr = true;
    else if (c === "-" && src[i + 1] === "-") inLine = true;
    else if (c === "/" && src[i + 1] === "*") { inBlock = true; i++; }
  }
  return inBlock;
}

// A statement that is only comments (line and/or block). The CLI REPL
// produces no output for these; runners treat them as silent steps rather
// than evaluating them (a lone `--` comment is an eval error in the VM).
export function isCommentOnly(src) {
  const noBlocks = src.replace(/\/\*[\s\S]*?\*\//g, "");
  const noLines = noBlocks.split("\n").map((l) => {
    const i = commentStart(l);
    return i < 0 ? l : l.slice(0, i);
  }).join("\n");
  return noLines.trim() === "";
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

// Statement-by-statement runner over a documented block, against `vm`
// ({ eval, is_complete }). step() executes the next statement and returns
// an event { afterIdx, text, ok, expected, expectedIdx } — ok is true/false
// when the statement had a documented value (`-- Returns:` conventions),
// null otherwise; expectedIdx marks an annotation-only line a renderer may
// replace with the live result. Returns null when the block is exhausted.
export function createRunner(vm, source) {
  const rawLines = source.split("\n");
  const codeLines = rawLines.map((l) => (annotationValue(l) !== null ? "" : l));
  const units = normalizeUnits(codeLines.join("\n"));
  let i = 0;

  function step() {
    let buffer = "", bufferStart = -1, bufferEnd = -1, bufferExpected = null;
    while (i < units.length) {
      const { text, startIdx, endIdx } = units[i];
      i++;
      if (buffer === "" && text.trim() === "") continue;
      if (buffer === "") bufferStart = startIdx;
      buffer = buffer === "" ? text : buffer + "\n" + text;
      bufferEnd = endIdx;
      const t = trailingAnnotation(text);
      if (t !== null) bufferExpected = t;
      if (!hasOpenBlockComment(buffer) &&
          (text.trim() === "" || vm.is_complete(buffer))) break;
    }
    if (buffer.trim() === "") return null;

    // Comment-only statements advance the cursor without output.
    if (isCommentOnly(buffer)) {
      return { startIdx: bufferStart, afterIdx: bufferEnd, text: "", ok: null,
               expected: null, expectedIdx: null, silent: true };
    }

    // An annotation-only line right after the statement is its expectation
    // (trailing annotations on the statement's own lines take precedence).
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
    return { startIdx: bufferStart, afterIdx: bufferEnd, text: out, ok, expected, expectedIdx };
  }

  return {
    step,
    get done() { return units.slice(i).every((u) => u.text.trim() === ""); },
  };
}

// Run a whole documented block: drain a fresh runner, return all events.
export function runBlock(vm, source) {
  const runner = createRunner(vm, source);
  const events = [];
  for (let e = runner.step(); e !== null; e = runner.step()) events.push(e);
  return events;
}
