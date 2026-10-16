# System: lib.soeutil.crypto

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2e700` | `DtorBody` | Crypto::ArraySecure<unsigned_char,32,1> | Releases the storage of the 32-byte secure byte array embedded in SymmetricKey at +4. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e760` | `Ctor` | Crypto::SymmetricKey | Creates an empty SymmetricKey. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e7c0` | `CtorFromBytes` | Crypto::SymmetricKey | Creates a SymmetricKey that holds a copy of the supplied bytes. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e840` | `DtorBody` | Crypto::SymmetricKey | Destructor body of SymmetricKey. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e8a0` | `ScalarDeletingDtor` | Crypto::ArraySecure<unsigned_char,32,1> | Scalar deleting destructor of the secure byte array. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e8c0` | `ScalarDeletingDtor` | Crypto::SymmetricKey | Scalar deleting destructor of SymmetricKey. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2e480` | `HasNonZeroKeyByte` | Crypto::SymmetricKey | Key-presence test: reports whether the SymmetricKey holds any nonzero byte. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e650` | `WipeAndFreeBuffer` | Crypto::ArraySecure<unsigned_char,32,1> | Secure release of the array storage: wipes live bytes and frees a heap buffer the caller hands in. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e780` | `WipeAndClear` | Crypto::SymmetricKey | Clears the key: overwrites the stored bytes and resets the count. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2e3e0` | `Lcg64NextState ?` |  | Brackets a 64-bit state update with calls to 00fc45f0 and 00fc4610. The state at DAT_01cb53b8/DAT_01cb53bc becomes state * 0x27BB2EE687B0B0FD + 0xB504F32D (mod 2^64) and is stored back; the new state is returned in EDX:… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e680` | `ResetThenAssign ?` |  | Calls the object's own vtable slot 2 with no arguments (the call is 00d2e780 when the object is a SymmetricKey), then assigns (param_2, param_3) into the embedded byte array at this+4 through 00d2e5d0. Returns 8 bytes (… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00d2e4d0` | `— ⚠reject` |  | Overwrites secure-array storage with generator output so key bytes are wiped before release; the SymmetricKey and ArraySecure wipe slots call it. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

