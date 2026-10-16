# System: engine.lobby

21 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00baff30` | `Lobby_OnJoinedGame` |  | Records the joined-game value and raises the Lobby:JoinedGame event. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bb0b10` | `ClientLobby_ScalarDeletingDtor` | ClientLobby | Destroys a ClientLobby object. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `010488d0` | `LobbyGame::RemoveId64` | LobbyGame | Virtual slot 1 of LobbyGame: removes a 64-bit id entry from its list and hands the node to a release routine through the object at +0x28. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01048940` | `LobbyGame::Ctor` | LobbyGame | Constructor of LobbyGame: installs its vtable and an embedded 64-bit id list at +0x28 and initialises its state fields. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01048a00` | `LobbyGame::CtorWithArgs` | LobbyGame | Second LobbyGame constructor that takes three values into fields +0x38, +0x3c and +0x40 on top of the standard base initialisation. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049900` | `Lobby::Ctor` | Lobby | Constructor of Lobby; it owns a 400-byte table at +0x14 (100 dwords). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010499a0` | `Lobby::CtorWithArg` | Lobby | Second Lobby constructor that also takes one value stored at +0x1a4, right after the 400-byte table. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049a20` | `Lobby::EhCleanupSetVtable` | Lobby | Exception-cleanup funclet-style routine that resets the Lobby vtable on the unwind path (called from two EH unwind funclets and from 00bb0780). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049ab0` | `Lobby::FindThenSlot3` | Lobby | Virtual slot 4 of Lobby (also in ClientLobby): finds an entry by key in the 100-bucket table and forwards the node to the object's virtual slot 3. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049af0` | `Lobby::ScalarDeletingDtor` | Lobby | Scalar deleting destructor of Lobby (vtable slot 0). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01049ba0` | `Lobby::ReleaseNode` | Lobby | Virtual slot 3 of Lobby (and ClientLobby): destroys a node handed over by Lobby::FindThenSlot3 (01049ab0). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb03d0` | `ClientLobby::ContainsPairKey` | ClientLobby | Membership test for a lobby pair key. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bb0780` | `ClientLobby_DtorBody` | ClientLobby | Destroys the client lobby state. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bb0b30` | `ClientLobby::AddPairIfAbsent` | ClientLobby | Adds a lobby pair key only when it is not already present. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `01048860` | `LobbyGame::RequeueId64` | LobbyGame | Virtual slot 0 of LobbyGame: moves a 64-bit id to the tail of its list (removing any earlier copy) and marks the lobby at +0x44. Not a destructor: the body does not match the scalar-deleting shape. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `0127ecb0` | `InitializeLobbyMatches` |  | Builds the InitializeLobbyMatches lobby request for the group identified by the GroupID key. | [[Native Map/Shards/S0667 0127d1b0-01281de0|S0667]] |
| `0127ede0` | `InitializeLobbyPlayers` |  | Builds the InitializeLobbyPlayers lobby request carrying the GroupID and GroupCount keys for the group. | [[Native Map/Shards/S0667 0127d1b0-01281de0|S0667]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb0450` | `ClientLobby::FindPairNode ?` | ClientLobby | Virtual slot [17] of ClientLobby: the same xor-hash bucket walk as ClientLobby::ContainsPairKey over the table at this+0x1bc, returning the matching node pointer or 0. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb0aa0` | `— ⚠reject` | ClientLobby | Constructs the client lobby object. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `00bb0c30` | `— ⚠reject` | ClientLobby | Reports a lobby error code to the player through a localized message and an error event. | [[Native Map/Shards/S0365 00bac0d0-00bb1110|S0365]] |
| `0127f010` | `— ⚠reject` |  | Handles the OnJoinedGroup lobby notification for the group identified by GroupID. | [[Native Map/Shards/S0667 0127d1b0-01281de0|S0667]] |

