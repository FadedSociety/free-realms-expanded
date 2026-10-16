# Class: ClosestInteractionTargetDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00baf690` | `ClosestInteractionTargetDataSource::FieldName` | ✅ | virtual | Returns a static field-name string for the index in param_1: 0 gives 'NpcName', 1 gives 'NpcGuid', 2 gives 'NpcIcon', any other value gives '<error>'. |
| `00baf6e0` | `— ⚠reject` | ⚪ | function | Thiscall: resets the cached target pair at this+0x258/+0x25c to the sentinel values, scans the tracked-actor list (008e4330, 008f5120) for candidates whose distance (sqrt of squared offset, compared with DAT_01762a30 an… |
| `00bafb90` | `ClosestInteractionTargetDataSource::GetField` | 🟢 | virtual | Virtual slot [4] of ClosestInteractionTargetDataSource: resolves the object for the cached pair through 00984960 and checks its state (virtual slot [5]); mode 0 returns the name through slot [10] and 0083d250, mode 1 re… |
| `00bafd00` | `ClosestInteractionTargetDataSource_DtorBody` | 🟢 | dtor | Destructor body: stores ClosestInteractionTargetDataSource::vftable at +0, +0x30 and +0x34, then calls 0091bf60 (destroys the remaining sub-object; receiver not shown in the pseudocode). |
| `00bafd80` | `ClosestInteractionTargetDataSource_Ctor` | 🟢 | ctor | Constructor: calls 00922890 with the 'BaseClient.ClosestInteractionTargetData' name, stores the ClosestInteractionTargetDataSource vtables at +0, +0x30 and +0x34, stores param_2 at +0x254, the sentinel pair at +0x258/+0… |
| `00bafe30` | `ClosestInteractionTargetDataSource_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00bafd00 on ECX, then _free when the delete flag bit 0 is set, and returns the object. |
