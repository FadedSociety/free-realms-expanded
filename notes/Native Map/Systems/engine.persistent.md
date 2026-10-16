# System: engine.persistent

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01279330` | `PersistentBase::DtorBody` | PersistentBase | Base-class destructor step of the persistent object chain, run during cleanup. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279370` | `PersistentBase::ScalarDeletingDtor` | PersistentBase | Heap-delete path of the PersistentBase object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279390` | `PersistentComponent::DtorBody` | PersistentComponent | Destructor step of the PersistentComponent chain, run during cleanup. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279400` | `PersistentComponent::GetClassName` | PersistentComponent | Class-name tag of the PersistentComponent base. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279440` | `PersistentComponent::ScalarDeletingDtor` | PersistentComponent | Heap-delete path of the PersistentComponent object. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `0127a250` | `PersistentComponent::SetBaseVftables` | PersistentComponent | Base-phase vtable store for the PersistentBase and PersistentComponent chain. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01279410` | `PersistentComponent::GetGlobal_01cbdcf8 ?` | PersistentComponent | Returns the 32-bit global DAT_01cbdcf8 (6 bytes: mov eax, [01cbdcf8]; ret). Virtual slot [12] of PersistentComponent::vftable. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |
| `01279420` | `PersistentComponent::SetGlobal_01cbdcf8 ?` | PersistentComponent | Stores its single stack argument into the global DAT_01cbdcf8 and returns, popping 4 bytes (12 bytes: mov eax, [esp+4]; mov [01cbdcf8], eax; ret 4). Virtual slot [13] of PersistentComponent::vftable. | [[Native Map/Shards/S0666 012773d0-0127d160|S0666]] |

