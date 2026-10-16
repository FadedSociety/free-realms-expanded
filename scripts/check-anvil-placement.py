"""Isolated anvil package/migration checks; never reads or writes campaign saves."""
from copy import deepcopy
import json
from pathlib import Path
import tempfile
from types import SimpleNamespace
import anvil_placement as migration
from journal_text_overlay import validate_journal_text


def main():
    old, marker = migration.verify_package()
    original_client = migration.LAB / "working/OSFR.Client/OSFR Client"
    locale = validate_journal_text(migration.PACKAGE, migration.PACKAGE / "content", original_client, migration.LAB)
    assert locale["proof"]["opening_sha256"] == migration.NEW
    stages = {}
    for index, stage in enumerate(old["stages"]):
        completed = index < 6
        stages[str(stage["id"])] = {"accepted": index <= 6, "completed": completed, "rewarded": completed,
                                   "counts": [g["required"] if completed else 0 for g in stage["objectives"]]}
    document = {"version": 2, "character": {"guid": 123, "zone_id": 0, "position": [-2083.8, -56.1, 194.0], "rotation": [0, 0, 1, 0]},
                "gameplay": {"identity_fingerprint": "synthetic", "inventory": {"coins": 140, "items": [{"definition_id": 799, "instance_id": 9876543210123456789}]},
                             "profiles": {"active": 45, "xp": {"16": 25, "45": 40}},
                             "opening": {"version": 1, "content_sha256": migration.OLD, "stages": stages,
                                         "discoveries": [old["discoveries"][0]["id"]]},
                             "forging": {"active": None, "history": [{"phase": "won", "score": 6600}]},
                             "cooking": {"active": None, "history": [{"score": 5133}]},
                             "smelting": {"active": None}, "encounter": {"active": None},
                             "future_component": {"keep": [True, None, "unknown field"]}}, "future_top": {"opaque": "preserve"}}
    raw = (json.dumps(document, indent=3) + "\n  ").encode()
    forward, _ = migration.migrate_bytes(raw)
    assert forward == raw.replace(migration.OLD.encode(), migration.NEW.encode())
    rollback, _ = migration.migrate_bytes(forward, direction="rollback")
    assert rollback == raw
    rejected = []

    def reject(name, action):
        try:
            action()
        except (ValueError, OSError):
            rejected.append(name)
        else:
            raise AssertionError("accepted unsafe case: " + name)

    def altered(name, edit):
        value = deepcopy(document)
        edit(value)
        reject(name, lambda: migration.migrate_bytes(migration.encode(value)))

    opening = lambda d: d["gameplay"]["opening"]
    altered("wrong snapshot version", lambda d: d.update(version=True))
    altered("wrong opening version", lambda d: opening(d).update(version=True))
    altered("wrong content hash", lambda d: opening(d).update(content_sha256="0" * 64))
    altered("missing stage", lambda d: opening(d)["stages"].pop(str(old["stages"][0]["id"])))
    first = str(old["stages"][0]["id"])
    altered("count width", lambda d: opening(d)["stages"][first]["counts"].append(0))
    altered("boolean count", lambda d: opening(d)["stages"][first]["counts"].__setitem__(0, True))
    altered("overlarge count", lambda d: opening(d)["stages"][first]["counts"].__setitem__(0, 9999))
    altered("reward mismatch", lambda d: opening(d)["stages"][first].update(rewarded=False))
    altered("out of order acceptance", lambda d: opening(d)["stages"][str(old["stages"][8]["id"])].update(accepted=True))
    altered("unknown discovery", lambda d: opening(d).update(discoveries=[99999999]))
    altered("duplicate discovery", lambda d: opening(d)["discoveries"].append(old["discoveries"][0]["id"]))
    altered("ambiguous repeated hash", lambda d: d.update(extra_hash=migration.OLD))
    for name in ("forging", "cooking", "smelting", "encounter"):
        for phase in ("playing", "won", "lost"):
            altered("active " + name + "/" + phase, lambda d, n=name, p=phase: d["gameplay"][n].update(active={"phase": p}))
    reject("oversized snapshot", lambda: migration.migrate_bytes(b" " * (migration.MAX_STATE + 1)))
    reject("duplicate JSON key", lambda: migration.migrate_bytes(raw.replace(b'"version": 2,', b'"version": 2, "version": 2,', 1)))
    reject("nonfinite JSON", lambda: migration.migrate_bytes(raw.replace(b'"opaque": "preserve"', b'"opaque": NaN')))
    reject("wrong direction", lambda: migration.migrate_bytes(raw, direction="other"))

    # Existing process guard must reject native or gateway output, independently
    # of the fixture-only stopped-boundary stub used below.
    real_run = migration.subprocess.run
    try:
        for output in ("1234", "[1234,5678]"):
            migration.subprocess.run = lambda *a, **k: SimpleNamespace(stdout=output)
            reject("running process " + output, migration.require_stopped)
        migration.subprocess.run = lambda *a, **k: SimpleNamespace(stdout="[]")
        migration.require_stopped()
    finally:
        migration.subprocess.run = real_run

    temporary_root = migration.LAB / "tmp"
    temporary_root.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="anvil-placement-check-", dir=temporary_root) as directory:
        temporary = Path(directory)
        package = temporary / "private-package"
        migration.prepare(package)
        reject("existing package", lambda: migration.prepare(package))
        for name in ("content/opening.json", "content/encounters.json", "content/inventory.json", "content/journal-text.json", "locale/en_us_data.dir", migration.MARKER):
            path = package / name
            before = path.read_bytes()
            path.write_bytes(before + b" ")
            reject("tampered " + name, lambda: migration.verify_package(package))
            path.write_bytes(before)
        extra = package / "unapproved-file.txt"
        extra.write_text("unapproved")
        reject("unapproved package file", lambda: migration.verify_package(package))
        extra.unlink()
        # All files below are synthetic and disposable. Actual process checking
        # is stubbed only within this fixture; no campaign path is inspected.
        stopped = migration.require_stopped
        migration.require_stopped = lambda: None
        try:
            source, output = temporary / "synthetic-source.json", temporary / "synthetic-output.json"
            source.write_bytes(raw)
            digest = migration.sha(raw)
            reject("source SHA mismatch", lambda: migration.migrate_file(source, "0" * 64, output, package))
            reject("source equals output", lambda: migration.migrate_file(source, digest, source, package))
            reject("output inside package", lambda: migration.migrate_file(source, digest, package / "bad.json", package))
            proof = migration.migrate_file(source, digest, output, package)
            assert source.read_bytes() == raw and output.read_bytes() == forward
            assert migration.migrate_file(source, digest, output, package) == proof
            reverse = temporary / "synthetic-rollback.json"
            migration.migrate_file(output, migration.sha(forward), reverse, package, "rollback")
            assert reverse.read_bytes() == raw
            conflict = temporary / "existing.json"
            conflict.write_bytes(b"existing unrelated data")
            reject("existing different output", lambda: migration.migrate_file(source, digest, conflict, package))
            assert conflict.read_bytes() == b"existing unrelated data"
            proof_conflict = temporary / "proof-conflict.json"
            proof_path = proof_conflict.with_suffix(".json.migration.json")
            proof_path.write_bytes(b"different proof")
            reject("existing different proof", lambda: migration.migrate_file(source, digest, proof_conflict, package))
            assert not proof_conflict.exists()
            real_migrate = migration.migrate_bytes
            def changing_source(*args, **kwargs):
                result = real_migrate(*args, **kwargs)
                source.write_bytes(raw + b" ")
                return result
            migration.migrate_bytes = changing_source
            reject("source changes during preparation", lambda: migration.migrate_file(source, digest, temporary / "race.json", package))
            assert not (temporary / "race.json").exists()
            migration.migrate_bytes = real_migrate
        finally:
            migration.require_stopped = stopped
    migration.verify_package()
    print(json.dumps({"package": str(migration.PACKAGE), "opening_sha256": migration.NEW,
                      "package_marker_sha256": migration.sha((migration.PACKAGE / migration.MARKER).read_bytes()),
                      "progressed_state_forward_and_byte_exact_rollback": True, "locale_launch_validation": True,
                      "filesystem_idempotency_and_source_preservation": True, "rejected_cases": len(rejected),
                      "rejections": rejected, "campaign_saves_read_or_written": 0}, indent=2))


if __name__ == "__main__":
    main()
