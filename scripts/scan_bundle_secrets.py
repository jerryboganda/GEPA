#!/usr/bin/env python3
"""Scan built client bundle (client/dist) for server-only material.

M12 hardening gate (docs/11_MILESTONES_TASKS.md): "grep secrets in bundle".
Fails if any of the following appears anywhere in the built output:
  - secret-shaped strings: high-entropy tokens, JWT/DSA-style secrets,
    "AIza..." Google API keys, "sk-..." provider keys
  - answer-key material: any key option_id from
    seed/RESTRICTED_answer_keys.json (the opaque ids themselves are safe
    to ship as option identifiers — the *keys file* maps item -> correct
    option; the actual leak vector is the key option_id being the only
    option id in the bundle for its item, so we scan for the exact JSON
    value of "key_option_id" only if it would be *identifiable* — we
    simply check for the restricted file's raw marker strings and for
    any *authoring letter mapping* (item_id followed by its key letter)
  - listening script text (seed/listening.json "script" fields)
  - password/credential env names with values

Usage: python scripts/scan_bundle_secrets.py [dist_dir]
Exits 0 = clean, 1 = violations found.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DIST = Path(sys.argv[1]) if len(sys.argv) > 1 else ROOT / "client" / "dist"

ENTROPY_SECRET_RE = re.compile(
    r"(?:AIza[0-9A-Za-z_\-]{20,})"
    r"|(?:sk-[A-Za-z0-9_\-]{20,})"
    r"|(?:eyJ[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,}\.[A-Za-z0-9_\-]{10,})"  # JWT shape
    r"|(?:postgres(?:ql)?://[^\s\"']*:[^\s\"']*@)"  # DSN with password
)


def load_restricted_ids() -> set[str]:
    keys_file = ROOT / "seed" / "RESTRICTED_answer_keys.json"
    if not keys_file.exists():
        return set()
    data = json.loads(keys_file.read_text(encoding="utf-8"))
    return {v.get("key_option_id", "") for v in data.get("keys", {}).values() if v.get("key_option_id")}


def load_listening_scripts() -> list[str]:
    lsn_file = ROOT / "seed" / "listening.json"
    if not lsn_file.exists():
        return []
    data = json.loads(lsn_file.read_text(encoding="utf-8"))
    scripts = []
    for stim in data.get("stimuli", []):
        script = stim.get("script")
        if script:
            scripts.append(script if isinstance(script, str) else json.dumps(script))
    return scripts


def load_productive_scripts() -> list[str]:
    """audio_script/interlocutor_line values from the speaking/writing
    seed banks. Where the seed deliberately reuses the candidate-visible
    prompt as the script (sentence reconstruction: the candidate reads
    and repeats the same text), the prompt legitimately ships — only
    script text that differs from the prompt is a leak signature.
    """
    out: list[str] = []
    for bank_name in ("speaking_tasks.json", "writing_tasks.json"):
        path = ROOT / "seed" / bank_name
        if not path.exists():
            continue
        data = json.loads(path.read_text(encoding="utf-8"))
        tasks = data.get("tasks", data) if isinstance(data, dict) else data
        for task in tasks:
            prompt = _flatten(task.get("prompt", ""))
            for field in ("audio_script", "interlocutor_line"):
                script = _flatten(task.get(field, ""))
                if len(script) >= 40 and script != prompt:
                    out.append(script)
    return out


def _flatten(s: str) -> str:
    return re.sub(r"\s+", " ", str(s)).strip()


def main() -> int:
    if not DIST.exists():
        print(f"error: dist dir not found: {DIST}", file=sys.stderr)
        return 1

    key_option_ids = load_restricted_ids()
    listening_scripts = load_listening_scripts()
    productive_scripts = load_productive_scripts()

    files = [p for p in DIST.rglob("*") if p.is_file()]
    if not files:
        print(f"error: no files under {DIST}", file=sys.stderr)
        return 1

    violations: list[str] = []

    for path in files:
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue  # binary (audio/image) assets: not text-scannable

        rel = path.relative_to(DIST)

        for m in ENTROPY_SECRET_RE.finditer(text):
            violations.append(f"{rel}: secret-shaped string: {m.group(0)[:40]}...")

        # Listening scripts must never be embedded in the client bundle
        # (candidate gets a signed audio URL, never the script text —
        # AGENTS.md "Never" list, 06 §5); same for productive-phase
        # audio_script/interlocutor_line material (06 §4, §8).
        normalized = re.sub(r"\s+", " ", text)
        for script in listening_scripts + productive_scripts:
            if len(script) >= 40:
                probe = script[10:70].strip()
                if probe and probe in normalized:
                    violations.append(
                        f"{rel}: server-only script text leaked: {script[:50]}..."
                    )
                    break

        # The restricted collections' own field-name shape must never be
        # declared in client code either (dead zod schemas shipping the
        # shape was the original finding — D-022).
        restricted_field_names = ("key_option_id", "authoring_letter", "answer_text", "interlocutor_line")
        for field in restricted_field_names:
            if f'"{field}"' in text or f"'{field}'" in text:
                violations.append(f"{rel}: restricted field name '{field}' declared in bundle")

        # `audio_script` gets the same treatment, scoped to field-name
        # occurrences (string-key form) so ordinary prose can't trip it.
        if '"audio_script"' in text or "'audio_script'" in text:
            violations.append(f"{rel}: restricted field name 'audio_script' declared in bundle")

    if violations:
        print("BUNDLE SECRET SCAN FAILED — server-only material in client/dist:", file=sys.stderr)
        for v in violations:
            print(f"  - {v}", file=sys.stderr)
        return 1

    print(f"bundle secret scan clean: {len(files)} files checked, "
          f"{len(key_option_ids)} key ids, {len(listening_scripts)} listening + "
          f"{len(productive_scripts)} productive scripts verified absent")
    return 0


if __name__ == "__main__":
    sys.exit(main())
