"""pets_import.py: every pet model, pet item and pet trick from client-shaped data into data/content/pets.json (T454).

Reads only; writes pets.json deterministically (same bytes on every run). Run with `python -I scripts/pets_import.py`.
Sources (data facts only; the AGPL reference's code is not copied):
  Pets.json             lab/references/Sanctuary-main-20261019/src/Resources/Pets.json (per-model rows)
  ClientItemDefinitions lab/references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json (Type 2 = pet items)
  Models.txt            lab/working/OSFR.Client/OSFR Client/Resources/Models.txt (client model ids)
  locale                lab/reports/locale-id-to-text.json (item display names; a name id the client lacks drops it)
Client evidence wins: an item whose model is not in Models.txt, or whose name id the locale lacks, is dropped.
(R) Two client item rows name a model Models.txt does not spell the same way: firefly.adr has two client ids and
no Pets.json row (the lower id is used), and Candy Corn's vanitypet_candycorn_critter.adr names the candycorn
critter by its item TextureAlias (candycorn-critter -> candycorn_critter.adr, which Models.txt row 1530
describes as "halloween candy corn pet pal"). Both are bound so every client Type-2 item can be adopted.
"""
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
LAB = ROOT.parent / "lab"
PETS_JSON = ROOT / "prototype" / "osfr-tools" / "data" / "content" / "pets.json"
LOCALE = LAB / "reports/locale-id-to-text.json"
# The shipped client item catalogue (the rows `Inventory::presentation` reads). Client evidence wins over the
# Legacy item file: it is where item 17849's icon tint 260 comes from (Legacy says 453).
CATALOGUE = LAB / "reports/round70-native-20261016T061000Z/content/item-catalogue.json"
SOURCES = {
    "pets": (LAB / "references/Sanctuary-main-20261019/src/Resources/Pets.json",
             "10f129e7f3e2094304d20ccee6e3f7136a5e889d"),
    "items": (LAB / "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
              "9e082025391177d5a33ee18c714ddd06b457a695"),
    "models": (LAB / "working/OSFR.Client/OSFR Client/Resources/Models.txt", None),
    "locale": (LOCALE, None),
    "catalogue": (CATALOGUE, None),
}
# ADRs the models table needs even though Pets.json has no row: the four bound before this import, plus
# firefly.adr (item 17543). Models.txt lists firefly.adr as 483 and 3243 with no other difference; the lower
# id is used (R), the rule the brief gives when Pets.json lists none.
EXTRA_MODELS = {"bat_m_pinkeye.adr": 4566, "dog_large_stbernard.adr": 4109, "robgoblin_minimech.adr": 4068,
                "vulture_pet.adr": 3985, "firefly.adr": 483}


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


def resolved_model_ids(item, by_adr, models):
    """The item's Models.txt model ids among the `models` rows. (R) When its ModelName has no entry
    (Candy Corn: vanitypet_candycorn_critter.adr), the ADR its client TextureAlias names
    (candycorn-critter -> candycorn_critter.adr, Models.txt 1530)."""
    names = [(item.get("ModelName") or "").lower()]
    alias = (item.get("TextureAlias") or "").lower().replace("-", "_").replace(" ", "_")
    if alias:
        names.append(alias if alias.endswith(".adr") else f"{alias}.adr")
    for name in names:
        ids = [mid for mid in by_adr.get(name, []) if mid in models]
        if ids:
            return ids
    return []


def main():
    content = json.loads(PETS_JSON.read_text(encoding="utf-8"))
    pets, items_all = read_json(SOURCES["pets"][0]), read_json(SOURCES["items"][0])
    client_models = models_txt(SOURCES["models"][0])
    texts = {r["id"]: r["text"] for r in json.loads(LOCALE.read_text(encoding="utf-8"))}
    # Icon tint from the shipped client catalogue (hex at the definition's +0x10, the field
    # `Inventory::presentation` reads); the Legacy row is the fallback for an id it lacks.
    tints = {row["id"]: int.from_bytes(bytes.fromhex(row["serialized_hex"])[16:20], "little")
             for row in json.loads(CATALOGUE.read_text(encoding="utf-8"))["definitions"]
             if len(row.get("serialized_hex") or "") >= 40}

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
    dropped_tricks = sorted(i for i, f in trick_fields.items() if f["name_id"] and f["name_id"] not in texts)
    tricks = {t["id"]: t for t in content["tricks"]}
    for trick_id, fields in sorted(trick_fields.items()):
        if trick_id in dropped_tricks:
            continue
        display_source = {"kind": "reference",
                          "evidence": f"Pets.json PetTricks Id={trick_id} (display fields; the client gesture row's "
                                      "own source stays)"}
        row = tricks.get(trick_id)
        if row is None:
            row = tricks[trick_id] = {"id": trick_id, "source": [
                {"kind": "reference", "evidence": f"Pets.json PetTricks Id={trick_id} (display fields; no "
                                                  "PetTrickGesturePoints.txt row, so no gesture: client 00cc9330 "
                                                  "scores nothing)"}]}
        elif row.get("waypoint_count") is not None and display_source not in row["source"]:
            # The seven client gesture rows keep their gesture source and gain where their display fields came from.
            row["source"].append(display_source)
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
                           "unknown_reasons": {"pet_row": "(R) Pets.json has no row for this model; a bound ADR "
                                                          "or a Type-2 item needs it, so it keeps model_id and "
                                                          "adr only"}}
    content["models"] = [models[k] for k in sorted(models)]

    # Items: Legacy Type-2 items whose ModelName resolves through Models.txt to a model row.
    by_adr = {}
    for mid, (adr, _) in client_models.items():
        by_adr.setdefault(adr.lower(), []).append(mid)
    items, unresolved = [], []
    for item in sorted((i for i in items_all if i.get("Type") == 2), key=lambda i: i["Id"]):
        ids = resolved_model_ids(item, by_adr, models)
        if not ids or item["NameId"] not in texts:
            unresolved.append(item["Id"])
            continue
        icon = item.get("Icon") or {}
        # Client evidence wins where the shipped catalogue disagrees with the Legacy icon tint (17849: 260 not 453).
        tint = icon.get("TintId")
        if tint is not None and tints.get(item["Id"]) not in (None, tint):
            tint = tints[item["Id"]]
        items.append({"item_id": item["Id"], "model_id": min(ids), "name_id": item["NameId"],
                      "name": texts[item["NameId"]],
                      "image_set_id": icon.get("Id"),
                      "icon_tint_id": tint,
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
