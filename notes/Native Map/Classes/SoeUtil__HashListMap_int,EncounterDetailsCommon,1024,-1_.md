# Class: SoeUtil::HashListMap<int,EncounterDetailsCommon,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a22850` | `SoeUtil::HashListMapEncounterDetails1024Ctor` | ✅ | ctor | Constructor of the 1024-bucket hash map for EncounterDetailsCommon records: stores SoeUtil::HashListMap<int,EncounterDetailsCommon,1024,-1>::vftable, zeroes the head/tail/count slots at +4, +8, +0xc and +0x10 and memset… |
| `00a22890` | `SoeUtil::HashListMapEncounterDetails1024Find` | 🟢 | method | Hash lookup: takes the key from *param_2, indexes the bucket array at this+0x14 with (key & 0x3ff), walks the chain through each node's +0x2dc link comparing the node key at +0x2d8 with the key, and returns the matching… |
| `00a22990` | `SoeUtil::HashListMapEncounterDetails1024RemoveNode` | 🟢 | method | Unlinks a node from the map: recomputes the bucket from the node key at +0x2d8 (& 0x3ff), walks the chain through the link at +0x2dc until it finds the node by identity and splices it out by writing the node's next poin… |
| `00a23400` | `— ⚠reject` | ⚪ | virtual | Node-acquire implementation for the 1024-bucket encounter-details map and the matching HashList: pops a recycled node from the pool (006fb9c0/006fb860) and initialises it via virtual slot 4, otherwise allocates a fresh … |
| `00a23440` | `SoeUtil::HashListMapEncounterDetails1024FreeNode` | ✅ | virtual | Deletes one node of the encounter-details map (and of the matching HashList) with 00401c00(node, 8). |
| `00a28db0` | `EncounterDetailsMapDtor → SoeUtil::HashListMap<int,EncounterDetailsCommon,1024,…` | ✅ | dtor | Installs map vtable and repeatedly removes records from linked list until empty. |
| `00a28de0` | `EncounterDetailsMapScalarDeletingDtor` | ✅ | virtual | Destroys map entries and frees object if deletion flag low bit is set. |
| `00a29300` | `EncounterDetailsMapClear` | ✅ | dtor | Installs map vtable and removes every linked entry until empty. |
