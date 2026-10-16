# 075 Postman delivery routes

## Scope

Recovered the Postman delivery-route evidence: the 23 mailbox actors and their street addresses,
the 15 delivery activity rows and their localized names/descriptions, the route quest text that
names the delivery flow, the mailbox spawn transforms, the client "Mail Delivered!" notification,
and the two delivery interaction modes (click a mailbox, or toss mail with the ability). This is a
recovery and validation deliverable. It grants no job, item, currency or experience and sends no
packet.

One helper carries real behaviour, test-first:

- `DeliveryRun::start` / `DeliveryRun::deliver` validate a caller-supplied route against the
  recovered mailbox actors and report completion. The original per-activity target list, the
  delivery order and the numeric timer were **not** recovered, so the helper takes the targets from
  the caller and the module keeps `ROUTE_MEMBERSHIP_RECOVERED = false`, `ROUTE_ORDER_RECOVERED =
  false`, `DELIVERY_TIMER_SECONDS = None`.

## Evidence

| Source | Path | sha256 |
| --- | --- | --- |
| Stock original client locale (hashed ids) | `lab/working/OSFR.Client/OSFR Client/locale/en_us_data.dat` | `9c764d76b05138e607f23276a3a03f364b22c44d2787e31bb96be1b4fe4fc097` |
| Preserved original locale export (85,932 rows) | `lab/reports/job-catalogue-20261009T022023Z/locale-en_us.json` | see `sources` block of `piece-075-evidence.json` |
| Original client activity table | `lab/working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` | see evidence JSON |
| Decompiled client Lua | `lab/decompiled/client-ui/ScriptsBase.lua` | see evidence JSON |
| Community mailbox actor table | `lab/references/Sanctuary/src/Resources/Npcs.json` | see evidence JSON |
| Community model table | `lab/references/Sanctuary/src/Resources/Models.txt` | see evidence JSON |
| Community zone spawn script | `lab/references/Sanctuary/src/Scripts/Zone/FabledRealms.lua` | see evidence JSON |
| Read-only wiki corpus | `lab/references/freerealms-wiki/wiki.sqlite` | see evidence JSON |

Every hash above is recomputed by `scripts/pieces/piece_075.py` into
`lab/reports/piece-075-20261009T042543Z/piece-075-evidence.json`, and the stock locale hash is
compared against the value the journal-text overlay work pins (`scripts/journal_text_overlay.py`
STOCK), reported as `sources.stock_locale.hash_match`.

## Original facts

Stock-locale route text quoted verbatim by the module and re-checked on every run:

- `29679012` "Deliver mail along the Sanctuary route before time runs out, then return to Felipe."
- `708303488` "Deliver mail to mailboxes in Sanctuary"; `557373038` "Click on each of the mailboxes
  specified before time runs out."; `1179122926` "Inform him of your progress on the mail route.";
  `1313439205` "Complete the Running The Route minigame"; `707853610` "Postman: Running the Route".
- `667138698` "Interact with each of the specified people to deliver the mail to them." - the NPC
  interaction mode. The same locale holds 181 distinct `Deliver to <name>` objective names, which
  the checker counts and reports (`npc_delivery_objectives`).
- `1049400037` "Use the Throw Mail ability to deliver mail to the addresses specified. First,
  interact with a mail box to target it, then click the Square Button again to toss mail into it."
  and `1150441937` (mail-toss tutorial line) - the second interaction mode.
- `330643173` "Deliver mail to various addresses in Downtown Sanctuary." and `276975481`
  (Shrouded Glade postmaster thank-you naming "new postal routes").

Mailbox addresses. The stock locale holds every address as its own whole-string row (for example
`2478533725` "1 Seaspray Terrace"), and the preserved locale export resolves the actor `NameId`s to
the same text. The community actor table lists 23 mailbox actors with two models: 910
`sg_mailbox_01.adr` (street mailbox) and 340 `sg_mailbox_group_01.adr` (postbox group). The
community zone script spawns all 23 with world transforms.

Delivery activities. `MiniGameData.txt` holds the 15 recovered Postman delivery rows
(3/5/6/12/13/14/16/24/72/238/239/241/248/263/1002). Live examples: `Can You Deliver?` (238,
difficulty 1, "If you want to be a postman, you've got to learn your territory. See if you can find
these addresses in Sanctuary before time runs out."), `Mail Marksman` (239), `Scattered Mail` (241,
the one row the client types as 1 rather than 2), `Deliver the Mail in Merry Vale` (248) and
`Seaside Mail Delivery` (263). The rest ship with the `NOT LIVE - ` prefix, including
`Special Delivery` (12) and two `Postal Route: Sanctuary` rows (24 and 1002).

