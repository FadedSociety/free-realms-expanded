# Class: SoeUtil::List<SoeUtil::HashList<EncounterParticipant,25,5>,-2>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a27890` | `BoundedParticipantMapListInit` | ✅ | function | Installs base and derived list vtables and zeros head, tail, count and extra bookkeeping. |
| `00a282f0` | `BoundedParticipantMapListAllocateNode` | ✅ | virtual | Uses free-node helper for active pool, otherwise allocates one 0x374-byte node from configured allocator. |
| `00a28340` | `BoundedParticipantMapListFreeNode` | ✅ | virtual | Returns node to active pool or directly frees it with four-byte alignment. |
| `00a28360` | `BoundedParticipantMapListDtor` | ✅ | dtor | Installs the bounded list vtable and drains the head at +4, calling 00a27070 to destroy each nested participant map. It then installs the base list vtable and repeats the loop on the same head, which is already empty. |
| `00a28400` | `BoundedParticipantMapListScalarDeletingDtor` | ✅ | virtual | Calls bounded list destructor and frees object when deletion flag low bit is set. |
