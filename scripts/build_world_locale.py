"""Write the live world's authored text into a client locale pair.

World Builder text (`<world>/text.json`, ids from 7,000,000) becomes rows in
`en_us_data.dat`/`.dir` using the unchanged, pinned MIT FreeRealmsLocaleTools:
each string is added at the id the server assigned (`Preimaging.GenerateEntry`),
so ids stay stable between builds. Every input row is preserved; the output is
re-read strictly and checked before it replaces anything.

    python -I project/scripts/build_world_locale.py --world lab/world \
        --locale <client>/locale [--out <dir>]

Without --out the pair in --locale is updated in place (a disposable run's
client copy, never the preserved original). Players see new text after their
client restarts with the rebuilt pair.
"""
import argparse
import hashlib
import json
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

LAB = Path(__file__).resolve().parents[2] / "lab"
TOOL_REVISION = "ba2ede225ff33edab8e8e0d20c45d4c5156309e5"
CSHARP = r'''using FreeRealmsLocaleTools.IdHashing;
using FreeRealmsLocaleTools.LocaleParser;
using System.Security.Cryptography;
using System.Text.Json;

var locale = new LocaleFileInfo(args[0], args[1], ParseOptions.Strict);
var original = locale.HashToEntry.SelectMany(x => x.Value).ToHashSet();
using var doc = JsonDocument.Parse(File.ReadAllText(args[2]));
int added = 0, kept = 0;
var expected = new Dictionary<uint, string>();
foreach (var e in doc.RootElement.GetProperty("entries").EnumerateArray())
{
    int id = e.GetProperty("id").GetInt32();
    string text = e.GetProperty("text").GetString()!;
    uint hash = Preimaging.GetHash(id);
    expected[hash] = text;
    if (locale.HashToEntry.TryGetValue(hash, out var rows))
    {
        // Already present from an earlier build: same text is fine, anything else collides.
        if (rows.Count == 1 && rows[0].Text == text) { kept++; continue; }
        throw new InvalidDataException($"World text id {id} collides with an existing locale row");
    }
    var entry = Preimaging.GenerateEntry(id, text);
    locale.HashToEntry.Add(hash, [entry]);
    added++;
}
Directory.CreateDirectory(args[3]);
string dat = Path.Combine(args[3], "en_us_data.dat"), dir = Path.Combine(args[3], "en_us_data.dir");
locale.WriteEntries(dat, dir);
var check = new LocaleFileInfo(dat, dir, ParseOptions.Strict);
var stored = check.HashToEntry.SelectMany(x => x.Value).ToHashSet();
if (!original.IsSubsetOf(stored)
    || expected.Any(x => !check.HashToEntry.TryGetValue(x.Key, out var r) || r.Count != 1 || r[0].Text != x.Value)
    || check.Metadata.MD5Checksum != Convert.ToHexString(MD5.HashData(File.ReadAllBytes(dat)))
    || check.Metadata.Count != stored.Count)
    throw new InvalidDataException("Locale round trip failed; nothing was replaced");
Console.WriteLine($"{{\"added\": {added}, \"kept\": {kept}, \"rows\": {stored.Count}}}");
'''


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--world", type=Path, required=True)
    parser.add_argument("--locale", type=Path, required=True, help="folder holding en_us_data.dat/.dir")
    parser.add_argument("--out", type=Path, help="output folder (default: update --locale in place)")
    args = parser.parse_args()
    text = args.world / "text.json"
    if not text.is_file():
        print("no world text yet; locale unchanged")
        return
    source = args.locale.resolve()
    for name in ("en_us_data.dat", "en_us_data.dir"):
        if not (source / name).is_file():
            sys.exit(f"{source / name} is missing")
        if "originals" in (source / name).parts:
            sys.exit("refusing to modify a preserved original")
    community = LAB / "references/FreeRealmsLocaleTools"
    if json.loads((community / "SOURCE.json").read_text())["revision"] != TOOL_REVISION:
        sys.exit("FreeRealmsLocaleTools revision changed; review before use")
    tool = LAB / "tools/world-locale"
    tool.mkdir(parents=True, exist_ok=True)
    (tool / "Program.cs").write_text(CSHARP, encoding="utf-8")
    (tool / "WorldLocale.csproj").write_text(
        '<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType>'
        '<TargetFramework>net10.0</TargetFramework><ImplicitUsings>enable</ImplicitUsings>'
        '<Nullable>enable</Nullable></PropertyGroup><ItemGroup><Compile Include="'
        + str(community / "src/FreeRealmsLocaleTools/**/*.cs")
        + '" /></ItemGroup></Project>', encoding="utf-8")
    with tempfile.TemporaryDirectory() as scratch:
        result = subprocess.run(
            [str(LAB / "tools/dotnet/dotnet.exe"), "run", "--project", str(tool / "WorldLocale.csproj"),
             "-c", "Release", "--", str(source / "en_us_data.dat"), str(source / "en_us_data.dir"),
             str(text), scratch], capture_output=True, text=True)
        if result.returncode != 0:
            sys.exit((result.stdout + result.stderr).strip()[-2000:])
        out = (args.out or source).resolve()
        out.mkdir(parents=True, exist_ok=True)
        for name in ("en_us_data.dat", "en_us_data.dir"):
            shutil.copyfile(Path(scratch) / name, out / name)
    summary = json.loads(result.stdout.strip().splitlines()[-1])
    summary["dat_sha256"] = sha(out / "en_us_data.dat")
    print(json.dumps(summary))


if __name__ == "__main__":
    main()
