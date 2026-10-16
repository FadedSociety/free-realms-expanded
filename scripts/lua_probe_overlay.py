"""Validate and replace ScriptsBase.bin only in a fresh run copy."""
import hashlib
import json
import shutil
from pathlib import Path

MAX_JSON = 1024 * 1024
BASE = "UI/ScriptsBase.bin"
LOG = "Logs/QuestOfferLuaProbe.log"
HEADER = bytes.fromhex("1b4c75615100010404040800")
SENTINEL = b"probe.scriptsbase.loaded"


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()


def read_json(path):
    path = Path(path)
    if path.stat().st_size > MAX_JSON:
        raise ValueError("Lua probe manifest exceeds size bound")
    return json.loads(path.read_text(encoding="utf-8"))


def validate_lua_probe(package, original_client, lab):
    """Verify the source-only package without touching the shared client."""
    if package is None:
        return None
    package, lab, original_client = (Path(x).resolve() for x in (package, lab, original_client))
    if package != (lab / "content/lua-probe").resolve():
        raise ValueError("Lua probe package must be lab/content/lua-probe")

    manifest_path = package / "lua-probe.json"
    manifest = read_json(manifest_path)
    if (manifest.get("schema_version"), manifest.get("kind"), manifest.get("client_target"),
            manifest.get("log_relative_path"), manifest.get("load_command"),
            manifest.get("manual_load_required")) != (
            3, "quest-offer-lua-probe", BASE, LOG, None, False):
        raise ValueError("Lua probe manifest schema/target/load mode mismatch")
    if manifest.get("native_client_load_tested") is not False or manifest.get("source_luac_used") is not False:
        raise ValueError("Lua probe source provenance flags mismatch")

    toolchain_path = (package / manifest.get("toolchain", "")).resolve()
    toolchain_root = (lab / "tools/lua-probe").resolve()
    if not toolchain_path.is_relative_to(toolchain_root) or not toolchain_path.is_file():
        raise ValueError("Lua probe toolchain manifest path mismatch")
    toolchain = read_json(toolchain_path)
    if (toolchain.get("runtime"), toolchain.get("source_format"),
            toolchain.get("source_luac_used"), toolchain.get("native_client_load_tested")) != (
            "embedded Lua 5.1", "UTF-8 Lua 5.1 source without BOM", False, False):
        raise ValueError("Lua probe source/runtime provenance mismatch")
    if digest(toolchain_path) != manifest.get("toolchain_sha256"):
        raise ValueError("Lua probe toolchain manifest hash mismatch")

    base = (original_client / BASE).resolve()
    if not base.is_relative_to(original_client) or digest(base) != manifest.get("original_scriptsbase_sha256"):
        raise ValueError("Lua probe original ScriptsBase.bin hash mismatch")
    if base.read_bytes()[:len(HEADER)] != HEADER:
        raise ValueError("Lua probe original Lua bytecode header mismatch")

    source_name = manifest.get("source_file")
    if source_name != Path(BASE).name:
        raise ValueError("Lua ScriptsBase source filename mismatch")
    source = (package / source_name).resolve()
    if not source.is_relative_to(package) or not source.is_file():
        raise ValueError("Lua probe source path mismatch")
    source_bytes = source.read_bytes()
    source_hash = manifest.get("scriptsbase_source_sha256")
    if (digest(source) != source_hash or not source_bytes
            or source_bytes.startswith(b"\xef\xbb\xbf")):
        raise ValueError("Lua ScriptsBase source hash/encoding mismatch")
    if SENTINEL not in source_bytes.splitlines()[0] or b"probe.loaded" not in source_bytes:
        raise ValueError("Lua ScriptsBase source lacks its first-line sentinel or appended probe")

    probe_name = manifest.get("probe_source_file")
    if probe_name != "QuestOfferLuaProbe.lua":
        raise ValueError("Lua probe source filename mismatch")
    probe_source = (package / probe_name).resolve()
    if (not probe_source.is_relative_to(package) or not probe_source.is_file()
            or digest(probe_source) != manifest.get("probe_source_sha256")):
        raise ValueError("Lua probe append-source hash mismatch")
    syntax = toolchain.get("syntax_checker", {})
    if (syntax.get("source_sha256") != source_hash
            or syntax.get("sha256") != manifest.get("syntax_checker_sha256")
            or syntax.get("token_count") != manifest.get("syntax_token_count")
            or manifest.get("syntax_source_sha256") != source_hash):
        raise ValueError("Lua 5.1 syntax-check provenance mismatch")

    legacy = manifest.get("legacy_failed_bytecode", {})
    legacy_path = (package / legacy.get("path", "")).resolve()
    if (not legacy_path.is_relative_to(package) or not legacy_path.is_file()
            or digest(legacy_path) != legacy.get("sha256")):
        raise ValueError("archived W70 bytecode proof mismatch")

    return {
        "source": source,
        "target": BASE,
        "log_relative_path": LOG,
        "original_sha256": manifest["original_scriptsbase_sha256"],
        "source_sha256": source_hash,
        "probe_source_sha256": manifest["probe_source_sha256"],
        "load_command": None,
        "proof": {
            "package": str(package),
            "manifest_sha256": digest(manifest_path),
            "original_scriptsbase_sha256": manifest["original_scriptsbase_sha256"],
            "scriptsbase_source_sha256": source_hash,
            "probe_source_sha256": manifest["probe_source_sha256"],
            "toolchain_sha256": manifest["toolchain_sha256"],
            "load_command": None,
            "manual_load_required": False,
            "native_client_load_tested": False,
            "log_relative_path": LOG,
        },
    }


def apply_lua_probe(validated, client, run, lab):
    """Copy the source probe only into a new disposable run client."""
    if validated is None:
        return None
    client, run, lab = Path(client).resolve(), Path(run).resolve(), Path(lab).resolve()
    runs_root = (lab / "runs").resolve()
    if client != run / "client" or run == runs_root or not run.is_relative_to(runs_root):
        raise ValueError("Lua probe target must be a new disposable run client")
    if any((run / name).exists() for name in ("server-stdout.log", "client-stdout.log", "external-inputs.jsonl")):
        raise ValueError("Cannot apply Lua probe to an active or existing run")

    base = (client / BASE).resolve()
    if not base.is_relative_to(client) or not base.is_file() or digest(base) != validated["original_sha256"]:
        raise ValueError("Run-copy ScriptsBase.bin changed before Lua probe application")
    source = Path(validated["source"]).resolve()
    if digest(source) != validated["source_sha256"]:
        raise ValueError("Lua probe source changed after validation")
    if validated["target"] != BASE:
        raise ValueError("Lua probe target mismatch")
    log = (client / validated["log_relative_path"]).resolve()
    if not log.is_relative_to(client):
        raise ValueError("Lua probe log path escapes run client")
    log.unlink(missing_ok=True)
    shutil.copy2(source, base)
    base_after = digest(base)
    if base_after != validated["source_sha256"]:
        raise ValueError("Run-copy ScriptsBase source replacement hash mismatch")

    proof = dict(validated["proof"])
    proof.update(
        run_target=str(base),
        run_source_sha256=base_after,
        run_scriptsbase_sha256_before=validated["original_sha256"],
        run_scriptsbase_sha256_after=base_after,
        run_log=str(log),
    )
    return proof
