# System: crt.wctype

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004015a0` | `IsWideDigit` |  | CRT-style wide digit classification helper used by string parsing code. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00489f1c` | `IsDigitW` |  | Thin wrapper around the CRT _iswdigit classifier; it returns whether the pointed-to wide character is classified as a digit. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |

