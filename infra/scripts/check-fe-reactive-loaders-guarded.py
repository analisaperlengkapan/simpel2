#!/usr/bin/env python3
"""Fail when a frontend loader re-runs on a filter change but can publish a STALE answer.

THE DEFECT CLASS

    Effect::new(move |_| {
        let filter = read(search.get(), kondisi.get(), page.get());   // reactive reads
        spawn_local(async move {
            let result = fetch(&filter).await;                        // 10 s on 624 533 rows
            set_items.set(result.data);                               // published unconditionally
        });
    });

Every filter change starts another request, HTTP answers do not come back in the
order the questions were asked, and the answer that lands LAST wins -- not the one
that was asked last.

Measured on staging, 2026-11-06 (Bank Aset list): the search was sent twice (the
input event and the "Terapkan" click), took ~10 s, and the second answer landed
after the "Rusak Ringan" answer and replaced it. The dropdown said Rusak Ringan, the
table listed five Baik rows -- 4 of 12 runs, always the same event order, and green
in CI because an empty database answers in milliseconds.

The repo already HAD the cure, once: `urutan` in `pemakaian_bmn_form.rs`, with a
comment explaining exactly this. It was applied to the one form that was hit and to
none of the seven list loaders with the identical shape -- the usual failure of a fix
that is a pattern to remember instead of a rule to enforce
(project_gate_scope_must_be_derived). The cure is now `lib_ui::hooks::LatestOnly`;
this script makes it mandatory.

WHAT THIS CHECKS, precisely. For every `Effect::new(...)` under `antarmuka/*/src`:

  * it READS a signal (`.get()` / `.with(`) in its synchronous part (before
    `spawn_local`), other than `reload_tick` -- so a filter, page or search change
    re-runs it; and
  * inside `spawn_local` it PUBLISHES (`.set(` / `.update(`) after an `.await`;

then the effect must carry a staleness guard: `.is_current(` (LatestOnly), or the
value-equality re-check `.get_untracked()` that a debounce uses.

WHAT IT DOES NOT CHECK. That the guard sits on the right line, or that every signal
written after the await is covered: that needs the data flow, and a wrong derivation
is worse than none. It also ignores effects that depend on `reload_tick` alone (a
refresh button, a timer): nothing in them can be out of date relative to a CHANGED
filter. And it ignores ACTION effects: this repo's idiom for "the user pressed a
button" is a `*_trigger` counter whose effect runs the mutation (portal: create, save,
reset, toggle, passkey). A request that already committed on the server must still
report its outcome, so dropping its answer would hide a result rather than prevent a
wrong one; those effects have a different concern (double submit) and are skipped by
name. The presence rule has no false negatives for the shape above and the self-test
pins both directions.

The scan strips comments and string literals with a single pass that understands
nested block comments, raw strings and char literals. An earlier regex stripper in
this repo read a quote inside a comment as the start of a string and ate real code.

No YAML, no third-party imports: the ARC runner image carries neither PyYAML nor yq
(project_arc_runner_image_missing_tools).

usage: check-fe-reactive-loaders-guarded.py            # scan antarmuka/*/src
       check-fe-reactive-loaders-guarded.py --self-test
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SRC_GLOB = "antarmuka/*/src/**/*.rs"

# Signals whose change means "refresh the SAME query" rather than "a filter moved".
REFRESH_ONLY = {"reload_tick"}
# `*_trigger` counters mark ACTION effects (a button ran a mutation), not loaders.
ACTION_TRIGGER_SUFFIX = "_trigger"
# What counts as a staleness guard inside the effect.
GUARD_MARKS = (".is_current(", ".get_untracked()")

# Effects that are exempt, by file, with the reason. A stale entry (the file no longer
# has a flagged effect) FAILS the run, so the list cannot outlive its reason.
ALLOW: dict[str, str] = {}


# --------------------------------------------------------------------------- lexing
def blank_comments_and_strings(src: str) -> str:
    """Replace comments and the CONTENT of string/char literals with spaces.

    Same length and same newlines as the input, so offsets and line numbers still
    line up. Handles `//`, nested `/* */`, "..." with escapes, r#"..."#, and tells a
    char literal ('x', '\\n') from a lifetime ('a).
    """
    out = list(src)
    n = len(src)
    i = 0

    def blank(a: int, b: int) -> None:
        for k in range(a, b):
            if out[k] != "\n":
                out[k] = " "

    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if c == "/" and nxt == "/":
            j = src.find("\n", i)
            j = n if j == -1 else j
            blank(i, j)
            i = j
        elif c == "/" and nxt == "*":
            depth, j = 1, i + 2
            while j < n and depth:
                if src.startswith("/*", j):
                    depth += 1
                    j += 2
                elif src.startswith("*/", j):
                    depth -= 1
                    j += 2
                else:
                    j += 1
            blank(i, j)
            i = j
        elif c == "r" and re.match(r'r#*"', src[i : i + 40] or ""):
            m = re.match(r'r(#*)"', src[i:])
            hashes = m.group(1)
            end = src.find('"' + hashes, i + len(m.group(0)))
            j = n if end == -1 else end + 1 + len(hashes)
            blank(i + len(m.group(0)), max(i + len(m.group(0)), j - 1 - len(hashes)))
            i = j
        elif c == '"':
            j = i + 1
            while j < n and src[j] != '"':
                j += 2 if src[j] == "\\" else 1
            blank(i + 1, j)
            i = j + 1
        elif c == "'":
            # char literal: 'x' or '\n' or '\u{..}'. A lifetime has no closing quote
            # right after one char, so it falls through untouched.
            m = re.match(r"'(?:\\.[^']*|[^\\'])'", src[i : i + 12])
            if m:
                blank(i + 1, i + len(m.group(0)) - 1)
                i += len(m.group(0))
            else:
                i += 1
        else:
            i += 1
    return "".join(out)


def balanced(src: str, open_idx: int) -> int:
    """Index just past the `)` matching the `(` at `open_idx` (strings already blanked)."""
    depth = 0
    for k in range(open_idx, len(src)):
        if src[k] == "(":
            depth += 1
        elif src[k] == ")":
            depth -= 1
            if depth == 0:
                return k + 1
    return len(src)


# --------------------------------------------------------------------------- analysis
SIGNAL_READ_RE = re.compile(r"\b([A-Za-z_]\w*)\s*\.\s*(?:get|with)\s*\(")
PUBLISH_RE = re.compile(r"\.\s*(?:set|update)\s*\(")


class Finding:
    def __init__(self, path: str, line: int, reads: list[str]):
        self.path, self.line, self.reads = path, line, reads

    def __str__(self) -> str:
        return (
            f"{self.path}:{self.line}: Effect reads {', '.join(self.reads)} and publishes "
            "after an `.await` with no staleness guard (use lib_ui::hooks::LatestOnly: "
            "`let t = latest.ticket();` before spawn_local, `if !latest.is_current(t) { return; }` "
            "after the await)"
        )


def unguarded_loaders(path: str, src: str) -> list[Finding]:
    clean = blank_comments_and_strings(src)
    found: list[Finding] = []
    for m in re.finditer(r"\bEffect\s*::\s*new\s*\(", clean):
        open_idx = m.end() - 1
        block = clean[open_idx : balanced(clean, open_idx)]
        sp = block.find("spawn_local")
        if sp == -1:
            continue
        sync_part, async_part = block[:sp], block[sp:]
        every_read = list(dict.fromkeys(SIGNAL_READ_RE.findall(sync_part)))
        if any(name.endswith(ACTION_TRIGGER_SUFFIX) for name in every_read):
            continue  # an action effect: runs a mutation when a button is pressed
        reads = [name for name in every_read if name not in REFRESH_ONLY]
        if not reads:
            continue  # re-runs only on refresh, or never: nothing can be stale vs a filter
        aw = async_part.find(".await")
        if aw == -1 or not PUBLISH_RE.search(async_part, aw):
            continue  # fires and forgets, or never publishes after waiting
        if any(mark in block for mark in GUARD_MARKS):
            continue
        found.append(Finding(path, src.count("\n", 0, open_idx) + 1, reads))
    return found


def scan() -> int:
    findings: list[Finding] = []
    flagged_files: set[str] = set()
    for p in sorted(ROOT.glob(SRC_GLOB)):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        got = unguarded_loaders(rel, p.read_text(encoding="utf-8"))
        if got:
            flagged_files.add(rel)
        if rel in ALLOW:
            continue
        findings.extend(got)

    stale = sorted(set(ALLOW) - flagged_files)
    for f in findings:
        print(f)
    for rel in stale:
        print(f"{rel}: ALLOW entry is stale (no flagged effect any more) — remove it: {ALLOW[rel]}")
    if findings or stale:
        print(f"\n{len(findings)} unguarded loader(s), {len(stale)} stale allow entr(y/ies).")
        return 1
    scanned = len(list(ROOT.glob(SRC_GLOB)))
    print(f"OK: every filter-driven loader in {scanned} frontend files carries a staleness guard.")
    return 0


# --------------------------------------------------------------------------- self-test
BAD = """
let (reload_tick, _) = signal(0);
Effect::new(move |_| {
    let _ = reload_tick.get();
    let f = Filter { search: search.get(), page: page.get() };
    set_loading.set(true);
    spawn_local(async move {
        let r = fetch(&f).await;
        set_items.set(r.data);
    });
});
"""
GOOD_TICKET = BAD.replace(
    "    spawn_local(async move {\n        let r = fetch(&f).await;\n",
    "    let t = latest.ticket();\n    spawn_local(async move {\n        let r = fetch(&f).await;\n"
    "        if !latest.is_current(t) { return; }\n",
)
GOOD_DEBOUNCE = """
Effect::new(move |_| {
    let pending = search_input.get();
    spawn_local(async move {
        TimeoutFuture::new(300).await;
        if search_input.get_untracked() == pending { set_q.set(pending); }
    });
});
"""
ACTION_EFFECT = """
Effect::new(move |_| {
    let count = save_trigger.get();
    if count == 0 { return; }
    spawn_local(async move {
        let req = Body { name: edit_name.get() };
        match api.save(&req).await { Ok(_) => set_msg.set(Some("ok")), Err(e) => set_err.set(Some(e)) }
    });
});
"""
KEYED_DETAIL_LOADER = """
Effect::new(move |_| {
    let _ = reload_tick.get();
    let id = id_signal.get();
    spawn_local(async move {
        match fetch_detail(&id).await { Ok(d) => set_detail.set(Some(d)), Err(e) => set_error.set(Some(e)) }
    });
});
"""
REFRESH_ONLY_SRC = """
Effect::new(move |_| {
    let _ = reload_tick.get();
    spawn_local(async move { let r = fetch().await; set_data.set(r); });
});
"""
MOUNT_ONLY = """
Effect::new(move |_| {
    spawn_local(async move { let r = fetch_options().await; set_opts.set(r); });
});
"""
IN_COMMENT_ONLY = """
// Effect::new(move |_| { let a = search.get(); spawn_local(async move { f().await; s.set(1); }); });
/* Effect::new(move |_| { let a = search.get(); spawn_local(async move { f().await; s.set(1); }); }); */
let msg = "Effect::new(move |_| { search.get(); spawn_local(async { f().await; s.set(1) }) })";
"""
QUOTE_IN_COMMENT_HIDES_REAL_CODE = """
// the user typed a " and then
Effect::new(move |_| {
    let a = search.get();
    spawn_local(async move { let r = fetch(a).await; set_items.set(r); });
});
// another " later
"""
GUARD_ONLY_IN_COMMENT = BAD.replace("let r = fetch(&f).await;", "let r = fetch(&f).await; // .is_current(t)")
NESTED_BLOCK_COMMENT = "/* outer /* inner */ still comment */\n" + BAD
LIFETIME_AND_CHAR = "fn f<'a>(x: &'a str) { let c = '\"'; }\n" + BAD


def self_test() -> int:
    cases = [
        ("unguarded filter loader is flagged", BAD, 1),
        ("LatestOnly ticket + is_current passes", GOOD_TICKET, 0),
        ("debounce equality re-check passes", GOOD_DEBOUNCE, 0),
        ("refresh-only effect is ignored", REFRESH_ONLY_SRC, 0),
        ("action effect (*_trigger + payload read) is ignored", ACTION_EFFECT, 0),
        ("loader keyed by the id in the URL is flagged", KEYED_DETAIL_LOADER, 1),
        ("mount-only effect (no reads) is ignored", MOUNT_ONLY, 0),
        ("code that lives only in comments/strings is ignored", IN_COMMENT_ONLY, 0),
        ("a quote inside a comment does not hide real code", QUOTE_IN_COMMENT_HIDES_REAL_CODE, 1),
        ("a guard named only in a comment does not count", GUARD_ONLY_IN_COMMENT, 1),
        ("nested block comments are stripped", NESTED_BLOCK_COMMENT, 1),
        ("lifetimes and a quote char literal do not derail the lexer", LIFETIME_AND_CHAR, 1),
    ]
    bad = 0
    for name, src, want in cases:
        got = len(unguarded_loaders("selftest.rs", src))
        ok = got == want
        bad += 0 if ok else 1
        print(f"{'ok  ' if ok else 'FAIL'}  {name}  (flagged {got}, want {want})")
    if bad:
        print(f"\nself-test FAILED: {bad} case(s) wrong")
        return 1
    print("\nself-test OK: both directions pinned")
    return 0


if __name__ == "__main__":
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    sys.exit(scan())
