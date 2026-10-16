"""Prepare private reconstructed objective text with pinned FreeRealmsLocaleTools.

Only a NEW package/content clone is written. No game-format parser is implemented
here: the unchanged MIT community reader/writer handles the paired locale files.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
from pathlib import Path

from journal_text_overlay import STOCK


TOOL_REVISION = "ba2ede225ff33edab8e8e0d20c45d4c5156309e5"
CSHARP = r'''using FreeRealmsLocaleTools.LocaleParser;
using System.Security.Cryptography;
using System.Text.Json;

var stock = new LocaleFileInfo(args[0], args[1], ParseOptions.Strict);
var stockIds = stock.IdToEntry.ToDictionary(x => x.Key, x => x.Value);
var originalEntries = stock.Entries.ToHashSet();
if (originalEntries.Count != stock.Entries.Length)
    throw new InvalidDataException("Duplicate original locale entries require explicit review");
var texts = JsonSerializer.Deserialize<string[]>(File.ReadAllText(args[2]))!;
if (texts.Length != texts.Distinct().Count())
    throw new InvalidDataException("Duplicate proposed strings");
var added = texts.Select(text => new {id = stock.AddEntry(text), text}).ToArray();
if (added.Any(x => stockIds.ContainsKey(x.id)))
    throw new InvalidDataException("New ID collides with stock ID");
var destination = args[3];
Directory.CreateDirectory(destination);
stock.WriteEntries(Path.Combine(destination, "en_us_data.dat"), Path.Combine(destination, "en_us_data.dir"));
var roundtrip = new LocaleFileInfo(Path.Combine(destination, "en_us_data.dat"),
    Path.Combine(destination, "en_us_data.dir"), ParseOptions.Strict);
if (roundtrip.Entries.Length != originalEntries.Count + added.Length
    || !originalEntries.IsSubsetOf(roundtrip.Entries.ToHashSet())
    || stockIds.Any(x => !roundtrip.IdToEntry.TryGetValue(x.Key, out var value) || value != x.Value)
    || added.Any(x => !roundtrip.IdToEntry.TryGetValue(x.id, out var value) || value.Text != x.text)
    || roundtrip.Metadata.Locale != stock.Metadata.Locale
    || roundtrip.Metadata.MD5Checksum != Convert.ToHexString(MD5.HashData(File.ReadAllBytes(Path.Combine(destination, "en_us_data.dat"))))
    || roundtrip.Metadata.Count != roundtrip.Entries.Length)
    throw new InvalidDataException("Community writer roundtrip/preservation failed");
File.WriteAllText(args[4], JsonSerializer.Serialize(new {original_entry_count=originalEntries.Count,
    original_id_count=stockIds.Count, output_entry_count=roundtrip.Entries.Length,
    all_original_entries_preserved=true, all_original_ids_preserved=true,
    new_ids_noncolliding=true, strict_roundtrip=true, md5_metadata_matches=true, added},
    new JsonSerializerOptions {WriteIndented=true}));
Console.WriteLine($"Community locale roundtrip: {originalEntries.Count} stock entries preserved; {added.Length} new strings");
'''


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def json_write(path, value):
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def presentation(goal):
    trigger = goal["trigger"]
    kind = trigger["kind"]
    if goal["name_id"] == 0:
        if kind == "class_selected":
            job = {1: "Adventurer", 11: "Medic", 14: "Miner", 16: "Blacksmith", 35: "Archer", 43: "Brawler", 45: "Chef"}[trigger["profile_id"]]
            return (f"Switch to {job}", f"Choose {job} in the Jobs menu.")
        if kind == "cosmetic_equipped":
            item = {13597: "Student Chef Shirt", 14794: "Student Chef Hat"}[trigger["definition_id"]]
            return (f"Equip the {item}", f"Wear the {item} in your Chef equipment screen.")
        raise ValueError(f"Unreviewed zero-ID goal: {goal['id']}")
    if kind == "gather" and trigger["resource_id"] in (1001, 1002):
        ingredient = {1001: "Rainleaf", 1002: "Gummyfruit"}[trigger["resource_id"]]
        return (f"Harvest {ingredient}", f"Gather {ingredient} bundles at the farm while playing as Chef.")
    if kind == "interact":
        fire = {5715144128442925065: "first", 5715144128442925066: "second", 5715144128442925067: "third"}
        artifact = {5715144128442925077: "first", 5715144128442925078: "second", 5715144128442925079: "third"}
        guid = trigger["actor_guid"]
        if guid in fire:
            return (f"Put out the {fire[guid]} farm fire", "Use the filled bucket on this fire. Follow the objectives in order.")
        if guid in artifact:
            return (f"Destroy the {artifact[guid]} Gloam artifact", "Approach this Gloam artifact and interact with it. Follow the objectives in order.")
    return None


def main():
    lab = Path(__file__).resolve().parents[2] / "lab"
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-content", type=Path, default=lab / "references/rust-opening-cooking-original-assets")
    parser.add_argument("--output", type=Path, required=True, help="NEW private package directory under lab")
    args = parser.parse_args()
    source, output = args.source_content.resolve(), args.output.resolve()
    if not output.is_relative_to(lab.resolve()) or output.exists() or source == output:
        raise ValueError("Output must be a NEW directory below lab")
    if (source / "journal-text.json").exists():
        raise ValueError("Source already has a presentation mapping")
    original = lab / "working/OSFR.Client/OSFR Client/locale"
    for name, expected in STOCK.items():
        if sha(original / Path(name).name) != expected:
            raise ValueError("Preserved original locale pair changed")
    community = lab / "references/FreeRealmsLocaleTools"
    provenance = json.loads((community / "SOURCE.json").read_text())
    if provenance["revision"] != TOOL_REVISION:
        raise ValueError("Locale community source revision changed")
    opening = json.loads((source / "opening.json").read_text(encoding="utf-8"))
    stock_locale = {r["id"]: r["text"] for r in json.loads((lab / "reports/locale-id-to-text.json").read_text())}
    proposed, texts = [], []
    for stage in opening["stages"]:
        for goal in stage["objectives"]:
            label = presentation(goal)
            if label is None:
                if not stock_locale.get(goal["name_id"]) or not stock_locale.get(goal["description_id"]):
                    raise ValueError("Unmapped objective lacks original text")
                continue
            name, description = label
            proposed.append({"quest_id": stage["id"], "objective_id": goal["id"],
                             "source_name_id": goal["name_id"], "source_description_id": goal["description_id"],
                             "name": name, "description": description})
            for text in label:
                if text not in texts:
                    texts.append(text)
    output.mkdir()
    tool = lab / "tools/journal-locale"
    tool.mkdir(exist_ok=True)
    (tool / "Program.cs").write_text(CSHARP, encoding="utf-8")
    (tool / "JournalLocale.csproj").write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType>'
        '<TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings>'
        '<Nullable>enable</Nullable></PropertyGroup><ItemGroup><Compile Include="'
        + str(community / "src/FreeRealmsLocaleTools/**/*.cs")
        + '" /></ItemGroup></Project>', encoding="utf-8")
    json_write(output / "proposed-strings.json", texts)
    subprocess.run([str(lab / "tools/dotnet/dotnet.exe"), "run", "--project", str(tool / "JournalLocale.csproj"),
                    "-c", "Release", "--", str(original / "en_us_data.dat"), str(original / "en_us_data.dir"),
                    str(output / "proposed-strings.json"), str(output / "locale"), str(output / "roundtrip.json")], check=True)
    roundtrip = json.loads((output / "roundtrip.json").read_text())
    by_text = {r["text"]: r["id"] for r in roundtrip["added"]}
    for entry in proposed:
        entry["name_id"] = by_text[entry["name"]]
        entry["description_id"] = by_text[entry["description"]]
    content = output / "content"
    shutil.copytree(source, content)
    mapping = {"schema_version": 1, "opening_sha256": sha(source / "opening.json"), "locale": 8,
               "notice": "Local reconstructed objective labels for the opening restoration. Original stock text and gameplay definitions are preserved.",
               "entries": proposed}
    json_write(content / "journal-text.json", mapping)
    for original_file in source.iterdir():
        if original_file.is_file() and sha(original_file) != sha(content / original_file.name):
            raise ValueError("Gameplay content clone differs")
    manifest = {"schema_version": 1, "kind": "opening-journal-text", "locale": 8,
                "opening_sha256": mapping["opening_sha256"], "mapping_sha256": sha(content / "journal-text.json"),
                "notice": mapping["notice"], "community_tool": provenance,
                "source_content": str(source), "goal_count": sum(len(s["objectives"]) for s in opening["stages"]),
                "mapped_goal_count": len(proposed), "new_string_count": len(texts),
                "stock_locale_export_sha256": sha(lab / "reports/locale-id-to-text.json"),
                "roundtrip_sha256": sha(output / "roundtrip.json"), "files": []}
    for name in ("en_us_data.dat", "en_us_data.dir"):
        manifest["files"].append({"file": "locale/" + name, "source_sha256": sha(original / name),
                                  "sha256": sha(output / "locale" / name), "length": (output / "locale" / name).stat().st_size})
    json_write(output / "journal-text-overlay.json", manifest)
    print(json.dumps({"package": str(output), "content": str(content), "opening_sha256": manifest["opening_sha256"],
                      "mapping_sha256": manifest["mapping_sha256"], "mapped_goals": len(proposed), "new_strings": len(texts)}))


if __name__ == "__main__":
    main()
