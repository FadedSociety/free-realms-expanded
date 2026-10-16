"""Validate a versioned local journal locale package before creating a run.

The community tool owns locale parsing/writing. This module checks only JSON,
hashes and the two fixed filesystem targets; it does not parse a game format.
"""
import hashlib
import json
import shutil
from pathlib import Path


MAX_JSON = 2 * 1024 * 1024
MAX_LOCALE_ID = 5103267
STOCK = {
    "locale/en_us_data.dat": "9c764d76b05138e607f23276a3a03f364b22c44d2787e31bb96be1b4fe4fc097",
    "locale/en_us_data.dir": "a3835d4b69aab9308fabe188c1d72f211038b11f9efdbbddbab688212cd8c458",
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def read_json(path):
    if path.stat().st_size > MAX_JSON:
        raise ValueError("Overlay JSON exceeds size bound")
    return json.loads(path.read_text(encoding="utf-8"))


def validate_journal_text(package, content, original_client, lab):
    """Return a read-only copy plan; fail for absent/mismatched paired opt-in."""
    mapping_path = content / "journal-text.json" if content else None
    if package is None:
        if mapping_path and mapping_path.exists():
            raise ValueError("Content has journal-text.json; matching --journal-text is required")
        return None
    package, lab = Path(package).resolve(), Path(lab).resolve()
    if not package.is_relative_to(lab) or not mapping_path or not mapping_path.is_file():
        raise ValueError("Journal text package must be under lab and have matching content mapping")
    manifest_path = package / "journal-text-overlay.json"
    manifest = read_json(manifest_path)
    mapping = read_json(mapping_path)
    opening_hash = digest(content / "opening.json")
    if (manifest.get("schema_version"), manifest.get("kind"), manifest.get("locale")) != (1, "opening-journal-text", 8):
        raise ValueError("Invalid journal text package schema/kind/locale")
    if manifest.get("mapping_sha256") != digest(mapping_path) or manifest.get("opening_sha256") != opening_hash:
        raise ValueError("Journal text package does not match content")
    if mapping.get("schema_version") != 1 or mapping.get("locale") != 8 or mapping.get("opening_sha256") != opening_hash:
        raise ValueError("Journal text mapping does not match content/locale")
    if not mapping.get("notice") or not manifest.get("notice"):
        raise ValueError("Reconstructed label disclosure missing")
    community = manifest.get("community_tool", {})
    if community.get("revision") != "ba2ede225ff33edab8e8e0d20c45d4c5156309e5":
        raise ValueError("Unverified locale community writer")
    roundtrip_path = package / "roundtrip.json"
    if digest(roundtrip_path) != manifest.get("roundtrip_sha256"):
        raise ValueError("Journal text roundtrip proof changed")
    roundtrip = read_json(roundtrip_path)
    for field in ("all_original_entries_preserved", "all_original_ids_preserved", "new_ids_noncolliding", "strict_roundtrip", "md5_metadata_matches"):
        if roundtrip.get(field) is not True:
            raise ValueError("Journal text community roundtrip/preservation proof missing")
    stock_export = lab / "reports/locale-id-to-text.json"
    if digest(stock_export) != manifest.get("stock_locale_export_sha256"):
        raise ValueError("Stock locale export changed")
    stock_ids = {row["id"] for row in json.loads(stock_export.read_text(encoding="utf-8"))}
    if roundtrip.get("original_entry_count") != 113286 or roundtrip.get("original_id_count") != len(stock_ids):
        raise ValueError("Journal text original locale counts mismatch")
    added = roundtrip.get("added", [])
    new_ids = {row["id"]: row["text"] for row in added}
    if len(new_ids) != len(added) or not new_ids or any(type(ident) is not int or not 0 < ident <= MAX_LOCALE_ID or ident in stock_ids for ident in new_ids):
        raise ValueError("New locale IDs are duplicated, out of bounds or collide with stock")
    opening = read_json(content / "opening.json")
    goals = {(s["id"], g["id"]): g for s in opening["stages"] for g in s["objectives"]}
    mapped = set()
    used = set()
    for entry in mapping.get("entries", []):
        key = (entry["quest_id"], entry["objective_id"])
        source = goals.get(key)
        if key in mapped or source is None or (source["name_id"], source["description_id"]) != (entry["source_name_id"], entry["source_description_id"]):
            raise ValueError("Invalid journal text objective binding")
        mapped.add(key)
        for id_field, text_field in (("name_id", "name"), ("description_id", "description")):
            ident, text = entry[id_field], entry[text_field]
            if new_ids.get(ident) != text or not text or len(text.encode("utf-8")) > 512 or any(ord(c) < 32 for c in text):
                raise ValueError("Journal text label does not match community locale output")
            used.add(ident)
    if used != set(new_ids) or any((g["name_id"] == 0 or g["description_id"] == 0) and key not in mapped for key, g in goals.items()):
        raise ValueError("Journal text mapping is incomplete")
    if (manifest.get("goal_count"), manifest.get("mapped_goal_count"), manifest.get("new_string_count")) != (len(goals), len(mapped), len(new_ids)):
        raise ValueError("Journal text manifest count mismatch")
    files = manifest.get("files", [])
    if len(files) != 2 or {row.get("file") for row in files} != set(STOCK):
        raise ValueError("Locale override must contain exactly the fixed paired targets")
    plan = []
    for entry in files:
        name = entry["file"]
        source = (package / name).resolve()
        if not source.is_relative_to(package) or source.stat().st_size != entry.get("length") or not 0 < source.stat().st_size <= 32 * 1024 * 1024:
            raise ValueError("Invalid journal text locale path/size")
        if entry.get("source_sha256") != STOCK[name] or digest(original_client / name) != STOCK[name]:
            raise ValueError("Original client locale pair changed")
        if digest(source) != entry.get("sha256"):
            raise ValueError("Journal text locale output hash mismatch")
        plan.append((source, name, STOCK[name], entry["sha256"]))
    return {"plan": plan, "proof": {"package": str(package), "manifest_sha256": digest(manifest_path),
            "mapping_sha256": digest(mapping_path), "opening_sha256": opening_hash, "locale": 8,
            "notice": manifest["notice"], "mapped_goals": len(mapped), "new_strings": len(new_ids), "files": files}}


def apply_journal_text(validated, client, run, lab):
    """Apply only to the just-created disposable run, before any process starts."""
    if validated is None:
        return
    client, run, lab = Path(client).resolve(), Path(run).resolve(), Path(lab).resolve()
    if client != run / "client" or not run.is_relative_to(lab / "runs"):
        raise ValueError("Journal text target must be the new run client")
    if any((run / name).exists() for name in ("server-stdout.log", "client-stdout.log", "external-inputs.jsonl")):
        raise ValueError("Cannot change locale in an existing/active run")
    # Check both before writing either; no process is running if copying fails.
    for source, name, expected, output in validated["plan"]:
        if not (client / name).resolve().is_relative_to(client) or digest(client / name) != expected or digest(source) != output:
            raise ValueError("Locale pair changed between validation and application")
    for source, name, _, output in validated["plan"]:
        shutil.copy2(source, client / name)
        if digest(client / name) != output:
            raise ValueError("Copied journal text locale hash mismatch")


def validate_character_select_title(package, lab):
    """Validate the separately reviewed one-operand original-movie repair."""
    if package is None:
        return None
    package, lab = Path(package).resolve(), Path(lab).resolve()
    if not package.is_relative_to(lab):
        raise ValueError("Character title package must be under lab")
    manifest_path = package / "character-select-title.json"
    manifest = read_json(manifest_path)
    target = "custom/characterSelectV2.gfx"
    original_hash = "1aa6779ca0168a01dc25860d0ea4e94e5f8c2ccd01bc6781acc46847bbbbe176"
    if (manifest.get("schema_version"), manifest.get("kind"), manifest.get("file"), manifest.get("target"), manifest.get("proof_file")) != (1, "character-select-title", target, target, "proof.json"):
        raise ValueError("Invalid character title overlay schema/target")
    original = lab / "extracted/client-assets/pack_18/006072.gfx"
    movie = (package / target).resolve()
    if not movie.is_relative_to(package) or manifest.get("original_movie_sha256") != original_hash or digest(original) != original_hash:
        raise ValueError("Character title original/path mismatch")
    if movie.stat().st_size != manifest.get("patched_movie_bytes") or not 0 < movie.stat().st_size < MAX_JSON or digest(movie) != manifest.get("patched_movie_sha256"):
        raise ValueError("Character title movie output hash/length mismatch")
    proof_path = package / "proof.json"
    if digest(proof_path) != manifest.get("proof_sha256"):
        raise ValueError("Character title proof hash mismatch")
    proof = read_json(proof_path)
    if (proof.get("original", {}).get("sha256"), proof.get("patched", {}).get("sha256"), proof.get("patched", {}).get("bytes")) != (original_hash, digest(movie), movie.stat().st_size):
        raise ValueError("Character title proof does not bind movie pair")
    if proof.get("xml_difference_count") != 1 or proof.get("original_movie_unchanged") is not True or len(proof.get("exact_script_comparisons", [])) != 2 or any(
            row.get("script_count") != 56 or row.get("byte_identical_export_count") != 55 or row.get("exact_renderer_one_operand_change") is not True
            for row in proof["exact_script_comparisons"]):
        raise ValueError("Character title exact one-operand proof missing")
    return {"source": movie, "target": target, "original_sha256": original_hash,
            "proof": {"package": str(package), "manifest_sha256": digest(manifest_path),
                      "original_sha256": original_hash, "patched_sha256": digest(movie),
                      "proof_sha256": digest(proof_path), "native_acceptance_pending": True}}


def apply_character_select_title(validated, client, run, lab):
    if validated is None:
        return
    client, run, lab = Path(client).resolve(), Path(run).resolve(), Path(lab).resolve()
    if client != run / "client" or not run.is_relative_to(lab / "runs") or any((run / name).exists() for name in ("server-stdout.log", "client-stdout.log", "external-inputs.jsonl")):
        raise ValueError("Character title target must be a new disposable run")
    target = (client / validated["target"]).resolve()
    if not target.is_relative_to(client) or (target.exists() and digest(target) != validated["original_sha256"]):
        raise ValueError("Character title would overwrite an unrelated custom movie")
    if digest(validated["source"]) != validated["proof"]["patched_sha256"]:
        raise ValueError("Character title source changed after validation")
    target.parent.mkdir(exist_ok=True)
    shutil.copy2(validated["source"], target)
    if digest(target) != validated["proof"]["patched_sha256"]:
        raise ValueError("Character title copied hash mismatch")
