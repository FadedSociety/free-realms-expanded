# System: net.packet.lobby_game_def

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a4970` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacket` | BaseLobbyGameDefinitionPacket | Base constructor for opcode-102 lobby-game definitions; outgoing request path uses it through 00a221d0, while receive handler 0091e060 handles responses. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a49d0` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacketSub` | BaseLobbyGameDefinitionPacket | Variable-sub-opcode constructor for opcode-102 definitions; response ctor 008a4b00 uses sub-opcode 2 and request builder 00a221d0 uses sub-opcode 1. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4a40` | `BaseLobbyGameDefinitionPacket::~BaseLobbyGameDefinitionPacket` | BaseLobbyGameDefinitionPacket | Destructor of the opcode-102 lobby game definition base class; the two vtable restores inside the body name the class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4b00` | `LobbyGameDefinitionPacketDefinitionsResponse::LobbyGameDefinitionPacketDefiniti…` | LobbyGameDefinitionPacketDefinitionsRes… | Constructs the opcode-102/sub-2 definitions-response object in receive handler 0091e060; stores two parsed payload words at +0xc and +0x10. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4b70` | `LobbyGameDefinitionPacketDefinitionsResponse::~LobbyGameDefinitionPacketDefinit…` | LobbyGameDefinitionPacketDefinitionsRes… | Destructor of the definitions-response packet; the three in-body vtable restores name the full class chain. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4bd0` | `LobbyGameDefinitionPacketDefinitionsResponse::ScalarDeletingDestructor` | LobbyGameDefinitionPacketDefinitionsRes… | Compiler-generated delete wrapper for the definitions-response class; reached through the class vtable. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00919b60` | `ParseKartLobbyDefinitionTables` |  | Parses the five lobby game-definition tables of the kart lobby definition response. | [[Native Map/Shards/S0241 009162f0-0091aa10|S0241]] |
| `0091e060` | `BaseClient::HandleLobbyGameDefinitions` | BaseClient | Client-side receipt path for lobby game definitions; virtual slot 77 of BaseClient/PlayClient. The vault note recovers the complete 102/2 framing and the five-table body layout from this function. | [[Native Map/Shards/S0242 0091af50-009200d0|S0242]] |
| `00a221d0` | `LobbyGameDefinitionPacketDefinitionsRequest::LobbyGameDefinitionPacketDefinitio…` | LobbyGameDefinitionPacketDefinitionsReq… | Builds the request the client sends to ask the server for the lobby game-definition table. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a22290` | `LobbyGameDefinitionPacketDefinitionsRequest::ScalarDeletingDtor` | LobbyGameDefinitionPacketDefinitionsReq… | Heap-delete path for the lobby game-definition request packet; vtable slot 0 of this class. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a4a90` | `BaseLobbyGameDefinitionPacket::ScalarDeletingDestructor ⚠downgrade` | BaseLobbyGameDefinitionPacket | Compiler-generated delete wrapper for the opcode-102 lobby game definition packet class; reached through the class vtable, no direct callers. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00a22230` | `LobbyGameDefinitionPacketDefinitionsRequest::Dtor` | LobbyGameDefinitionPacketDefinitionsReq… | Cleanup for the lobby game-definition request packet, called from an EH unwind funclet at 015b04d0. | [[Native Map/Shards/S0290 00a1de60-00a243e0|S0290]] |
| `00a273f0` | `SendLobbyGameDefinitionRequest` |  | Sends the definitions request constructed by caller 00a27790; BasePacket is a local wrapper, not this function?s class. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `00a27790` | `SendLobbyGameDefinitionPacketDefinitionsRequest` |  | Builds and conditionally sends the definitions request. Caller 00a27820 applies the delay and one-shot state gate. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a26330` | `WriteTwoU16Fields ?` |  | Serializes two short request fields for the lobby definitions send path. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a4970` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacket` | send | BaseLobbyGameDefinitionPacket | 102 | Hypothesis: the peer accepts opcode-102/sub-1 requests and sends opcode-102/sub-2 definitions responses. Client request construction is in 00a221d0 and response handling is in 0091e060; original-server catalogue contents are unknown. |
| `008a4970` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacket` | recv | BaseLobbyGameDefinitionPacket | 102 | Hypothesis: the peer accepts opcode-102/sub-1 requests and sends opcode-102/sub-2 definitions responses. Client request construction is in 00a221d0 and response handling is in 0091e060; original-server catalogue contents are unknown. |
| `008a49d0` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacketSub` | send | BaseLobbyGameDefinitionPacket | 102 | Hypothesis: the peer accepts definitions requests and sends definitions responses in family 102. Client request path 00a221d0 and response handler 0091e060 establish both directions; original-server catalogue contents are unknown. |
| `008a49d0` | `BaseLobbyGameDefinitionPacket::BaseLobbyGameDefinitionPacketSub` | recv | BaseLobbyGameDefinitionPacket | 102 | Hypothesis: the peer accepts definitions requests and sends definitions responses in family 102. Client request path 00a221d0 and response handler 0091e060 establish both directions; original-server catalogue contents are unknown. |
| `008a4b00` | `LobbyGameDefinitionPacketDefinitionsResponse::LobbyGameDefinitionPacketDefiniti…` | recv | LobbyGameDefinitionPacketDefinitionsResponse |  | Hypothesis: the peer sends the catalogue response parsed by 0091e060. Request constructor 00a221d0 establishes the paired client request; this code does not establish how the peer chooses definitions. |
| `00919b60` | `ParseKartLobbyDefinitionTables` | recv | LobbyGameDefinitionPacketDefinitionsResponse |  | The server sends the kart lobby definition tables in the fixed order the client reads them; the reader is the body of a 102/2 response. |
| `0091e060` | `BaseClient::HandleLobbyGameDefinitions` | recv | LobbyGameDefinitionPacketDefinitionsResponse | 2 | Hypothesis: the original server answered the lobby-definition request with 102/2 carrying the five tables (games, options, values, display order, settings) that this handler unpacks; the note derives the inner 33-subtype map from the captured frames. |
| `00a221d0` | `LobbyGameDefinitionPacketDefinitionsRequest::LobbyGameDefinitionPacketDefinitio…` | send | LobbyGameDefinitionPacketDefinitionsRequest |  | Hypothesis: the original server answered 102/1 with LobbyGameDefinitionPacketDefinitionsResponse (102/2) listing the game configuration options; the client only asked once per lobby session (sender caller 00a27790). |
| `00a273f0` | `SendLobbyGameDefinitionRequest` | send | LobbyGameDefinitionPacketDefinitionsRequest |  | Hypothesis: the request expects LobbyGameDefinitionPacketDefinitionsResponse; this depends on the request construction at 00a27790 and the community packet catalogue, not recovered server logic. |
| `00a27790` | `SendLobbyGameDefinitionPacketDefinitionsRequest` | send | LobbyGameDefinitionPacketDefinitionsRequest |  | Hypothesis: the request expects LobbyGameDefinitionPacketDefinitionsResponse; this depends on the request and the community packet catalogue, not recovered server logic. |
| `00a26330` | `WriteTwoU16Fields ?` | send | LobbyGameDefinitionPacketDefinitionsRequest |  |  |
