# System: net.tcg

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a9de0` | `TradingCardStartGamePacket::TradingCardStartGamePacket` | TradingCardStartGamePacket | Builds the trading-card-game start packet (sub-opcode 40 per the pinned community catalogue and the vault's TCG note); constructed by 009bf890. Direction is inferred from the class semantics; no capture backs it. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009a9ec0` | `RequestTCGChallengePacket::RequestTCGChallengePacket` | RequestTCGChallengePacket | Builds the client's trading-card-game challenge request (sub-opcode 38 per the pinned community catalogue and the vault's TCG note); constructed by 009bdaf0. | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009a9e70` | `PacketDtor_009a9e70` |  | Destructor of the TradingCardStartGamePacket class; its class vftable at 0180e844 holds the matching scalar deleting destructor 009ac750 in slot 0 (binding read from the vtable data). | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009a9f30` | `PacketDtor_009a9f30` |  | Destructor of the RequestTCGChallengePacket class; its class vftable at 0180e84c holds the matching scalar deleting destructor 009ac7b0 in slot 0 (binding read from the vtable data). | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009ac750` | `DeletingDtor_009ac750` | TradingCardStartGamePacket | Slot 0 of the TradingCardStartGamePacket vftable at 0x0180e844 (bound by reading the vtable data in .rdata; the constructor 009a9de0 stores that same address). The class name is not carried by this function's own eviden… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |
| `009ac7b0` | `DeletingDtor_009ac7b0` | RequestTCGChallengePacket | Slot 0 of the RequestTCGChallengePacket vftable at 0x0180e84c (bound by reading the vtable data in .rdata; the constructor 009a9ec0 stores that same address). The class name is not carried by this function's own evidenc… | [[Native Map/Shards/S0269 009a6cf0-009aceb0|S0269]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `009a9de0` | `TradingCardStartGamePacket::TradingCardStartGamePacket` | recv |  | 0x28 | Hypothesis: the server started a TCG match by sending 39/40 (base MiniGamePacket 39 + sub-opcode 40) with two payload words after a 39/38 challenge request; depends on the TCG matchmaking handler. |
| `009a9ec0` | `RequestTCGChallengePacket::RequestTCGChallengePacket` | send |  | 0x26 | Hypothesis: the server received 39/38 (challenge target in the header) and answered with 39/40 TradingCardStartGamePacket once the match was accepted; depends on the TCG matchmaker. |
