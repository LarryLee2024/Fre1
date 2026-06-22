#!/usr/bin/env python3
"""
Engramory index-size guard — a Claude Code PreToolUse hook.

Keeps the memory index (MEMORY.md) from growing past the window the host actually
loads. Claude Code loads the first 200 lines / 25 KB of MEMORY.md at the start of
every conversation (documented behavior; configurable below for other hosts), so
anything past EITHER limit silently stops being recalled. Lines alone are not
enough: an index can be well under the line cap yet over the byte cap because its
lines are long (content leaked into the index). This hook predicts the index size
*after* the pending edit on BOTH dimensions, then:

  - over a HARD cap AND the edit GROWS the index -> DENY (compact first).
  - over a cap but the edit SHRINKS/keeps size (compaction in progress), or only
    over the WARN cap -> inject a non-blocking nudge to compact via
    additionalContext with NO permissionDecision.
  - otherwise -> silent pass-through.

Shrinking edits are always allowed even while still over a cap, so the index can
be compacted incrementally (e.g. 210 -> 205 -> 198) instead of in one edit.

Edit/MultiEdit are predicted by *simulating* the sub-edits sequentially on a copy
of the current text.

Fail-SAFE, not fail-open, for the things that matter.

Wire it up in settings.local.json (PreToolUse, matcher "Edit|Write|MultiEdit"):
  "command": "python /ABSOLUTE/PATH/TO/engramory/hooks/engramory_index_guard.py"

Config via environment variables (all optional):
  ENGRAMORY_HARD          hard line ceiling,  default 200
  ENGRAMORY_WARN          soft line warning,  default 150
  ENGRAMORY_HARD_BYTES    hard byte ceiling,  default 25600  (25 KB)
  ENGRAMORY_WARN_BYTES    soft byte warning,  default 20480  (20 KB)
  ENGRAMORY_INDEX_NAME    index filename to guard, default "MEMORY.md"
  ENGRAMORY_INDEX_PATH    absolute path of the one index to guard
"""
import json
import os
import sys


def _allow_silently():
    sys.exit(0)


def _emit(decision=None, reason=None, context=None):
    hso = {"hookEventName": "PreToolUse"}
    if decision is not None:
        hso["permissionDecision"] = decision
    if reason is not None:
        hso["permissionDecisionReason"] = reason
    if context is not None:
        hso["additionalContext"] = context
    print(json.dumps({"hookSpecificOutput": hso}))
    sys.exit(0)


def _envint(name, default):
    raw = os.environ.get(name)
    if raw is None:
        return default
    try:
        val = int(raw.strip())
    except (TypeError, ValueError):
        return default
    return val if val > 0 else default


def _lines(text):
    if not text:
        return 0
    return text.count("\n") + (0 if text.endswith("\n") else 1)


def _bytes(text):
    return len(text.encode("utf-8"))


def _kb(n):
    return f"{n} B" if n < 1024 else f"{n / 1024:.1f} KB"


def _plural(n, word):
    return f"{n} {word}" if n == 1 else f"{n} {word}s"


def _which(p_lines, p_bytes, line_cap, byte_cap, cur_lines=None, cur_bytes=None):
    parts = []
    if p_lines > line_cap and (cur_lines is None or p_lines > cur_lines):
        parts.append(f"{_plural(p_lines, 'line')} > {line_cap}")
    if p_bytes > byte_cap and (cur_bytes is None or p_bytes > cur_bytes):
        parts.append(f"{_kb(p_bytes)} > {_kb(byte_cap)}")
    return " and ".join(parts)


def _apply_edits(current, edits):
    result = current
    for e in edits:
        old = e.get("old_string", "") or ""
        new = e.get("new_string", "") or ""
        if not old:
            continue
        if e.get("replace_all"):
            result = result.replace(old, new)
        elif result.count(old) == 1:
            result = result.replace(old, new, 1)
    return result


def main():
    data = json.loads(sys.stdin.read())

    tool = data.get("tool_name", "")
    ti = data.get("tool_input", {}) or {}
    file_path = ti.get("file_path", "") or ""
    if not file_path:
        _allow_silently()

    index_name = os.environ.get("ENGRAMORY_INDEX_NAME", "MEMORY.md")
    index_path = os.environ.get("ENGRAMORY_INDEX_PATH", "")

    if index_path:
        def _key(p):
            return os.path.normcase(os.path.realpath(p))
        if _key(file_path) != _key(index_path):
            _allow_silently()
    else:
        if os.path.basename(file_path).lower() != index_name.lower():
            _allow_silently()

    hard = _envint("ENGRAMORY_HARD", 200)
    warn = _envint("ENGRAMORY_WARN", 150)
    hard_b = _envint("ENGRAMORY_HARD_BYTES", 25600)
    warn_b = _envint("ENGRAMORY_WARN_BYTES", 20480)

    try:
        with open(file_path, "rb") as fh:
            raw = fh.read()
    except OSError:
        raw = b""
    current = raw.decode("utf-8", "replace")
    cur_lines = _lines(current)
    cur_bytes = len(raw)

    if tool == "Write":
        new_text = ti.get("content")
        if new_text is None:
            new_text = ti.get("file_text", "")
        result = new_text or ""
    elif tool in ("Edit", "MultiEdit"):
        edits = [ti] if tool == "Edit" else (ti.get("edits", []) or [])
        result = _apply_edits(current, edits)
    else:
        _allow_silently()

    p_lines = _lines(result)
    p_bytes = _bytes(result)

    over_hard = p_lines > hard or p_bytes > hard_b
    over_warn = p_lines > warn or p_bytes > warn_b
    worsens_cap = ((p_lines > hard and p_lines > cur_lines)
                   or (p_bytes > hard_b and p_bytes > cur_bytes))

    size = f"{_plural(p_lines, 'line')} / {_kb(p_bytes)}"
    caps = f"{hard} lines / {_kb(hard_b)}"

    if worsens_cap:
        which = _which(p_lines, p_bytes, hard, hard_b, cur_lines, cur_bytes)
        _emit(
            "deny",
            reason=(
                f"Engramory: this edit would GROW the memory index to {size}, past the "
                f"host's load window ({which}; cap {caps}). Beyond it the "
                f"tail of the index is silently truncated and those memories stop being "
                f"recalled. Do NOT append. Run the compaction procedure: "
                f"(1) pointer-ify prose that leaked into index lines, (2) merge duplicate "
                f"pointers, (3) archive cold/superseded memories. Shrinking edits ARE "
                f"allowed, so you can compact step by step; only growth while over the cap "
                f"is blocked. If you still cannot get under the cap, ask the user which "
                f"memories to retire. "
                f"(If this file is NOT your memory index, set the ENGRAMORY_INDEX_PATH env "
                f"var to your real index's absolute path so the hook only gates that file.)"
            ),
        )
    elif over_hard:
        _emit(
            context=(
                f"Engramory: index will be {size}, still over the load window "
                f"({_which(p_lines, p_bytes, hard, hard_b)}; cap {caps}), but this edit "
                f"shrinks/keeps it so it's allowed. Keep compacting until it's at or "
                f"under {caps}."
            ),
        )
    elif over_warn:
        _emit(
            context=(
                f"Engramory: index will be {size} — over "
                f"{_which(p_lines, p_bytes, warn, warn_b)} (caps {caps}). Allowed, but tell "
                f"the user the index is getting long and offer a compaction pass."
            ),
        )
    else:
        _allow_silently()


if __name__ == "__main__":
    try:
        main()
    except SystemExit:
        raise
    except Exception:
        sys.exit(0)
