"""Exercise private presentation preflight without launching or touching a save."""
import argparse
import copy
import json
import shutil
import tempfile
from pathlib import Path

from journal_text_overlay import (STOCK, digest, validate_journal_text, apply_journal_text,
                                  validate_character_select_title, apply_character_select_title)


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--package", type=Path, required=True)
    parser.add_argument("--character-select-title", type=Path)
    parser.add_argument("--report", type=Path)
    args = parser.parse_args()
    package = args.package.resolve()
    original = lab / "working/OSFR.Client/OSFR Client"
    baseline = validate_journal_text(package, package / "content", original, lab)
    assert validate_journal_text(None, lab / "references/rust-opening-cooking-original-assets", original, lab) is None
    checks = []

    def rejects(name, operation):
        try:
            operation()
        except (ValueError, OSError, KeyError, TypeError):
            checks.append(name)
            return
        raise AssertionError("Unsafe transformation accepted: " + name)

    rejects("mapped-content-without-opt-in", lambda: validate_journal_text(None, package / "content", original, lab))
    rejects("overlay-with-unmapped-content", lambda: validate_journal_text(package, lab / "references/rust-opening-cooking-original-assets", original, lab))
    with tempfile.TemporaryDirectory(prefix="journal-preflight-", dir=lab / "reports") as temporary:
        work = Path(temporary).resolve()
        assert work.is_relative_to(lab.resolve())
        clone = work / "package"
        shutil.copytree(package, clone)
        manifest_path, mapping_path, roundtrip_path = clone / "journal-text-overlay.json", clone / "content/journal-text.json", clone / "roundtrip.json"
        saved = {p: p.read_bytes() for p in (manifest_path, mapping_path, roundtrip_path)}
        manifest = json.loads(saved[manifest_path]); mapping = json.loads(saved[mapping_path]); roundtrip = json.loads(saved[roundtrip_path])

        def write(path, value):
            path.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")

        for case in ("opening-hash", "mapping-hash", "locale", "pair-missing", "traversal-target", "source-hash", "output-hash", "proof-hash", "stock-export-hash", "source-goal", "unmapped-zero", "zero-id", "stock-id-collision"):
            for path, data in saved.items():
                path.write_bytes(data)
            changed_manifest, changed_mapping, changed_proof = copy.deepcopy(manifest), copy.deepcopy(mapping), copy.deepcopy(roundtrip)
            if case == "opening-hash": changed_manifest["opening_sha256"] = "00"
            elif case == "mapping-hash": changed_manifest["mapping_sha256"] = "00"
            elif case == "locale": changed_manifest["locale"] = 4
            elif case == "pair-missing": changed_manifest["files"].pop()
            elif case == "traversal-target": changed_manifest["files"][0]["file"] = "../en_us_data.dat"
            elif case == "source-hash": changed_manifest["files"][0]["source_sha256"] = "00"
            elif case == "output-hash": changed_manifest["files"][0]["sha256"] = "00"
            elif case == "proof-hash": changed_manifest["roundtrip_sha256"] = "00"
            elif case == "stock-export-hash": changed_manifest["stock_locale_export_sha256"] = "00"
            elif case == "source-goal": changed_mapping["entries"][0]["source_name_id"] = 99
            elif case == "unmapped-zero": changed_mapping["entries"].pop(0)
            elif case == "zero-id": changed_mapping["entries"][0]["name_id"] = 0
            elif case == "stock-id-collision": changed_proof["added"][0]["id"] = 1
            if changed_mapping != mapping:
                write(mapping_path, changed_mapping)
                changed_manifest["mapping_sha256"] = digest(mapping_path)
            if changed_proof != roundtrip:
                write(roundtrip_path, changed_proof)
                changed_manifest["roundtrip_sha256"] = digest(roundtrip_path)
            write(manifest_path, changed_manifest)
            rejects(case, lambda: validate_journal_text(clone, clone / "content", original, lab))
        for path, data in saved.items():
            path.write_bytes(data)
        validated = validate_journal_text(clone, clone / "content", original, lab)
        with tempfile.TemporaryDirectory(prefix="journal-copy-check-", dir=lab / "runs") as disposable:
            run = Path(disposable).resolve(); client = run / "client"
            assert run.is_relative_to((lab / "runs").resolve())
            shutil.copytree(original / "locale", client / "locale")
            (run / "client-stdout.log").write_text("active")
            rejects("active-run", lambda: apply_journal_text(validated, client, run, lab))
            (run / "client-stdout.log").unlink()
            rejects("outside-run", lambda: apply_journal_text(validated, client, work, lab))
            apply_journal_text(validated, client, run, lab)
            assert all(digest(client / name) == output for _, name, _, output in validated["plan"])
            if args.character_select_title:
                title = validate_character_select_title(args.character_select_title, lab)
                apply_character_select_title(title, client, run, lab)
                assert digest(client / title["target"]) == title["proof"]["patched_sha256"]
                (run / "client-stdout.log").write_text("active")
                rejects("title-active-run", lambda: apply_character_select_title(title, client, run, lab))
                (run / "client-stdout.log").unlink()
                (client / title["target"]).write_bytes(b"unrelated custom movie")
                rejects("title-unrelated-custom-movie", lambda: apply_character_select_title(title, client, run, lab))
                title_clone = work / "title"
                shutil.copytree(args.character_select_title, title_clone)
                title_manifest = title_clone / "character-select-title.json"
                title_saved = json.loads(title_manifest.read_text())
                for field in ("target", "patched_movie_sha256", "original_movie_sha256", "proof_sha256"):
                    wrong = copy.deepcopy(title_saved); wrong[field] = "../wrong"
                    write(title_manifest, wrong)
                    rejects("title-" + field, lambda: validate_character_select_title(title_clone, lab))
    assert all(digest(original / name) == value for name, value in STOCK.items())
    report = {"schema_version": 1, "scope": "Offline presentation preflight; no process launched and no save accessed", "package": str(package),
              "opening_sha256": baseline["proof"]["opening_sha256"], "mapping_sha256": baseline["proof"]["mapping_sha256"],
              "paired_copy_verified": True, "original_pair_unchanged": True, "rejected_count": len(checks), "rejected": checks}
    if args.report:
        if args.report.exists(): raise ValueError("Report must be a NEW file")
        args.report.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
