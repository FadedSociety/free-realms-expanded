# Inventory catalog framing compatibility

2026-10-08. Native restart172324 restores120 coins and all16 saved items in the complete original player serializer, but All Items, Clothes/Hats and Chef Head gear display no items. Earlier171251 dynamically granted food/hat rows were visible. Native acceptance of the repair below remains pending.

The startup35/37 catalog omitted the byte-length prefix. The original `PlayerUpdatePacketItemDefinitions.Serialize` writes35/37, then `PacketWriter.WritePayload(Payload)`, which writes an int32 byte length before the payload. The payload itself begins with the int32 definition count. The original client dispatcher0092f460 selects008f81f0, which reads the header through008d6830 and the length/span through008bfc00. It requires end-of-input before calling00a3db70 to read the definition count and definitions. The malformed startup instead supplies1647 as a byte length, leaving309963 bytes unread; its catalog is rejected before registration.

The preserved captured template already has this length. Actual171251/172324 startup catalogs were byte-identical and contained the prepared award definitions, but lacked the length. All1701 disk rows reduce to1647 unique definitions through54 byte-identical duplicates. The compiled unchanged community writer yields311622 bytes, with length311614 and count1647. Its complete payload equals the actual malformed catalog after its four-byte header. No definition, ordering, saved item, equipment, profile, wallet or content hash changes are needed.

Only `Inventory::definitions_packet` now wraps the same count/definition bytes with their original required length. The synthetic complete-packet regression failed before the patch and passes afterward. Twelve ordinary inventory checks and the complete private1647-definition compiled original writer comparison pass. Independent review separately passes the older1386-item captured-self/reward regression with its intended reference content.

Original All Items is a supported local category: UI0.1 floors to native category0, and native predicate00a6dd70 accepts all items for0. The native inventory datasource00a6d9b0 first requires a registered definition through00b0d810, so catalog rejection explains all categories and equipment appearing empty. New Items can be empty after restart because its new-item queue is client-local; that does not establish loss of persisted items.

Evidence:

- Lab `reports/inventory-catalog-framing-proof.json`, SHA2561da63eada651afafb9ff7f1b5c7fa8a07b3cb1acd380b20826bdb04a61b0f741; reproducer `prove-inventory-catalog-framing.py` binds native code, unchanged community sources, captured original/failed startup and exact corrected payload.
- Lab `tools/inventory-catalog-oracle`: compiles unchanged Sanctuary packet/writer/reader sources. Full private output `reports/inventory-catalog-current-community-oracle.bin`, SHA256206ba7cb5006f5bcac1684f3746a775de558a9e0f151ec41b9ed01a3cbd4c9ce. Only small synthetic writer vectors enter repository tests.
- Authoritative restart audit `runs/client-baseline-20261008T172324Z/analysis/native-post-cooking-restart-proof.json`, SHA25692bb4f124550f6a20819657eb1f6bf9c54c9fe981a4bbe4a7ce4a13f44af0830; whole player/save preservation passes, visibility explicitly failed.

Next native gate: fresh launch with the reviewed framing repair, browse All Items, Supplies/Food & Drink, Clothes & Armor/Hats and Chef Head gear; confirm existing food35 and hat14794 display with the same16 items and120 coins. A new grant is unnecessary. No release, client input, save or content mutation was performed by this investigation.
