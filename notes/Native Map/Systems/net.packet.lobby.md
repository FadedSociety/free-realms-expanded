# System: net.packet.lobby

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a3ce0` | `BaseLobbyPacket::BaseLobbyPacket` | BaseLobbyPacket | Shared base constructor of the opcode-101 lobby packet family; the nine lobby sub-packet constructors at 008a3da0..008a4360 call it with sub-opcodes 1,2,3,4,6,7,8,11,12. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3da0` | `JoinLobbyGamePacket::JoinLobbyGamePacket` | JoinLobbyGamePacket | Constructs lobby sub-opcode 1 under family opcode 101. Client sender 00bb1890 sends this class, and parser 00923210 also constructs it while handling the matching incoming sub-opcode. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3e60` | `LeaveLobbyGamePacket::LeaveLobbyGamePacket` | LeaveLobbyGamePacket | Constructs lobby sub-opcode 2 under family opcode 101. Client sender 00bb1920 sends this class, and parser 00923210 also constructs it while handling the matching incoming sub-opcode. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3f20` | `StartLobbyGamePacket::StartLobbyGamePacket` | StartLobbyGamePacket | Constructs lobby sub-opcode 3 under family opcode 101. Client sender 00bb19b0 sends this class, and parser 00923210 also constructs it while handling the matching incoming sub-opcode. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3fe0` | `UpdateLobbyGamePacket::UpdateLobbyGamePacket` | UpdateLobbyGamePacket | Constructs opcode-101 sub-opcode 4 for the receive path; dispatcher 00923210 dispatches the received update. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4090` | `SendLobbyToClientPacket::SendLobbyToClientPacket` | SendLobbyToClientPacket | Constructs opcode-101 sub-opcode 6 for the receive path; dispatcher 00923210 creates/stores lobby state. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4140` | `SendLeaveLobbyToClientPacket::SendLeaveLobbyToClientPacket` | SendLeaveLobbyToClientPacket | Constructs opcode-101 sub-opcode 7 for the receive path; dispatcher 00923210 clears lobby state. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a41f0` | `RemoveLobbyGamePacket::RemoveLobbyGamePacket` | RemoveLobbyGamePacket | Constructs opcode-101 sub-opcode 8 for the receive path; dispatcher 00923210 dispatches the received remove. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a42a0` | `LobbyErrorMessagePacket::LobbyErrorMessagePacket` | LobbyErrorMessagePacket | Constructs opcode-101 sub-opcode 11 for the receive path; dispatcher 00923210 dispatches the received error. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4360` | `ShowLobbyUiPacket::ShowLobbyUiPacket` | ShowLobbyUiPacket | Constructs opcode-101 sub-opcode 12 for the receive path; dispatcher 00923210 dispatches the received UI command. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008b7880` | `BaseLobbyPacket::ScalarDeletingDtor` | BaseLobbyPacket | Releases a BaseLobbyPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b78e0` | `JoinLobbyGamePacket::ScalarDeletingDtor` | JoinLobbyGamePacket | Releases a JoinLobbyGamePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7940` | `LeaveLobbyGamePacket::ScalarDeletingDtor` | LeaveLobbyGamePacket | Releases a LeaveLobbyGamePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b79a0` | `StartLobbyGamePacket::ScalarDeletingDtor` | StartLobbyGamePacket | Releases a StartLobbyGamePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7a00` | `UpdateLobbyGamePacket::ScalarDeletingDtor` | UpdateLobbyGamePacket | Releases an UpdateLobbyGamePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7a60` | `SendLobbyToClientPacket::ScalarDeletingDtor` | SendLobbyToClientPacket | Releases a SendLobbyToClientPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7ac0` | `SendLeaveLobbyToClientPacket::ScalarDeletingDtor` | SendLeaveLobbyToClientPacket | Releases a SendLeaveLobbyToClientPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7b20` | `RemoveLobbyGamePacket::ScalarDeletingDtor` | RemoveLobbyGamePacket | Releases a RemoveLobbyGamePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7b80` | `LobbyErrorMessagePacket::ScalarDeletingDtor` | LobbyErrorMessagePacket | Releases a LobbyErrorMessagePacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7be0` | `ShowLobbyUiPacket::ScalarDeletingDtor` | ShowLobbyUiPacket | Releases a ShowLobbyUiPacket allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00bb1890` | `ClientLobby::SendJoinGame` | ClientLobby | Lobby join request path: takes the lobby/game id from ClientLobby+0x1a4 and ships a JoinLobbyGamePacket (base opcode 101 sub-opcode 1). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1920` | `ClientLobby::SendLeaveGame` | ClientLobby | Lobby leave request path (base opcode 101 sub-opcode 2) for the current lobby id. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb19b0` | `ClientLobby::SendStartGame` | ClientLobby | Lobby start-game request path (base opcode 101 sub-opcode 3). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a3d50` | `BaseLobbyPacket::~BaseLobbyPacket` | BaseLobbyPacket | Base teardown for opcode-101 lobby packet objects; the parser calls it on a local temporary whose storage is reclaimed with the caller frame. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3e10` | `JoinLobbyGamePacket::~JoinLobbyGamePacket` | JoinLobbyGamePacket | Base-object teardown step for the opcode-101 JoinLobbyGamePacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3ed0` | `LeaveLobbyGamePacket::~LeaveLobbyGamePacket` | LeaveLobbyGamePacket | Base-object teardown step for the opcode-101 LeaveLobbyGamePacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a3f90` | `StartLobbyGamePacket::~StartLobbyGamePacket` | StartLobbyGamePacket | Base-object teardown step for the opcode-101 StartLobbyGamePacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4040` | `UpdateLobbyGamePacket::~UpdateLobbyGamePacket` | UpdateLobbyGamePacket | Base-object teardown step for the opcode-101 UpdateLobbyGamePacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a40f0` | `SendLobbyToClientPacket::~SendLobbyToClientPacket` | SendLobbyToClientPacket | Base-object teardown step for the opcode-101 SendLobbyToClientPacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a41a0` | `SendLeaveLobbyToClientPacket::~SendLeaveLobbyToClientPacket` | SendLeaveLobbyToClientPacket | Base-object teardown step for the opcode-101 SendLeaveLobbyToClientPacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4250` | `RemoveLobbyGamePacket::~RemoveLobbyGamePacket` | RemoveLobbyGamePacket | Base-object teardown step for the opcode-101 RemoveLobbyGamePacket instance; bound to this class through the dispatcher that constructs and destroys it. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4310` | `LobbyErrorMessagePacket::~LobbyErrorMessagePacket` | LobbyErrorMessagePacket | Base teardown step for the opcode-101 LobbyErrorMessagePacket instance; the class binding comes from the dispatcher that destroys this packet type, not from RTTI inside this body. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a43d0` | `ShowLobbyUiPacket::~ShowLobbyUiPacket` | ShowLobbyUiPacket | Base teardown step for the opcode-101 ShowLobbyUiPacket instance; the class binding comes from the dispatcher that destroys this packet type, not from RTTI inside this body. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00bb1a40` | `ClientLobby::SendLeaveLobby` | ClientLobby | Leave-lobby request used by ClientLobby slot 16; unlike slot 13 (LeaveLobbyGamePacket) this sends the LeaveLobbyPacket RTTI class with a caller-supplied id. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb1cd0` | `ClientLobby::SendCreateGame` | ClientLobby | Create-lobby-game request path of the lobby UI; the packet carries the game definition produced by the setup call. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb1ac0` | `ClientLobby::SendLeaveLobbyAndRemoveSlot44 ? ⚠downgrade` | ClientLobby | Lobby-leave plus removal of the currently selected lobby entry from the client-side list that slot 0x44 exposes. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a3ce0` | `BaseLobbyPacket::BaseLobbyPacket` | send |  | 101 | Hypothesis: the peer endpoint uses opcode 101 with the following sub-opcode. The client sends sub-ops 1, 2 and 3 and parses several server-to-client sub-ops; exact original-server behavior depends on sender methods 00bb1890/00bb1920/00bb19b0 and parser 00923210. |
| `008a3ce0` | `BaseLobbyPacket::BaseLobbyPacket` | recv |  | 101 | Hypothesis: the peer endpoint uses opcode 101 with the following sub-opcode. The client sends sub-ops 1, 2 and 3 and parses several server-to-client sub-ops; exact original-server behavior depends on sender methods 00bb1890/00bb1920/00bb19b0 and parser 00923210. |
| `008a3da0` | `JoinLobbyGamePacket::JoinLobbyGamePacket` | send |  | 1 | Hypothesis: the peer both accepts and emits opcode-101/sub-1 records. Sender 00bb1890 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3da0` | `JoinLobbyGamePacket::JoinLobbyGamePacket` | recv |  | 1 | Hypothesis: the peer both accepts and emits opcode-101/sub-1 records. Sender 00bb1890 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3e60` | `LeaveLobbyGamePacket::LeaveLobbyGamePacket` | send |  | 2 | Hypothesis: the peer both accepts and emits opcode-101/sub-2 records. Sender 00bb1920 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3e60` | `LeaveLobbyGamePacket::LeaveLobbyGamePacket` | recv |  | 2 | Hypothesis: the peer both accepts and emits opcode-101/sub-2 records. Sender 00bb1920 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3f20` | `StartLobbyGamePacket::StartLobbyGamePacket` | send |  | 3 | Hypothesis: the peer both accepts and emits opcode-101/sub-3 records. Sender 00bb19b0 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3f20` | `StartLobbyGamePacket::StartLobbyGamePacket` | recv |  | 3 | Hypothesis: the peer both accepts and emits opcode-101/sub-3 records. Sender 00bb19b0 and client receive path 00923210 establish both directions; original-server state transitions are unknown. |
| `008a3fe0` | `UpdateLobbyGamePacket::UpdateLobbyGamePacket` | recv |  | 4 | Hypothesis: the peer sends opcode-101/sub-4 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `008a4090` | `SendLobbyToClientPacket::SendLobbyToClientPacket` | recv |  | 6 | Hypothesis: the peer sends opcode-101/sub-6 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `008a4140` | `SendLeaveLobbyToClientPacket::SendLeaveLobbyToClientPacket` | recv |  | 7 | Hypothesis: the peer sends opcode-101/sub-7 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `008a41f0` | `RemoveLobbyGamePacket::RemoveLobbyGamePacket` | recv |  | 8 | Hypothesis: the peer sends opcode-101/sub-8 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `008a42a0` | `LobbyErrorMessagePacket::LobbyErrorMessagePacket` | recv |  | 11 | Hypothesis: the peer sends opcode-101/sub-11 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `008a4360` | `ShowLobbyUiPacket::ShowLobbyUiPacket` | recv |  | 12 | Hypothesis: the peer sends opcode-101/sub-12 to the client path handled by 00923210. This constructor establishes the client subtype only; server-side rules are unknown. |
| `00bb1890` | `ClientLobby::SendJoinGame` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the original lobby service accepts base opcode 101 sub-opcode 1 (JoinLobbyGame) carrying the 0x14-byte record (u16 opcode + byte sub-opcode + two dwords) and answers by moving the client into the lobby/game; the community catalogue models base 101 only as GatewayLoginReply, so this mapping is inferred and depends on 008a3da0 and the accept/leave handlers 00bb1920/00bb19b0. |
| `00bb1920` | `ClientLobby::SendLeaveGame` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the lobby service removes the player from the lobby game on base opcode 101 sub-opcode 2 and answers with the corresponding SendLeaveLobbyToClientPacket (sub-opcode 7, ctor 008a4140). Depends on 008a3e60 and the catalogue rows for the 101 family. |
| `00bb19b0` | `ClientLobby::SendStartGame` | send | PacketTunneledClientWorldPacket |  | Hypothesis: on base opcode 101 sub-opcode 3 the server starts the lobby game and pushes the actual minigame launch to the clients (the client-side counterpart is ClientMiniGameActivityLaunchProcessor). Depends on 008a3f20 and the processor handlers in this shard. |
| `00bb1a40` | `ClientLobby::SendLeaveLobby` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the lobby service receives base opcode 101 sub-opcode 9 (LeaveLobbyPacket) with one u32 and detaches the client from the lobby UI; community catalogue rows stop at sub-opcode 4/6/7/8 for the 101 family, so this mapping is inferred and depends on 00bafe60. |
| `00bb1cd0` | `ClientLobby::SendCreateGame` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the lobby service creates a game from the 0xe0-byte game definition produced by the client-side setup call and answers with ShowLobbyUiPacket/SendLobbyToClientPacket (101 sub-opcodes 0xc/6). Depends on 00bb0b80, the tunnel sender and the setup method called at vtable offset 0x54+0x10. |
| `00bb1ac0` | `ClientLobby::SendLeaveLobbyAndRemoveSlot44 ? ⚠downgrade` | send | PacketTunneledClientWorldPacket |  | Hypothesis: the same lobby leave contract as 00bb1a40 (101/9) plus a local list cleanup that has no wire counterpart; inferred together with 00bafe60 and the tunnel sender 00bb1680. |
