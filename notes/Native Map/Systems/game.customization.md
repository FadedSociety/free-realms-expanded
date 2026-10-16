# System: game.customization

23 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095fec0` | `NewHatHairAttachmentGroup::~NewHatHairAttachmentGroup` | NewHatHairAttachmentGroup | Destroys a new hat/hair attachment group. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ff70` | `ProxiedCharacterAttachmentGroup::GetByte9D` | ProxiedCharacterAttachmentGroup | Shared implementation also appears in NewHatHairAttachmentGroup, PlayerSwitchProfileAttachmentGroup, and GuiModelAttachmentGroup vtables. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095ff80` | `NewHatHairAttachmentGroup::ScalarDeletingDtor` | NewHatHairAttachmentGroup | Implements the scalar deleting destructor. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00bd9850` | `HashListMap_PlayerCustomizationData::~HashListMap_PlayerCustomizationData` | SoeUtil::HashListMap<int,PlayerCustomiz… | Destructor body for the PlayerCustomizationData hash-list map. It drains all nodes. Two EH unwind funclets call it, which marks it as cleanup. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd9880` | `HashListMap_PlayerCustomizationData::ScalarDeletingDtor` | SoeUtil::HashListMap<int,PlayerCustomiz… | Tear down the PlayerCustomizationData hash map on delete. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bd9920` | `HashListMap_PlayerCustomizationData::~HashListMap_PlayerCustomizationData` | SoeUtil::HashListMap<int,PlayerCustomiz… | Non-deleting destructor of the PlayerCustomizationData hash map, called at the end of the owning object's destructor 00bd9990 and from EH unwind funclets. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bd9d70` | `HashListMap_PlayerCustomizationData::Ctor` | SoeUtil::HashListMap<int,PlayerCustomiz… | Construct the hash map that holds PlayerCustomizationData records; called with the map subobject address from the PlayerCustomization constructor 00bd9e20. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |
| `00bd9df0` | `HashList_PlayerCustomizationData::ScalarDeletingDtor` | SoeUtil::HashList<PlayerCustomizationDa… | Deleting destructor for the base-class view of the PlayerCustomizationData hash list. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00958110` | `SetHairTintProperty` |  | Sets and notifies the HairTint property; the concrete owning class is not established here. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958130` | `SetEyeTintProperty` |  | Sets and notifies the EyeTint property; the concrete owning class is not established here. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958150` | `GetEyeTint` |  | The DebugDump_EyeTint caller uses this getter; it prefers a linked customization value and falls back to field +0x5f4. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958180` | `GetHairTint` |  | The DebugDump_HairTint caller uses this getter; it prefers a linked customization value and falls back to field +0x5f0. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00bd5980` | `CommittedSlotMatch` |  | Predicate that tests whether a customization record value equals the value currently held for its slot. The offsets match the PlayerCustomization committed block recorded in the vault (S0373). Callers are 00a10730 (adju… | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd5d00` | `HashListMap10_FreeNode` | SoeUtil::HashListMap<int,PlayerCustomiz… | Node-free slot of the PlayerCustomizationData map, the counterpart of the node-destroy slot [2] that HashListMap8_RemoveNode calls for the FotomatPortraitData map (shape match, not verified for the removal path of this … | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd8a00` | `PendingResetSlots` |  | Discards staged (pending) customization edits for one slot or all slots and reverts them to committed values. The domain comes from the neighbours 00bd8e00 and 00bd9560, not from this function's own strings. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd8b20` | `PendingCommitSlots` |  | Commit step that turns staged slot edits into committed values and reports which slot changed. The apply role is inferred from callers 00bd8e00 and 00bd9560. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd8e00` | `PlayerCustomization_ApplyChangeRecord` |  | Per-record apply for a customization change addressed to a target guid, with a try-on (preview) path. The domain rests on the packet anchor PlayerUpdatePacketCustomizationChange and on caller 0092f460. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd9560` | `Slots_CommitChangedApply` |  | Flushes staged slot edits and applies the changed ones as records, either for all slots or for one. The domain comes only from its callee 00bd8e00 and from the neighbours. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd9730` | `PlayerCustomization_ReplaceFromPacket` |  | Applies a parsed PlayerUpdatePacketCustomizationData (opcode 65) by replacing the local container with the packet's record list and resetting pending edits. The dispatch is case 0x41 of caller 0092f460. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd9e20` | `PlayerCustomization_Ctor ⚠downgrade` |  | Builds the per-character customization object that the ClientPcData constructor (00a1de60) creates, holding one item-group data source per customization part plus the PlayerCustomizationData hash map. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bd5cc0` | `HashListMap10_Slot1Helper ?` | SoeUtil::HashListMap<int,PlayerCustomiz… | Virtual slot [1] of the PlayerCustomizationData 10-bucket map (read from the binary): calls 006fb9c0, then calls vtable slot [1] of the object returned by 006fb860 when it is non-null, otherwise of the global DAT_01bdf2… | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bd5e90` | `— ⚠reject` |  | Resolves the PlayerCustomization ItemGroup name keys against the item-group hash table at object+0x1f4. Purpose is inferred from the key strings; the caller 00a10740 is unlabelled. | [[Native Map/Shards/S0373 00bd5e90-00bd9850|S0373]] |
| `00bd9990` | `— ⚠reject` |  | Tear the PlayerCustomization object down; its constructor 00bd9e20 loads the seven ItemGroup.PlayerCustomization* groups and a CustomizationItemDataSource. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bd8e00` | `PlayerCustomization_ApplyChangeRecord` | recv | PlayerUpdatePacketCustomizationChange |  | Hypothesis only: the original server sent PlayerUpdatePacketCustomizationChange (39) as a per-target delta list. Each entry carries a slot Id, an optional StringParam, a Param value and an ItemId, addressed by the 64-bit Guid. Preview=true asked for a try-on that is not stored. This depends on 00bd9730 (opcode 65 full set, which replaces the container) and on 00bd8b20 and 00bd8a00 (commit and rev… |
| `00bd9730` | `PlayerCustomization_ReplaceFromPacket` | recv | PlayerUpdatePacketCustomizationData |  | Hypothesis only: the original server sent PlayerUpdatePacketCustomizationData (65) as the full customization set, one record per Id with StringParam, Param and ItemId. The client replaces its whole set and resets pending edits at the same time. This depends on 00bd8cf0 and 00bd8d70 (the container store) and on 00bd8e00 (per-record apply for opcode 39). No original server logic has been verified. |
