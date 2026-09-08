#!/usr/bin/env python3
"""
GEPA v2 — PDF -> machine-readable seed files.

Parses "GEPA_Master_Draft_v2_All_in_One_Review.pdf" (Pilot Item Bank + Keys) into:
  seed/language_systems.json      candidate-safe items (NO keys)
  seed/reading.json               stimuli + items (NO keys)
  seed/listening.json             admin scripts + items (NO keys) — scripts are admin assets
  seed/speaking_tasks.json        48 prompts, 7 task types, TTS scripts extracted
  seed/writing_tasks.json         24 tasks incl. 6 diagnostics
  seed/RESTRICTED_answer_keys.json   keys by opaque option identity  (SERVER-ONLY)
  seed/item_bank_review.md        human-readable bank (no keys)
  seed/RESTRICTED_keys_review.md  human-readable keys (SERVER-ONLY)
  seed/manifest.json              counts + sha256 checksums

Validation: counts and authoring key-letter distributions must match the figures printed in the
blueprint (LS 52 {A15,B16,C13,D8}; RD 42 / LSN 42 {A10,B14,C14,D4}; SPK 48; WRT 24).

Usage:  python scripts/parse_pdf_to_seed.py <pdf_path> <out_dir>
Needs:  poppler-utils (pdftotext) on PATH.
"""
import hashlib
import json
import os
import re
import subprocess
import sys
from collections import Counter, OrderedDict

BANDS = ["Pre-A1", "A1", "A2", "B1", "B2", "C1", "C2"]
BAND_FROM_CODE = {"PRE": "Pre-A1", "A1": "A1", "A2": "A2", "B1": "B1", "B2": "B2", "C1": "C1", "C2": "C2"}
FOOTER = re.compile(r"GEPA (Master Draft|Pilot Item Bank|Keys & Rubrics) v2 \| Review Edition v2 \| \d+")

ITEM_HDR = re.compile(r"^(LS|RD|LSN)-(PRE|A1|A2|B1|B2|C1|C2)-(\d{2}) \| (.*)$")
STIM_HDR = re.compile(r"^(RD|LSN)-(PRE|A1|A2|B1|B2|C1|C2)-S(\d) \| (.*)$")
OPT = re.compile(r"^([A-D])\. (.*)$")
BAND_LINE = re.compile(r"^(Pre-A1|A1|A2|B1|B2|C1|C2)$")
ROUTE_LINE = re.compile(r"^(Pre-A1/A1|A1/A2|A2/B1|B1/B2|B2/C1|C1/C2)$")
SPK_HDR = re.compile(r"^SPK-([A-Za-z0-9]+)-(OR|SR|FS|RT|ER1|ER2|INT1|INT2) \| (.*)$")
WRT_HDR = re.compile(r"^WRT-([A-Za-z0-9]+)-(\d|D) \| (.*?) \| target: (.*?) \| focus: (.*)$")
KEY_ROW = re.compile(r"^\s*((?:LS|RD|LSN)-(?:PRE|A1|A2|B1|B2|C1|C2)-\d{2})\s+(Pre-A1|A1|A2|B1|B2|C1|C2)\s+([ABCD])(?:\s+(.*))?$")

SPK_ROUTE = {"PreA1A1": "PreA1-A1", "A1A2": "A1-A2", "A2B1": "A2-B1", "B1B2": "B1-B2", "B2C1": "B2-C1", "C1C2": "C1-C2"}
WRT_ROUTE = {"PA": "PreA1-A1", "A12": "A1-A2", "A2B1": "A2-B1", "B1B2": "B1-B2", "B2C1": "B2-C1", "C1C2": "C1-C2"}
ROUTE_BANDS = {"PreA1-A1": ["Pre-A1", "A1"], "A1-A2": ["A1", "A2"], "A2-B1": ["A2", "B1"],
               "B1-B2": ["B1", "B2"], "B2-C1": ["B2", "C1"], "C1-C2": ["C1", "C2"]}

