# System: game.mount

16 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095ad80` | `SetPendingMountInfo` |  | Records pending rider and mount state for later processing. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00b03c90` | `SetAutoMountId` |  | Setter for the auto-mount id on the client mount object; the field at this+0x64 is compared against the id of each mount-list entry in FUN_00b047b0, and the mount handler 00b06100 is one of the two callers. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b056a0` | `UpdateMountTrialTime` |  | Raises the mount trial time callback when elapsed time advances by a positive whole-minute boundary. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b04240` | `StoreTimerAt68` |  | Stamps a timestamp into the mount object; its only caller is the mount packet handler FUN_00b06100, so the stamp is taken while a mount message is being handled. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b042e0` | `SetSubObjectByte2B1AndRefresh` |  | Flag setter plus refresh pair for the sub-object at this+0x70; its only caller is the mount code FUN_00bff440, and its sibling FUN_00b04310 runs the same refresh pair on mount and dismount. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04310` | `ClearTwoSubObjectFlags` |  | Reset pair run on both the mount and dismount paths (callers FUN_00972d30 and FUN_00973050 log Mount and Dismount with the rider and mount guids). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b047b0` | `ListEntryMatchesIdAndGuid` |  | Membership test used by the automount path: the id field at this+0x64 is the one FUN_00b03c90 (SetAutoMountId) writes, and the only caller is FUN_0096de80, whose log line is Mountable npc ready, automount %d, guid %I64u. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b047f0` | `ListLookupIdByGuid` |  | Guid-to-id lookup over the client's mount list; both callers (FUN_00bdc060, FUN_00bdc420) are in the mount/vehicle area. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b048a0` | `NodeDtorCallWrapper` |  | Second call site of the same destructor body, emitted as its own function (typical for a delete-path or cleanup variant). | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04980` | `NodeInitWithIStringAndGuid` |  | Field initialisation of one mount-list record; the same layout is walked by FUN_00b047b0 and FUN_00b047f0 (id at +0, guid at +0x20) and destroyed by FUN_00b04830. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04a10` | `NodeCtorWithKeyAt38` |  | Creates the record inserted by FUN_00b050d0; caller FUN_00b050d0 is the container insert, so this is the element constructor of the client's mount list. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b04a90` | `NodeScalarDeletingDtor` |  | Deletes heap-allocated mount list records; the same body shape as the other scalar deleting destructors in this range. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |
| `00b06d70` | `HandleDismountInput` |  | Checks local dismount eligibility and requests the dismount operation when the predicate is false. | [[Native Map/Shards/S0333 00b052f0-00b0a290|S0333]] |
| `00bdc330` | `SendPendingMountSpawn` |  | Flushes a queued mount spawn request, used after an item change resolves a mount; the pending id is stored by 00bdc060 and 00bdc420 through FUN_00b047f0. | [[Native Map/Shards/S0374 00bd9880-00be0090|S0374]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0095a710` | `— ⚠reject` |  | Finds the closest eligible pending mount entry to a supplied four-float point. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00b04830` | `— ⚠reject` |  | Member cleanup of the 0x40-byte mount list record whose ctor FUN_00b04980 installs the same IString at +0x10; the node removal FUN_00b04b70 calls it before freeing the node. | [[Native Map/Shards/S0332 00b006d0-00b05260|S0332]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bdc330` | `SendPendingMountSpawn` | send | PacketMountSpawn |  | Hypothesis: a server might process the mount-spawn request and report a resulting mount. This depends on the PacketMountSpawn send path; server validation and response are not established. |
