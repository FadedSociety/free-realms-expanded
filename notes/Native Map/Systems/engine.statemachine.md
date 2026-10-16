# System: engine.statemachine

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007071b0` | `ResetCallbackState ?` |  | For prior states 3, 6, 7, or 8 calls the stored callback with zero, then resets the state field to zero. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00707240` | `SetCallbackState4 ?` |  | Sets callback state four, stores two supplied dwords, and clears a callback from selected prior states. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00707290` | `SetCallbackState2 ?` |  | Sets callback state two and stores the supplied 64-bit value in the callback-state record. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007072e0` | `SetCallbackState5 ?` |  | Sets callback state five and stores the supplied byte in the callback-state record. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |

