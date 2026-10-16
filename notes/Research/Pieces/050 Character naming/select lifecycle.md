# 050 Character naming/select lifecycle

Queue item 050. Acceptance: duplicate/invalid name rules and saved appearance; complete native
select/delete behaviour only with evidence.

Status 2026-10-09 05:30 UTC: the check-name lifecycle (request/reply opcodes, the client's own
result-code switch, the four length messages behind codes 5/7/8/9/10, the preset-name grammar and
the delete wire) is recovered and tested. Duplicate and invalid-name rules are implemented in the
owned module, and the saved-appearance side of selection is exercised against the real login
service. Native **delete** is *not* implemented and no captured native 210/211 exchange has been
decoded, so no native naming acceptance is claimed. The module is not registered in `lib.rs`, and
no production behaviour changed.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_050.rs` | Check-name request/reply codecs, the recovered result-code table with message keys, the preset-name catalogue parser/resolver, the shape + custom-name policies, delete request/reply codecs and the local delete gate (new) |
| `prototype/osfr-tools/tests/piece_050.rs` | 10 tests; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_050.py` | Read-only checker over the preserved client resources, pseudocode, locale export, public manifest and the pinned community vector (new) |
| `tests/pieces/test_piece_050.py` | 9 stdlib tests: real-corpus extraction plus three fail-closed cases (new) |
| `../lab/reports/piece-050-20261009T053017Z/` | `catalogue.json`, `custom-name-policy.json`, `result-codes.json`, `vectors.json`, `rust-cross-check.json`, `findings.json`, `provenance.json` (Rust module SHA256 `e93aa75f…0edc`); `../lab/reports/piece-050-20261009T052852Z/checks.txt` holds the RED/GREEN log, and the earliest `piece-050-20261009T052718Z` bundle is superseded |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save, native-UI or release path
was touched.

## Recovered original facts

1. **Check-name is a real, still-present client flow.** In
   `lab/decompiled/client/FreeRealms/pseudocode.c` the `PacketCheckNameRequest` constructor
   (`FUN_00953950`) stores opcode `0xd2` (210) and `PacketCheckNameReply` (`FUN_00953a10`) stores
   `0xd3` (211). The request carries five length-prefixed strings: first name, last name, name
   definition, last-prefix definition and last-suffix definition. The reply carries first name,
   last name, an `int32` result and a suggestion list whose rows the client logs as
   `Name` / `LastPrefix` / `LastSuffix`.
2. **The client's own result switch is the code authority.**
   `ClientAccountManager::OnRequestCheckName` (`FUN_00710e20`) maps the result to literal message
   keys: `1` available, `2` `Char_Name_Taken_PS3`, `3` `Char_Name_Profane_PS3`,
   `5` `Char_Name_Incorrect_Length_PS3`, `6` `Char_Server_Failure_PS3`,
   `7` `First_Name_Short_PS3`, `8` `Last_Name_Short_PS3`, `9` `First_Name_Long_PS3`,
   `10` `Last_Name_Long_PS3`, `-1 | 4` `Char_Name_Invalid_PS3`, and default
   `Char_Unable_Log_Server_PS3`. There is **no** case for `11`, so the pinned community enum's
   `IllegalCharacters = 11` would render the server-failure text and is not used as a rule here.
3. **The four length messages state their own thresholds.** `CodeStringMappings.txt` maps the keys
   onto locale ids and `lab/reports/locale-id-to-text.json` resolves them:

   | Key | Locale | Text | Rule |
   | --- | ---: | --- | --- |
   | `First_Name_Short_PS3` | 432111 | "The First Name you entered is too short.  Make sure it contains at least two characters." | first ≥ 2 |
   | `Last_Name_Short_PS3` | 432112 | "The Last Name you entered is too short.  Make sure it contains at least two characters." | last ≥ 2 |
   | `First_Name_Long_PS3` | 432097 | "The Custom name you have entered is invalid.  The first name must be less than 15 characters." | first ≤ 14 |
   | `Last_Name_Long_PS3` | 432096 | "The Custom name you have entered is invalid.  The last name must be less than 15 characters." | last ≤ 14 |
   | `Char_Name_Incorrect_Length_PS3` | 432109 | "The name you entered is not the right length.  Your first and last name must contain at least 15 characters." | combined ≥ 15 |

   The two "long" strings say *Custom name*, and the shipped preset tables compose a 9-character
   name ("Ari" + "Ox" + "arm"), so the combined rule cannot have applied to preset names. These
   thresholds come from client text; the server-side enforcement behind them is unverified.
4. **Preset names are composed from three glossary families.**
   `Resources/CharacterCreate/CharacterCreateNames.txt` (39,844 bytes) holds
   `character.name.format = [*char_firstname*] [*char_lastname*]` and
   `character.lastname.format = [*char_lastname_prefix*][*char_lastname_suffix*]`, plus 840
   `first.name` rows (PART_ID 1, 3–11 characters), 234 `last.name.prefix` and 242 `last.name.suffix`
   rows (PART_ID 2, 2–8 characters). The shortest composition is 9 characters; the recovered
   long-name bound is 14.
5. **The supplied native character name is provably a catalogue composition.**
   `lab/state/opening-supported-identity.json` saves `Capo Indigocauldron`, which decomposes
   uniquely into `male.first.name.56` + `last.name.prefix.102` (`Indigo`) +
   `last.name.suffix.32` (`cauldron`). That is the end-to-end link between the creator, the wire
   fields and this glossary.
6. **The numeric error table is gone from the shipped game.** `ErrorCodesCheckName.txt` is a
   36-byte header-only table in the supplied client (SHA256 `465fd089…cf05`) and the pinned public
   manifest lists the same 36 bytes / XXHash64 `9897321053530986055`; the client therefore renders
   the message-key locale rows instead of a code table.
7. **Delete has a real wire but no recovered exchange.** `CharacterDeleteRequest` is opcode 9 with
   a `u64` entity key and `CharacterDeleteReply` is opcode 10 with the key, an `int32` status and a
   length-prefixed payload (pinned community serializers); `characterSelectV2.gfx` exports a
   `deleteButton` handler. No captured original delete exchange exists, and the local service
   answers opcode 9 with status 2 because the separate gameplay save is not coordinated.
8. **Saved appearance on selection already has native acceptance.** Run
   `client-baseline-20261008T163518Z` accepted the saved-name selection row and Go restored the
   same identity fingerprint (see [[Native Character Selection Labels]]). Item 050 adds no new
   native run and does not re-claim that gate.

## Implemented acceptance

`piece_050.rs` provides:

- `NameCheckRequest` / `NameCheckReply` codecs (210/211) with bounds, strict trailing-byte
  rejection and a decode that refuses unrecovered result codes.
- `NameResultCode` with the recovered code ↔ message-key table, `from_code` folding `-1` onto `4`,
  and `UNKNOWN_RESULT_MESSAGE_KEY` for the client default branch.
- `classify_name`, the production-matching duplicate/invalid rule: same window as
  `identity::valid_name` (trimmed, first non-empty, letters plus space/hyphen/apostrophe, ≤ 32
  characters and ≤ 128 bytes per part), plus a case-insensitive duplicate comparison over a
  supplied name set.
- `classify_custom_name`, the recovered custom-name policy (2–14 per part, combined ≥ 15,
  short/long codes 7/8/9/10, code 5 for the combined bound) — deliberately separate so adopting it
  stays a lead decision.
- `NameCatalogue::parse` / `resolve`, which rebuilds the concrete name from the submitted
  definitions using the recovered format rows and rejects unknown ids, half-submitted last names
  and definitions that disagree with the typed name.
- `CharacterDeleteRequest` / `CharacterDeleteReply` codecs and `delete_decision`, the local gate
  (key must match the saved character; a separate gameplay save blocks deletion).

`tests/piece_050.rs` pins the wire against the community `name_ok` vector, cross-checks
`classify_name` against `identity::valid_name` over a case table, and drives the **real**
`LoginService`: login → available check → invalid check → create → duplicate check → verify the
saved identity byte-for-byte → selection reply still equals the saved appearance. That last test
is the item's saved-appearance acceptance.

## Original vs reconstructed

- **Original:** opcodes 210/211, the five request fields, reply first/last/result plus suggestions,
  the result-code switch and message keys, the four length messages and their numbers, the name
  format rows and family counts, the 36-byte empty error table, delete opcodes 9/10.
- **Reconstructed (labelled in code):** the case-insensitive duplicate comparison, the character
  set/32-character window reused from `identity::valid_name`, the custom-name length policy as an
  enforced rule, the formatter that requires typed names to agree with submitted definitions, and
  the local delete gate.
- **Explicit gaps:** the reply suggestion row layout; the numeric error-code table; any profanity
  list; server-side confirmation of the 2/14/15 thresholds; a captured native 210/211 exchange;
  native delete; and the original per-account name-uniqueness scope (the local service still has
  one character).

## Checks and results

| Command | Result |
| --- | --- |
| `scripts/cargo.ps1 test --test piece_050` (RED, placeholder module) | `1 passed; 8 failed` |
| `scripts/cargo.ps1 test --test piece_050` (GREEN) | `10 passed; 0 failed` |
| `..\lab\tools\wiki-env\Scripts\python.exe tests\pieces\test_piece_050.py` | `Ran 9 tests … OK` |
| `scripts/cargo.ps1 test --test login` | `7 passed; 2 ignored` (private oracles) |
| `scripts/cargo.ps1 test --lib` | `24 passed; 0 failed` |
| `scripts/cargo.ps1 test` (whole workspace) | red before this item: other workers' unregistered `tests/piece_047.rs` and `tests/piece_068.rs` do not compile; untouched here |

The checker also fails closed: removing the two format rows and perturbing the Rust opcode both
exit 2 with a named reason.

## Remaining integration / native gates

1. Lead registers `pub mod piece_050;` and decides whether `login.rs` should resolve the three
   definition fields through `NameCatalogue` instead of trusting the typed name, and whether the
   recovered custom-length policy replaces today's 32-character window (that changes which custom
   names the local realm accepts).
2. A captured native create flow is needed to decode a real 210/211 exchange — especially the
   suggestion rows — before claiming native naming acceptance.
3. Native delete needs a coordinated gameplay-save story (item 095 multi-character plus the save
   migration path) before the gate can be allowed; until then opcode 9 keeps answering status 2.

## Skills applied

- **RTK:** normal commands and normal output; no blind prefixing.
- **Ponytail:** reused the existing `identity::valid_name` window, the login service and the pinned
  community vectors; no new dependency, no new crate registration, no speculative scaffold.
- **Caveman:** terse status lines only; this note and the code carry the detail.
- **game-research-websearch / reverse-engineering:** pinned community readers first (Sanctuary,
  FreeRealms-Legacy), then the preserved pseudocode switch, the supplied resource tables and the
  read-only wiki corpus for the naming policy; no custom format reader was written.
