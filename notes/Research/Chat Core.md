# Chat Core

Task W16, first slice of Build Ledger item 20. The module and tests are scoped under `prototype/osfr-tools`; this note separates facts recovered from the client from server policy (R).

## Client facts

- Family 15 mode 1 is `PacketChat`. The writer at `00b14fa0` emits family/mode, channel, sender and target GUIDs, two `NameData` records, length-prefixed message bytes, a four-float vector, guild GUID, language, and an area id only for channel 8. The reader at `00b11600` mirrors this shape; `00b140c0` bounds the read and checks complete consumption.
- The receive switch at `00b14350` maps channels 0–10: say, tell, system, generic channel, group, yell, trade, LFG, area, guild, members-only. The sender table in `00b15450` uses 0, 1, 4–10. The channel-3 sender exists at `00b16350`, but no registered command reaches it. Modes 2–5 are separate chat payloads and are outside this slice.
- The chat processor reads `ChatSpeed` values Any 330 ms, Dup 1000 ms, Dup3 2000 ms, Max 30000 ms (`00b15450`). Its event handler `00b164b0` uses the 256-entry recent-send list and suppresses similar text using the character edit-distance helper `00b12b70` at threshold 0.2. The client gate is advisory.
- Quick chat uses family 67: sub 1 is the table, sub 2 is phrase tell, and sub 3 is channel phrase (`00b966e0`). The startup row stores the id in `QuickChatRow.id`; display and say localization ids remain separate. The base serializer `00b96d40` writes the common header, phrase id, sender GUID and name data. Channel serialization `00b96f80` adds channel, one dword, and guild GUID; tell serialization `00b97210` adds target GUID and a length-prefixed target string. The manager reads rows through `00b90750` and sends tells through `00b976a0`.
- The client keeps the last ten received lines per speaker (`00b14200`). `/report` builds family 68 sub 1 with the target name and up to ten history rows (`0093b730`); its packet constructor allocates ten row slots (`008dd550`). A row contains two integers and a string in the client history. The server-side reader and exact serialized row envelope were not recovered.
- Family 22 carries the target name for the client’s “player not online” notice; the main dispatcher handles it at `009471a0`. The two account bytes are read from self-data by `00944fe0`. Client Lua blocks typed chat when underage is true and open-chat is false; `src/account.rs` correctly marks server interpretation as reconstructed.

## Implemented behavior

- `social_chat.rs` decodes and encodes family 15 mode 1, preserves the area-only tail, rejects truncated/trailing data, and rejects server-only channels on typed requests. `server_chat_message` rebuilds sender identity and position from a selected session.
- Quick-chat request decoding and the 67/2 and 67/3 builders validate phrase ids against a caller-built catalog of `QuickChatRow.id` values. The builders follow the client serializer fields above. No phrase text is copied into this module; the client resolves localization ids.
- The spam gate uses the client timings, a normalized character edit distance, and a 256-entry recent list. The exact server threshold comparisons remain R.
- `AccountChatPermissions` treats underage accounts without open chat as quick-chat-only (R). A configurable word-list filter rejects digits and words outside its supplied list (R); the permissive filter is the default hook (R), not a claim about historical moderation.
- The in-memory ignore map drops lines for recipients who ignore the sender (R). Recipient resolution accepts an explicit session slice and returns session ids. Implemented scopes are local say within 30 m and same zone (R), tell by online name, group id, zone-wide yell, server-wide trade/LFG, same-zone area id, guild membership after checking the claimed guild id, and members-only after checking membership (all R). The 30 m radius is provisional because no original server radius was recovered.
- Report intake parses the family 68/1 envelope, resolves the reported name through the server’s chat-history name index (including offline targets), ignores client-supplied lines, and copies that character’s server-owned last ten lines into an in-memory moderation inbox. It writes no logs or files. Report envelope count/order and all moderation decisions are R.
- Offline tells produce the family 22 name packet expected by the client. Its delivery point remains the gateway integration.

## Tests and provenance

`tests/social_chat.rs` uses only synthetic identities and synthetic message text. Its byte vectors are independently assembled against the client writer/reader facts above. It covers all eleven `PacketChat` channels, area-tail placement, quick-chat id validation and packet shapes, spam timing/similarity, quick-chat-only permissions, filtering, ignore fanout, offline tell response, report history, and the local-radius boundary. No test writes chat text to disk.

Recovered facts cited above come from the client’s native map and the reviewed Social build guide. The closest original code locations are: `00b11600`, `00b140c0`, `00b14350`, `00b14fa0`, `00b15450`, `00b16350`, `00b164b0`, `00b12b70`, `00b90750`, `00b966e0`, `00b96d40`, `00b96f80`, `00b97210`, `00b97380`, `00b97580`, `00b976a0`, `00b14200`, `0093b730`, `008dd550`, `009471a0`, and `00944fe0`. `nmq dis` checked the unknown `PacketChat` writer, the channel-3 sender, and the speculative/unknown quick-chat channel senders before using their fields.

## Lead integration and open gates

1. Register `social_chat` from the lead-owned module tree; do not copy the existing `chat.rs` developer-command helper into it.
2. Build `ChatSession` values from the live registry, including account membership, zone and chat area, group/guild ids, and authoritative position. Pass account flags from `AccountFlags` to `AccountChatPermissions`. Call `ChatHistory::record` for each accepted line so reports remain available when the speaker disconnects.
3. Route typed 15/1 and quick-chat 67/2–3 through the parser, account/filter checks, spam gate, quick-chat catalogue, recipient resolver, and server-message builders. Send family 22 for offline tells. Keep every reconstructed scope and radius configurable or reviewable at this boundary.
4. Connect `ReportInbox` to the lead’s moderation store only after choosing a retention policy; current implementation is process memory only. Verify the report serializer envelope from client bytes before relying on its exact wire layout.
5. Native two-account acceptance is still required for each channel, quick chat, ignoring, offline tell, and reporting. No native client or gateway integration test was run for this slice.

Open facts: original server recipient rules, local radius, exact chat text/name maxima, safe-chat vocabulary, language filtering, cross-zone trade/LFG scope, report packet list encoding, and moderation retention. The client capture does not settle these values.
