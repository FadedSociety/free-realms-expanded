# System: crt.time

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c5d50` | `GetTimestamp64` |  | Small time wrapper returning a 64-bit time plus milliseconds into a caller-supplied 3-word slot; used by many event/timestamp producers. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00702670` | `CachedWallClockTime ?` |  | Checks the cached tick count, refreshes the stored 64-bit wall-clock value after the interval, and returns the cached pair. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007026b0` | `UnixTimeToCalendar ?` |  | Converts a supplied time value into calendar fields and uses a fallback date when conversion fails. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00702760` | `CalendarToUnixTime ?` |  | Builds a calendar structure from the supplied fields and converts it to a time value. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |

