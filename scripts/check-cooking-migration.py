"""Read-only migration proof using synthetic snapshots; no campaign save is written."""
import argparse
from copy import deepcopy
import importlib.util
import json
from pathlib import Path
import shutil
import tempfile
from cooking_campaign_proof import sha, verify_variant


def main():
    project = Path(__file__).resolve().parents[1]
    lab = project.parent / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--content", type=Path, default=lab / "references/rust-opening-cooking-original-assets")
    content = parser.parse_args().content.resolve()
    spec = importlib.util.spec_from_file_location("cooking_migration", project / "scripts/migrate-cooking-campaign.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    marker, old = verify_variant(content, lab)
    document = {"version": 2, "character": {"guid": 1, "zone_id": 0, "position": [1, 2, 3], "rotation": [0, 0, 1]},
                "gameplay": {"inventory": {"coins": 100, "items": [{"synthetic_preservation": True}], "equipment": []},
                             "identity_fingerprint": "synthetic-preservation-test", "opening": {
                                 "version": 1, "content_sha256": module.OLD, "discoveries": [1, 2, 3],
                                 "stages": {str(s["id"]): {"accepted": False, "completed": False, "rewarded": False,
                                                          "counts": [0]*len(s["objectives"])} for s in old["stages"]}},
                             "encounter": {"active": None}, "smelting": {"active": None}, "forging": {"active": None}}}
    for key in ["910001", "910002"]:
        document["gameplay"]["opening"]["stages"][key] = {"accepted": True, "completed": True, "rewarded": True, "counts": [1, 1]}
    document["gameplay"]["opening"]["stages"]["910003"]["accepted"] = True
    document["gameplay"]["opening"]["stages"]["910003"]["counts"] = [1, 1, 0, 0]
    encode = lambda d: json.dumps(d).encode()
    data, _ = module.migrate(encode(document), content)
    result = json.loads(data)
    assert result["gameplay"]["opening"]["content_sha256"] == marker["opening_sha256"]
    result["gameplay"]["opening"]["content_sha256"] = module.OLD
    assert result == document
    rejected = 0
    for change in ["hash", "accepted", "counts", "completed", "rewarded", "layout", "missing", "smelting", "forging", "cooking", "encounter", "version", "size"]:
        bad = deepcopy(document)
        p = bad["gameplay"]["opening"]["stages"]["910004"]
        if change == "hash": bad["gameplay"]["opening"]["content_sha256"] = "0"*64
        elif change in ["accepted", "completed", "rewarded"]: p[change] = True
        elif change == "counts": p["counts"][0] = 1
        elif change == "layout": p["counts"].append(0)
        elif change == "missing": del bad["gameplay"]["opening"]["stages"]["910004"]
        elif change in ["smelting", "forging", "cooking", "encounter"]: bad["gameplay"][change] = {"active": {"synthetic": True}}
        elif change == "version": bad["version"] = 1
        try:
            module.migrate(b" "*1048577 if change == "size" else encode(bad), content)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError(f"accepted unsafe state: {change}")
    # Corrupt a NEW temporary private copy and update claimed hashes so checks
    # exercise exact transformation proof rather than only checksum mismatch.
    lab_tmp = lab / "tmp"
    lab_tmp.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="cooking-migration-", dir=lab_tmp) as temporary:
        target = Path(temporary)
        for change in ["earlier-stage", "future-threshold", "actor-x", "artifact-model", "discovery", "inventory"]:
            for path in content.iterdir():
                if path.is_file(): shutil.copy2(path, target / path.name)
            path = target / ("inventory.json" if change == "inventory" else "opening.json")
            value = json.loads(path.read_bytes())
            if change == "earlier-stage": value["stages"][0]["reward"]["coins"] += 1
            elif change == "future-threshold": value["stages"][3]["objectives"][3]["required"] = 5
            elif change == "actor-x": value["actors"][0]["position"][0] += 1
            elif change == "artifact-model":
                actor = next(a for a in value["actors"] if a["guid"] == 5715144128442925077)
                actor["model_id"] = 1579
                actor["model_file_name"] = "gw_precursor_artifact_01.adr"
            elif change == "discovery": value["discoveries"][0]["radius"] += 1
            elif change == "inventory": value["definitions"][0]["default_tint"] += 1
            path.write_bytes(encode(value))
            fake_marker = json.loads((target / "cooking-campaign.json").read_bytes())
            if change != "inventory": fake_marker["opening_sha256"] = sha(path.read_bytes())
            (target / "cooking-campaign.json").write_bytes(encode(fake_marker))
            try:
                module.migrate(encode(document), target)
            except ValueError:
                rejected += 1
            else:
                raise AssertionError(f"accepted unproved content: {change}")
    assert rejected == 19
    print(json.dumps({"new_content_sha256": marker["opening_sha256"], "synthetic_preservation": "passed",
                      "invalid_transformations_rejected": rejected, "save_files_written": 0}))


if __name__ == "__main__":
    main()
