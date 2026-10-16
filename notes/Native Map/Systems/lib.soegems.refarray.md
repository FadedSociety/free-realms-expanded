# System: lib.soegems.refarray

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0107fd20` | `RefArrayPooledUcharCtor` | SoeGems::RefArrayPooled<unsigned_char> | Constructor of SoeGems::RefArrayPooled<unsigned_char>, a pooled byte array with two vtable bases; container infrastructure. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01080060` | `RefArrayPooledSecondaryDtorThunk` | SoeGems::RefArrayPooled<unsigned_char> | Slot-0 thunk for the second base of SoeGems::RefArrayPooled<unsigned_char>; forwards the deleting path to 01080970. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01080590` | `RefArrayPooledDtorBody` | SoeGems::RefArrayPooled<unsigned_char> | Destructor body of SoeGems::RefArrayPooled<unsigned_char>: releases the byte buffer; container cleanup, no game role shown. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01080970` | `RefArrayPooledScalarDeletingDtor` | SoeGems::RefArrayPooled<unsigned_char> | Scalar deleting destructor of SoeGems::RefArrayPooled<unsigned_char>. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081b60` | `RefArrayPooledFreeBuffer` | SoeGems::RefArrayPooled<unsigned_char> | Buffer free slot of SoeGems::RefArrayPooled<unsigned_char>; shared with the ListEmbedded and RefObjectPool Node variants. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0107fd80` | `RefArrayReplaceBuffer ?` | SoeGems::RefArrayPooled<unsigned_char> | Thiscall: obtains a new buffer through vslot 1 with (size at +0x8, address of the argument, argument); if it differs from the buffer at +0x4 it copies the size at +0x8 bytes with _memcpy, releases the old buffer through… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01081a70` | `— ⚠reject` | SoeGems::RefArrayPooled<unsigned_char> | Buffer allocation slot of SoeGems::RefArrayPooled<unsigned_char>; the same slot is in RefArrayPooledListEmbedded<unsigned_char> and the RefObjectPool Node variants. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

