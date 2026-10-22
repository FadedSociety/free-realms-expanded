"""T459: the boombox catalogue; run with python -I scripts/build_boomboxes.py.

Writes prototype/osfr-tools/data/content/boomboxes.json for the 15 client boombox items (the
Class 171 rows of the reference item definitions W84's item catalogue is built from). Each row
comes from Sanctuary's Consumables.json `Boomboxes` row (AGPL data rows only, no code copied).
Client evidence wins: where the client item names a model file, model_id is that file's
Models.txt id, and the aliases come from the same item row. Sanctuary-only boxes are left out.
"""
import json
from pathlib import Path
import re
import subprocess
import sys

sys.dont_write_bytecode = True
PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "prototype/osfr-tools/data/content/boomboxes.json"
LAB = PROJECT.parent / "lab"
SANCTUARY, SANCTUARY_REV, BOXES = "Sanctuary-main-20261019", "10f129e", "src/Resources/Consumables.json"
LEGACY, LEGACY_REV, ITEMS = "FreeRealms-Legacy", "9e08202", "src/Resources/ClientItemDefinitions.json"
MODELS = LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt"
CAPTURED = LAB / "reports/kart-weapons-probe-v20-20261010/content/inventory.json"  # W84 baseline rows
# (R) Sanctuary BoomboxDefinition defaults (10f129e src/Sanctuary.Game/Resources/Definitions/
# BoomboxDefinition.cs:9-11); the recast is the duration with Sanctuary's 3000 ms floor (14914ac).
DURATION_MS, RANGE_M, SPAWN_OFFSET_M, MIN_RECAST_MS = 180_000, 15, 2, 3_000
TUNABLES = ("(R) clip and blend lengths were measured by Sanctuary, not read from client data; "
            "duration, range and spawn offset are Sanctuary's BoomboxDefinition defaults "
            "(BoomboxDefinition.cs:9-11) and the recast is the duration with its 3000 ms floor (14914ac)")
FIELDS = {"Comment", "ItemId", "ModelId", "DanceSequence", "SynchronizedDances", "DanceBlendMs",
          "DanceDurationsMs", "EffectIds", "StandingDanceAnimationId", "Priority", "TransformModelId",
          "TransformReapplyDelayMs", "IndependentDanceDurationsMs"}


def git_show(repo, rev, path):
    run = subprocess.run(["git", "-C", str(LAB / "references" / repo), "show", f"{rev}:{path}"],
                         capture_output=True, check=True)
    return run.stdout.decode("utf-8-sig")


def line_of(text, pattern):
    lines = [n for n, line in enumerate(text.splitlines(), 1) if re.search(pattern, line)]
    assert len(lines) == 1, (pattern, lines)
    return lines[0]


