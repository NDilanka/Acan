#!/usr/bin/env python3
"""Replay pinned TypeScript merge histories without checking out or editing source."""

import argparse
import collections
import difflib
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile


CORPUS = [
    ("kvarm", "/tmp/kvarm", "c80253b14bbf2c16d67461f6e4dbe58b2e1d98e4"),
    ("hono", "https://github.com/honojs/hono.git", "5ca49b0720755c585c9a9835f0a479ceea36fb52"),
    ("trpc", "https://github.com/trpc/trpc.git", "6d92747fa267c51829ec99242be15e0ed968d0de"),
]
PARSER_VERSION = "5.9.3"
PARSER_JS = r"""
const ts = require(process.argv[1]);
const readline = require('node:readline');
const crypto = require('node:crypto');
function parse(text, path) {
  const sf = ts.createSourceFile(path, text, ts.ScriptTarget.Latest, true,
    path.endsWith('.tsx') ? ts.ScriptKind.TSX : ts.ScriptKind.TS);
  const byte = pos => Buffer.byteLength(text.slice(0, pos), 'utf8');
  const span = n => [byte(n.getFullStart()), byte(n.end)];
  const tokenHash = source => {
    const scanner = ts.createScanner(ts.ScriptTarget.Latest, false,
      ts.LanguageVariant.Standard, source);
    const parts = [];
    for (let k = scanner.scan(); k !== ts.SyntaxKind.EndOfFileToken; k = scanner.scan()) {
      if (k !== ts.SyntaxKind.WhitespaceTrivia && k !== ts.SyntaxKind.NewLineTrivia)
        parts.push([k, scanner.getTokenText()]);
    }
    return crypto.createHash('sha256').update(JSON.stringify(parts)).digest('hex');
  };
  const regions = [], imports = [], slots = [];
  const names = new Map();
  const declaration = n => ts.isVariableStatement(n) || ts.isFunctionDeclaration(n)
    || ts.isClassDeclaration(n) || ts.isInterfaceDeclaration(n)
    || ts.isTypeAliasDeclaration(n) || ts.isEnumDeclaration(n) || ts.isModuleDeclaration(n);
  function label(n) {
    if (ts.isVariableStatement(n))
      return n.declarationList.declarations.map(d => d.name.getText(sf)).join(',');
    return n.name ? n.name.getText(sf) : '<anonymous>';
  }
  function region(id, kind, start, end) {
    regions.push({id, kind, start: byte(start), end: byte(end),
      line: sf.getLineAndCharacterOfPosition(start).line + 1});
  }
  for (const n of sf.statements) {
    if (ts.isImportDeclaration(n) || ts.isImportEqualsDeclaration(n)) {
      region('commons:imports', 'commons', n.getFullStart(), n.end);
      imports.push(n);
    } else if (!declaration(n)) {
      region('commons:statements', 'commons', n.getFullStart(), n.end);
    } else {
      const stem = `${ts.SyntaxKind[n.kind]}:${label(n)}`;
      const ordinal = names.get(stem) || 0;
      names.set(stem, ordinal + 1);
      const id = `${stem}#${ordinal}`;
      if (ts.isClassDeclaration(n)) {
        let cursor = n.getFullStart();
        const members = new Map();
        for (const member of n.members) {
          region(id + ':shell', 'declaration', cursor, member.getFullStart());
          const name = `${ts.SyntaxKind[member.kind]}:${label(member)}`;
          const number = members.get(name) || 0;
          members.set(name, number + 1);
          region(`${id}/${name}#${number}`, 'declaration', member.getFullStart(), member.end);
          cursor = member.end;
        }
        region(id + ':shell', 'declaration', cursor, n.end);
      } else region(id, 'declaration', n.getFullStart(), n.end);
    }
  }
  let cursor = 0;
  const covered = [...regions].sort((a,b) => a.start-b.start || a.end-b.end);
  for (const r of covered) {
    if (r.start > cursor) regions.push({id:'commons:trivia', kind:'commons',
      start:cursor, end:r.start, line:1});
    cursor = Math.max(cursor, r.end);
  }
  if (cursor < Buffer.byteLength(text) || !regions.length)
    regions.push({id:'commons:trivia', kind:'commons', start:cursor,
      end:Buffer.byteLength(text), line:1});
  function importEntries(nodes) {
    const entries = [];
    for (const n of nodes) {
      if (!ts.isImportDeclaration(n)) return null;
      const module = n.moduleSpecifier.text;
      const clause = n.importClause;
      const extra = n.attributes ? tokenHash(n.attributes.getText(sf)) : '';
      const add = (kind, imported, local, type) => {
        const key = JSON.stringify([module, kind, imported, local, !!type, extra]);
        entries.push([key, key]);
      };
      if (!clause) add('side-effect', '', '', false);
      else {
        if (clause.name) add('default', 'default', clause.name.text, clause.isTypeOnly);
        const bindings = clause.namedBindings;
        if (bindings && ts.isNamespaceImport(bindings))
          add('namespace', '*', bindings.name.text, clause.isTypeOnly);
        if (bindings && ts.isNamedImports(bindings))
          for (const e of bindings.elements)
            add('named', e.propertyName?.text || e.name.text, e.name.text,
              clause.isTypeOnly || e.isTypeOnly);
      }
    }
    return entries;
  }
  if (imports.length) slots.push({kind:'imports',
    start:span(imports[0])[0], end:span(imports.at(-1))[1],
    entries:importEntries(imports)});
  function key(n) {
    if (ts.isStringLiteralLike(n) || ts.isNumericLiteral(n)) return n.text;
    if (ts.isIdentifier(n)) return n.text;
    return null;
  }
  function arrayKey(n) {
    if (ts.isStringLiteralLike(n) || ts.isNumericLiteral(n)) return n.text;
    if (ts.isCallExpression(n) && n.arguments.length &&
        ts.isStringLiteralLike(n.arguments[0])) return n.arguments[0].text;
    if (ts.isObjectLiteralExpression(n)) {
      for (const field of ['id','key','name','path']) {
        const p = n.properties.find(p => ts.isPropertyAssignment(p) && key(p.name) === field);
        if (p && (ts.isStringLiteralLike(p.initializer) || ts.isNumericLiteral(p.initializer)))
          return p.initializer.text;
      }
    }
    return null;
  }
  function visit(n) {
    if (ts.isObjectLiteralExpression(n) || ts.isArrayLiteralExpression(n)) {
      const object = ts.isObjectLiteralExpression(n);
      const children = object ? n.properties : n.elements;
      const entries = [];
      let valid = true;
      for (const e of children) {
        const k = object && (ts.isPropertyAssignment(e) || ts.isShorthandPropertyAssignment(e)
          || ts.isMethodDeclaration(e)) ? key(e.name) : object ? null : arrayKey(e);
        if (k === null) { valid = false; break; }
        entries.push([k, tokenHash(e.getText(sf))]);
      }
      slots.push({kind: object ? 'object' : 'array', start:byte(n.getStart(sf)),
        end:byte(n.end), entries:valid ? entries : null});
    }
    ts.forEachChild(n, visit);
  }
  visit(sf);
  return {version:ts.version, regions, slots, whitespaceHash:tokenHash(text),
    errors:sf.parseDiagnostics.map(d => ({code:d.code,
      line:sf.getLineAndCharacterOfPosition(d.start || 0).line + 1,
      message:ts.flattenDiagnosticMessageText(d.messageText, ' ')}))};
}
readline.createInterface({input:process.stdin}).on('line', line => {
  try {
    const {text,path} = JSON.parse(line);
    console.log(JSON.stringify(parse(text,path)));
  } catch (e) {
    console.log(JSON.stringify({fatal:String(e)}));
  }
});
"""