Client notification. `ScriptsBase.lua` defines `SH_DeliveryGame.DeliverLetter`, which calls
`NotificationHandler:DeliverLetter(0.7)`, which completes an objective with `Ui.GetString(388255)`
and slides icon 1237. Locale 388255 is "Mail Delivered!".

## Community and wiki facts

The wiki `Postman` page documents the job as a freestyle delivery job, names the trainers (Felipe,
Flitter, Postmaster Vixie, Postchug Runalug, Loryn, Brin Galeglow) and lists 30+ quest entries
including `Postman: Running the Route`, `Postman: Can You Deliver?`, `Postman: Special Delivery`
and `Postman: What a Mess!`. The mailbox spawn transforms are community-pinned world data, not
original recovered server rows; they are labelled `CommunityTable` in the module.

## Implementation

`prototype/osfr-tools/src/restoration/piece_075.rs` (test-first, `#[path]` included by
`prototype/osfr-tools/tests/piece_075.rs` until the lead registers the module):

- Recovered tables: `MAILBOXES` (23 rows with address, `NameId`, model, kind and spawn transform),
  `DELIVERY_ACTIVITIES` (15 rows with exact names/descriptions), `LOCALE_TEXT_CLAIMS`,
  `ROUTE_TEXT_CLAIMS` and the delivery notification constants.
- Behaviour: `DeliveryRun` (start/deliver/progress/remaining/is_complete) plus `mailbox` and
  `mailbox_by_address` lookups and a `validate()` structural check.
- Gap flags: `ROUTE_MEMBERSHIP_RECOVERED`, `ROUTE_ORDER_RECOVERED`,
  `DELIVERY_OBJECTIVE_PACKETS_RECOVERED` and `DELIVERY_TIMER_SECONDS`.

`scripts/pieces/piece_075.py` re-reads every pinned source, cross-validates the Rust module against
it, and fails closed with a named check: `MailboxRowDrift`, `MailboxSpawnMissing`,
`MailboxSpawnDrift`, `ActivityRowDrift`, `RouteTextDrift`, `LocaleTextDrift`,
`DeliveryNotificationDrift`, `RouteGapClaimed`, `WikiPostmanMissing`, `EvidenceUnreadable`.

## Checks

| Check | Command | Result |
| --- | --- | --- |
| Rust RED | `scripts/cargo.ps1 test --test piece_075 --manifest-path prototype/osfr-tools/Cargo.toml` before the module existed | `error: couldn't find file tests\..\src\restoration\piece_075.rs`, exit 101 - `lab/reports/piece-075-20261009T042543Z/red-test-rust.log` |
| Python RED | `lab/tools/wiki-env/Scripts/python.exe tests/pieces/test_piece_075.py -v` before the checker existed | `Ran 10 tests ... FAILED (failures=9, errors=1)`, exit 1 - `red-test-python.log` |
| Rust GREEN | same cargo command | `6 passed; 0 failed; 0 ignored`, exit 0 - `green-test-rust.log`, repeated after `rustfmt` as `green-test-rust-postfmt.log` |
| Python GREEN | same unittest command | `Ran 10 tests ... OK`, exit 0 - `green-test-python.log`, repeated after `rustfmt` as `green-test-python-postfmt.log` |
| Evidence document | `lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_075.py --output lab/reports/piece-075-20261009T042543Z/piece-075-evidence.json` | `piece 075 evidence ok: 23 mailboxes, 15 delivery activities, 16 route lines, 181 NPC delivery objectives, 4 gaps` - `checker-run.log` |
| Format | `lab/tools/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin/rustfmt.exe --edition 2024 prototype/osfr-tools/src/restoration/piece_075.rs prototype/osfr-tools/tests/piece_075.rs` | clean |

Python tamper cases in the suite: a drifted mailbox `NameId` (1755), a removed zone spawn (2297),
a rewritten activity `NAME_ID` (238), a rewritten route line (29679012), a module that claims a
recovered route order, and a missing stock locale and wiki corpus. Each must exit 2 with its named
check.

## Artifacts

See the final worker report for the sha256 of each artifact as written; recompute with
`Get-FileHash` before relying on them.

## Limitations and next acceptance

- The per-activity target list, the delivery order and the numeric timer remain missing; nothing
  here invents them. The smallest missing evidence is the server-side quest/objective rows for the
  Postman delivery quests (or a captured delivery session), which would name the ordered targets
  and the timer.
- No preserved packet capture covers a delivery activity; the archived captures cover soccer,
  racing and demolition derby only.
- The mailbox spawn transforms come from the community zone script. They are world data, not
  proof that the original server placed the same actors.
- The helper is wired into no client or gateway path and no client run was made. Native acceptance
  stays with the lead.
- No shared file, gateway, save, content or release path was touched.
