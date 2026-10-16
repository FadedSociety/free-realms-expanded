# Class: SoeUtil::Array<CharacterStat,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad7070` | `— ⚠reject` | ⚪ | virtual | Reuses storage when capacity suffices, returns null for zero count, otherwise allocates count*12 bytes and updates capacity. |
| `00ad7110` | `CharacterStatArrayFreeBuffer` | ✅ | virtual | Frees the supplied CharacterStat buffer using alignment four. |
| `00ad7130` | `CharacterStatArrayMoveStorage` | ✅ | virtual | With move flag, dispatches the source array operation; otherwise swaps buffer pointers and exchanges size/capacity fields. |
| `00ad7880` | `CharacterStatArrayDtor` | ✅ | dtor | Installs CharacterStat array vtable, clears size, frees backing storage and zeros its pointer. |
| `00ad7930` | `CharacterStatArrayScalarDeletingDtor` | ✅ | dtor | Frees CharacterStat backing storage and frees the array object when the deleting flag is set. |
| `00ad7b80` | `— ⚠reject` | ⚪ | function | Grows the CharacterStat buffer when required, changes element count and zeros each newly added 12-byte entry. |
