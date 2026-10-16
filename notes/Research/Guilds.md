---
type: research
updated: 2026-10-13
---

# Guilds

## Scope

Task W48 adds a standalone Rust guild store and family-129 request decoder. The module is included
by `tests/guilds.rs` with `#[path]`; it is not registered in the shared library or gateway. Card-service
guild commands are a different protocol and remain outside this slice.

`notes/Research/Pieces/090 Guild lifecycle.md` was read as earlier evidence. Its sub-opcode table and
client field widths agree with the Build Guide. Its community-only rank ids and permission rules are
not treated as original behavior here. No community source or live guild traffic was used.

## Recovered client facts

The main world connection uses family 129. Client base constructor `00a09b70` stores `0x81` as the
family field and the supplied sub-opcode in the adjacent word; the function was checked with
`nmq dis`. Family and guild sub-opcodes are 16-bit fields. The common client readers are
`00c7ef80`, `008e7290` and `00911c30`; strings use signed 32-bit byte lengths and GUIDs are two
little-endian dwords (`notes/Build Guides/Gameplay/Social (Friends, Guilds, Chat).md`, Data and
formats).

| Sub | Client body | Client evidence |
|---|---|---|
| 1 Create | Three length-prefixed strings | `00c0f3b0` directly calls `00911c30` three times; constructor `00c0ea50` initializes three strings. |
| 2 Delete | One 64-bit GUID | `00c0f4e0` writes the 8-byte field at packet offset `+0x10`. |
| 3 Invite | Two 64-bit GUIDs, one string | `00c0f830`; the decoder preserves GUID order without assigning names to the two fields. |
| 4 Promote | Two 64-bit GUIDs | `00c0fba0`. |
| 5 Demote | Two 64-bit GUIDs | `00c0fcc0`. |
| 6 Remove | Two 64-bit GUIDs | `00c0f960`. |
| 7 Quit | One 64-bit GUID | `00c0fa80`. |
| 8 Accept | One 64-bit GUID | Client responder `00a0f1d0`; guild invite handlers `00a0ee70` and `00a0ed50`. |
| 9 Decline | One 64-bit GUID | Client responder `00a0f130`; invite decline handler `00a0ed50`. |
| 10 Timeout | One 64-bit GUID | Client responder `00a0f290`; timeout handler `00a0ef90`. |
| 11 MOTD | One 64-bit GUID, one string | `00c0f710` and packet constructor `00b19560`. |
| 13 Rename | One 64-bit GUID, two strings, one flag byte | `00c0f600` delegates to `00c0eef0`; its disassembly writes the fields in this order. Parsed only; rename behavior is out of scope. |
| 14 Name request | One 64-bit GUID | `00b1e960` constructs the request; `00b1e4e0` writes one 64-bit field. |
| 15 Member-location request | One 64-bit GUID, a 32-bit count, then 64-bit GUIDs | `00b1e600` writes the GUID; helper `00b1d280` writes the count and array. Parsed only. |
| 16 Paid rename check | One 64-bit GUID, one string | Constructor `00a16c50` stores the GUID and name string. Parsed only. |

The Create and Delete sender labels are `unknown` in the accepted map. Their relevant field writes
were checked directly with `nmq dis 00c0f3b0 300` and `nmq dis 00c0f4e0 300`. Rename writer `00c0eef0`
was also checked directly. The module decodes every client-to-server guild request sub-opcode in
this table; sub-opcode 12 is absent from the recovered enum.

The client maps guild member-status values 1 through 7 to joined, removed, quit, promoted, demoted,
presence change, and silent update (`00b1d7d0`, Build Guide Data and formats). Its status reader
`00b1c840` reads two 16-bit fields, two 64-bit identifiers, a name record, and further scalar fields.
The field meanings are not all recovered. Full guild data uses reader `00b1e730` and nested reader
`00b1e160`; the packet contains counted records and a role map, but its complete field meaning and
server writer are not recovered. The client guild datasource exposes guild name and MOTD plus member
name, rank and online status (Build Guide, Main classes and Data and formats).

Family 129/23 carries a locale-key string (`00b1cd30`; key list in Build Guide Data and formats).
`GuildResultCode` maps the task-relevant client keys, and `error_message` builds this response.
The client uses `Guild.Invite` sub-opcode 3 and pop-up responses 8/9/10. AutoRefuse option index 2
declines through sub-opcode 9. The invitation handler `00b1e7e0` has a speculative map label; its
bytes were inspected with `nmq dis 00b1e7e0 220`, which shows an inviter query and a decline path for
the ignored-inviter branch. These are client behaviors; server enforcement remains reconstructed.

