"""Nameplates piece: which AddNpc names the 2014 servers hid; run with python -I.

Writes prototype/osfr-tools/data/content/nameplate_rules.json (read by src/nameplate.rs) from two
pinned tables. Only names and ids are copied, never rows:
* Sanctuary's NPC table (AGPL data, rebuilt from 2014 captures): a name or model is "hidden" when
  every row carrying it has HideNamePlate, "shown" when none has; mixed ones are left out.
* the client's own Models.txt (ID^MODEL_FILE_NAME^RACE_ID^...): the models whose RACE_ID is a
  critter or prop race (the races client-rs fr-client names.rs treats as things), and the
  raceless invisible cubes and particle launchers.
"""
import collections
import json
import subprocess
from pathlib import Path

PROJECT = Path(__file__).resolve().parents[1]
OUT = PROJECT / "prototype/osfr-tools/data/content/nameplate_rules.json"
LAB = PROJECT.parent / "lab"
SANCTUARY, SANCTUARY_REV = "Sanctuary-main-20261019", "10f129e"
NPCS = "src/Resources/Npcs.json"
MODELS = LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt"
MODELS_CITE = "client Resources/Models.txt (byte-identical to Sanctuary@10f129e src/Resources/Models.txt)"
# Models.txt RACE_ID (RaceDefinitions.txt): 21 birds, 22 antelope, 23 goats, 24 pigs, 32 cows,
# 46 butterflies, 48 ewes, 50 squirrels, 51 rabbits, 52 pets and mounts; 45, 99, 107 props.
CRITTER_RACES = [21, 22, 23, 24, 32, 46, 48, 50, 51, 52]
PROP_RACES = [45, 99, 107]


def sanctuary_rows():
    repo = LAB / "references" / SANCTUARY
    run = subprocess.run(["git", "-C", str(repo), "show", f"{SANCTUARY_REV}:{NPCS}"],
                         capture_output=True, check=True)
    return json.loads(run.stdout.decode("utf-8-sig"))


def split(groups):
    """Keys whose every row hides, and keys whose no row hides; mixed keys are in neither."""
    hidden = sorted(key for key, flags in groups.items() if all(flags))
    shown = sorted(key for key, flags in groups.items() if not any(flags))
    return hidden, shown


def main():
    rows = sanctuary_rows()
    by_name, by_model = collections.defaultdict(list), collections.defaultdict(list)
    for row in rows:
        hidden = bool(row.get("HideNamePlate"))
        if row.get("Name", "").strip():
            by_name[row["Name"].strip()].append(hidden)
        by_model[row["ModelId"]].append(hidden)
    hidden_names, shown_names = split(by_name)
    hidden_models, shown_models = split(by_model)
    races = {}
    markers = []
    for line in MODELS.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        model_id, file_name, race = line.split("^")[:3]
        races[int(model_id)] = int(race or 0)
        if int(race or 0) == 0 and file_name.lower().startswith("invisible_"):
            markers.append(int(model_id))

    def models_of(wanted):
        return sorted(model for model, race in races.items() if race in wanted)

    source = f"lab/references/{SANCTUARY}@{SANCTUARY_REV} {NPCS}, {len(rows)} definitions: "
    document = {
        "schema_version": 1,
        "provenance": {
            "generator": "scripts/build_nameplate_rules.py (python -I)",
            "license": "AGPL data (Sanctuary): names and ids only, no rows or code copied",
            "order": "src/nameplate.rs hidden(): players, quest NPCs and vendors, explicit row flag,"
                     " pick-up kind, then these blocks top to bottom; the first that names the row wins",
        },
        "hidden_names": {
            "source": source + "every row with this trimmed Name has HideNamePlate",
            "names": hidden_names,
        },
        "shown_names": {
            "source": source + "no row with this trimmed Name has HideNamePlate (the named characters"
                      " and named props of 2014)",
            "names": shown_names,
        },
        "hidden_models": {
            "source": source + "every row with this ModelId has HideNamePlate",
            "model_ids": hidden_models,
        },
        "shown_models": {
            "source": source + "no row with this ModelId has HideNamePlate",
            "model_ids": shown_models,
        },
        "critter_races": {
            "source": f"(R) {MODELS_CITE}: models whose RACE_ID is a critter race, as client-rs"
                      " fr-client names.rs; by analogy for rows the 2014 table does not decide",
            "race_ids": CRITTER_RACES,
            "model_ids": models_of(CRITTER_RACES),
        },
        "prop_races": {
            "source": f"(R) {MODELS_CITE}: models whose RACE_ID is a prop race, as client-rs"
                      " fr-client names.rs; by analogy for rows the 2014 table does not decide",
            "race_ids": PROP_RACES,
            "model_ids": models_of(PROP_RACES),
        },
        "marker_models": {
            "source": f"(R) {MODELS_CITE}: raceless models whose file name starts with invisible_"
                      " (cubes and particle launchers; the race-99 invisible_cube.adr 271 is a prop)",
            "model_ids": sorted(markers),
        },
    }
    with open(OUT, "w", encoding="utf-8", newline="\n") as handle:
        handle.write(json.dumps(document, indent=2, ensure_ascii=False) + "\n")
    print(f"{OUT}: names {len(hidden_names)} hidden / {len(shown_names)} shown, models"
          f" {len(hidden_models)} hidden / {len(shown_models)} shown, race models"
          f" {len(models_of(CRITTER_RACES))} critter / {len(models_of(PROP_RACES))} prop, markers {len(markers)}")


if __name__ == "__main__":
    main()
