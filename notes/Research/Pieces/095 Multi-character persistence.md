# 095 Multi-character persistence

2026-10-08. Work item 095: remove single-character assumptions with a reversible
migration; prove isolation and restart behavior. This note separates original
recovered facts, the pinned community cross-check, the reconstructed local policy
and the remaining integration/native gates. No native client run is claimed.

## The single-character assumptions being removed

The local storage assumed exactly one character in four places:

* `prototype/osfr-tools/src/character_store.rs` stores one version-1 document
  `{"version":1,"character":{...}}` per file; `load_or_create` returns a single
  `CharacterState`.
* `prototype/osfr-tools/src/identity.rs` returns `Option<CharacterIdentity>` and
  its `validate` rejects anything but `guid == 1` with the literal message
  `invalid single-character identity`; a missing file means "offer the creator".
* `prototype/osfr-tools/src/login.rs` answers selection with
  `selection_reply(identity: Option<&CharacterIdentity>)`, i.e. status 1 and at
  most one entity row.
* `prototype/osfr-tools/src/game_store.rs` and the gateway bind one state file to
  one character (`--state .../rust-gameplay-character.json`).

Item 095 owns only the local save format and its migration. The login/wire side
stays a gate; see "Remaining gates".

## Original facts (first-party client inputs, outside Git)

All paths are relative to `%USERPROFILE%/Documents/Projects/CartoonMMO/lab`.

`decompiled/client-ui/ScriptsBase.lua`, SHA256
`c7c71c7cbb177c975a7cdb12ed0c2e1b74d662f5ca5ef6a4e9ff86d2d332a8f6`:

* `AccountInfo_DS` declares the account datasource columns
  (`ScriptsBase.lua:3103-3110`): `IsMember = 0`, `MaxCharacters = 1`,
  `IsAdminAccount = 2`, `HeadshotUrl = 3`, `CharacterGuid = 4`,
  `CharacterName = 5`. The client therefore renders **a list** of characters and
  reads a per-account slot count, not a single identity.
* `CharacterSelect:Populate` (`ScriptsBase.lua:16187-16202`) calls
  `addCharacter` once per account-info row with `AccountInfo_DS.CharacterGuid` and
  `AccountInfo_DS.CharacterName`, then always appends
  `addCharacter", 300, 0, Ui.GetStringById(436106), "add"` - the "Add a Character"
  row with GUID 0.
* `CharacterSelect:GetMaxCharacters` (`ScriptsBase.lua:16204-16209`) returns
  `ds:GetData(0, AccountInfo_DS.MaxCharacters)`.
* `CharacterSelect:OnUserEvent` (`ScriptsBase.lua:16211-16234`): for the
  `addCharacter` event the client continues to the creator only when
  `self:GetMaxCharacters() > 1`; otherwise it shows
  `NotificationHandler:SystemMessage(msg, nil, "0xFF3333")` with
  `msg = Ui.GetStringById(435820)`.

`reports/locale-id-to-text.json`, SHA256
`f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844`:

* locale 436106 = `Add a Character`.
* locale 435820 = `Get more Characters! Become A Member!`.

So the original client already supported several characters per account, gated the
creator on a server-supplied slot count, and used an upsell message when the
account had only one slot. The *value* of that slot count is server data; it is not
recovered here.

## Community cross-check (pinned, not first-party)

Paths relative to `lab/references/FreeRealms-Legacy/src`.