The client seeds four visible role names (Leader, Officer, Member, Recruit) through
`ResolveGuildRoleName` at `00b1a510`. The numeric rank ids and permission matrix are not recovered.
The client locale says any job at level 15 can create a guild and a character can belong to one guild
at a time. Those client requirements do not prove original server enforcement. See the Build Guide's
Guilds and Network sections for the source inventory and known contradictions.

## Reconstructed policy in `src/guilds.rs`

Every rule in this section is local policy (R), not recovered server behavior:

- Guild names are trimmed, 3-32 Unicode characters, and limited to letters, digits, spaces, hyphens
  and apostrophes. Names compare by Unicode lowercase for uniqueness.
- The caller supplies the level-15 eligibility result. The store enforces one membership per
  character. New guilds start with one Leader; invitees join as Recruit.
- The member cap is 100, following the cited wiki value; the client itself does not encode this cap.
- Rank order is Leader, Officer, Member, Recruit. Leader can manage lower ranks. Officer can invite
  and manage Member/Recruit, but cannot act on Leader/Officer. Only Leader can disband or transfer
  leadership. A promote request cannot create a second Leader.
- Explicit transfer promotes the target and leaves the previous leader as Officer. If the leader
  leaves, the highest remaining rank succeeds; lowest character GUID breaks ties. A final member
  leaving dissolves the guild.
- Invite expiry uses caller-supplied monotonic milliseconds and a caller-supplied duration. The
  store allows one pending invite per target. Ignore state and AutoRefuse are passed by the host;
  ignored invitations fail without disclosing the ignore relation.
- MOTD is limited locally to 1,024 UTF-8 bytes. Presence and invite state are runtime-only; login
  notices include the joining member and connected guild members, logout notices reach the remaining
  connected members.
- The server-wide JSON file uses local schema version 1 and a 1 MiB bound. It persists guilds and a
  `character_refs` map keyed by character GUID. This provides the membership reference without
  changing `AccountStore::CharacterRecord`, which is outside W48's owned paths. Presence and pending
  invites do not survive restart.

`GuildNotice` is a typed event for the host to route. It is not a family-129 packet encoder.
`GuildRosterEntry` returns GUID/rank/online state; the host resolves display names from character
records. The client `GuildDataFullPacket` body remains too incomplete to safely encode a full native
roster. Existing `social_chat::resolve_recipients` consumes `ChatSession.guild_id`; callers can set
that value from `GuildStore::guild_guid_for`.

## Checks

Commands ran from `project/prototype/osfr-tools` unless a repository-root path is shown:

```powershell
$rustTools = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools'
$env:CARGO_HOME = Join-Path $rustTools 'cargo'
$env:RUSTUP_HOME = Join-Path $rustTools 'rustup'
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH
& (Join-Path $env:CARGO_HOME 'bin\rustfmt.exe') --edition 2024 src/guilds.rs tests/guilds.rs

$env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W48'
& '..\..\scripts\cargo.ps1' test --test guilds
```

The requested test target passed: **11 passed, 0 failed**. Cargo emitted one unrelated existing
dead-code warning for `housing.rs:567`. The test includes the module with `#[path]`; its small
test-only replacement shim stands in for `character_store::replace_bytes`. The production module
calls the shared atomic replacement helper when registered in the library.

Native-map commands were read-only: `python -I scripts/native_map/nmq.py --help`; `brief` and `show`
lookups for the guild senders/readers named above; and `dis` for `00a09b70`, `00c0f3b0`, `00c0f4e0`,
`00c0eef0`, `00b1c840`, `00b1d7d0`, `00b1e7e0`, `00a0f1d0`, `00a0f130` and `00a0f290`. These checks
do not establish original server behavior or native playability.

## Open gates

- No original guild packet capture, server guild implementation, or native guild-window acceptance
  exists in this evidence set.
- Module registration, request routing, session-to-character lookup, persistence wiring and packet
  delivery remain lead integration work.
- Full roster serialization (129/22) and member-status packet serialization (129/17) remain open;
  the module returns typed notices and roster data only. Do not present these tests as native guild
  acceptance.
- Guild rename/payment, member location replies, invitation queueing for offline targets, original
  numeric roles, permission rules, exact name policy, cap, invitation timing, succession and MOTD
  limits remain unverified or reconstructed.
