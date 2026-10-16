# System: ui.help

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009959f0` | `OnlineHelpWindow::Slot2FloatStateTick` | OnlineHelpWindow | OnlineHelpWindow virtual slot 2 that advances its state on a float-equality check (3 and 4 transitions), copying a string from this+0x1c to this+0x28 on the 4-to-5 path. | [[Native Map/Shards/S0266 00995720-0099c9a0|S0266]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00995a70` | `OnlineHelpWindow::Slot3IfFalseSetState4 ? ⚠rename` | OnlineHelpWindow | OnlineHelpWindow virtual slot 3 that enters the state-4 transition (009954d0(4)) when its bool argument is false. | [[Native Map/Shards/S0266 00995720-0099c9a0|S0266]] |

