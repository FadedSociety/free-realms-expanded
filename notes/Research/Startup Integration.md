# Startup Integration

Research snapshot: 2026-10-13. W19 registers the reviewed startup builders in the gateway. This supersedes the pending integration handoff in [[Research/Startup Message Builders]]. It remains capture-compatibility work, not proof of original retail content or server policy.

## Runtime behavior

- `src/lib.rs` registers `startup_messages`, `startup_world`, `startup_catalogues`, `startup_snapshot`, `startup_selfdata`, `account`, `account_store`, and `rewards`. Tests for registered modules now import `osfr_tools::...`.
- The gateway builds startup indices 00-06 and 08-45 from typed snapshot values and existing builders, preserving the eight-message login phase and 38-message ready phase. Opening gameplay still omits the 23 captured NPC rows as before.
- Index 07 (opcode 12 self data) is built from typed self data parsed from template 7; its action-bar rows remain the captured ones.
- The gateway never replays captured startup bytes. The 46 captured templates are still loaded and hash-checked at startup, and only template 7 supplies the typed self-data seed.
- Set `OSFR_GATEWAY_STARTUP_SNAPSHOT` to override the typed JSON path. With no override, the gateway uses `lab/content/startup-snapshot/startup.json`, derived from the configured reference directory under `lab/references`.
- The ignored exporter writes only to `lab/content/startup-snapshot/startup.json`. It omits index 07 and has no fields for the player's name, GUID, owned inventory, or chat. Item definitions are static catalogue rows; their serialized record bytes come from the paired `rust-gameplay/inventory.json` and are checked against captured index 05.

## Evidence and limits

No new native-client behavior was claimed or recovered in this integration. Packet layouts and field claims reuse the reviewed sources:

- The 46-file order, 8/38 phase split, outer tunnel framing, captured lengths, index 07 exception, and startup message map are in [[Research/Startup Message Builders]] and `lab/reports/startup-builders-20261013T191103Z/capture-inventory.json`.
- World-content layouts and reader provenance are in [[Research/Startup World Content]]: quick-chat reader `00b90750` and row reader `00b8f090`; POI handler/readers `00b98b70`, `00b98990`, `00b98290`; AddNpc reader `00921a10`; journal readers `00a45d80`, `00a45d10`, `00a45740`, `00a457e0`, `00a45880`, `00a45920`; welcome readers `00939200`, `009264b0`, `00ad0790`; and MOTD constructor `008ea4e0`.
- Catalogue and status layouts are in [[Research/Startup Catalogues]]: item readers/cache `0092f460`, `00a3db70`, `008e3760`, `00919e10`, `00a3c290`; coin-store handler/reader `00c75fc0`, `00c71510`; ability set dispatch/readers `00a35cc0`, `00a32fe0`, `00a2ccf0`, `008fa5c0`, `008e6760`; effect-tag route/readers `009392c0`, `008d8870`, `00a19f00`, `00a19e70`; stat handler/readers `0091e1a0`, `0090a540`, `008fc4c0`, `008e7640`; combat-flag path `00aa36c0`, `00a9ed60`, `009058d0`.
- The seven `startup_messages` packet facts are recorded in [[Research/Startup Message Builders]]: login reply `PacketLoginReply` has no mapped body reader; environment reader `0090e6a0`; zone constructor `008cebc0` with no mapped receive reader; HP/mana family reader `009392c0` and mana field reader `0089d360`; journal-set parser/row reader `00a45bf0`/`00a45680`; preload marker constructor `0089e8c0`. Index 07's identity/position patch remains the existing gateway path: dispatch `009471a0`, self handling `00944fe0`, item/self path `0090a630`, constructor `0089b240`; the full reader layout remains incomplete.

`(R)` values remain capture-backed reconstruction: unknown settings fields, opaque opcode 193/2 tail, coin-store columns, ability tails, captured character HP/mana/journal/stat rows, and the type-2 value used when a gameplay zone override lacks a zone-type field. Snapshot values are a convenient seed for this private capture, not recovered retail definitions or account rules. Quick-chat text is not exported; NPC display strings and server MOTD are static captured content.

## Export

Run the ignored test exporter with `OSFR_GATEWAY_REFERENCE` set to the private reference directory. It validates the reference manifest and packet hashes, parses the typed world/status data, builds each index except 07, compares all 45 complete tunnel messages byte-for-byte, then writes the JSON in the lab. The committed tree contains no raw captures or exported snapshot.

## Verification

Private Rust target: `%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W19`.

- `rustfmt --edition 2024` ran only on the changed Rust files; exit 0. No crate-wide formatter ran.
- RED before the gateway switch: the synthetic startup-order test saw the fixture's placeholder opcodes instead of the required startup order.
- `cargo test --test gateway startup_builders_use_typed_snapshot_and_keep_self_data_on_its_patch_path -- --exact` — 1 passed.
- `cargo test --test gateway synthetic_startup_sequence_has_the_expected_phase_order_and_lengths -- --exact` — 1 passed; 8 login messages, 38 ready messages, tunnel lengths valid.
- `cargo test --test startup_snapshot empty_snapshot_serializes_as_typed_json_and_loads -- --exact` — 1 passed.
- Filtered ignored private gates passed: `gateway::built_startup_sequence_matches_the_captured_character_byte_for_byte`, `startup_messages::mapped_messages_reproduce_the_private_capture`, `startup_world::mapped_world_content_reproduces_capture_and_exports_typed_rows`, `startup_catalogues::mapped_catalogue_and_status_messages_reproduce_the_private_capture`, and `startup_snapshot::exports_typed_snapshot_to_lab_and_matches_all_45_builders`. Each was run with `-- --ignored --exact`; the typed snapshot path was used.
- Exported `lab/content/startup-snapshot/startup.json` (1,911,797 bytes). Gateway private oracle compared the complete 46-message sequence for the captured character; the 45 builder messages and unchanged index 07 all matched byte-for-byte.
- Required whole ordinary command `scripts/cargo.ps1 test --no-fail-fast` exited 1. Every reported target passed except `quest_engine`, where `prerequisite_level_job_member_and_limit_gates_are_server_owned` and `source_tagged_opening_fixture_matches_the_existing_seventeen_stage_engine` failed. An isolated rerun without startup environment variables reproduced those two failing test names with differing assertion points. No `quest_engine` source or test file was changed; the cause remains open.
- No release binary was built and no native client was launched. Native acceptance remains with the lead.
