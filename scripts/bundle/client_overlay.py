"""Build the client overlay a realm serves as its launcher patch manifest (W121).

It holds the community character-creator tables and the custom/ dressing-room scene,
at the paths the client opens. The native harness installs the same files through
install() (scripts/run-local-baseline.py). Provenance and the pending distribution
decision: the launcher repo's README (github.com/FadedSociety/free-realms-launcher).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle_common import LAB, make_manifest, require_regular_source, sha256_file  # noqa: E402

CACHE = LAB / "references" / "asset-cache"

# (cached file, client path, SHA-256). The three table pins are the harness's.
# Client paths are read from FreeRealms.exe:
# - StartingSets: "./Resources/CharacterCreate/" (0x017e802c, set by 0x0071efb0) +
#   "CharacterCreateItemStartingSets.txt" (0x017e8dc8, read by 0x00723e90).
# - Customization tables: "Resources/" (0x017ed7d8, passed by 0x008ebf70) +
#   "CharacterCreateCustomization" (0x0180b638, 0x0097f570) + the category suffix from
#   CharacterCreateCustomizationCategories.txt + ".txt" (0x00982f10). Not CharacterCreate/.
# - custom/ is not a hard-coded name: the client's loose-file index scans every
#   subdirectory of its root (notes/Research/Native Character Selection Labels.md).
OVERLAY = (
    (
        "CharacterCreateItemStartingSets.txt",
        "Resources/CharacterCreate/CharacterCreateItemStartingSets.txt",
        "89cc60e8979179e9af77aada3f3e8b64042757b249ee92745c361ce40b153c24",
    ),
    (
        "public-CharacterCreateCustomizationSkinTone.txt",
        "Resources/CharacterCreateCustomizationSkinTone.txt",
        "485d20174e96c53021d9644d3ba943f0a016fb0413d1be8fa8f728bea528dd9d",
    ),
    (
        "public-CharacterCreateCustomizationFaceType.txt",
        "Resources/CharacterCreateCustomizationFaceType.txt",
        "9d4d7acd63c7926ed3d4c03f99116e929061eef4ae25e1a2b6208c738f2c8525",
    ),
    *((name, "custom/" + name, pin) for name, pin in (
        ("sg_dressing_room.gzne", "bf49bec0f03aefc6d1bf14fc41049023b31f408efc0345da3f67595c3133a3c6"),
        ("sg_dressing_room_0_0.gck2", "19da101cd06d820983156c5c257550d3105f7e06784d30cf512e7798d4b62626"),
        ("sg_dressing_room_0_0.gcnk", "b914368be071c1ad1b50af6558fd6b05e09d01c08d76b278e3e14fc59e6c0aee"),
        ("sg_dressing_room_0_4.gck2", "5a84441f17e19390ab727fe3291080a7b428fc22b742520fc98615751e65fbba"),
        ("sg_dressing_room_0_4.gcnk", "81b8b0ec435fb90b0b24e4b90f54c2b5f48996ab07c5b784143a1de283a6aa83"),
        ("sg_dressing_room_0_8.gck2", "366c78da9bd4c2d663b09fce714398ff499497036b2c0399ef9e4f54213d2ac2"),
        ("sg_dressing_room_0_8.gcnk", "7a1a36c1ae9ca701eb67a469ad40f3e40f1bdc1af4d29141f80370f740a70537"),
        ("sg_dressing_room_4_0.gck2", "0da5ddf394aaec8be9c7f5a8d94b772c446aa001e32309fc40a8356525626d4c"),
        ("sg_dressing_room_4_0.gcnk", "4aa545ca68323f23a7fe9576720021edd30b9e9f83c5424fa07992c19d635291"),
        ("sg_dressing_room_4_4.gck2", "9860b4e2515c515053ccaaeefe32df308ceb40f1b74e94038c7d377f3c1dc342"),
        ("sg_dressing_room_4_4.gcnk", "c58f065fe9cb7a2664730cbf4253a5478c27f0087422b2d7aa182ad89b401530"),
        ("sg_dressing_room_4_8.gck2", "5a66e6629dda786f80434bac864580e5926fc5ddcb8f31fb3a4d090359340977"),
        ("sg_dressing_room_4_8.gcnk", "35647418448753fab50a365ec7811bdb2e978b66bc79f41eb0ce64ccba371b5a"),
        ("sg_dressing_room_8_0.gck2", "785717a9d939e8200f014d03cd05b785ca4cc078fe7812228a2b05433f2cd2ff"),
        ("sg_dressing_room_8_0.gcnk", "17eac9a5f01b9f8fb4b0adbe298a8217d32e8ad28975793ae4b957f99f973181"),
        ("sg_dressing_room_8_4.gck2", "9bdd369b4e2e9dd14a2c4bbd843f2b78f21f79c443c82ee18f1a3534ab9dd5ef"),
        ("sg_dressing_room_8_4.gcnk", "8769d331e649240c5ceed613c5430892a833bad1082dfcddc83ee2c3e129eda4"),
        ("sg_dressing_room_8_8.gck2", "efc6f6167180cef6976e64e0024932157b8e175fb8b2bc5598ade139702893e7"),
        ("sg_dressing_room_8_8.gcnk", "adb407d869408fc37bef6a9283d27c331ab92c9e3114f9146b04fbd67f99b06e"),
    )),
)


def install(root: Path, cache: Path = CACHE) -> None:
    """Pin-check every source first, so a refusal writes nothing; then copy each under root."""
    checked = []
    for name, path, pin in OVERLAY:
        data = require_regular_source(cache / name, cache).read_bytes()
        if hashlib.sha256(data).hexdigest() != pin:
            raise ValueError(f"client overlay source does not match its pin: {name}")
        checked.append((root / path, data))
    for target, data in checked:
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)


def build(out: Path, cache: Path = CACHE) -> dict:
    """A new folder holding the overlay and its W121 manifest.json."""
    if out.exists():
        raise FileExistsError(f"overlay folder already exists: {out}")
    install(out, cache)
    manifest = make_manifest(out)
    (out / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    return manifest


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True, help="new folder to serve at the patch URL")
    out = parser.parse_args().out
    build(out)
    print(f"Client overlay: {out} ({len(OVERLAY)} files)")
    print(f"Manifest SHA-256 pin: {sha256_file(out / 'manifest.json')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
