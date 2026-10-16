# Class: SoeUtil::Deque<SoeUtil::IString<unsigned short>,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9bb70` | `SoeUtil_DequeIStringU16_256_AllocBlock` | 🟢 | method | Allocates a deque storage block of the requested byte size: calls 006fb9c0, asks 006fb860 for the current memory manager and, when one is available, tail-calls its vtable allocate slot; otherwise falls back to FUN_006fb… |
| `00a9bbc0` | `SoeUtil_DequeIStringU16_256_FreeBlock` | 🟢 | method | Forwards its first argument to 00401c00 (FreeMemory) with alignment 4 and returns; the size argument the callers pass (0x1000) is not used by the free route. |
