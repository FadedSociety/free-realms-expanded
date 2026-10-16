# System: net.namechange

18 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a14f10` | `BaseNameChangePacket_Dtor` | NameChange::BaseNameChangePacket | In-place destruction of the base object shared by CheckNamePacket, CheckNameResponsePacket, ChangeNameRequestPacket, NameChangeResponsePacket and CheckNameStatusResponsePacket (base opcode 192). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14f70` | `BaseNameChangePacket_Dtor_00a14f70` | NameChange::BaseNameChangePacket | Second emitted copy of the same destructor, used by CheckNameResponsePacket (00a160e0). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a14fd0` | `BaseNameChangePacket_Dtor_00a14fd0` | NameChange::BaseNameChangePacket | Third emitted copy, used by ChangeNameRequestPacket (00a16ab0). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a15030` | `BaseNameChangePacket_Dtor_00a15030` | NameChange::BaseNameChangePacket | Fourth emitted copy, used by NameChangeResponsePacket (00a16160). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a15090` | `BaseNameChangePacket_Dtor_00a15090` | NameChange::BaseNameChangePacket | Fifth emitted copy, used by CheckNameStatusResponsePacket (00a161e0). | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16000` | `CheckNamePacket_Ctor` | NameChange::CheckNamePacket | Builds the client-to-server name availability check packet; the emulator catalogue lists CheckNamePacket as sub-opcode 1 of base 192 and deserialises it, i.e. the client sends it. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16060` | `CheckNamePacket_ScalarDeletingDtor` | NameChange::CheckNamePacket | Heap-delete path of the name check request packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16080` | `CheckNameResponsePacket_Ctor` | NameChange::CheckNameResponsePacket | Builds the packet object used to parse the server's answer to a name check; the dispatcher 00a21840 constructs it in its sub-opcode 2 branch. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a160e0` | `CheckNameResponsePacket_ScalarDeletingDtor` | NameChange::CheckNameResponsePacket | Heap-delete path of the name check response packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16100` | `NameChangeResponsePacket_Ctor` | NameChange::NameChangeResponsePacket | Builds the packet object that carries the outcome of an actual rename; the dispatcher 00a21840 constructs it in its sub-opcode 4 branch and passes it to the status handler 00a16e30. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16160` | `NameChangeResponsePacket_ScalarDeletingDtor` | NameChange::NameChangeResponsePacket | Heap-delete path of the rename response packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a161e0` | `CheckNameStatusResponsePacket_ScalarDeletingDtor` | NameChange::CheckNameStatusResponsePack… | Heap-delete path of the name status response packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16990` | `CheckNamePacket_CtorWithName` | NameChange::CheckNamePacket | The property-style builder of the name check request used by the three in-shard callers 00a1fa10, 00a1ff40 and 00a21a50. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16ab0` | `ChangeNameRequestPacket_ScalarDeletingDtor` | NameChange::ChangeNameRequestPacket | Heap-delete path of the rename request packet. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16ad0` | `ChangeNameRequestPacket_CtorWithName` | NameChange::ChangeNameRequestPacket | Property-style builder of the rename request, used by 00a20460, 00a207e0 and 00a20f30. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a16180` | `CheckNameStatusResponsePacket_Ctor ⚠downgrade` | NameChange::CheckNameStatusResponsePack… | Builds the third server-to-client name-change reply class, the one the community catalogue does not contain (it stops at sub-opcode 4), created by the dispatcher at 00a21840. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a16900` | `— ⚠reject` | NameChange::CheckNamePacket | Fills a name check request directly from caller-supplied parameters (caller 00a21c00) instead of the property setters used by 00a16990. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |
| `00a16a30` | `— ⚠reject` | NameChange::ChangeNameRequestPacket | Builds the rename request with a payload name; emulator catalogue lists ChangeNameRequestPacket as base 192 sub 3. | [[Native Map/Shards/S0288 00a13190-00a18a10|S0288]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a16000` | `CheckNamePacket_Ctor` | send | CheckNamePacket | 1 | Hypothesis: a peer may interpret the emitted 192/1 packet as a candidate-name check and answer with 192/2. The native send path is 00a1f8a0 -> 00a1e9e0 and the receive path is dispatcher 00a21840 with parser 00a1ab30; the third-party catalogue supports packet layout only, not original-server behavior. |
| `00a16080` | `CheckNameResponsePacket_Ctor` | recv | CheckNameResponsePacket | 2 | Hypothesis: a peer response may carry the name-check result and echoed identifiers used by the UI. This depends on parser 00a1ab30 and applier 00a20f30 under dispatcher 00a21840; the emulator catalogue supports layout only, not original-server behavior. |
| `00a16100` | `NameChangeResponsePacket_Ctor` | recv | NameChangeResponsePacket | 4 | Hypothesis: a peer response may report a rename result, with client text handling in 00a16e30. This depends on parser 00a1ac10 and dispatcher 00a21840; the catalogue supports packet layout only, not original-server behavior. |
| `00a16990` | `CheckNamePacket_CtorWithName` | send | CheckNamePacket | 1 | Hypothesis: a peer may treat the assigned string as the candidate name in a 192/1 request. This depends on send path 00a1e9e0 and the emulator packet layout; the client request does not establish original-server validation behavior. |
| `00a16ad0` | `ChangeNameRequestPacket_CtorWithName` | send | ChangeNameRequestPacket | 3 | Hypothesis: a peer may interpret this 192/3 packet as a rename request and return a result represented by 192/4. This depends on the client send path, parser/dispatcher 00a1ac10/00a21840, and status handler 00a16e30; it does not establish original-server behavior. |
| `00a16180` | `CheckNameStatusResponsePacket_Ctor ⚠downgrade` | recv |  | 6 | Hypothesis: a peer may send a 192/6 name-change status or eligibility reply; only receive dispatch at 00a21840 establishes this client-side direction. The parser and field layout remain unidentified, and this does not prove original-server behavior. |
| `00a16900` | `— ⚠reject` | send | CheckNamePacket | 1 | Hypothesis: a peer may interpret the emitted CheckNamePacket as a request to validate the supplied name. This depends on caller 00a21c00, send path 00a1e9e0, and the catalogue layout; it does not establish original-server behavior. |
| `00a16a30` | `— ⚠reject` | send | ChangeNameRequestPacket | 3 | Hypothesis: a peer may interpret this 192/3 packet as a rename request and reply with 192/4. This depends on caller 00a20b50, the send helper for this packet, and receive parser/dispatcher 00a1ac10/00a21840; the emulator layout is not proof of original-server behavior. |
