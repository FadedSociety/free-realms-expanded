# System: unknown.dtor

1 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b4b3e0` | `ScalarDeletingDtor_00b4acb0 ?` |  | Scalar-deleting destructor shape: with ECX as the object it calls 00b4acb0 (the body that frees the pointer at +0xc), then calls _free(this) when bit 0 of its stack byte is set; returns this with ret 4. | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |

