# Class: SoeUtil::HashListMap<int,EncounterParticipant,25,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a22b30` | `SoeUtil::HashListMapEncounterParticipant25RemoveNode` | 🟢 | method | Unlinks a node from the 25-bucket hash map: bucket = node key at +0x88 modulo 0x19, bucket head from this+0x14+bucket*4, walk through the +0x8c links until the node pointer matches, then write the node's next pointer in… |
| `00a22d40` | `SoeUtil::HashListMapEncounterParticipant25Ctor` | ✅ | ctor | Constructor of the 25-bucket encounter-participant hash map: stores SoeUtil::HashListMap<int,EncounterParticipant,25,-1>::vftable, zeroes the head/tail/count fields at +4/+8/+0xc/+0x10 and memsets the 100-byte bucket ar… |
| `00a22dc0` | `SoeUtil::HashListMapEncounterParticipant25InitNode` | 🟢 | function | SEH-wrapped node initialiser: calls 01062540 with the caller's this (that function sets up the node's SoeUtil::IString vtables and clears the other header fields) and then stores the dword pointed to by the argument int… |
| `00a231e0` | `— ⚠reject` | ⚪ | virtual | Node-acquire implementation for the 25-bucket participant map: pops a recycled node from the pool (006fb9c0/006fb860) and initialises it via virtual slot 4, otherwise allocates a fresh 0x90-byte, 16-aligned block with 0… |
| `00a23220` | `— ⚠reject` | ⚪ | virtual | Deletes one map node: calls 00401c00 with the node pointer and the payload size 0x10 (16 bytes, the size of one EncounterParticipant). |
| `00a23bb0` | `— ⚠reject` | ⚪ | method | Removes a participant node: first unlinks it from its 25-bucket chain with 00a22b30, then unlinks it from the doubly-linked list through the node's +0x80 (predecessor) and +0x84 (successor) fields, fixes the map's head/… |
| `00a24180` | `SoeUtil::HashListMapEncounterParticipant25InsertNode` | 🟢 | function | Links a prepared node into the map: sets the node's +0x84 predecessor to the current tail and +0x80 to 0, points the old tail's +0x80 at the node (or the head slot when the list was empty), updates the tail at this+0xc,… |
| `00a24b00` | `EncounterParticipantHashMapDtor` | ✅ | dtor | Installs the map vtable and repeatedly frees the linked entry at field index two. |
| `00a24b60` | `EncounterParticipantHashMapScalarDeletingDtor` | ✅ | virtual | Destroys linked entries and frees the object only when deletion flag bit zero is set. |
