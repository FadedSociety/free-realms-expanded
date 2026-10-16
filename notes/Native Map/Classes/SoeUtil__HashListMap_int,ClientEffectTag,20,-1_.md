# Class: SoeUtil::HashListMap<int,ClientEffectTag,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a10c60` | `HashListMapIntClientEffectTag20_Ctor` | ✅ | ctor | Constructs the 20-bucket hash-list map: stores the class vtable, zeroes the count/head/tail fields at +8, +0xc and +0x10, memsets the 0x50-byte bucket array at +0x14 through FUN_00d836a0 (_memset) and zeroes +4. |
| `00a14750` | `— ⚠reject` | ⚪ | method | Unlinks one ClientEffectTag node from the bucket chain and ordered list, updates head/tail/count, writes ClientEffectTag::vftable into the node, calls FUN_0103da90 with the node in ECX (the EffectTag base destructor res… |
| `00a162b0` | `HashListMap_ClientEffectTag_20_minus1_Dtor` | ✅ | dtor | Destructor body: installs the class vftable and loops while the list head at this+8 is non-null, calling the node removal 00a14750 for each ClientEffectTag until the map is empty. |
| `00a16410` | `HashListMap_ClientEffectTag_20_minus1_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): installs the class vftable, drains the per-client effect-tag map by calling 00a14750 on each head node, then frees the object with _free when bit 0 of the flag argument is set. |
| `00a166c0` | `HashListMap_ClientEffectTag_InsertAtTail` | 🟢 | method | Appends a ClientEffectTag node to the 20-bucket map: sets node+0x7c to the current tail (this+0xc) and node+0x78 to zero, updates head at this+8 when the list was empty or the old tail's next link otherwise, replaces th… |
