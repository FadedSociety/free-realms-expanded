"""Narrow private actor appearance/placement migration proof, shared by helpers."""
from copy import deepcopy
import hashlib
import math
from pathlib import Path


def apply_actor_repairs(actors, repairs, lab):
    if repairs.get("schema_version") != 1:
        raise ValueError("unsupported actor repair proof")
    # Proof files are immutable local evidence, not fetched during preparation.
    files = repairs.get("proof_files") or []
    if not files:
        raise ValueError("actor repair requires source hash proofs")
    for proof in files:
        path = Path(proof["path"]).resolve()
        if not path.is_relative_to(lab.resolve()) or hashlib.sha256(path.read_bytes()).hexdigest() != proof["sha256"]:
            raise ValueError("actor asset proof file differs")
    result = deepcopy(actors)
    by_guid = {a["guid"]: a for a in result}
    seen = set()
    allowed = {"model_id", "model_file_name", "position", "provenance"}
    for repair in repairs["actors"]:
        guid, before, after = repair["guid"], repair["before"], repair["after"]
        if guid in seen or guid not in by_guid or not before or set(before) != set(after) or not set(before).issubset(allowed):
            raise ValueError("actor repair changes unsupported fields or identity")
        seen.add(guid)
        actor = by_guid[guid]
        if any(actor[k] != v for k, v in before.items()):
            raise ValueError("actor repair source differs")
        if "model_id" in after and (type(after["model_id"]) is not int or after["model_id"] <= 0):
            raise ValueError("invalid actor model identity")
        if "model_file_name" in after and not after["model_file_name"].endswith(".adr"):
            raise ValueError("unsupported actor model file")
        if "position" in after and (len(after["position"]) != 3 or any(not math.isfinite(n) or abs(n) > 1000000 for n in after["position"])):
            raise ValueError("invalid actor position")
        actor.update(after)
        notice = repair.get("provenance")
        if not isinstance(notice, str) or not notice.strip():
            raise ValueError("actor repair requires explicit provenance")
        actor["provenance"] = actor["provenance"] + " " + notice
    if not seen:
        raise ValueError("empty actor repair proof")
    return result
