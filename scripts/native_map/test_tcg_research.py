"""Bounded regression checks for the T031 card-game research claims.

Run from the project root:

    python -I scripts/native_map/test_tcg_research.py

The checks compare the note with preserved raw evidence. They do not execute
the game, contact a service, or mutate the lab.
"""

from pathlib import Path
import json
import re


ROOT = Path(__file__).resolve().parents[2]
LAB = ROOT.parent / "lab"
NOTE = ROOT / "notes/Research/Card Game.md"
ANALYSIS = LAB / "content/tcg/analysis"
MINIGAME_GUIDE = ROOT / "notes/Build Guides/Gameplay/Minigame Framework.md"
TCG_GUIDE = ROOT / "notes/Build Guides/Gameplay/Trading Card Game.md"


def _read(path):
    return path.read_text(encoding="utf-8")


def _between(text, start, end):
    _head, sep, tail = text.partition(start)
    assert sep, f"missing section start: {start}"
    body, sep, _ = tail.partition(end)
    assert sep, f"missing section end: {end}"
    return body


def _table_direction(section, command_id):
    pattern = rf"^\|\s*{command_id}\s*\([^)]*\)\s*\|[^|]+\|\s*([^|]+?)\s*\|"
    match = re.search(pattern, section, re.MULTILINE)
    assert match, f"missing command row {command_id}"
    return match.group(1).strip()


def check_exe_only_network_commands_are_not_attributed_to_dll():
    table = json.loads(_read(ANALYSIS / "message-table.json"))
    curated_ids = {row["id"] for row in table["messages"]}
    assert len(curated_ids) == 60
    assert not ({546, 575} & curated_ids)

    registrations = {}
    for row in _read(ANALYSIS / "dll-registration.tsv").splitlines()[1:]:
        command_id, *fields = row.split("\t")
        registrations[int(command_id, 16)] = fields
    assert len(registrations) == 499
    assert curated_ids <= registrations.keys()
    assert not ({546, 575} & registrations.keys())

    note = _read(NOTE)
    dll_section = _between(
        note, "### Separate TCG card service", "### Framing and routing"
    )
    dll_table = _between(
        dll_section, "| ID | Class / operation |", "The manual challenge exchange"
    )
    assert not re.search(r"^\|\s*(?:546|575)\b", dll_table, re.MULTILINE)
    exe_only = _between(
        dll_section, "Two network IDs in the Build Guide", "Unresolved operations"
    )
    assert re.search(r"^\|\s*546\b", exe_only, re.MULTILINE)
    assert re.search(r"^\|\s*575\b", exe_only, re.MULTILINE)
    assert "no DLL factory registration" in exe_only
    guide = _read(TCG_GUIDE)
    assert "`NetworkCommand_UpdateDataVersion` (546" in guide
    assert "`NetworkCommand_RequestData` (575)" in guide


def check_manual_challenge_direction_matches_execute_path():
    note = _read(NOTE)
    dll_section = _between(
        note, "### Separate TCG card service", "### Framing and routing"
    )
    assert _table_direction(dll_section, 78) == "C>S"
    assert _table_direction(dll_section, 79) == "S>C"
    assert _table_direction(dll_section, 80) == "C>S"

    registrations = {}
    for row in _read(ANALYSIS / "dll-registration.tsv").splitlines()[1:]:
        fields = row.split("\t")
        names = (
            "initializer",
            "creator",
            "class_constructor",
            "class_name",
            "vtable",
            "reader",
            "writer",
            "dispatch",
            "execute",
            "routing",
        )
        registrations[fields[0]] = dict(zip(names, fields[1:]))
    assert registrations["0x4f"]["execute"] == "1017a080"

    decompiled = _read(ANALYSIS / "w61b-decompiled-excerpts.c")
    body = _between(
        decompiled,
        "/* 1017a080 FUN_1017a080 */",
        "/* 1017b2d0 FUN_1017b2d0 */",
    )
    assert registrations["0x50"]["class_constructor"] == "101790a0"
    assert "FUN_101790a0();" in body
    assert "FUN_10068af0(auStack_170);" in body
    assert "78 C>S, 79 S>C, 80 C>S" in dll_section


def check_start_request_is_header_only():
    note = _read(NOTE)
    family_39 = _between(note, "### Main game, family 39", "### Separate TCG card service")
    start_row = next(
        line for line in family_39.splitlines() if line.startswith("| `39/5`")
    )
    assert "header only (15 bytes)" in start_row
    assert "[instance, -1, -1]" in start_row
    assert "header plus" not in start_row

    guide = _read(MINIGAME_GUIDE)
    assert "u16 39, u8 sub-opcode, i32 id1, i32 id2, i32 id3" in guide
    assert "A 15-byte client request" in guide
    assert "009be7d0" in guide
    assert "009bdc60" in guide


def main():
    check_exe_only_network_commands_are_not_attributed_to_dll()
    check_manual_challenge_direction_matches_execute_path()
    check_start_request_is_header_only()
    print(
        "tcg_research=ok exe_only_546_575=not_dll "
        "challenge_direction=78_c2s_79_s2c_80_c2s "
        "start_request=header_only_15_bytes"
    )


if __name__ == "__main__":
    main()