def main():
    boxes_text = git_show(SANCTUARY, SANCTUARY_REV, BOXES)
    items_text = git_show(LEGACY, LEGACY_REV, ITEMS)
    boxes = {row["ItemId"]: row for row in json.loads(boxes_text)["Boomboxes"]}
    items = {row["Id"]: row for row in json.loads(items_text)}
    models = {}
    for n, line in enumerate(MODELS.read_text(encoding="latin-1").splitlines(), 1):
        cells = line.split("^")
        if cells[0].strip().isdigit():
            models[int(cells[0])] = (n, cells[1].strip(), cells[4].strip())
    by_file = {}
    for model_id, (_, name, _) in models.items():
        by_file.setdefault(name.lower(), []).append(model_id)
    captured = {d["id"]: d for d in json.loads(CAPTURED.read_text(encoding="utf-8"))["definitions"]}

    ids = sorted(i for i, item in items.items() if item.get("Class") == 171)
    assert len(ids) == 15 and set(ids) <= set(boxes), ids
    print("left out (Sanctuary only):", sorted(set(boxes) - set(ids)))

    def model_cite(model_id):
        line, name, _ = models[model_id]
        return {"kind": "client", "evidence": f"Models.txt:{line} {model_id}^{name}"}

    out = []
    for item_id in ids:
        box, item = boxes[item_id], items[item_id]
        assert set(box) <= FIELDS, set(box) - FIELDS
        name = box["Comment"]
        model_file = item.get("ModelName")
        notes = []
        if model_file:
            (model_id,) = by_file[model_file.lower()]
            if model_id != box["ModelId"]:
                other = models[box["ModelId"]]
                hint = f" ('{other[2]}' in Models.txt)" if other[2] else ""
                notes.append(f"(R) model conflict until the lead checks it natively: the client item's model "
                             f"{model_file} is {model_id} (used); Sanctuary uses {box['ModelId']} {other[1]}{hint}")
        else:
            model_id = box["ModelId"]
            notes.append(f"(R) the client item names no model file; Sanctuary's {model_id} "
                         f"{models[model_id][1]} is kept")
        transform = box.get("TransformModelId", 0)
        kind = "transform" if transform else "synchronized" if box.get("SynchronizedDances") else "independent"
        clips = box.get("DanceDurationsMs", {})
        # Independent boxes pick random clips per dancer; those tables are boombox-random-dances' slice.
        assert (kind == "independent") == (not clips), item_id
        assert all(len(c) == len(box["DanceSequence"]) for c in clips.values()), item_id
        item_line = line_of(items_text, f'"Id": {item_id},')
        box_line = line_of(boxes_text, f'"ItemId": {item_id},')
        defined = (f"../lab/references/{LEGACY}@{LEGACY_REV} {ITEMS}:{item_line} '{name}': Class 171, "
                   f"ModelName {model_file or 'none'}, TextureAlias {item.get('TextureAlias') or 'none'}, "
                   f"TintAlias {item['TintAlias']}")
        if item_id in captured:
            row = captured[item_id]
            defined += (f"; the captured definition the runtime loads first (W84 baseline inventory.json) "
                        f"has class {row['class_id']} and model '{row['model']}', texture '{row['texture']}'")
        source = [
            {"kind": "reference", "evidence": f"../lab/references/{SANCTUARY}@{SANCTUARY_REV} {BOXES}:"
                f"{box_line} ({name}): dances, sync, blend, clip lengths, song effects, standing dance, "
                "priority, transform"},
            {"kind": "reference", "evidence": defined},
            model_cite(model_id),
            *([model_cite(transform)] if transform else []),
            {"kind": "reconstructed", "evidence": TUNABLES},
            *({"kind": "reconstructed", "evidence": note} for note in notes),
        ]
        out.append({
            "item_id": item_id,
            "model_id": model_id,
            "kind": kind,
            "dance_sequence": box["DanceSequence"],
            "clip_ms": clips,
            "blend_ms": box["DanceBlendMs"],
            "song_effect_ids": box.get("EffectIds", []),
            "standing_animation_id": box.get("StandingDanceAnimationId", 0),
            "priority": box.get("Priority", 0),
            "transform_model_id": transform,
            "transform_reapply_ms": box.get("TransformReapplyDelayMs", 0),
            "duration_ms": DURATION_MS,
            "range_m": RANGE_M,
            "spawn_offset_m": SPAWN_OFFSET_M,
            "recast_ms": max(DURATION_MS, MIN_RECAST_MS),
            "texture_alias": item.get("TextureAlias", ""),
            "tint_alias": item["TintAlias"],
            "source": source,
        })
    text = json.dumps({"schema_version": 1, "boomboxes": out}, indent=2, ensure_ascii=False)
    # One line per number list, so a row stays readable.
    text = re.sub(r"\[([-\d,\s]*)\]", lambda m: "[" + ", ".join(re.findall(r"-?\d+", m.group(1))) + "]", text)
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(text + "\n")
    print(f"wrote {len(out)} boomboxes to {OUT}")


if __name__ == "__main__":
    main()