# Speaking task metadata (blueprint §8 / Keys §6)
SPK_TYPE = {
    "OR":   dict(task_type="oral_reading",            weight=0.10, spontaneous=False, delivery="text_read_aloud",
                 traits=["intelligibility", "fluency"]),
    "SR":   dict(task_type="sentence_reconstruction", weight=0.10, spontaneous=False, delivery="audio_only_repeat",
                 traits=["intelligibility", "fluency", "communication"]),
    "FS":   dict(task_type="functional_situation",    weight=0.15, spontaneous=True,  delivery="text_prompt",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
    "RT":   dict(task_type="retell_summarise",        weight=0.15, spontaneous=True,  delivery="audio_then_speak",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
    "ER1":  dict(task_type="extended_response",       weight=0.15, spontaneous=True,  delivery="text_prompt",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
    "ER2":  dict(task_type="extended_response",       weight=0.15, spontaneous=True,  delivery="text_prompt",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
    "INT1": dict(task_type="simulated_interaction",   weight=0.10, spontaneous=True,  delivery="interlocutor_audio_then_speak",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
    "INT2": dict(task_type="simulated_interaction",   weight=0.10, spontaneous=True,  delivery="interlocutor_audio_then_speak",
                 traits=["intelligibility", "fluency", "grammar", "vocabulary", "communication"]),
}
# INT1+INT2 together = 20% (one two-turn task). Weight above is per turn.

# Kit-assigned topic families for productive tasks (blueprint gives none). Reviewer may rename.
SPK_TOPIC = {
    "PreA1A1": dict(OR="personal_intro", SR="bus_time", FS="cafe_request", RT="shop_schedule", ER1="home",
                    ER2="food", INT1="meeting_time", INT2="meeting_time"),
    "A1A2":    dict(OR="commute_weather", SR="meeting_time", FS="class_booking", RT="library_hours", ER1="town_place",
                    ER2="learning_new", INT1="english_study_habits", INT2="english_study_habits"),
    "A2B1":    dict(OR="event_weather", SR="registration", FS="shift_swap", RT="workshop_places",
                    ER1="learning_alone_vs_group", ER2="problem_solved", INT1="noise_complaint", INT2="noise_complaint"),
    "B1B2":    dict(OR="booking_system", SR="proposal_costs", FS="meeting_format", RT="market_day_buses",
                    ER1="hybrid_work", ER2="advice_sources", INT1="training_attendance", INT2="training_attendance"),
    "B2C1":    dict(OR="policy_consistency", SR="evidence_caution", FS="claim_calibration", RT="simplification_clarity",
                    ER1="simplification_clarity", ER2="policy_consistency", INT1="pilot_scaling", INT2="pilot_scaling"),
    "C1C2":    dict(OR="recommendation_systems", SR="forecasting", FS="claim_calibration", RT="dictionary_usage",
                    ER1="information_transparency", ER2="framing_language", INT1="model_accuracy", INT2="model_accuracy"),
}
WRT_TOPIC = {
    "PA":   {"1": "meeting_time", "2": "library_hours", "3": "weekly_routine", "D": "bus_time"},
    "A12":  {"1": "class_booking", "2": "pool_closure", "3": "english_study_habits", "D": "meeting_time"},
    "A2B1": {"1": "order_complaint", "2": "workshop_places", "3": "learning_alone_vs_group", "D": "workshop_cancelled"},
    "B1B2": {"1": "meeting_format", "2": "training_format", "3": "hybrid_work", "D": "evidence_caution"},
    "B2C1": {"1": "claim_calibration", "2": "simplification_clarity", "3": "uncertainty_communication", "D": "evidence_caution"},
    "C1C2": {"1": "claim_calibration", "2": "dictionary_usage", "3": "information_transparency", "D": "uncertainty_communication"},
}
WRT_TASK_TYPE = {"1": ("functional_communication", 0.30), "2": ("mediation_synthesis", 0.30),
                 "3": ("extended_response", 0.40), "D": ("integrated_accuracy_diagnostic", 0.0)}

EXPECTED = {
    "LS": (52, {"A": 15, "B": 16, "C": 13, "D": 8}),
    "RD": (42, {"A": 10, "B": 14, "C": 14, "D": 4}),
    "LSN": (42, {"A": 10, "B": 14, "C": 14, "D": 4}),
    "SPK": 48, "WRT": 24,
}


def pdftotext(pdf, layout=False):
    args = ["pdftotext"] + (["-layout"] if layout else []) + [pdf, "-"]
    return subprocess.check_output(args, text=True)


def clean_lines(text):
    return [FOOTER.sub("", ln.replace("\f", "")).rstrip() for ln in text.split("\n")]


def opt_id(item_id, letter):
    """Opaque, deterministic option identity. The authoring letter is NOT recoverable client-side."""
    return "opt_" + hashlib.sha256(f"GEPA-v2|{item_id}|{letter}".encode()).hexdigest()[:10]


def section(lines, start_marker, end_marker, search_from=0):
    s = next(i for i in range(search_from, len(lines)) if lines[i].strip() == start_marker)
    e = next(i for i in range(s + 1, len(lines)) if lines[i].strip() == end_marker)
    return s, e, lines[s + 1:e]


def parse_meta_ls(rest):
    parts = [p.strip() for p in rest.split("|")]
    locator = any(p.upper() == "LOCATOR CANDIDATE" for p in parts)
    parts = [p for p in parts if p.upper() != "LOCATOR CANDIDATE"]
    topic = next((p[len("topic:"):].strip() for p in parts if p.lower().startswith("topic:")), None)
    parts = [p for p in parts if not p.lower().startswith("topic:")]
    return dict(construct=parts[0], domain=parts[1], topic_family=topic, locator_candidate=locator)


def parse_objective(lines, module):
    items, stimuli, cur, cur_stim, mode = [], [], None, None, None

    def finalize():
        nonlocal cur
        if cur:
            cur["stem"] = " ".join(cur["stem"]).strip()
            cur["options"] = [dict(option_id=opt_id(cur["item_id"], L), text=t.strip()) for L, t in cur["_opts"]]
            cur["_letters"] = [L for L, _ in cur["_opts"]]
            del cur["_opts"]
            items.append(cur)
            cur = None

    for ln in lines:
        s = ln.strip()
        if not s or BAND_LINE.match(s):
            continue
        m = STIM_HDR.match(s)
        if m:
            finalize()
            parts = [p.strip() for p in m.group(4).split("|")]
            topic = next((p[len("topic:"):].strip() for p in parts if p.lower().startswith("topic:")), None)
            parts = [p for p in parts if not p.lower().startswith("topic:")]
            stim = dict(stimulus_id=s.split(" | ")[0], band=BAND_FROM_CODE[m.group(2)], topic_family=topic, text=[], item_ids=[])
            if module == "RD":
                stim.update(stimulus_type=parts[0], domain=parts[1])
            else:
                wpm = int(re.search(r"target (\d+) wpm", parts[1]).group(1))
                stim.update(speakers=parts[0], target_wpm=wpm, domain=parts[2])
            stimuli.append(stim); cur_stim = stim; mode = "stim_wait"
            continue
        m = ITEM_HDR.match(s)
        if m:
            finalize()
            item_id = s.split(" | ")[0]
            cur = dict(item_id=item_id, module=module, band=BAND_FROM_CODE[m.group(2)], stem=[], _opts=[])
            if module == "LS":
                cur.update(parse_meta_ls(m.group(4)))
            else:
                cur.update(evidence_focus=m.group(4).replace("focus:", "").strip(), stimulus_id=cur_stim["stimulus_id"])
                cur_stim["item_ids"].append(item_id)
            mode = "stem"
            continue
        if cur is not None:
            mo = OPT.match(s)
            if mo:
                cur["_opts"].append([mo.group(1), mo.group(2)]); mode = "options"
            elif mode == "stem":
                cur["stem"].append(s)
            elif mode == "options":
                cur["_opts"][-1][1] += " " + s
            continue
        if cur_stim is not None:
            if s == "Stimulus" or s.startswith("ADMIN AUDIO SCRIPT"):
                mode = "stim_text"
            elif mode == "stim_text":
                cur_stim["text"].append(s)
    finalize()
    for st in stimuli:
        st["text"] = "\n".join(st["text"]).strip() if module == "RD" else " ".join(st["text"]).strip()
        if module == "LSN":
            words = len(st["text"].split())
            st["word_count"] = words
            st["est_duration_sec"] = round(words / st["target_wpm"] * 60, 1)
            st["speaker_count"] = {"One": 1, "Two": 2, "Three": 3}[st["speakers"].split()[0]]
            st["admin_only"] = True
        else:
            st["word_count"] = len(st["text"].split())
    return items, stimuli


def extract_quoted(text):
    m = re.search(r"[\"“](.+?)[\"”]", text)
    return m.group(1).strip() if m else None


def parse_speaking(lines):
    tasks, cur, route = [], None, None

    def finalize():
        nonlocal cur
        if cur:
            cur["prompt"] = " ".join(cur["prompt"]).strip()
            code = cur["_code"]; del cur["_code"]
            meta = SPK_TYPE[code]
            cur.update(task_type=meta["task_type"], weight=meta["weight"], spontaneous_or_interactive=meta["spontaneous"],
                       delivery=meta["delivery"], traits_scored=meta["traits"])
            cur["topic_family"] = SPK_TOPIC[cur["_rc"]][code]; del cur["_rc"]
            cur["turn"] = int(code[-1]) if code.startswith("INT") else None
            cur["task_group"] = re.sub(r"\d$", "", cur["task_id"]) if code.startswith("INT") else cur["task_id"]
            if code == "SR":
                cur["audio_script"] = cur["prompt"]; cur["candidate_sees_text"] = False
            elif code == "RT":
                cur["audio_script"] = extract_quoted(cur["prompt"]); cur["candidate_sees_text"] = True
            elif code.startswith("INT"):
                cur["interlocutor_line"] = extract_quoted(cur["prompt"]); cur["candidate_sees_text"] = True
            else:
                cur["candidate_sees_text"] = True
            tasks.append(cur); cur = None

    for ln in lines:
        s = ln.strip()
        if not s:
            continue
        if ROUTE_LINE.match(s):
            finalize(); route = s; continue
        m = SPK_HDR.match(s)
        if m:
            finalize()
            cur = dict(task_id=s.split(" | ")[0], module="SPK", route=SPK_ROUTE[m.group(1)], route_bands=ROUTE_BANDS[SPK_ROUTE[m.group(1)]],
                       label=m.group(3), prompt=[], _code=m.group(2), _rc=m.group(1))
            continue
        if cur is not None:
            cur["prompt"].append(s)
    finalize()
    return tasks


def parse_writing(lines):
    tasks, cur = [], None

    def finalize():
        nonlocal cur
        if cur:
            cur["prompt"] = " ".join(cur["prompt"]).strip()
            n = cur["_n"]; rc = cur["_rc"]; del cur["_n"], cur["_rc"]
            tt, w = WRT_TASK_TYPE[n]
            cur.update(task_type=tt, weight=w, scored_in_writing_level=(n != "D"), topic_family=WRT_TOPIC[rc][n])
            if n == "D":
                cur["audio_script"] = extract_quoted(cur["prompt"])
            m = re.match(r"(\d+)-(\d+) words", cur["target"])
            cur["word_guidance"] = dict(min=int(m.group(1)), max=int(m.group(2))) if m else None
            tasks.append(cur); cur = None

    for ln in lines:
        s = ln.strip()
        if not s:
            continue
        if ROUTE_LINE.match(s):
            finalize(); continue
        m = WRT_HDR.match(s)
        if m:
            finalize()
            cur = dict(task_id=s.split(" | ")[0], module="WRT", route=WRT_ROUTE[m.group(1)], route_bands=ROUTE_BANDS[WRT_ROUTE[m.group(1)]],
                       label=m.group(3), target=m.group(4), focus=m.group(5), prompt=[], _n=m.group(2), _rc=m.group(1))
            continue
        if cur is not None:
            cur["prompt"].append(s)
    finalize()
    return tasks


def parse_keys(layout_lines):
    """Keys from -layout text. Returns {item_id: (level, letter, answer_text_fragment, rationale)}."""
    s = next(i for i, l in enumerate(layout_lines) if l.strip() == "2. Language Systems key")
    e = next(i for i, l in enumerate(layout_lines) if l.strip().startswith("5. Objective routing rules"))
    seg = layout_lines[s:e]
    keys = OrderedDict()
    cut = None
    para = []

    def flush(para):
        nonlocal cut
        for l in para:
            if "Rationale" in l and "Key" in l:
                cut = l.index("Rationale"); return
        row = next((KEY_ROW.match(l) for l in para if KEY_ROW.match(l)), None)
        if not row:
            return
        item_id, level, letter, rest = row.group(1), row.group(2), row.group(3), (row.group(4) or "")
        rationale = None
        # rationale column start = indent of the rationale-only lines in this paragraph (wrapped cell)
        # (rationale is always the right-most column; construct cells may wrap at a smaller indent)
        starts = []
        for x in para:
            if KEY_ROW.match(x) or not x.strip():
                continue
            last = list(re.finditer(r"\S+(?: \S+)*", x))[-1]   # last cluster of words separated by 2+ spaces
            starts.append(last.start())
        if starts:
            cut = max(starts)
        if item_id.startswith("LS-") and cut is not None:
            rationale = " ".join(x[cut:].strip() for x in para if len(x) > cut and x[cut:].strip()) or None
            rest = rest[:max(0, cut - row.start(4))] if row.group(4) else ""
        keys[item_id] = (level, letter, rest.strip(), rationale)

    for l in seg:
        if l.strip() == "":
            if para:
                flush(para); para = []
        else:
            para.append(l)
    if para:
        flush(para)
    return keys


def sha256_file(p):
    return hashlib.sha256(open(p, "rb").read()).hexdigest()


def main(pdf, out):
    os.makedirs(out, exist_ok=True)
    raw = clean_lines(pdftotext(pdf))
    lay = clean_lines(pdftotext(pdf, layout=True))

    ls_s, ls_e, ls_lines = section(raw, "2. Language Systems", "3. Reading")
    _, rd_e, rd_lines = section(raw, "3. Reading", "4. Listening", ls_e)
    _, ln_e, ln_lines = section(raw, "4. Listening", "5. Speaking", rd_e)
    _, sp_e, sp_lines = section(raw, "5. Speaking", "6. Writing", ln_e)
    _, wr_e, wr_lines = section(raw, "6. Writing", "7. Form-assembly constraints", sp_e)

    ls_items, _ = parse_objective(ls_lines, "LS")
    rd_items, rd_stim = parse_objective(rd_lines, "RD")
    ln_items, ln_stim = parse_objective(ln_lines, "LSN")
    spk = parse_speaking(sp_lines)
    wrt = parse_writing(wr_lines)
    keys = parse_keys(lay)

    problems = []
    all_items = {i["item_id"]: i for i in ls_items + rd_items + ln_items}
    # --- validate counts
    for mod, lst in (("LS", ls_items), ("RD", rd_items), ("LSN", ln_items)):
        if len(lst) != EXPECTED[mod][0]:
            problems.append(f"{mod}: expected {EXPECTED[mod][0]} items, parsed {len(lst)}")
    if len(rd_stim) != 21: problems.append(f"RD stimuli: expected 21, parsed {len(rd_stim)}")
    if len(ln_stim) != 21: problems.append(f"LSN stimuli: expected 21, parsed {len(ln_stim)}")
    if len(spk) != EXPECTED["SPK"]: problems.append(f"SPK: expected 48, parsed {len(spk)}")
    if len(wrt) != EXPECTED["WRT"]: problems.append(f"WRT: expected 24, parsed {len(wrt)}")
    # --- validate keys
    restricted = OrderedDict()
    dist = {"LS": Counter(), "RD": Counter(), "LSN": Counter()}
    for item_id, item in all_items.items():
        if item_id not in keys:
            problems.append(f"no key for {item_id}"); continue
        level, letter, frag, rationale = keys[item_id]
        if level != item["band"]:
            problems.append(f"{item_id}: band mismatch item={item['band']} key={level}")
        if letter not in item["_letters"]:
            problems.append(f"{item_id}: key letter {letter} not among options {item['_letters']}")
            continue
        ans = item["options"][item["_letters"].index(letter)]["text"]
        if frag:
            focus = (item.get("construct") or item.get("evidence_focus") or "")
            frag_clean = re.sub(r"\s+", " ", frag).replace(focus, "").strip()
            ok = (frag_clean.lower().startswith(re.sub(r"\s+", " ", ans).lower()) if item_id.startswith("LS-")
                  else frag_clean.lower() in re.sub(r"\s+", " ", ans).lower())
            if frag_clean and not ok:
                problems.append(f"{item_id}: key text '{frag_clean}' not found in option {letter} '{ans}'")
        dist[item["module"]][letter] += 1
        restricted[item_id] = OrderedDict(module=item["module"], band=item["band"],
                                          key_option_id=opt_id(item_id, letter), authoring_letter=letter,
                                          answer_text=ans, option_count=len(item["options"]),
                                          evidence_focus=item.get("construct") or item.get("evidence_focus"),
                                          rationale=rationale)
    for mod in ("LS", "RD", "LSN"):
        if dict(dist[mod]) != EXPECTED[mod][1]:
            problems.append(f"{mod} key distribution {dict(dist[mod])} != expected {EXPECTED[mod][1]}")
    # strip authoring letters from candidate-safe files
    for it in all_items.values():
        del it["_letters"]
    # option count rule: 3 options below B1, 4 at B1+
    for it in all_items.values():
        exp = 4 if BANDS.index(it["band"]) >= 3 else 3
        if len(it["options"]) != exp:
            problems.append(f"{it['item_id']}: {len(it['options'])} options, expected {exp}")
    # speaking weights: OR+SR <=0.20, spontaneous >=0.60 per route
    for route in ROUTE_BANDS:
        rt = [t for t in spk if t["route"] == route]
        diag = sum(t["weight"] for t in rt if not t["spontaneous_or_interactive"])
        spon = sum(t["weight"] for t in rt if t["spontaneous_or_interactive"])
        if len(rt) != 8 or round(diag, 2) > 0.20 or round(spon, 2) < 0.60 or round(diag + spon, 2) != 1.0:
            problems.append(f"SPK route {route}: n={len(rt)} diag={diag:.2f} spon={spon:.2f}")
        wt = [t for t in wrt if t["route"] == route]
        if len(wt) != 4 or round(sum(t["weight"] for t in wt), 2) != 1.0:
            problems.append(f"WRT route {route}: n={len(wt)} weights={sum(t['weight'] for t in wt):.2f}")

    def dump(name, obj):
        p = os.path.join(out, name)
        with open(p, "w", encoding="utf-8") as f:
            json.dump(obj, f, ensure_ascii=False, indent=2)
        return p

    hdr = dict(package="GEPA Pilot Item Bank v2", source_pdf=os.path.basename(pdf), prepared="2026-09-08",
               status="pilot — expert review / cognitive labs / closed beta only; NOT a public production bank",
               note="Option order printed in source = authoring layout only. Runtime MUST shuffle per session and store the permutation.")
    files = []
    files.append(dump("language_systems.json", dict(**hdr, module="LS", reported="diagnostic_only",
                                                    locator_rule="first two items in each A1-C2 band are locator candidates", items=ls_items)))
    files.append(dump("reading.json", dict(**hdr, module="RD", reported=True, stimuli=rd_stim, items=rd_items)))
    files.append(dump("listening.json", dict(**hdr, module="LSN", reported=True,
                                             audio_policy="questions visible before first play; 1 required play; 2nd play allowed, logged, never penalised",
                                             stimuli=ln_stim, items=ln_items)))
    files.append(dump("speaking_tasks.json", dict(**hdr, module="SPK", reported=True,
                                                  rules=["OR+SR combined <= 20% of Speaking estimate", ">= 60% from spontaneous/interactive production",
                                                         "INT1+INT2 form one two-turn task worth 20%", "accent similarity is never scored"],
                                                  topic_family_source="kit-assigned (blueprint gives none) — reviewer may rename", tasks=spk)))
    files.append(dump("writing_tasks.json", dict(**hdr, module="WRT", reported=True,
                                                 rules=["task weights 30/30/40", "diagnostic (D) weight 0 — never in Writing level",
                                                        "Task 3 rotates rhetorical function"], tasks=wrt)))
    restricted_path = dump("RESTRICTED_answer_keys.json", dict(
        RESTRICTED="SERVER-ONLY. Never ship to client bundle, never return in any API response, never log.",
        source_pdf=os.path.basename(pdf), authoring_key_distribution={m: dict(dist[m]) for m in dist},
        key_identity_rule="keys are stored by opaque option_id, not by displayed letter", keys=restricted))
    files.append(restricted_path)

    # --- human-readable review markdown
    md = ["# GEPA Pilot Item Bank v2 — review rendering (NO KEYS)", "", "Generated from the source PDF by `scripts/parse_pdf_to_seed.py`.", ""]
    md += ["## Language Systems (52 items — diagnostic only)", ""]
    for it in ls_items:
        loc = " **[LOCATOR]**" if it["locator_candidate"] else ""
        md.append(f"### {it['item_id']} · {it['band']} · {it['construct']} · {it['domain']}/{it['topic_family']}{loc}")
        md.append(f"{it['stem']}"); md += [f"- ({o['option_id']}) {o['text']}" for o in it["options"]]; md.append("")
    for title, stim, items, key in (("Reading (21 stimuli / 42 items)", rd_stim, rd_items, "text"),
                                    ("Listening (21 scripts / 42 items) — scripts are ADMIN assets", ln_stim, ln_items, "text")):
        md += [f"## {title}", ""]
        for st in stim:
            extra = f" · {st['speakers']} · {st['target_wpm']} wpm · ~{st['est_duration_sec']}s" if "target_wpm" in st else f" · {st['stimulus_type']}"
            md.append(f"### {st['stimulus_id']} · {st['band']} · {st['domain']}/{st['topic_family']}{extra}")
            md.append("> " + st["text"].replace("\n", "\n> ")); md.append("")
            for it in [i for i in items if i["stimulus_id"] == st["stimulus_id"]]:
                md.append(f"**{it['item_id']}** ({it['evidence_focus']}): {it['stem']}")
                md += [f"- ({o['option_id']}) {o['text']}" for o in it["options"]]; md.append("")
    md += ["## Speaking (48 prompts)", ""]
    for t in spk:
        md.append(f"- **{t['task_id']}** · {t['route']} · {t['label']} · w={t['weight']} · {t['topic_family']}  \n  {t['prompt']}")
    md += ["", "## Writing (24 tasks incl. 6 diagnostics)", ""]
    for t in wrt:
        md.append(f"- **{t['task_id']}** · {t['route']} · {t['label']} · {t['target']} · w={t['weight']} · {t['topic_family']}  \n  {t['prompt']}")
    with open(os.path.join(out, "item_bank_review.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(md) + "\n")
    kmd = ["# RESTRICTED — GEPA v2 answer keys (server-only review copy)", "", "| item | band | key option_id | authoring letter | answer | focus | rationale |", "|---|---|---|---|---|---|---|"]
    for k, v in restricted.items():
        kmd.append(f"| {k} | {v['band']} | `{v['key_option_id']}` | {v['authoring_letter']} | {v['answer_text']} | {v['evidence_focus']} | {v['rationale'] or ''} |")
    with open(os.path.join(out, "RESTRICTED_keys_review.md"), "w", encoding="utf-8") as f:
        f.write("\n".join(kmd) + "\n")

    manifest = dict(generated_from=os.path.basename(pdf), counts=dict(language_systems=len(ls_items), reading_items=len(rd_items),
                    reading_stimuli=len(rd_stim), listening_items=len(ln_items), listening_stimuli=len(ln_stim),
                    speaking_prompts=len(spk), writing_tasks=len(wrt)),
                    authoring_key_distribution={m: dict(dist[m]) for m in dist},
                    files={os.path.basename(p): sha256_file(p) for p in files}, validation_problems=problems)
    dump("manifest.json", manifest)
    print(json.dumps(manifest["counts"]), json.dumps(manifest["authoring_key_distribution"]))
    if problems:
        print("VALIDATION PROBLEMS:"); [print(" -", p) for p in problems]; sys.exit(1)
    print("OK — all validation checks passed")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        print(__doc__); sys.exit(2)
    main(sys.argv[1], sys.argv[2])