| File | SHA256 | Fact |
| --- | --- | --- |
| `Sanctuary.Packet.Common/Login/AccountInfo.cs` | `82731f26bd4c8d3467519c340d23ef072a92de1320b13eb6cb7a9ad8c6484e2b` | writes `IsMember`, `MaxCharacters`, `IsAdminAccount` in that order (lines 17-19), matching `AccountInfo_DS` |
| `Sanctuary.Packet/ExternalLogin/CharacterSelectInfoReply.cs` | `80138158d96989652618472761f0da637463e5ca2acf273e9be0a7abdc41d023` | opcode 12 (line 10); serializes a `List<EntityDetails>` (lines 15, 26) |
| `Sanctuary.Login/Handlers/CharacterCreateRequestHandler.cs` | `5c809ca1e9a6195c3516bccba60f78ee91afdd51b683979f2d2f87683d918d01` | refuses creation when `characterCount >= dbUser.MaxCharacters` (lines 208-214) with result 6 |
| `Sanctuary.Database.MySql/Configuration/DbUserConfiguration.cs` | `3b6da0de4001be1903a7e5902d76cf37e474805c710e6ee5fc51f3b3e0d22f89` | `HasDefaultValue(10)` for `MaxCharacters`, line 22 |
| `Sanctuary.Database.Sqlite/Migrations/20250823191050_Initial.cs` | `7300a1c179ef31e70ad19abbc132bc5462a6af144b2df64f3f72b970714f7f51` | `defaultValue: 10` for `MaxCharacters`, line 24 |
| `Sanctuary.Database/Entities/DbUser.cs` | `8a9aa6ca2989f95b3d9d73c9837082e361f24f93c9e63069f5a40aaebd9a26dd` | `ICollection<DbCharacter> Characters` (line 28) owns many characters |
| `Sanctuary.Database/Entities/DbCharacter.cs` | `a347ce2788ca1ec60a5564b4b8ceccd0616abfdbbce54f1e96fcb1bcd4f98bf7` | `public ulong UserId` (line 76) is the owner key; the same character row also holds `PositionX/Y/Z` / `RotationX/Z` |
| `Sanctuary.Core/Helpers/GuidHelper.cs` | `fe19b76732d892f9fe6de3084dfb3b2d0496eecdb87aeda7f0c8d3b8ffbc5a2e` | player guid = `(id << 4) | 1` (lines 10, 33) |

The community project is a cross-check only: it also *chooses* 10 with no citation
to original server data, so 10 is not historical evidence for the real slot count.

## Reconstructed local policy (this item)

`prototype/osfr-tools/src/restoration/piece_095.rs` is the version boundary in
front of `character_store.rs`. It reuses the real version-1 reader/writer by
`#[path]` include, so the legacy bytes cannot drift from what the single-character
build wrote.

Version-2 document (reconstructed shape, not recovered):

```json
{
  "version": 2,
  "active": 1,
  "characters": [
    {"guid": 1, "zone_id": 2, "position": [234.5, -15.25, 800.0], "rotation": [-1.0, 2.0, 0.125]}
  ],
  "migrated_from": {"version": 1, "sha256": "<digest of the legacy file>"}
}
```

Rules implemented:

* Each entry keeps the legacy `CharacterState` shape and validation (nonzero guid,
  finite coordinates inside the existing local magnitude bound). Guids are unique;
  the ordered list is the client row order; `active` is the last selected guid.
* `MAX_CHARACTERS = 10` is a **local** bound mirroring the community default of the
  recovered `MaxCharacters` field. The original per-account value stays unrecovered
  and belongs to the login layer.
* `migrate_legacy` never rewrites `character.json`. It records the SHA256 of the
  exact legacy bytes in the store, so a later change to the legacy file is refused
  (`legacy save changed after migration`) instead of silently overwritten.
* `rollback_to_legacy` writes the canonical version-1 document with the real
  `character_store::save` and refuses when the store holds more than one character,
  so a rollback cannot drop a character. An unchanged character round-trips
  byte-for-byte.
* Corrupt, unsupported, oversized, duplicate-guid and out-of-range documents are
  errors; the file is preserved, never reset. Writes use the existing same-directory
  atomic replacement with the Windows retry.

## Artifact and evidence

