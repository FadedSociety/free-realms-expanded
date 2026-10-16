# System: net.packet.player_title

22 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b49140` | `BasePlayerTitlePacket::CtorDefault` | BasePlayerTitlePacket | Base constructor of the player-title packet family; the 0x98 opcode matches the catalogue's BasePlayerTitlePacket (opcode 152) whose sub-opcodes select add/remove/update-all/request-select. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49210` | `BasePlayerTitlePacket::Dtor` | BasePlayerTitlePacket | Base-class destructor of the title packets, invoked from unwind funclets whenever a title packet is destroyed on a stack that is being unwound. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b492d0` | `PlayerTitleAddPacket::Ctor` | PlayerTitleAddPacket | Constructor for the 'add player title' message (152/1); the four zeroed words are the title payload the handler 00b4a3a0 later fills from the wire. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49340` | `PlayerTitleAddPacket::Dtor` | PlayerTitleAddPacket | Destructor of the add-title packet, called by the packet handler 00b4a3a0 after the parsed data has been consumed and from one unwind funclet. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b493a0` | `PlayerTitleAddPacket::ScalarDeletingDtor` | PlayerTitleAddPacket | Vtable slot 0 of PlayerTitleAddPacket (delete-through-base entry point). | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49410` | `PlayerTitleRemovePacket::Ctor` | PlayerTitleRemovePacket | Constructor for the 'remove player title' message (152/2); the id of the title to remove is supplied afterwards by the sender (00b49540's sibling path uses +0xc for its parameter). | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49470` | `PlayerTitleRemovePacket::Dtor` | PlayerTitleRemovePacket | Destructor of the remove-title packet, called by the handler 00b4a3a0 and from one compiler-generated unwind funclet. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b494d0` | `PlayerTitleRemovePacket::ScalarDeletingDtor` | PlayerTitleRemovePacket | Vtable slot 0 of PlayerTitleRemovePacket. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49540` | `PlayerTitleRequestSelectPacket::Ctor` | PlayerTitleRequestSelectPacket | Constructor for the 'request to select a title' message (152/4); the caller 00b4a880 passes the title id that the server must validate/apply, then sends it through 00b4a620. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b495b0` | `PlayerTitleRequestSelectPacket::Dtor` | PlayerTitleRequestSelectPacket | Destructor of the title-select request, used by the sender path 00b4a880 via its unwind funclet. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49610` | `PlayerTitleRequestSelectPacket::ScalarDeletingDtor` | PlayerTitleRequestSelectPacket | Vtable slot 0 of PlayerTitleRequestSelectPacket. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49680` | `PlayerTitleRefreshRequestPacket::Ctor` | PlayerTitleRefreshRequestPacket | Constructor for the 'refresh my titles' request (152/5) sent by 00b4a910 through the world-packet sender 00b4a730. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b496e0` | `PlayerTitleRefreshRequestPacket::Dtor` | PlayerTitleRefreshRequestPacket | Destructor of the title refresh request, reached from a compiler-generated unwind funclet in the sender 00b4a910. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49740` | `PlayerTitleRefreshRequestPacket::ScalarDeletingDtor` | PlayerTitleRefreshRequestPacket | Vtable slot 0 of PlayerTitleRefreshRequestPacket. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b497b0` | `PlayerTitleUpdateAllPacket::Ctor` | PlayerTitleUpdateAllPacket | Constructor for the 'update all titles' message (152/3, catalogue row PlayerTitleUpdateAllPacket); the handler 00b4a3a0 builds it before parsing the title list. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49810` | `PlayerTitleUpdateAllPacket::Dtor` | PlayerTitleUpdateAllPacket | Destructor of the update-all message, called by 00b4a3a0 and from one unwind funclet. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49870` | `PlayerTitleUpdateAllPacket::ScalarDeletingDtor` | PlayerTitleUpdateAllPacket | Vtable slot 0 of PlayerTitleUpdateAllPacket. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4a3a0` | `HandlePlayerTitlePacket` |  | Receive path for the whole player-title feature: title add/remove/update-all messages are decoded here and applied to the client's title table. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4a880` | `SendPlayerTitleRequestSelect` |  | Sends the client's request to select a title for the local player; the packet's sub-opcode 4 and single payload word identify the title. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4a910` | `SendPlayerTitleRefreshRequest` |  | Asks the server to resend the player's title list; the header-only payload matches the constructor, which initialises nothing beyond opcode and sub-opcode. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b491a0` | `— ⚠reject` | BasePlayerTitlePacket | Shared base constructor for all six title packets; each derived constructor calls it with 1 (add), 2 (remove), 3 (update-all), 4 (request-select) or 5 (refresh request). | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b49260` | `— ⚠reject` | BasePlayerTitlePacket | Vtable slot 0 of BasePlayerTitlePacket, i.e. the delete-through-base entry point for the title packet family. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b4a3a0` | `HandlePlayerTitlePacket` | recv | PlayerTitleUpdateAllPacket | 3 | Hypothesis: the server sent opcode-152 messages with a 32-bit sub-opcode (1 add, 2 remove, 3 update-all) plus payload, and owned the authoritative title list because the client only applies what it receives and requests a refresh with 152/5. Depends on 00b491a0, 00b492d0, 00b49410, 00b497b0 and the senders 00b4a880/00b4a910. |
| `00b4a880` | `SendPlayerTitleRequestSelect` | send | PlayerTitleRequestSelectPacket |  | Hypothesis: the server validated the requested title (the client only asks, it does not apply locally) and answered with 152/3 update-all or 152/1 add messages; the eventual authoritative title list came from the server. Depends on 00b49540, 00b4a620 and 00b4a3a0. |
| `00b4a910` | `SendPlayerTitleRefreshRequest` | send | BasePlayerTitlePacket |  | Hypothesis: the refresh request implies the server owned the title list and could resend it at any time (152/3 update-all); the client had no local title set to push. Depends on 00b49680 and 00b4a3a0. |
