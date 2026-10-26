"""pets_import.py: every pet model, pet item and pet trick from client-shaped data into data/content/pets.json (T454).

Reads only; writes pets.json deterministically (same bytes on every run). Run with `python -I scripts/pets_import.py`.
Sources (data facts only; the AGPL reference's code is not copied):
  Pets.json             lab/references/Sanctuary-main-20261019/src/Resources/Pets.json (per-model rows)
  ClientItemDefinitions lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json (Type 2 = pet items)
  Models.txt            lab/working/OSFR.Client/OSFR Client/Resources/Models.txt (client model ids)
  locale                lab/reports/locale-id-to-text.json (a name id the client lacks drops its trick or item)
Client evidence wins: an item whose model is not in Models.txt, or whose name id the locale lacks, is dropped.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT.parent / "lab"
PETS_JSON = ROOT / "prototype" / "osfr-tools" / "data" / "content" / "pets.json"
SOURCES = {
    "pets": (LAB / "references/Sanctuary-main-20261019/src/Resources/Pets.json",
             "10f129e7f3e2094304d20ccee6e3f7136a5e889d"),
    "items": (LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
              "9e082025391177d5a33ee18c714ddd06b457a695"),
    "models": (LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt", None),
}
LOCALE = LAB / "reports/locale-id-to-text.json"
# ADRs the server already bound before this import (src/pets.rs MODEL_IDS) that Pets.json has no row for.
# Client evidence wins over the Legacy item file: the shipped client item catalogue's icon tint for these items
# (tests/pet_catalogue.rs every_item_row_matches_the_shipped_item_catalogue found the one disagreement).
CLIENT_ICON_TINT = {17849: 260}
EXTRA_MODELS = {"bat_m_pinkeye.adr": 4566, "dog_large_stbernard.adr": 4109, "robgoblin_minimech.adr": 4068,
                "vulture_pet.adr": 3985}


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def models_txt(path):
    """client model id -> (adr, gender)."""
    out = {}
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if not line or line.startswith("#"):
            continue
        c = line.split("^")
        out[int(c[0])] = (c[1], int(c[5] or 0))
    return out


def main():
    content = json.loads(PETS_JSON.read_text(encoding="utf-8"))
    pets, items_all = read_json(SOURCES["pets"][0]), read_json(SOURCES["items"][0])
    client_models = models_txt(SOURCES["models"][0])
    locale = {r["id"] for r in json.loads(LOCALE.read_text(encoding="utf-8"))}

    content["imports"] = [{"name": name, "path": str(path.relative_to(LAB.parent)).replace("\\", "/"),
                           "commit": commit, "sha256": sha256(path)}
                          for name, (path, commit) in SOURCES.items()]

    # Tricks: display fields per trick id, identical on every model that lists it.
    trick_fields = {}
    for row in pets:
        for t in row.get("PetTricks") or []:
            trick_fields.setdefault(t["Id"], {"icon_id": t["IconId"], "name_id": t["NameId"],
                                              "tooltip_id": t["TooltipStringId"], "skill_level": t["SkillLevel"],
                                              "unknown5": t["Unknown5"], "unknown6": t["Unknown6"]})
    dropped_tricks = sorted(i for i, f in trick_fields.items() if f["name_id"] and f["name_id"] not in locale)
    tricks = {t["id"]: t for t in content["tricks"]}
    for trick_id, fields in sorted(trick_fields.items()):
        if trick_id in dropped_tricks:
            continue
        row = tricks.setdefault(trick_id, {"id": trick_id, "source": [
            {"kind": "reference", "evidence": f"Pets.json PetTricks Id={trick_id} (display fields; no "
                                              "PetTrickGesturePoints.txt row, so no gesture: client 00cc9330 "
                                              "scores nothing)"}]})
        row.update(fields)
    content["tricks"] = [tricks[k] for k in sorted(tricks)]

    # Models: one row per Pets.json entry whose Id is a client model, plus the extra bound ADRs.
    models = {}
    for row in pets:
        mid = row["Id"]
        if mid not in client_models:
            continue
        adr, gender = client_models[mid]
        order = sorted((t for t in row.get("PetTricks") or [] if t["Id"] not in dropped_tricks),
                       key=lambda t: (t["Unknown8"], t["Id"]))
        models[mid] = {"model_id": mid, "adr": adr, "gender": gender, "icon_id": row.get("IconId"),
                       "default_tint_id": row.get("DefaultTintId"), "default_tint_alias": row.get("DefaultTintAlias"),
                       "vanity": bool(row.get("IsVanityPet")),
                       "tricks": [{"id": t["Id"], "known": t["Known"]} for t in order],
                       "item_classes": [{"key": c["Key"], "id": c["Value"]["Id"], "unknown": c["Value"]["Unknown"]}
                                        for c in row.get("ItemClasses") or []],
                       "src": f"Pets.json Id={mid}; Models.txt {mid}"}
    for adr, mid in EXTRA_MODELS.items():
        if mid not in models and client_models.get(mid, ("",))[0] == adr:
            models[mid] = {"model_id": mid, "adr": adr, "src": f"Models.txt {mid}",
                           "unknown_reasons": {"pet_row": "(R) Pets.json has no row for this model; the server "
                                                          "bound it before T454, so it keeps model_id and adr only"}}
    content["models"] = [models[k] for k in sorted(models)]

    # Items: Legacy Type-2 items whose ModelName resolves through Models.txt to a model row.
    by_adr = {}
    for mid, (adr, _) in client_models.items():
        by_adr.setdefault(adr.lower(), []).append(mid)
    items, unresolved = [], []
    for item in sorted((i for i in items_all if i.get("Type") == 2), key=lambda i: i["Id"]):
        ids = [m for m in by_adr.get((item.get("ModelName") or "").lower(), []) if m in models]
        if not ids or item["NameId"] not in locale:
            unresolved.append(item["Id"])
            continue
        icon = item.get("Icon") or {}
        items.append({"item_id": item["Id"], "model_id": min(ids), "name_id": item["NameId"],
                      "image_set_id": icon.get("Id"),
                      "icon_tint_id": CLIENT_ICON_TINT.get(item["Id"], icon.get("TintId")),
                      "tint_alias": item.get("TintAlias"), "texture_alias": item.get("TextureAlias"),
                      "members_only": bool(item.get("MembersOnly")),
                      "src": f"ClientItemDefinitions.json Id={item['Id']}"})
    content["items"] = items

    PETS_JSON.write_text(json.dumps(content, indent=2, ensure_ascii=False) + "\n", encoding="utf-8", newline="")
    print(json.dumps({"models": len(models), "vanity": sum(1 for m in models.values() if m.get("vanity")),
                      "items": len(items), "type2_items": len(items) + len(unresolved),
                      "unresolved_items": unresolved, "tricks": len(content["tricks"]),
                      "dropped_tricks": dropped_tricks}))


if __name__ == "__main__":
    main()