Changed files (all inside the item's exclusive paths):

* `prototype/osfr-tools/src/restoration/piece_095.rs` - store, migration, rollback.
  SHA256 `973b511d3da8bff50ddbb58d0392f5482a27505901bef0bfc249998dda8cbec8`.
* `prototype/osfr-tools/tests/piece_095.rs` - ten tests including a fresh-process
  restart proof and a two-reader isolation proof. SHA256
  `db6079766324434845dfaff735498822381d5d64f147b65bd004a5168512e01e`.
* `scripts/pieces/piece_095.py` - read-only evidence checker over the Lua, locale,
  community and Rust inputs. SHA256 `4bc1efc21219867d02f7e61ab396ac0c4da1bf22a9838ea2476e2b0a1fb9ad89`.
* `tests/pieces/test_piece_095.py` - hermetic tamper tests plus the real-corpus
  test. SHA256 `7d54eae4ba9b2d5460b3405daba73d46716cf52eb19ae0914ed512ebe59797e6`.
* Lab bundle: `lab/reports/piece-095-20261009T050357Z/` (RED/GREEN logs, regression
  log, `piece-095-check.json`, `piece-095-provenance.json`).

Commands and results (from `project/`, PowerShell):

| Command | Result |
| --- | --- |
| `./scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_095` before the module existed | RED: `error: couldn't find file tests\..\src\restoration\piece_095.rs`, exit 101 (`red-rust-test.log`) |
| same command after the module, the fresh-install fix and the isolation test | GREEN: 10 passed, 0 failed (`green-final-rust.log`) |
| `./scripts/cargo.ps1 test ... --test character_store --test game_store --test piece_095` | 8 + 1 + 10 passed, 0 failed (`regression-legacy-store.log`, plus the final run) |
| `../lab/tools/wiki-env/Scripts/python.exe -m unittest discover -s tests/pieces -p "test_piece_095.py" -v` before the checker existed | RED: 5 errors, `FileNotFoundError ... scripts/pieces/piece_095.py` (`red-python-test.log`) |
| same command after the checker | GREEN: 5 passed (`green-python-test2.log`) |
| `../lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_095.py --out ../lab/reports/piece-095-20261009T050357Z` | `ok 095 multi-character persistence: 6 rust entry points ...` (`checker-run.log`) |

The checker's real-corpus run pins the Lua and locale hashes above and re-derives
the six account-info column indices, the 436106/435820 rows, the community writer
order, opcode 12, the result-6 gate, the community default 10 and the guid encoding.
Its hermetic tests tamper each claim (including a `MAX_CHARACTERS` rewrite and a
rollback that stops using the real writer) and require the named check to fail.

## Remaining gates (not claimed by this item)

* Integration: the lead registers the module (`pub mod restoration;` /
  `piece_095`) and replaces the `#[path]` include with `use crate::character_store;`.
* Login/wire: `identity.rs` and `login.rs` still load and answer a single identity;
  `selection_reply` still emits at most one entity, so no second character can be
  selected natively yet. The recovered opcode-12 `CharacterSelectInfoReply` entity
  list is the wire shape the login service must adopt.
* Account slot source: the original per-account `MaxCharacters` value is
  unrecovered. The client gate (`> 1`) and the upsell locale are recovered; the
  server value is not.
* Character creation/deletion persistence for a second character (name uniqueness,
  guid allocation, appearance storage) is untouched; `GuidHelper`'s `id << 4 | 1`
  encoding is community evidence for the guid side, not first-party.
* Native acceptance: no run exercises two characters, so multi-character
  selection, add and delete remain natively unproven. The restart proof here is a
  Rust process proof over the save format only.

## Skills applied

* RTK: normal commands first; condensed output kept as the result.
* Ponytail: the module reuses the real version-1 reader/writer instead of copying
  the format; one store, one document version, no speculative abstraction.
* Caveman: terse progress reports; this note keeps the full evidence.
* game-recon / scientific method in reverse-engineering: engine-agnostic save-format
  work; evidence separated from reconstruction.
* reverse-engineering: static reading of the decompiled client Lua and the pinned
  community sources; no custom binary reader was needed.
