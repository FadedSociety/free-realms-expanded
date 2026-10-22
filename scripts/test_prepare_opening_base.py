"""Regression check for creator clothing classes (T620): python -I scripts/test_prepare_opening_base.py.

Runs the generator on a temporary copy of the private lab inputs; skips when the lab is absent.
"""
import json
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

SCRIPTS = Path(__file__).resolve().parent
LAB = SCRIPTS.parents[1] / "lab"
CREATOR = "working/OSFR.Client/OSFR Client/Resources/CharacterCreate"
INPUTS = [CREATOR, "references/rust-gameplay", "references/rust-gateway-reference",
          "references/Sanctuary/src/Resources/ItemClasses.txt",
          "references/FreeRealms-Legacy/src/Resources/Profiles.json",
          "references/FreeRealms-Legacy/src/Resources/ClientItemDefinitions.json",
          "working/OSFR.Server/OSFR Server/Customize/PacketSendSelfToClient.json"]


def run(root, edit=lambda text: text):
    """Copy the inputs and both scripts under root, edit the creator item table, run the generator."""
    for name in INPUTS:
        (root / "lab" / name).parent.mkdir(parents=True, exist_ok=True)
        (shutil.copytree if (LAB / name).is_dir() else shutil.copy2)(LAB / name, root / "lab" / name)
    (root / "project/scripts").mkdir(parents=True)
    for name in ("prepare-opening-base.py", "prepare-gameplay-inventory.py"):
        shutil.copy2(SCRIPTS / name, root / "project/scripts" / name)
    table = root / "lab" / CREATOR / "ItemDefinitions.txt"
    table.write_text(edit(table.read_text()))
    return subprocess.run([sys.executable, "-I", str(root / "project/scripts/prepare-opening-base.py")],
                          capture_output=True, text=True)


if not (LAB / CREATOR).is_dir():
    print("skip: private lab inputs absent")
    sys.exit(0)
with tempfile.TemporaryDirectory() as temp:
    done = run(Path(temp) / "fixed")
    assert done.returncode == 0, done.stderr
    inventory = json.loads((Path(temp) / "fixed/lab/references/rust-opening/inventory.json").read_text())
    # Human female sets 30-35 ship ITEM_CLASS 0; they get class 1 (R) and pass the profile 1 audit.
    female_sets = list(range(77679, 77697))
    assert {d["class_id"] for d in inventory["definitions"] if d["id"] in female_sets} == {1}
    assert inventory["provenance"]["creator_definitions"]["reconstructed_class_1"] == female_sets
    # Canary: an outfit whose class profile 1 lacks (201 -> class 5) stops the generator.
    refused = run(Path(temp) / "refused", lambda text: text.replace(
        "\n201^1^8506^1114^3646^0^0^0^270^1^", "\n201^1^8506^1114^3646^0^0^0^270^5^", 1))
    assert refused.returncode and "creator clothing [201] cannot be equipped on profile 1" in refused.stderr, \
        refused.stderr
print("ok")
