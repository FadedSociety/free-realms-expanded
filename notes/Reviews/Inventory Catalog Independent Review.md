# Inventory catalog independent review

2026-10-08. **No blocking findings in the narrow initial catalog repair. Fresh native inventory visibility remains pending.**

Finalized restart172324 proves exact saved ownership and whole original self serialization while native All Items/Hats/Chef Head are empty. Initial35/37 is311618 bytes and places definition count1647 at inner offset4, where the original parser expects a payload byte length. No client inventory category request/rejection was observed. Food/hat/shirt/pants definitions are present; the entire body equals the bound sorted catalog.

The unchanged original native constructor/dispatcher/reader and pinned community `PlayerUpdatePacketItemDefinitions`/`PacketWriter.WritePayload` require a four-byte byte-length before count/definitions. The native parser requires successful exact end-of-input. Treating1647 as byte length leaves309963 trailing bytes and prevents the actual catalog consumer from running.

Independently checked213 original executable instructions with Capstone, all original pseudocode/dispatcher assertions, the captured original packet and all seven unchanged linked community source hashes. Rebuilt the original writer to a new private output directory: corrected packet311622 bytes, length311614 at offset4,count1647 at offset8, SHA256 `206ba7cb5006f5bcac1684f3746a775de558a9e0f151ec41b9ed01a3cbd4c9ce`.

The correction inserts only the missing four-byte wrapper length. Its entire311614-byte count/definition body equals actual failed startup bytes. Disk1701 rows have54 identical duplicate IDs and produce1647 sorted unique definitions, matching `Inventory::load`. Source changes only `definitions_packet`; it does not touch item state, category data, item definitions, saved schema, economy, recipe/quest rewards or self serialization.

Independent checks pass:12 ordinary inventory tests, the complete current catalog original-writer oracle and the older1386-item captured-self/reward regression using its intended reference. Synthetic full-packet vectors match the independently compiled writer and distinguish byte length from count while exercising sorted duplicate input. Existing consumption/equipment/atomic failure/reconnect checks pass. The immutable post-Mae campaign hash remains `d4a1df6ce4688bf81669172b293e4d186adef4bc5a1942668e2bac7988735341`.

Evidence:

- `lab/reports/inventory-catalog-independent-review.json`, SHA256 `f573cf48daada469ad2e97c02c741998d07fa3df61f72da6d655062e4fb6fbcb`.
- Read-only checker `lab/reports/review-inventory-catalog-independent.py`.
- Native/source proof `lab/reports/inventory-catalog-framing-proof.json`, SHA256 `1da63eada651afafb9ff7f1b5c7fa8a07b3cb1acd380b20826bdb04a61b0f741`.
- Independent compiled output directory `lab/reports/inventory-catalog-independent-review-v1`.
- Final failed-visibility restart proof `lab/runs/client-baseline-20261008T172324Z/analysis/native-post-cooking-restart-proof.json`, SHA256 `92bb4f124550f6a20819657eb1f6bf9c54c9fe981a4bbe4a7ce4a13f44af0830`.

Root was informed that the release gate is clear. The next native launch must show restored items in All Items,Food & Drink,Hats and Chef Head without replaying awards. Reviewer changed no shared source, gameplay save, movie, release or native input. An initial combined include-ignored test command lacked the older private test's intended reference; targeted reruns with current/legacy references all passed.
