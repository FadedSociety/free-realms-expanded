"""Read-only migration proof checks against private content; no save is written."""
from copy import deepcopy
import argparse
import importlib.util
import json
from pathlib import Path


def main():
    project = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--content", type=Path, default=project.parent / "lab/references/rust-opening-forging")
    content = parser.parse_args().content.resolve()
    spec = importlib.util.spec_from_file_location("forging_migration", project / "scripts/migrate-forging-campaign.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    old = json.loads((content / "prior-opening.json").read_bytes())
    # Fully synthetic early save exercises preservation, not a user checkpoint.
    document = {"version": 2, "character": {"guid": 1, "zone_id": 0,
        "position": [1, 2, 3], "rotation": [0, 0, 1]}, "gameplay": {
        "inventory": {"coins": 75, "items": [], "equipment": []},
        "identity_fingerprint": "synthetic-preservation-test",
        "opening": {"version": 1, "content_sha256": module.OLD, "discoveries": [1, 2, 3],
                    "stages": {str(s["id"]): {"accepted": False, "completed": False,
                         "rewarded": False, "counts": [0]*len(s["objectives"])} for s in old["stages"]}},
        "encounter": {"active": None}, "smelting": {"active": None}}}
    document["gameplay"]["opening"]["stages"]["910001"] = {
        "accepted": True, "completed": True, "rewarded": True, "counts": [1, 1]}
    document["gameplay"]["opening"]["stages"]["910002"] = {
        "accepted": True, "completed": True, "rewarded": True, "counts": [1, 1]}
    document["gameplay"]["opening"]["stages"]["910003"]["accepted"] = True
    def encode(d):
        return json.dumps(d).encode()
    data, marker = module.migrate(encode(document), content)
    result = json.loads(data)
    assert result["gameplay"]["opening"]["content_sha256"] == marker["opening_sha256"]
    result["gameplay"]["opening"]["content_sha256"] = module.OLD
    assert result == document
    rejected = 0
    for change in ["hash", "accepted", "counts", "smelting", "forging", "encounter", "version", "size"]:
        bad = deepcopy(document)
        if change == "hash": bad["gameplay"]["opening"]["content_sha256"] = "0"*64
        elif change == "accepted": bad["gameplay"]["opening"]["stages"]["910004"]["accepted"] = True
        elif change == "counts": bad["gameplay"]["opening"]["stages"]["910015"]["counts"][0] = 1
        elif change in ["smelting", "forging", "encounter"]: bad["gameplay"][change] = {"active": {"synthetic": True}}
        elif change == "version": bad["version"] = 1
        try:
            module.migrate(b" "*1048577 if change == "size" else encode(bad), content)
        except ValueError:
            rejected += 1
        else:
            raise AssertionError(f"accepted unsafe transformation: {change}")
    assert rejected == 8
    print(json.dumps({"new_content_sha256": marker["opening_sha256"], "synthetic_preservation": "passed",
                      "invalid_transformations_rejected": rejected, "save_files_written": 0}))


if __name__ == "__main__":
    main()
