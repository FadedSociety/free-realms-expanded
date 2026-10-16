# Class: SoeUtil::HashListMap<int,AbilityDefinition,100,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a2d9d0` | `AbilityDefinitionMapCtor` | ✅ | ctor | Stores SoeUtil::HashListMap<int,AbilityDefinition,100,-1>::vftable at +0, zeroes the head/tail and count fields at +4..+0x10 and memsets the 100 bucket heads at +0x14 (400 bytes) to zero. |
| `00a312f0` | `HashListMap_AbilityDefinition_Destruct_A` | ✅ | dtor | Stores SoeUtil::HashListMap<int,AbilityDefinition,100,-1>::vftable at +0, then removes head nodes through 00a2f170 until the head at +8 is null. |
| `00a313a0` | `HashListMap_AbilityDefinition_ScalarDeletingDtor` | ✅ | dtor | Virtual slot 0 of SoeUtil::HashListMap<int,AbilityDefinition,100,-1>::vftable. Stores the HashListMap vtable at +0, removes head nodes through 00a2f170 until the head at +8 is null, and frees the object (_free) only whe… |
| `00a317f0` | `HashListMap_AbilityDefinition_Destruct_B` | ✅ | dtor | Fastcall with an SEH frame. Stores the HashListMap<int,AbilityDefinition,100,-1> vtable at +0, then removes the head node at +8 through 00a2f170 until the head is null. |
| `00a31990` | `HashListMap_AbilityDefinition_Ctor ? ⚠rename` | 🟡 | ctor | Stores the HashListMap<int,AbilityDefinition,100,-1> vtable, clears the fields at +4, +8, +0xc, memsets the 400-byte bucket table starting at +0x14, then installs the SoeUtil::HashList<AbilityDefinition,100,-1> vtable a… |
