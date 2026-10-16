"""Build a private, versioned Free Realms playtest content bundle."""

from __future__ import annotations

import argparse
import json
import secrets
import shutil
import subprocess
import sys
import tarfile
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle_common import (  # noqa: E402
    ACTIVE_CONTENT,
    BUNDLE_ROOT,
    CLIENT_RESOURCES,
    GENERATED_CONTENT,
    OPTIONAL_REPOSITORY_CONTENT,
    LOGIN_RESOURCE_FILES,
    LOCALE_FILES,
    JOURNAL_PACKAGE,
    PRIVATE_CONTENT_FILES,
    PROJECT,
    REFERENCE,
    STARTUP_SNAPSHOT,
    MANIFEST_VERSION,
    check_runtime_content,
    sha256_file,
    TERRAIN,
    empty_account_store,
    make_manifest,
    read_json,
    require_regular_source,
    safe_manifest_path,
    scan_bundle_sources,
    verify_manifest,
)
from item_content import check_item_content  # noqa: E402


def copy_file(source: Path, source_root: Path, target: Path) -> None:
    require_regular_source(source, source_root)
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, target)


def write_private_json(source: Path, target: Path) -> None:
    value = read_json(source)
    if not isinstance(value, dict):
        raise ValueError(f"private runtime JSON is not an object: {source.name}")
    changed = False
    # Keep content provenance; drop only the active save digest from package
    # overlays so a private character snapshot cannot be correlated by hash.
    provenance = value.get("provenance")
    if isinstance(provenance, dict) and "character_json_sha256" in provenance:
        provenance.pop("character_json_sha256")
        changed = True
    if source.name == "inventory.json":
        if "initial_state" not in value:
            raise ValueError("active inventory has no saved initial_state to remove")
        value.pop("initial_state")
        changed = True
    target.parent.mkdir(parents=True, exist_ok=True)
    if changed:
        target.write_text(
            json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
    else:
        shutil.copyfile(source, target)


def copy_repository_content(target: Path) -> int:
    prefix = "prototype/osfr-tools/data/content/"
    process = subprocess.Popen(
        ["git", "-C", str(PROJECT), "archive", "--format=tar", "HEAD", "prototype/osfr-tools/data/content"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    count = 0
    try:
        with tarfile.open(fileobj=process.stdout, mode="r|") as archive:
            for member in archive:
                member_name = member.name.rstrip("/")
                if member.isdir() and (
                    member_name == prefix.rstrip("/")
                    or prefix.startswith(member_name + "/")
                ):
                    continue
                if not member.name.startswith(prefix):
                    if member.isdir():
                        continue
                    raise ValueError(f"Git archive escaped the content tree: {member.name}")
                if member.isdir():
                    continue
                if not member.isreg():
                    raise ValueError(f"Git content is not a regular file: {member.name}")
                relative = member.name[len(prefix):]
                if not relative:
                    raise ValueError("Git archive contains an empty content path")
                safe_manifest_path("content/" + relative)
                destination = target / "content" / Path(*Path(relative).parts)
                destination.parent.mkdir(parents=True, exist_ok=True)
                source = archive.extractfile(member)
                if source is None:
                    raise ValueError(f"could not read Git content file: {relative}")
                with source, destination.open("wb") as output:
                    shutil.copyfileobj(source, output)
                count += 1
        process.stdout.close()
        errors = process.stderr.read().decode("utf-8", errors="replace")
        status = process.wait()
        if status:
            raise RuntimeError(f"git archive failed: {errors.strip()}")
    except Exception:
        if process.poll() is None:
            process.kill()
        process.wait()
        raise
    if not count:
        raise ValueError("HEAD repository content directory is empty")
    return count


def copy_private_content(target: Path) -> int:
    count = 0
    for name in PRIVATE_CONTENT_FILES:
        source = ACTIVE_CONTENT / name
        require_regular_source(source, ACTIVE_CONTENT)
        destination = target / "content" / name
        if source.suffix.lower() == ".json" and source.name != "item-catalogue.json":
            write_private_json(source, destination)
        else:
            copy_file(source, ACTIVE_CONTENT, destination)
        count += 1
    return count


def add_generated_content(target: Path) -> list[str]:
    """Write reconstructed (R) runtime content the committed tree does not carry."""
    written = []
    for name, value in GENERATED_CONTENT.items():
        destination = target / "content" / name
        if destination.exists():
            continue  # a committed copy was reviewed; keep it
        destination.write_text(json.dumps(value, indent=2) + "\n", encoding="utf-8")
        written.append(name)
    return written


def uncommitted_content() -> list[str]:
    """Content files edited or added in the working tree but absent from HEAD."""
    result = subprocess.run(
        ["git", "-C", str(PROJECT), "status", "--porcelain", "--untracked-files=all",
         "--", "prototype/osfr-tools/data/content"],
        check=True,
        capture_output=True,
        text=True,
    )
    return sorted(line[3:].rsplit("/", 1)[-1] for line in result.stdout.splitlines() if line[3:])


def copy_reference(target: Path) -> int:
    manifest_path = REFERENCE / "manifest.json"
    require_regular_source(manifest_path, REFERENCE)
    manifest = read_json(manifest_path)
    templates = manifest.get("templates")
    if manifest.get("schema_version") != 1 or not isinstance(templates, list):
        raise ValueError("gateway reference manifest schema is unsupported")
    output = target / "reference"
    copy_file(manifest_path, REFERENCE, output / "manifest.json")
    names = set()
    for entry in templates:
        name = entry.get("file") if isinstance(entry, dict) else None
        if not isinstance(name, str) or Path(name).name != name or name in names:
            raise ValueError("gateway reference contains an unsafe or duplicate template path")
        names.add(name)
        source = REFERENCE / name
        require_regular_source(source, REFERENCE)
        if source.stat().st_size != entry.get("length"):
            raise ValueError(f"gateway reference length mismatch: {name}")
        if sha256_file(source) != entry.get("sha256"):
            raise ValueError(f"gateway reference hash mismatch: {name}")
        copy_file(source, REFERENCE, output / name)
    if len(names) != 46:
        raise ValueError("gateway reference must contain the verified 46 templates")
    return len(names) + 1


def copy_fixed_inputs(target: Path) -> tuple[int, int, int]:
    for relative in LOGIN_RESOURCE_FILES:
        source = CLIENT_RESOURCES / Path(relative)
        copy_file(source, CLIENT_RESOURCES, target / "login-resources" / Path(relative))

    for name in LOCALE_FILES:
        source = JOURNAL_PACKAGE / "locale" / name
        copy_file(source, JOURNAL_PACKAGE / "locale", target / "client-locale" / name)

    terrain_files = sorted(TERRAIN.glob("zone_*.hgt"))
    if not terrain_files:
        raise ValueError("no terrain height grids were found")
    for source in terrain_files:
        copy_file(source, TERRAIN, target / "terrain" / source.name)
    return len(LOGIN_RESOURCE_FILES), len(LOCALE_FILES), len(terrain_files)


def copy_startup_snapshot(target: Path) -> int:
    copy_file(STARTUP_SNAPSHOT, STARTUP_SNAPSHOT.parent, target / "startup" / "startup.json")
    return 1


def source_revision() -> str:
    result = subprocess.run(
        ["git", "-C", str(PROJECT), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    revision = result.stdout.strip()
    if len(revision) != 40 or any(char not in "0123456789abcdef" for char in revision.lower()):
        raise ValueError("could not identify the current Git revision")
    return revision


def readme(bundle_id: str, revision: str) -> str:
    return f"""# Free Realms playtest bundle {bundle_id}

Private local bundle, manifest version {MANIFEST_VERSION}. Source Git revision: `{revision}`.

This folder contains current repository gameplay content, the frozen round70
activity package, the verified gateway startup reference, journal locale files,
the typed startup snapshot, character-creation resource tables, and terrain
grids. It contains no player identity or save, no populated account store, and
no private account key.

`server/accounts.json` is a fresh empty schema-3 store. `server/server-id.bin`
is a new 32-byte public server identifier. The gateway handoff key and every
player's Ed25519 private key must be created or kept outside this bundle.
`client-locale/` is the private journal-text overlay. `login-resources/` contains
only the character creator tables required by `osfr-login`.

## Build and local run

From the repository root, build the server tools with the private toolchain:

```powershell
.\\scripts\\cargo.ps1 build --release --manifest-path .\\prototype\\osfr-tools\\Cargo.toml --bin osfr-login --bin osfr-gateway --bin osfr-account --bin osfr-bot
```

Set up a new runtime directory for this playtest. Keep its saves and secrets
outside this bundle. `OSFR_CLIENT_LOGIN_KEY_FILE` must point to the locally held
legacy client protocol key; this key is deliberately not copied here.

```powershell
$bundle = (Resolve-Path '..\\lab\\bundles\\{bundle_id}').Path
$runtime = Join-Path $env:LOCALAPPDATA ('FreeRealms\\Playtest\\' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $runtime -ErrorAction Stop | Out-Null
Copy-Item "$bundle\\server\\accounts.json" "$runtime\\accounts.json"
python -I -c "import pathlib,secrets,sys; pathlib.Path(sys.argv[1]).write_bytes(secrets.token_bytes(32))" "$runtime\\handoff.key"
$clientKey = $env:OSFR_CLIENT_LOGIN_KEY_FILE
if (-not (Test-Path $clientKey -PathType Leaf)) {{ throw 'Set OSFR_CLIENT_LOGIN_KEY_FILE to the local client protocol key.' }}
$handle = Read-Host 'Account handle (3-32 ASCII characters)'
$publicKey = Read-Host 'Account Ed25519 public key (64 hex characters)'
if ($publicKey -notmatch '^[0-9a-fA-F]{{64}}$') {{ throw 'Enter only the account public key.' }}
& .\\prototype\\osfr-tools\\target\\release\\osfr-account.exe create `
  --store "$runtime\\accounts.json" --handle $handle --public-key $publicKey
if ($LASTEXITCODE -ne 0) {{ throw 'Account provisioning failed.' }}
Write-Host "Playtest runtime: $runtime"
```

`osfr-account` receives the account public key only. Never ask a player to send
their private key. Run the gateway first, then the login service, each in its
own PowerShell terminal. Both bind to loopback. In each terminal, start from the
repository root and enter the runtime path printed by setup.

```powershell
$bundle = (Resolve-Path '..\\lab\\bundles\\{bundle_id}').Path
$runtime = Read-Host 'Full playtest runtime path'
$env:OSFR_GATEWAY_STARTUP_SNAPSHOT = Join-Path $bundle 'startup\\startup.json'
& .\\prototype\\osfr-tools\\target\\release\\osfr-gateway.exe 127.0.0.1:20041 `
  --reference "$bundle\\reference" --state "$runtime\\character.json" `
  --content "$bundle\\content" --terrain "$bundle\\terrain" `
  --client-locale "$bundle\\client-locale" `
  --account-store "$runtime\\accounts.json" --handoff-key-file "$runtime\\handoff.key"
```

```powershell
$bundle = (Resolve-Path '..\\lab\\bundles\\{bundle_id}').Path
$runtime = Read-Host 'Full playtest runtime path'
$handle = Read-Host 'Account handle created during setup'
$clientKey = Read-Host 'Full path to the local client protocol key'
& .\\prototype\\osfr-tools\\target\\release\\osfr-login.exe `
  --bind 127.0.0.1:20042 --auth-bind 127.0.0.1:20043 --gateway 127.0.0.1:20041 `
  --resources "$bundle\\login-resources" --identity "$runtime\\identity.json" --session $handle `
  --state "$runtime\\login-state.json" --account-store "$runtime\\accounts.json" `
  --server-id-file "$bundle\\server\\server-id.bin" `
  --handoff-key-file "$runtime\\handoff.key" --key-file "$clientKey"
```

The local check runs `python -I scripts/bundle/check_bundle.py --bundle
..\\lab\\bundles\\{bundle_id}`. Its bot-created gameplay save is temporary and
is removed when the check exits.

## Manifest

`manifest.json` follows the W121 launcher schema exactly: `version: 1` and a
sorted `files` array of `{{path, size, sha256}}` entries. Every payload file
except `manifest.json` is listed with its normalized relative path, byte size,
and lowercase SHA-256. The folder name carries the playtest stamp; this README
records the source Git revision. The manifest cannot hash itself. These checksums
detect accidental or unreviewed payload changes; they are not a signature or
publisher identity.

The bundle manifest is directly consumable by the W121 parser. W121 still needs
an operator-pinned SHA-256 for the manifest itself; never trust a downloaded
manifest's own pin.

## Private data and release boundary

Do not publish this folder. It contains client-derived packet templates,
localized client text, client resource tables, terrain, and activity data that
includes Legacy-derived material. Keep the bundle in `lab/bundles/` until the
project's content and licensing review explicitly approves distribution. Do
not add user account stores, character identities, player saves, handoff keys,
client private keys, or signing keys.
"""


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--name",
        help="bundle folder name under lab/bundles (default: playtest-<UTC stamp>)",
    )
    args = parser.parse_args()
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    bundle_id = args.name or f"playtest-{stamp}"
    if bundle_id != Path(bundle_id).name or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-" for c in bundle_id):
        parser.error("--name must be a single safe folder name")

    revision = source_revision()
    bundle_root = BUNDLE_ROOT.resolve()
    if BUNDLE_ROOT.is_symlink():
        raise ValueError("refusing a linked lab/bundles directory")
    BUNDLE_ROOT.mkdir(parents=True, exist_ok=True)
    target = bundle_root / bundle_id
    if target.exists() or target.is_symlink():
        raise FileExistsError(f"bundle already exists: {bundle_id}")
    target.mkdir()
    try:
        repo_files = copy_repository_content(target)
        private_files = copy_private_content(target)
        generated = add_generated_content(target)
        reference_files = copy_reference(target)
        login_files, locale_files, terrain_files = copy_fixed_inputs(target)
        startup_files = copy_startup_snapshot(target)

        reference_manifest = read_json(target / "reference" / "manifest.json")
        expected_guid = reference_manifest["initial_state"]["guid"]
        inventory = read_json(target / "content" / "inventory.json")
        if inventory.get("player_guid") != expected_guid or "initial_state" in inventory:
            raise ValueError("fresh inventory seed does not match the gateway reference")

        server = target / "server"
        server.mkdir()
        (server / "accounts.json").write_text(
            json.dumps(empty_account_store(), indent=2) + "\n", encoding="utf-8"
        )
        (server / "server-id.bin").write_bytes(secrets.token_bytes(32))
        (target / "README.md").write_text(readme(bundle_id, revision), encoding="utf-8")

        present = check_runtime_content(target)
        item_content = check_item_content(target)  # a bundle whose items cannot load is not built
        manifest = make_manifest(target)
        (target / "manifest.json").write_text(
            json.dumps(manifest, indent=2) + "\n", encoding="utf-8"
        )
        _, file_count = verify_manifest(target)
        protected_count, _ = scan_bundle_sources(target, manifest)
        total_size = sum(entry["size"] for entry in manifest["files"])
        print(f"Bundle: {target}")
        print(f"Source revision: {revision}")
        print(f"Payload files: {file_count}; payload bytes: {total_size}")
        print(
            "Inputs: "
            f"repo-content={repo_files}, private-content={private_files}, "
            f"gateway-reference={reference_files}, login-resources={login_files}, "
            f"locale={locale_files}, terrain={terrain_files}, "
            f"startup-snapshot={startup_files}"
        )
        print(
            "Runtime content: "
            f"generated={generated or 'none'}; present={present or 'none'}; "
            "absent optional="
            f"{[n for n in OPTIONAL_REPOSITORY_CONTENT if n not in present] or 'none'}"
        )
        print(f"Item content: PASS ({item_content})")
        pending = uncommitted_content()
        if pending:
            print(
                "WARNING: content edited but not committed is NOT in this bundle "
                f"(it is built from HEAD): {', '.join(pending)}"
            )
        print(
            "Security scan: PASS; empty accounts=0, characters=0, public keys=0; "
            f"protected source files checked={protected_count}, matches=0"
        )
        return 0
    except Exception:
        shutil.rmtree(target)
        raise


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except Exception as error:
        print(f"make_bundle: {error}", file=sys.stderr)
        raise SystemExit(1)
