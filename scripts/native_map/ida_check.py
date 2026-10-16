"""Deterministic IDA facts for a sample of accepted native-map labels."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import os
import random
import re
import sqlite3
import sys
from datetime import datetime, timezone
from pathlib import Path
from types import SimpleNamespace
from typing import Any

sys.dont_write_bytecode = True


PROJECT_ROOT = Path(__file__).resolve().parents[2]
LAB_ROOT = PROJECT_ROOT.parent / "lab"
SAMPLE_ROOT = LAB_ROOT / "build" / "w97"
CLIENT_ROOT = LAB_ROOT / "working" / "ida"
CLIENT_BINARY = CLIENT_ROOT / "FreeRealms.exe"
IDA_INSTALL = Path(r"C:\Program Files\IDA Professional 9.4")
MAP_SCRIPT = PROJECT_ROOT / "scripts" / "native_map" / "nmq.py"
MAP_ROOT = LAB_ROOT / "native-map"
OUTPUT_DIR = MAP_ROOT / "run" / "ida-check"
CONFIDENCE = ("confirmed", "likely")
RUN_NAME = re.compile(r"^[A-Za-z0-9][A-Za-z0-9_.-]{0,79}$")
SERIALIZER = re.compile(r"serializ|marshal|encode|write.{0,20}(packet|bytes|blob)|packet.{0,20}write", re.I)


def _within(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
        return True
    except ValueError:
        return False


def validate_binary(binary: Path) -> tuple[Path, bool, bool]:
    """Return resolved path, whether it is the client, and whether to auto-analyze."""
    path = binary.expanduser().resolve(strict=True)
    if not path.is_file():
        raise ValueError(f"binary is not a file: {path}")

    sample_root = SAMPLE_ROOT.resolve()
    client_binary = CLIENT_BINARY.resolve()
    is_client = path == client_binary
    if not is_client and not _within(path, sample_root):
        raise ValueError(f"binary must be the FreeRealms client or live under {sample_root}")

    idb = Path(str(path) + ".i64")
    id0 = Path(str(path) + ".id0")
    if idb.exists() and not _within(idb.resolve(), path.parent):
        raise RuntimeError(f"IDA database sidecar escapes its binary directory: {idb}")
    if is_client:
        if not idb.is_file():
            raise RuntimeError(f"client IDA database is not ready: missing {idb}")
        if id0.exists():
            raise RuntimeError(f"client database is still open: {id0} exists; do not open it")
        return path, True, False

    if id0.exists():
        raise RuntimeError(f"sample database is still open: {id0} exists; close its IDA session first")
    return path, False, not idb.exists()


def load_ida_api() -> SimpleNamespace:
    if not (IDA_INSTALL / "idalib.dll").is_file():
        raise RuntimeError(f"IDA 9.4 idalib.dll is missing from {IDA_INSTALL}")
    os.environ["IDADIR"] = str(IDA_INSTALL)
    import idapro

    import ida_auto
    import ida_bytes
    import ida_funcs
    import ida_hexrays
    import ida_idp
    import ida_lines
    import ida_name
    import ida_nalt
    import ida_ua
    import idautils

    version = idapro.get_library_version()
    return SimpleNamespace(
        idapro=idapro,
        ida_auto=ida_auto,
        ida_bytes=ida_bytes,
        ida_funcs=ida_funcs,
        ida_hexrays=ida_hexrays,
        ida_idp=ida_idp,
        ida_lines=ida_lines,
        ida_name=ida_name,
        ida_nalt=ida_nalt,
        ida_ua=ida_ua,
        idautils=idautils,
        hexrays_available=False,
        version=version,
        opened=False,
        is_client=False,
    )


def open_database(binary: Path) -> tuple[SimpleNamespace, bool]:
    path, is_client, auto_analysis = validate_binary(binary)
    api = load_ida_api()
    if api.idapro.open_database(str(path), run_auto_analysis=auto_analysis) != 0:
        api.idapro.close_database(False)
        raise RuntimeError(f"IDA could not open {path}")
    api.opened = True
    api.is_client = is_client
    api.binary = path
    api.imagebase = int(api.ida_nalt.get_imagebase())
    api.hexrays_available = bool(api.ida_hexrays.init_hexrays_plugin())
    if auto_analysis:
        api.ida_auto.auto_wait()
    return api, is_client


def close_database(api: SimpleNamespace, save: bool) -> None:
    if getattr(api, "opened", False):
        api.idapro.close_database(save)
        api.opened = False


def _record(value: Any) -> dict[str, Any]:
    return json.loads(value) if isinstance(value, str) else dict(value or {})


def _addresses(value: Any) -> list[str]:
    items = json.loads(value) if isinstance(value, str) else (value or [])
    result = set()
    for item in items:
        raw = str(item).lower().removeprefix("0x").removeprefix("fun_")
        try:
            result.add(f"{int(raw, 16):08x}")
        except ValueError:
            continue
    return sorted(result)


def _string_list(value: Any) -> list[str]:
    items = json.loads(value) if isinstance(value, str) else (value or [])
    return [str(item) for item in items]


def select_labels(
    records: list[dict[str, Any]], confirmed_count: int, likely_count: int, seed: int
) -> list[dict[str, Any]]:
    if confirmed_count < 0 or likely_count < 0:
        raise ValueError("sample counts must be non-negative")
    confirmed, likely = [], []
    for row in records:
        confidence = _record(row.get("record")).get("confidence")
        if confidence == "confirmed":
            confirmed.append(row)
        elif confidence == "likely":
            likely.append(row)
    confirmed.sort(key=lambda row: int(row["addr"], 16))
    confirmed_sample = confirmed[:confirmed_count]
    likely_sample = random.Random(seed).sample(likely, min(likely_count, len(likely)))
    return confirmed_sample + likely_sample


def _string_text(api: SimpleNamespace, address: int) -> str | None:
    raw = api.ida_bytes.get_strlit_contents(address, -1, api.ida_nalt.STRTYPE_C)
    if isinstance(raw, bytes):
        return raw.decode("utf-8", errors="replace")
    if isinstance(raw, str):
        return raw
    return None


def _referenced_strings(api: SimpleNamespace, function_start: int) -> list[str]:
    strings = set()
    for instruction in api.idautils.FuncItems(function_start):
        for target in api.idautils.DataRefsFrom(instruction):
            value = _string_text(api, target)
            if value:
                strings.add(value)
    return sorted(strings)


def _is_call(api: SimpleNamespace, address: int) -> bool:
    instruction = api.ida_ua.insn_t()
    return bool(api.ida_ua.decode_insn(instruction, address)) and bool(api.ida_idp.is_call_insn(instruction))


def _function_start(api: SimpleNamespace, address: int) -> int | None:
    function = api.ida_funcs.get_func(address)
    return int(function.start_ea) if function else None


def _ida_callees(api: SimpleNamespace, function_start: int) -> list[dict[str, str]]:
    result = {}
    for instruction in api.idautils.FuncItems(function_start):
        if not _is_call(api, instruction):
            continue
        for target in api.idautils.CodeRefsFrom(instruction, False):
            start = _function_start(api, target)
            if start is not None:
                addr = f"{start:08x}"
                result[addr] = api.ida_name.get_name(start) or ""
    return [{"address": addr, "name": name} for addr, name in sorted(result.items())]


def _ida_callers(api: SimpleNamespace, function_start: int) -> list[dict[str, str]]:
    result = {}
    for source in api.idautils.CodeRefsTo(function_start, False):
        if not _is_call(api, source):
            continue
        start = _function_start(api, source)
        if start is not None:
            addr = f"{start:08x}"
            result[addr] = api.ida_name.get_name(start) or ""
    return [{"address": addr, "name": name} for addr, name in sorted(result.items())]


def _overlap(map_edges: list[str], ida_edges: list[str]) -> dict[str, Any]:
    map_set, ida_set = set(map_edges), set(ida_edges)
    common = map_set & ida_set
    union = map_set | ida_set
    return {
        "map": sorted(map_set),
        "ida": sorted(ida_set),
        "shared": sorted(common),
        "map_only": sorted(map_set - ida_set),
        "ida_only": sorted(ida_set - map_set),
        "map_edge_coverage": len(common) / len(map_set) if map_set else None,
        "jaccard": len(common) / len(union) if union else None,
    }


def _map_target_records(nmq: Any, addresses: list[str]) -> dict[str, dict[str, Any]]:
    result = {}
    for address in addresses:
        row = nmq.db.execute(
            "SELECT status,record FROM label WHERE addr=?", (address,)
        ).fetchone()
        if row:
            result[address] = {"status": row[0], "record": json.loads(row[1])}
    return result


def _required_strings(record: dict[str, Any]) -> list[str]:
    return sorted({str(item["v"]) for item in record.get("evidence", [])
                   if isinstance(item, dict) and item.get("t") == "string" and item.get("v")})


def _string_agreement(required: list[str], actual: list[str]) -> dict[str, Any]:
    def normalize(value: str) -> str:
        return " ".join(value.split()).casefold()

    matches = {}
    for value in required:
        key = normalize(value)
        matches[value] = [item for item in actual if key and key in normalize(item)]
    return {
        "required": required,
        "present": all(matches.values()) if required else None,
        "matches": matches,
        "missing": [value for value, found in matches.items() if not found],
    }


def _label_is_packet_writer(record: dict[str, Any]) -> bool:
    claims = " ".join(str(record.get(key) or "") for key in ("name", "does", "purpose", "subsystem"))
    claims = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", claims).casefold()
    writes = re.search(r"write|writer|serializ|marshal|encode", claims)
    packet = re.search(r"packet|payload|request|response|net\.packet", claims)
    return bool(writes and packet)


def inspect_label(
    api: SimpleNamespace, label_row: dict[str, Any], map_function: dict[str, Any], nmq: Any = None
) -> dict[str, Any]:
    address = f"{int(str(label_row['addr']).lower().removeprefix('0x'), 16):08x}"
    start = int(address, 16)
    record = _record(label_row.get("record"))
    function = api.ida_funcs.get_func(start)
    exact = bool(function and int(function.start_ea) == start)

    source = {
        "address": address,
        "status": label_row.get("status"),
        "confidence": record.get("confidence"),
        "name": record.get("name"),
        "kind": record.get("kind"),
        "subsystem": record.get("subsystem"),
        "does": record.get("does"),
        "purpose": record.get("purpose"),
        "evidence": record.get("evidence", []),
        "wire": record.get("wire", []),
        "function_size": map_function.get("bytes"),
        "strings": _string_list(map_function.get("strings", "[]")),
        "callers": _addresses(map_function.get("callers", "[]")),
        "callees": _addresses(map_function.get("callees", "[]")),
    }

    ida_callers = _ida_callers(api, start) if exact else []
    ida_callees = _ida_callees(api, start) if exact else []
    actual_strings = _referenced_strings(api, start) if exact else []
    required_strings = _required_strings(record)
    string_agreement = _string_agreement(required_strings, actual_strings)
    caller_edges = _overlap(source["callers"], [item["address"] for item in ida_callers])
    callee_edges = _overlap(source["callees"], [item["address"] for item in ida_callees])

    pseudocode = None
    pseudocode_error = None
    if exact and api.hexrays_available:
        try:
            cfunc = api.ida_hexrays.decompile(start)
            if cfunc:
                pseudocode = "\n".join(
                    api.ida_lines.tag_remove(line.line) for line in cfunc.get_pseudocode()
                )
        except Exception as error:
            pseudocode_error = str(error)[:300]
    pseudocode_limit = 20000
    full_pseudocode = pseudocode
    if pseudocode is not None and len(pseudocode) > pseudocode_limit:
        pseudocode = pseudocode[:pseudocode_limit]

    ida = {
        "address": address,
        "imagebase": f"{api.imagebase:08x}",
        "function_exists": function is not None,
        "function_start": f"{int(function.start_ea):08x}" if function else None,
        "function_end": f"{int(function.end_ea):08x}" if function else None,
        "function_size": int(function.end_ea - function.start_ea) if function else None,
        "function_name": api.ida_name.get_name(function.start_ea) if function else None,
        "boundary_match": exact,
        "hexrays_available": api.hexrays_available,
        "pseudocode": pseudocode,
        "pseudocode_chars": len(full_pseudocode) if full_pseudocode is not None else 0,
        "pseudocode_truncated": bool(full_pseudocode is not None and len(full_pseudocode) > pseudocode_limit),
        "pseudocode_error": pseudocode_error,
        "callers": ida_callers,
        "callees": ida_callees,
        "referenced_strings": actual_strings,
    }

    flags = []
    if not exact:
        flags.append({"code": "function_boundary_mismatch", "fact": "IDA has no function starting at the label address"})
    if source["callers"] and not caller_edges["shared"]:
        flags.append({"code": "caller_graph_disjoint", "fact": "map reports callers but IDA shares no caller address"})
    if source["callees"] and not callee_edges["shared"]:
        flags.append({"code": "callee_graph_disjoint", "fact": "map reports callees but IDA shares no callee address"})
    if string_agreement["missing"]:
        flags.append({"code": "label_string_evidence_missing", "fact": "IDA found no matching function reference for one or more label string claims"})

    serializer_targets = []
    if nmq is not None and _label_is_packet_writer(record):
        target_records = _map_target_records(nmq, source["callees"])
        serializer_targets = [
            address for address, target in target_records.items()
            if target["status"] == "accepted" and SERIALIZER.search(
                " ".join(str(target["record"].get(key) or "") for key in ("name", "does", "subsystem"))
            )
        ]
        ida_callee_set = {item["address"] for item in ida_callees}
        ida_named_serializer = any(SERIALIZER.search(item["name"]) for item in ida_callees)
        observed_serializer_targets = sorted(set(serializer_targets) & ida_callee_set)
        if serializer_targets and not observed_serializer_targets and not ida_named_serializer:
            flags.append({
                "code": "packet_writer_serializer_edge_missing",
                "fact": "map-labeled serializer callees have no matching IDA call edge",
                "map_serializer_targets": sorted(serializer_targets),
            })
        elif not serializer_targets and not ida_named_serializer and not ida_callees:
            flags.append({
                "code": "packet_writer_without_outbound_calls",
                "fact": "IDA found no resolved outbound call target for packet-writer label",
            })
        elif not serializer_targets and not ida_named_serializer:
            flags.append({
                "code": "packet_writer_without_labeled_serializer_callee",
                "fact": "no map or IDA callee name identifies a serializer target",
            })

    return {
        "source": source,
        "ida": ida,
        "agreement": {
            "boundary_match": exact,
            "size_delta": (ida["function_size"] - source["function_size"])
            if ida["function_size"] is not None and isinstance(source["function_size"], int) else None,
            "callers": caller_edges,
            "callees": callee_edges,
            "strings": string_agreement,
            "serializer_targets_in_map": sorted(serializer_targets),
        },
        "flags": flags,
    }


def _load_nmq() -> Any:
    if not MAP_SCRIPT.is_file():
        raise RuntimeError(f"native-map reader is missing: {MAP_SCRIPT}")
    active = MAP_ROOT / "ACTIVE"
    raw_db = os.environ.get("NM_DB") or (
        active.read_text(encoding="utf-8").strip()
        if active.is_file() else str(MAP_ROOT / "run" / "map.sqlite")
    )
    expected_db = Path(raw_db).expanduser().resolve(strict=True)
    map_root = MAP_ROOT.resolve()
    if not _within(expected_db, map_root):
        raise RuntimeError(f"map database must be under {map_root}: {expected_db}")
    module_name = "w97_native_map_nmq"
    module = sys.modules.get(module_name)
    if module is None:
        spec = importlib.util.spec_from_file_location(module_name, MAP_SCRIPT)
        if spec is None or spec.loader is None:
            raise RuntimeError(f"could not load native-map reader: {MAP_SCRIPT}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[module_name] = module
        spec.loader.exec_module(module)
    db_path = Path(module.DB).resolve(strict=True)
    if db_path != expected_db:
        raise RuntimeError(f"nmq database path changed during import: {db_path}")
    return module


def load_accepted_labels(nmq: Any) -> list[dict[str, Any]]:
    sql = """SELECT l.addr,l.status,l.record,f.bytes,f.callers,f.callees,f.strings
             FROM label AS l JOIN fn AS f USING(addr)
             WHERE l.status='accepted' AND json_extract(l.record,'$.confidence') IN (?,?)
             ORDER BY l.addr"""
    result = []
    for addr, status, record, size, callers, callees, strings in nmq.db.execute(sql, CONFIDENCE):
        result.append({
            "addr": addr,
            "status": status,
            "record": json.loads(record),
            "function": {"bytes": size, "callers": callers, "callees": callees, "strings": strings},
        })
    return result


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _rate(matched: int, total: int) -> float | None:
    return matched / total if total else None


def _edge_totals(rows: list[dict[str, Any]], direction: str) -> dict[str, Any]:
    total_map = total_ida = matched = eligible = 0
    for row in rows:
        edges = row["agreement"][direction]
        total_map += len(edges["map"])
        total_ida += len(edges["ida"])
        matched += len(edges["shared"])
        eligible += bool(edges["map"])
    return {
        "functions_with_map_edges": eligible,
        "map_edges": total_map,
        "ida_edges": total_ida,
        "shared_edges": matched,
        "map_edge_coverage": _rate(matched, total_map),
        "jaccard": _rate(matched, total_map + total_ida - matched),
    }


def summarize(
    rows: list[dict[str, Any]], target: Path, target_sha256: str, map_database: str | None,
    selection: dict[str, Any], ida_version: Any,
) -> dict[str, Any]:
    processed = len(rows)
    boundary_matches = sum(row["agreement"]["boundary_match"] for row in rows)
    decompiled = sum(bool(row["ida"]["pseudocode"]) for row in rows)
    required = [
        value for row in rows for value in row["agreement"]["strings"]["required"]
    ]
    present = [
        value for row in rows for value in row["agreement"]["strings"]["required"]
        if value not in row["agreement"]["strings"]["missing"]
    ]
    flags = {}
    for row in rows:
        for flag in row["flags"]:
            flags[flag["code"]] = flags.get(flag["code"], 0) + 1
    return {
        "schema_version": 1,
        "analysis": "deterministic IDA facts; no LLM verdicts",
        "call_graph_method": "resolved IDA code xrefs from/to call instructions; indirect targets are omitted",
        "string_method": "IDA data-reference targets that IDA recognizes as C strings",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "target": str(target),
        "target_sha256": target_sha256,
        "map_database": map_database,
        "ida_library_version": ida_version,
        "selection": selection,
        "results": {
            "processed": processed,
            "functions_found": sum(row["ida"]["function_exists"] for row in rows),
            "exact_boundaries": boundary_matches,
            "boundary_match_rate": _rate(boundary_matches, processed),
            "decompiled": decompiled,
            "decompile_rate": _rate(decompiled, processed),
            "functions_with_pseudocode_truncated": sum(row["ida"]["pseudocode_truncated"] for row in rows),
        },
        "agreement_rates": {
            "callers": _edge_totals(rows, "callers"),
            "callees": _edge_totals(rows, "callees"),
            "label_string_evidence": {
                "required_strings": len(required),
                "present_strings": len(present),
                "rate": _rate(len(present), len(required)),
            },
        },
        "contradiction_flags": flags,
    }


def write_report(run_name: str, rows: list[dict[str, Any]], summary: dict[str, Any]) -> tuple[Path, Path]:
    if not RUN_NAME.fullmatch(run_name):
        raise ValueError("run name must use 1-80 letters, digits, dots, underscores, or hyphens")
    lab_root = LAB_ROOT.resolve()
    map_root = MAP_ROOT.resolve()
    output_root = OUTPUT_DIR.resolve()
    if not _within(map_root, lab_root) or not _within(output_root, map_root):
        raise ValueError(f"output directory must stay under {lab_root}: {output_root}")
    OUTPUT_DIR.mkdir(parents=True, exist_ok=True)
    output_root = OUTPUT_DIR.resolve()
    if not _within(output_root, map_root):
        raise ValueError(f"output directory escaped {map_root}: {output_root}")
    jsonl_path = output_root / f"{run_name}.jsonl"
    summary_path = output_root / f"{run_name}.summary.json"
    if not _within(jsonl_path.resolve(strict=False), output_root) or not _within(summary_path.resolve(strict=False), output_root):
        raise ValueError("output path escapes the IDA-check directory")
    if jsonl_path.exists() or summary_path.exists():
        raise FileExistsError(f"refusing to overwrite an existing run named {run_name}")
    with jsonl_path.open("x", encoding="utf-8", newline="\n") as output:
        for row in rows:
            output.write(json.dumps(row, ensure_ascii=False, sort_keys=True) + "\n")
    with summary_path.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(summary, output, ensure_ascii=False, indent=2, sort_keys=True)
        output.write("\n")
    return jsonl_path, summary_path


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=SAMPLE_ROOT / "where.exe",
                        help="FreeRealms.exe or a small PE under lab/build/w97")
    parser.add_argument("--confirmed-count", type=int, default=200)
    parser.add_argument("--likely-count", type=int, default=100)
    parser.add_argument("--seed", type=int, default=97)
    parser.add_argument("--run-name", help="safe output stem; defaults to an UTC timestamp")
    args = parser.parse_args(argv)

    try:
        api, is_client = open_database(args.binary)
        target = api.binary
        nmq = _load_nmq()
        all_labels = load_accepted_labels(nmq)
        chosen = select_labels(all_labels, args.confirmed_count, args.likely_count, args.seed)
        rows = [inspect_label(api, label, label["function"], nmq) for label in chosen]
        selected_confirmed = sum(row["record"]["confidence"] == "confirmed" for row in chosen)
        selected_likely = len(chosen) - selected_confirmed
        summary = summarize(
            rows,
            target=target,
            target_sha256=sha256_file(target),
            map_database=str(Path(nmq.DB).resolve()),
            selection={
                "accepted_confirmed_available": sum(r["record"].get("confidence") == "confirmed" for r in all_labels),
                "accepted_likely_available": sum(r["record"].get("confidence") == "likely" for r in all_labels),
                "confirmed_selected": selected_confirmed,
                "likely_selected": selected_likely,
                "seed": args.seed,
                "order": "confirmed by address, then seeded random likely sample",
            },
            ida_version=api.version,
        )
        run_name = args.run_name or "ida-check-" + datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        jsonl_path, summary_path = write_report(run_name, rows, summary)
        print(json.dumps({"jsonl": str(jsonl_path), "summary": str(summary_path), "results": summary["results"]}))
        return 0
    except (ImportError, OSError, RuntimeError, ValueError, sqlite3.Error) as error:
        parser.error(str(error))
    finally:
        if "api" in locals():
            close_database(api, save=not is_client)
if __name__ == "__main__":
    raise SystemExit(main())
