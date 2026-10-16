# Class: SoeUtil::Map<int,ActorSoundEmitterDefinition*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad3c50` | `ActorSoundEmitterDefinitionMapCleanup ⚠downgrade` | 🟢 | virtual | Calls 006fb9c0, destroys an obtained allocator when present, otherwise destroys DAT_01bdf280 or requests an 0x18-byte allocation at alignment four. |
| `00ad41b0` | `MapDestroyNodeChildren` | ✅ | dtor | For a non-null root, recursively destroys its left and right child subtrees, invokes the node cleanup virtual slot, then clears map root/count fields. |
| `00ad4220` | `MapScalarDeletingDtor` | ✅ | dtor | Cleans the map root's two subtrees and node, clears root/count, and frees the map object if the deleting flag is set. |