def run(args, **kwargs):
    return subprocess.run(args, check=True, stdout=subprocess.PIPE, **kwargs).stdout


def git(repo, *args):
    return run(["git", "-C", str(repo), *args])


class Parser:
    def __init__(self, root):
        module = root / "node_modules/typescript/lib/typescript.js"
        self.process = subprocess.Popen(
            ["node", "-e", PARSER_JS, str(module)], stdin=subprocess.PIPE,
            stdout=subprocess.PIPE, text=True, encoding="utf-8")
        self.cache = {}

    def parse(self, data, path):
        identity = (hashlib.sha256(data).hexdigest(), path.endswith(".tsx"))
        if identity not in self.cache:
            self.process.stdin.write(json.dumps({"text": data.decode("utf-8"), "path": path}) + "\n")
            self.process.stdin.flush()
            answer = json.loads(self.process.stdout.readline())
            if "fatal" in answer:
                raise RuntimeError(answer["fatal"])
            assert answer["version"] == PARSER_VERSION, answer["version"]
            self.cache[identity] = answer
        return self.cache[identity]

    def close(self):
        self.process.stdin.close()
        if self.process.wait(timeout=30):
            raise RuntimeError("TypeScript parser exited with an error")


class Blobs:
    def __init__(self, repo):
        self.process = subprocess.Popen(
            ["git", "-C", str(repo), "cat-file", "--batch"],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
        self.cache = {"0" * 40: b""}

    def get(self, oid):
        if oid not in self.cache:
            self.process.stdin.write(oid.encode() + b"\n")
            self.process.stdin.flush()
            header = self.process.stdout.readline().split()
            if len(header) != 3 or header[1] != b"blob":
                raise RuntimeError(f"Missing blob {oid}: {header}")
            self.cache[oid] = self.process.stdout.read(int(header[2]))
            assert self.process.stdout.read(1) == b"\n"
        return self.cache[oid]

    def close(self):
        self.process.stdin.close()
        assert self.process.wait(timeout=30) == 0


def edits(base, side):
    a, b = base.splitlines(keepends=True), side.splitlines(keepends=True)
    ao, bo = [0], [0]
    for line in a:
        ao.append(ao[-1] + len(line))
    for line in b:
        bo.append(bo[-1] + len(line))
    result = []
    for tag, i, j, k, l in difflib.SequenceMatcher(None, a, b, autojunk=False).get_opcodes():
        if tag == "equal":
            continue
        left, right = base[ao[i]:ao[j]], side[bo[k]:bo[l]]
        for kind, x, y, u, v in difflib.SequenceMatcher(None, left, right, autojunk=False).get_opcodes():
            if kind != "equal":
                result.append({"start": ao[i] + x, "end": ao[i] + y,
                               "side_start": bo[k] + u, "side_end": bo[k] + v})
    return result


def touched(change, regions):
    lo, hi = change["start"], change["end"]
    if lo == hi:
        found = [r for r in regions if r["start"] < lo < r["end"]]
        if not found:
            neighbors = [r for r in regions if r["start"] == lo or r["end"] == lo]
            identity = "commons:imports" if any(r["id"] == "commons:imports" for r in neighbors) \
                else "commons:insertions"
            return [{"id": identity, "kind": "commons", "start": lo, "end": lo, "line": 1}]
        return found
    return [r for r in regions if r["start"] < hi and lo < r["end"]]


def merge_lines(base, ours, theirs, root):
    with tempfile.TemporaryDirectory(prefix="merge-", dir=root) as directory:
        paths = [Path(directory) / name for name in ("ours", "base", "theirs")]
        for path, data in zip(paths, (ours, base, theirs)):
            path.write_bytes(data)
        result = subprocess.run(
            ["git", "merge-file", "--stdout", "--diff3", "--marker-size=15",
             "-L", "p1", "-L", "base", "-L", "p2", *map(str, paths)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        if result.returncode < 0 or result.returncode > 127:
            raise RuntimeError(result.stderr.decode())
        return result.returncode != 0, result.stdout


def conflict_hunks(merged, ours, base, theirs, changes=None):
    rows = merged.splitlines(keepends=True)
    positions = [0, 0, 0]
    hunks = []
    i = 0
    while i < len(rows):
        if rows[i].rstrip(b"\r\n") != b"<" * 15 + b" p1":
            i += 1
            continue
        chunks = [[], [], []]
        i += 1
        for side, marker in enumerate((b"|" * 15 + b" base", b"=" * 15, b">" * 15 + b" p2")):
            while i < len(rows) and rows[i].rstrip(b"\r\n") != marker:
                chunks[side].append(rows[i])
                i += 1
            if i == len(rows):
                raise RuntimeError("Unterminated git conflict marker")
            i += 1
        starts, ambiguous = [], []
        for side, data in enumerate((ours, base, theirs)):
            # Repeated chunks remain uncertain unless an exact insertion map
            # establishes the base anchor from both unique side chunks.
            chunk = b"".join(chunks[side])
            start = data.find(chunk, positions[side]) if chunk else positions[side]
            if start < 0:
                start = data.find(chunk)
            if start < 0 and chunk.endswith(b"\n"):
                chunk = chunk[:-1]
                start = data.find(chunk, positions[side])
            if start < 0:
                raise RuntimeError("Conflict chunk absent from input")
            ambiguous.append(not chunk or data.find(chunk, start + 1) >= 0)
            starts.append(start)
            positions[side] = start + len(chunk)
        if not chunks[1] and changes is not None and not ambiguous[0] and not ambiguous[2]:
            anchors = []
            for side_index, side_changes in zip((0, 2), changes):
                start = starts[side_index]
                end = start + len(b"".join(chunks[side_index]))
                anchors.append({c["start"] for c in side_changes
                                if c["start"] == c["end"]
                                and c["side_start"] <= start and end <= c["side_end"]})
            if len(anchors[0]) == 1 and anchors[0] == anchors[1]:
                starts[1] = next(iter(anchors[0]))
                positions[1] = starts[1]
                ambiguous[1] = False
        hunks.append({
            "base_start": starts[1], "base_end": starts[1] + len(b"".join(chunks[1])),
            "base_line": base[:starts[1]].count(b"\n") + 1,
            "p1_line": ours[:starts[0]].count(b"\n") + 1,
            "p2_line": theirs[:starts[2]].count(b"\n") + 1,
            "ambiguous_locations": ambiguous,
            "p1": b"".join(chunks[0]).decode("utf-8")[:500],
            "base": b"".join(chunks[1]).decode("utf-8")[:500],
            "p2": b"".join(chunks[2]).decode("utf-8")[:500],
        })
    return hunks


def apply_segment(base, side, changes, lo, hi):
    result = base[lo:hi]
    for c in sorted(changes, key=lambda c: (c["start"], c["end"]), reverse=True):
        assert lo <= c["start"] <= c["end"] <= hi
        result = result[:c["start"] - lo] + side[c["side_start"]:c["side_end"]] + result[c["end"] - lo:]
    return result


def splice(base, sides, changes, regions):
    replacements = []
    for side, side_changes in zip(sides, changes):
        spans = []
        for c in side_changes:
            owners = touched(c, regions)
            if not owners:
                return None, "unmapped_edit"
            spans.append((min(r["start"] for r in owners), max(r["end"] for r in owners)))
        merged = []
        for lo, hi in sorted(spans):
            if merged and lo <= merged[-1][1]:
                merged[-1] = (merged[-1][0], max(hi, merged[-1][1]))
            else:
                merged.append((lo, hi))
        for lo, hi in merged:
            inside = [c for c in side_changes if lo <= c["start"] and c["end"] <= hi]
            replacements.append((lo, hi, apply_segment(base, side, inside, lo, hi)))
    for i, (lo, hi, _) in enumerate(replacements):
        for other_lo, other_hi, _ in replacements[i + 1:]:
            if lo < other_hi and other_lo < hi or lo == hi == other_lo == other_hi:
                return None, "overlapping_splice_spans"
    result = base
    for lo, hi, payload in sorted(replacements, key=lambda r: (r[0], r[1]), reverse=True):
        result = result[:lo] + payload + result[hi:]
    return result, None


def side_position(position, changes, after=False):
    delta = 0
    for c in changes:
        lo, hi = c["start"], c["end"]
        size = c["side_end"] - c["side_start"]
        if hi < position or hi == position and (lo != hi or after):
            delta += size - (hi - lo)
        elif lo < position < hi:
            return None
    return position + delta


def addition_slots(parsed, changes):
    found = []
    for slot in parsed[0]["slots"]:
        if slot["entries"] is None:
            continue
        base_entries = dict(slot["entries"])
        if len(base_entries) != len(slot["entries"]):
            continue
        added = []
        for side, side_changes in zip(parsed[1:], changes):
            lo = side_position(slot["start"], side_changes)
            hi = side_position(slot["end"], side_changes, after=True)
            matches = [s for s in side["slots"] if s["kind"] == slot["kind"]
                       and s["start"] == lo and s["end"] == hi and s["entries"] is not None]
            if len(matches) != 1:
                break
            entries = dict(matches[0]["entries"])
            if len(entries) != len(matches[0]["entries"]) or any(entries.get(k) != v for k, v in base_entries.items()):
                break
            new_keys = set(entries) - set(base_entries)
            if not new_keys:
                break
            added.append(new_keys)
        if len(added) == 2 and added[0].isdisjoint(added[1]):
            found.append({**slot, "added_keys": [sorted(keys) for keys in added]})
    return found


def slots_cover_hunks(hunks, changes, candidates):
    kinds = set()
    if not hunks:
        return []
    for h in hunks:
        if h["ambiguous_locations"][1]:
            return []
        relevant = [c for side in changes for c in side
                    if c["start"] <= h["base_end"] and h["base_start"] <= c["end"]]
        if not relevant:
            return []
        for c in relevant:
            slots = [s for s in candidates if s["start"] <= c["start"] and c["end"] <= s["end"]]
            if not slots:
                return []
            kinds.add(min(slots, key=lambda s: s["end"] - s["start"])["kind"])
    return sorted(kinds)


def analyze(base, ours, theirs, actual, path, parser, root):
    conflict, merged = merge_lines(base, ours, theirs, root)
    result = {"line_conflict": conflict, "line_merge_matches_actual": merged == actual}
    parsed = [parser.parse(data, path) for data in (base, ours, theirs)]
    result["parse_errors"] = [p["errors"] for p in parsed]
    result["hunks"] = conflict_hunks(merged, ours, base, theirs) if conflict else []
    if any(p["errors"] for p in parsed):
        result["overlap"] = "unknown_parse"
        return result
    changes = [edits(base, side) for side in (ours, theirs)]
    if conflict:
        result["hunks"] = conflict_hunks(merged, ours, base, theirs, changes)
    regions = parsed[0]["regions"]
    for side, side_changes in zip((ours, theirs), changes):
        assert apply_segment(base, side, side_changes, 0, len(base)) == side
        one_side, refusal = splice(base, (side,), [side_changes], regions)
        assert refusal is None and one_side == side, "Region reconstruction lost a side's bytes"
    owners = [{r["id"] for c in side for r in touched(c, regions)} for side in changes]
    common = owners[0] & owners[1]
    kind = "same_declaration" if any(not r.startswith("commons:") for r in common) \
        else "commons_only" if common else "region_disjoint"
    result.update(overlap=kind, shared_regions=sorted(common),
                  changed_regions=[sorted(o) for o in owners], changed_ranges=changes)
    result["slots_kinds"] = slots_cover_hunks(result["hunks"], changes, addition_slots(parsed, changes))
    result["slots_footprint_kinds"] = slots_cover_hunks(result["hunks"], changes, parsed[0]["slots"])
    if kind == "region_disjoint":
        composed, refusal = splice(base, (ours, theirs), changes, regions)
        if refusal:
            result["splice"] = {"built": False, "reason": refusal}
        else:
            check = parser.parse(composed, path)
            reference = parser.parse(actual, path)
            match = "identical" if composed == actual else "whitespace_only" \
                if check["whitespaceHash"] == reference["whitespaceHash"] else "differs"
            result["splice"] = {"built": True, "parses": not check["errors"],
                                "errors": check["errors"], "match": match,
                                "sha256": hashlib.sha256(composed).hexdigest()}
    return result


def changed_files(repo, base, parent):
    raw = git(repo, "diff-tree", "-r", "--no-commit-id", "--raw", "--no-renames",
              "--no-abbrev", "-z", base, parent, "--", "*.ts", "*.tsx").split(b"\0")
    result = {}
    for i in range(0, len(raw) - 1, 2):
        header = raw[i].decode().split()
        result[raw[i + 1].decode()] = {"base": header[2], "side": header[3], "status": header[4]}
    return result


def tree_files(repo, commit):
    result = {}
    for item in git(repo, "ls-tree", "-r", "-z", commit).split(b"\0"):
        if item:
            head, path = item.split(b"\t", 1)
            result[path.decode()] = head.split()[2].decode()
    return result


def totals(pairs):
    count = collections.Counter()
    for p in pairs:
        count["same_file_pairs"] += 1
        count["overlap_" + p["overlap"]] += 1
        if p["line_conflict"]:
            count["line_conflicts"] += 1
            count["conflict_" + p["overlap"]] += 1
            if p.get("slots_kinds"):
                count["slots_candidates"] += 1
                for kind in p["slots_kinds"]:
                    count["slots_" + kind] += 1
            if p.get("slots_footprint_kinds"):
                count["slots_footprint_pairs"] += 1
        else:
            count["line_clean"] += 1
        if "splice" in p:
            count["splice_eligible"] += 1
            if p["splice"]["built"]:
                count["splice_built"] += 1
                if p["splice"]["parses"]:
                    count["splice_parses"] += 1
                count["splice_" + p["splice"]["match"]] += 1
                if p["line_conflict"]:
                    count["conflict_splice_built"] += 1
                    count["conflict_splice_" + p["splice"]["match"]] += 1
                    if p["splice"]["parses"]:
                        count["conflict_splice_parses"] += 1
            else:
                count["splice_refused_" + p["splice"]["reason"]] += 1
    pairs_n, conflicts_n = count["same_file_pairs"], count["line_conflicts"]
    return {"counts": dict(sorted(count.items())), "rates": {
        "line_conflicts_per_same_file_pair": conflicts_n / pairs_n if pairs_n else None,
        "region_disjoint_line_conflicts_per_same_file_pair": count["conflict_region_disjoint"] / pairs_n if pairs_n else None,
        "slots_candidates_per_line_conflict": count["slots_candidates"] / conflicts_n if conflicts_n else None,
        "splice_parse_success": count["splice_parses"] / count["splice_built"] if count["splice_built"] else None,
        "splice_exact_or_whitespace_match": (count["splice_identical"] + count["splice_whitespace_only"]) / count["splice_built"] if count["splice_built"] else None,
    }}


def replay(name, source, head, root, parser, max_merges):
    repo = root / name
    rows = git(repo, "rev-list", "--merges", "--parents", head).decode().splitlines()
    if max_merges:
        rows = rows[:max_merges]
    pairs, merges, skipped = [], [], []
    blobs = Blobs(repo)
    try:
        for row in rows:
            merge, *parents = row.split()
            if len(parents) != 2:
                skipped.append({"merge": merge, "reason": "not_two_parents", "parents": parents})
                continue
            base_result = subprocess.run(["git", "-C", str(repo), "merge-base", "--all", *parents],
                                         stdout=subprocess.PIPE, check=False)
            if base_result.returncode not in (0, 1):
                raise RuntimeError(f"merge-base failed for {merge}")
            bases = base_result.stdout.decode().splitlines()
            if len(bases) != 1:
                skipped.append({"merge": merge, "reason": "not_one_merge_base", "bases": bases})
                continue
            base = bases[0]
            sides = [changed_files(repo, base, p) for p in parents]
            paths = sorted(sides[0].keys() & sides[1].keys())
            merges.append({"merge": merge, "parents": parents, "base": base, "same_file_pairs": len(paths),
                           "parent_ts_paths": [len(side) for side in sides],
                           "subject": git(repo, "show", "-s", "--format=%s", merge).decode().strip()})
            if not paths:
                continue
            actual_tree = tree_files(repo, merge)
            for path in paths:
                oid_b = sides[0][path]["base"]
                assert oid_b == sides[1][path]["base"]
                oids = [oid_b, sides[0][path]["side"], sides[1][path]["side"],
                        actual_tree.get(path, "0" * 40)]
                data = [blobs.get(oid) for oid in oids]
                evidence = {"repo": name, "merge": merge, "parents": parents, "base": base,
                            "path": path, "blob_oids": oids,
                            "statuses": [side[path]["status"] for side in sides],
                            "unique_key": hashlib.sha256(json.dumps(
                                [name, path, oid_b, sorted(oids[1:3])]).encode()).hexdigest()}
                try:
                    evidence.update(analyze(*data, path, parser, root))
                except UnicodeDecodeError:
                    conflict, _ = merge_lines(*data[:3], root)
                    evidence.update(line_conflict=conflict, overlap="unknown_utf8")
                pairs.append(evidence)
            print(f"{name} {len(merges)}/{len(rows)} merges, {len(pairs)} pairs", file=sys.stderr, flush=True)
    finally:
        blobs.close()
    unique = list({p["unique_key"]: p for p in pairs}.values())
    return {"name": name, "source": source, "head": head,
            "reachable_commits": int(git(repo, "rev-list", "--count", head)),
            "reachable_merges": int(git(repo, "rev-list", "--count", "--merges", head)),
            "summary": totals(pairs), "unique_blob_triples_summary": totals(unique),
            "merges": merges, "skipped_merges": skipped, "pairs": pairs}


def prepare(root, selected):
    root.mkdir(parents=True, exist_ok=True)
    for name, source, head in selected:
        repo = root / name
        if not repo.exists():
            command = ["git", "clone", "--no-checkout"]
            if name == "kvarm":
                command += ["--no-hardlinks"]
            command += [source, str(repo)]
            subprocess.run(command, check=True)
        git(repo, "cat-file", "-e", head + "^{commit}")
    module = root / "node_modules/typescript/lib/typescript.js"
    if not module.exists():
        subprocess.run(["npm", "install", "--prefix", str(root), "--no-audit",
                        "--no-fund", "typescript@" + PARSER_VERSION], check=True)


def self_test(parser, root):
    cases = [
        ("adjacent declarations", "const a = 1; const b = 1;\n",
         "const a = 2; const b = 1;\n", "const a = 1; const b = 3;\n",
         "const a = 2; const b = 3;\n", True, "region_disjoint", []),
        ("class members", "class C { a() { return 1; } b() { return 1; } }\n",
         "class C { a() { return 2; } b() { return 1; } }\n",
         "class C { a() { return 1; } b() { return 3; } }\n",
         "class C { a() { return 2; } b() { return 3; } }\n",
         True, "region_disjoint", []),
        ("same declaration", "const a = 1;\n", "const a = 2;\n", "const a = 3;\n",
         "const a = 3;\n", True, "same_declaration", []),
        ("import additions", "import { a } from 'm';\n", "import { a, b } from 'm';\n",
         "import { a, c } from 'm';\n", "import { a, b, c } from 'm';\n",
         True, "commons_only", ["imports"]),
        ("object additions", "const r = { a: 1 };\n", "const r = { a: 1, b: 2 };\n",
         "const r = { a: 1, c: 3 };\n", "const r = { a: 1, b: 2, c: 3 };\n",
         True, "same_declaration", ["object"]),
        ("array additions", "const r = ['a'];\n", "const r = ['a', 'b'];\n",
         "const r = ['a', 'c'];\n", "const r = ['a', 'b', 'c'];\n",
         True, "same_declaration", ["array"]),
        ("unicode byte offsets", "const a = '雪'; const b = 1;\n",
         "const a = '雨'; const b = 1;\n", "const a = '雪'; const b = 3;\n",
         "const a = '雨'; const b = 3;\n", True, "region_disjoint", []),
        ("add add", "", "const a = 1;\n", "const b = 1;\n",
         "const a = 1;\nconst b = 1;\n", True, "commons_only", []),
        ("clean disjoint", "const a = 1;\n\nconst b = 1;\n",
         "const a = 2;\n\nconst b = 1;\n", "const a = 1;\n\nconst b = 3;\n",
         "const a = 2;\n\nconst b = 3;\n", False, "region_disjoint", []),
        ("duplicate registry key", "const r = { a: 1 };\n",
         "const r = { a: 1, b: 2 };\n", "const r = { a: 1, b: 3 };\n",
         "const r = { a: 1, b: 3 };\n", True, "same_declaration", []),
        ("empty base import hunk", "import { a } from 'a';\nimport { z } from 'z';\n",
         "import { a } from 'a';\nimport { b } from 'b';\nimport { z } from 'z';\n",
         "import { a } from 'a';\nimport { c } from 'c';\nimport { z } from 'z';\n",
         "import { a } from 'a';\nimport { b } from 'b';\nimport { c } from 'c';\nimport { z } from 'z';\n",
         True, "commons_only", ["imports"]),
    ]
    for label, b, o, t, m, conflict, overlap, slots in cases:
        result = analyze(*(s.encode() for s in (b, o, t, m)), "fixture.ts", parser, root)
        assert result["line_conflict"] == conflict, (label, result)
        assert result["overlap"] == overlap, (label, result)
        assert result.get("slots_kinds") == slots, (label, result)
        if overlap == "region_disjoint":
            assert result["splice"]["built"] and result["splice"]["parses"], (label, result)
            assert result["splice"]["match"] == "identical", (label, result)
        assert apply_segment(b.encode(), o.encode(), edits(b.encode(), o.encode()), 0, len(b.encode())) == o.encode()
        print("PASS " + label, file=sys.stderr)
    invalid = analyze(b"const a = ;\n", b"const a = 2;\n", b"const a = 3;\n",
                      b"const a = 3;\n", "fixture.ts", parser, root)
    assert invalid["overlap"] == "unknown_parse" and invalid["parse_errors"][0]
    print("PASS parse failure is unknown", file=sys.stderr)
    jsx = analyze(b"const a = <div>one</div>; const b = 1;\n",
                  b"const a = <div>two</div>; const b = 1;\n",
                  b"const a = <div>one</div>; const b = 2;\n",
                  b"const a = <div>two</div>; const b = 2;\n",
                  "fixture.tsx", parser, root)
    assert jsx["overlap"] == "region_disjoint" and jsx["splice"]["match"] == "identical"
    assert jsx["splice"]["parses"]
    print("PASS TSX regions", file=sys.stderr)
    return len(cases) + 2


def main():
    arg = argparse.ArgumentParser(description=__doc__)
    arg.add_argument("--root", type=Path, default=Path("/tmp/same-file-replay"))
    arg.add_argument("--output", type=Path, default=Path(__file__).with_name("results.json"))
    arg.add_argument("--only", nargs="+", choices=[r[0] for r in CORPUS])
    arg.add_argument("--max-merges", type=int, default=0, help="0 replays all reachable merge commits")
    arg.add_argument("--self-test", action="store_true", help="run fixtures without replaying history")
    args = arg.parse_args()
    root = args.root.resolve()
    if root != Path("/tmp/same-file-replay") and Path("/tmp/same-file-replay") not in root.parents:
        arg.error("--root must be under /tmp/same-file-replay")
    selected = [r for r in CORPUS if not args.only or r[0] in args.only]
    prepare(root, [] if args.self_test else selected)
    parser = Parser(root)
    try:
        tests = self_test(parser, root)
        if args.self_test:
            print(json.dumps({"self_tests_passed": tests}))
            return
        repositories = [replay(*repo, root, parser, args.max_merges) for repo in selected]
        pairs = [p for repo in repositories for p in repo["pairs"]]
        unique = list({p["unique_key"]: p for p in pairs}.values())
        output = {"schema_version": 1, "parser": "TypeScript " + PARSER_VERSION,
                  "git": run(["git", "--version"]).decode().strip(),
                  "python": sys.version.split()[0], "self_tests_passed": tests,
                  "max_merges_per_repository": args.max_merges,
                  "summary": totals(pairs), "unique_blob_triples_summary": totals(unique),
                  "modified_both_summary": totals([p for p in pairs if p["statuses"] == ["M", "M"]]),
                  "repositories": repositories}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(output, indent=2) + "\n")
        print(json.dumps({"output": str(args.output), **output["summary"]}, indent=2))
    finally:
        parser.close()


if __name__ == "__main__":
    main()
