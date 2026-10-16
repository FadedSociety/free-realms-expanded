# System: unknown.timewindow

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01082660` | `FindTimeWindowEntry ?` |  | Thiscall: gets a 64-bit current time via 00702670 into a local, then walks the entry list at this+0x74 (next at entry+0x20); for each entry with flag byte +0x14 clear it compares the given (low dword param_2, high dword… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010826f0` | `FindTimeWindowEntryValue ?` |  | Stdcall (ret 8) with ECX passed through as the object: stores its two arguments as the low and high dwords of a local 64-bit value (bytes: mov edx,[esp+4]; sub esp,8; mov [eax],edx; mov [eax+4],second arg), calls 010826… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01082750` | `FindTimeWindowEntryFound ?` |  | Stdcall (ret 8) with ECX passed through: packs its two arguments as a 64-bit local as in 010826f0, calls 01082660 (FindTimeWindowEntry) and returns true when an entry was found (bytes: setne on the result). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

