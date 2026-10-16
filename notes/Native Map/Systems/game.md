# System: game

34 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009dc030` | `EffectTag::EffectTag` | EffectTag | value copy of an effect tag for the SoeUtil::List<EffectTag> members carried by the factory placement record (+0x44) and factory data records (+0x50) on the blueprint path; the NPC tooltip effect chain is a different RT… | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `01030030` | `RecordDtorBody44 → PointOfInterestDefinition::DtorBody` |  | Releases the string buffer embedded at +0x44 of a record; called from EH unwind funclets and the scalar-deleting teardowns. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `01030160` | `RecordCopyCtor44 → PointOfInterestDefinition::CopyCtor` |  | Copy-constructs a point-of-interest record of at least 0x54 bytes with an embedded IString at +0x44 (used by NodeInitCopyKey). | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009dc0d0` | `EffectTag::ScalarDeletingDtor` | EffectTag | virtual delete stub of the effect tag value type. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `009dd830` | `EffectTag_CopyCtorWrapper` |  | copy-constructs an EffectTag value for the List<EffectTag> add-tail 009df1d0, which serves the factory placement/data records (list members at +0x44 and +0x50), not any NPC tooltip list. | [[Native Map/Shards/S0278 009d89c0-009e00c0|S0278]] |
| `00c7f280` | `StringRecord::CopyConstruct` |  | Copies a string-bearing record used by client social data. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `0102dd10` | `Record::DeletingDtor` |  | Destroys a shared record object (member destructor 008d1550) and frees it when the deleting flag is set. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102f9e0` | `GroupStateDtorBody` |  | Tears down the group state object's pooled member and invitation lists. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `01030410` | `RowTableReloadCaretText → PointOfInterestRowsReloadCaretText` |  | Reloads a row table from a caret-delimited text blob, clearing existing rows first and optionally filtering by the parsed id. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |

## 🟡 speculative (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c7e640` | `ClientSocialData::Destruct ? ⚠rename` |  | Cleans up two owned social-data references. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `0102d0a0` | `ListFindRoundedKey1D24 ?` |  | Cdecl, one stack argument: if the argument is non-null, walks the list whose head is at arg+0x1d24 (next pointer at node+0x14) for a node with int at node+0x10 equal to -0x161d38bd and takes the double at node+0; if non… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102d100` | `ListFindRoundedKey334 ?` |  | Cdecl, one stack argument: if the argument is non-null, walks the list whose head is at arg+0x334 (next pointer at node+0x14) for a node with int at node+0x10 equal to -0x6c604739 and takes the double at node+0; if none… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102d340` | `PredicateStateGate ?` |  | Fastcall (ECX = object): returns 1 when state = this+0xc satisfies (state < 1) or (state == 1 and this+0x6f8 < 1 and byte this+0x10a0 == 0); returns 0 otherwise (state > 1, or state == 1 with this+0x6f8 >= 1 or a nonzer… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102d620` | `ListFindNodeByPair ?` |  | Thiscall (ECX = object, two stack args a, b; ret 8): walks the list from this+0x4 with next pointer at node+0xd0 and returns the first node whose int at +0x0 equals a and int at +0x4 equals b; returns 0 when none matche… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102d6b0` | `ListNodeAtIndex ?` |  | Thiscall (ECX = object, one stack argument = index; ret 4): if 0 <= index < this+0xc (count), walks the list from this+0x4 (next at node+0xd0) and returns the node at that index; otherwise returns 0. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102d8d0` | `ListContainsPair ?` |  | Thiscall (ECX = object, one stack argument = pointer to a two-int key; ret 4): walks the list from this+0x4 (next at node+0xd0) and returns 1 when a node matches the key's first and second ints, else 0. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102da50` | `RecordCopyCtor296 ?` |  | Thiscall (ECX = destination record, param_2 = source record): calls 0070bac0 twice (member initialisation), copies dwords 0 and 1, copies sub-objects at +0x8, +0x2c and +0x50 through 006f9e60, copies dwords at 0x84..0x9… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102dbd0` | `Wrapper_008d1550 ?` |  | Void, EH-framed wrapper (SEH setup, then call 008d1550 with ECX unchanged) that runs the shared record member destructor and returns. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102dc20` | `Wrapper_0102da50 ?` |  | Thiscall (ECX = destination, one stack argument = source; ret 4): SEH-framed wrapper that calls the record copy 0102da50 with this and the source, then returns this in EAX. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102dd70` | `Wrapper_008eb670 ?` |  | Thiscall (ECX = destination, one stack argument = source; ret 4): SEH-framed wrapper that calls 008eb670 with this and the source, then returns this. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102df90` | `ListTryPushUniquePair ?` |  | Thiscall (ECX = owner, param_2 = source record pointer; ret 4): if owner+0xc (count) is below owner+0x109c, checks the pair key with 0102d8d0; when it is absent appends a copy with 0102dee0 and returns 1; otherwise retu… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102dfd0` | `ListTryPushUniquePair ?` |  | Thiscall (ECX = owner, param_2 = source record pointer; ret 4): same body as 0102df90 (count below owner+0x109c, pair absent, append copy via 0102dee0, return 1; else 0). | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102e010` | `ListFindPairRemove ?` |  | Thiscall (ECX = list, param_2 = pointer to a two-int key): walks the list from list+0x4 (next at node+0xd0) for the first node whose ints at +0x0 and +0x4 equal the key; if found calls 008faac0 on that node and returns … | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102f810` | `GroupStateInit ?` |  | Thiscall (ECX = group state object, param_2 = capacity value): constructs the GroupMember pooled list (0102e190) and the Invitation pooled list at object+0x6ec (0102ecb0); stores param_2 at object+0x109c (capacity), cle… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102f8a0` | `GroupStateCopy ?` |  | Thiscall (ECX = destination group state, param_2 = source): constructs both pooled lists, copies +0x109c, the byte at +0x10a0 and the dword at +0x1098; then for each GroupMember node of the source (list head at +0x4, ne… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102fd90` | `FieldGet0x54Arg ?` |  | One stack argument (ret 4; bytes 8b 44 24 04 8b 40 54 c2 04 00): returns the dword at argument+0x54. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102fe10` | `RecordInitIString44 ? → PointOfInterestDefinition::Init` |  | Fastcall (ECX = record): zeroes dwords 0x0 to 0x10 and 0x13 and 0x14 (offsets 0x0-0x40 and 0x4c-0x50), stores the SoeUtil::IString<char> vftable at +0x44 and the empty-string buffer DAT_01cb64b0 at +0x48. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102feb0` | `ParseCaretRecord ? → PointOfInterestDefinition::ParseCaretRecord` |  | Thiscall (ECX = record out, param_2 = pointer to a text-reader object whose +0x4 is the current position): zero-initialises the record as 0102fe10 does, then parses '^'-delimited (0x5e) fields with 0072d900: dwords 0, 1… | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `01030280` | `NodeInitCopyKey ?` |  | Thiscall (ECX = node, param_2 = key pointer, param_3 = source record): copies the source record into the node through 01030160 and stores *param_2 at node+0x5c; returns the node. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |

## ⚪ unknown (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c171d0` | `— ⚠reject` |  | Script hook that tells the death/revive system whether the player accepted a paid revive; the name is the table row 0x0182e4e8 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c7e570` | `— ⚠reject` |  | Initializes two owned client social-data members. | [[Native Map/Shards/S0402 00c7a610-00c7f590|S0402]] |
| `0102d650` | `— ⚠reject` |  | Thiscall (ECX = object, two stack args; ret 8): walks the same list (head this+0x4, next at node+0xd0) looking for the pair (+0x0, +0x4) and returns without producing a value; it has no side effects. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `0102fb50` | `— ⚠reject` |  | Looks up a point-of-interest definition by id in the client table (used by the warpstone travel validation). | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |
| `01030360` | `— ⚠reject` | PointOfInterestDefinitionManager | Inserts one point-of-interest definition record into the manager's 1000-bucket table; the same body is slot [1] of ClientPointOfInterestManager. | [[Native Map/Shards/S0566 0102b8a0-01030cb0|S0566]] |

