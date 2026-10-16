---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Game Rules and State

Part of [[Build Guides]] › Gameplay. Related: [[Command System]], [[Client Core]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

1607 accepted functions: 677 confirmed, 467 likely, 279 speculative, 184 unknown. Network-linked (`wire`): 33.

Subsystems: `game.statemachine` (468), `game.ruleset` (369), `game.factory` (250), `game.client` (86), `game.requirement` (61), `game` (56), `game.value` (52), `game.time` (43), `game.player_event` (32), `game.instance` (22), `game.interaction` (15), `game.combat` (14), `game.inspect` (14), `game.account` (14), `game.garage` (11), `game.equipment` (11), `engine.player_event` (10), `game.pirates` (10), `game.wall_of_data` (9), `game.fte` (8), `game.items` (8), `net.player_update` (7), `game.objective` (7), `game.claim` (6), `game.poll` (5), `game.sort` (5), `game.titles` (3), `game.firsttimeevent` (2), `game.social` (2), `game.archetype` (2), `game.group` (1), `net.interaction` (1), `game.npc` (1), `game.pets` (1), `game.observer` (1)

_Large or library guide: only confirmed and likely functions are listed (1144 of 1607)._

### `engine.player_event`

**ActorEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093f630` | ActorEventHandler::HandleEvent | confirmed | For param_8==8 and a nonnull actor, builds a stack Target using actor+0x268 and sends param_5/param_6 through direct helper 008a90c0. It then forwards through … |

**EncounterCompleteEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00aa1e40` | EncounterCompleteEvent::EncounterCompleteEvent | confirmed | Constructor taking (a, byte, b, c): calls the GameEvent base constructor FUN_007e78e0, installs EncounterCompleteEvent::vftable at +0, constructs two member ob… |
| `00aa1f10` | EncounterCompleteEvent::~EncounterCompleteEvent | confirmed | Destructor body: rewrites the EncounterCompleteEvent vtable at +0, destroys the two member objects through FUN_0103d010 (called twice, second under the SEH sta… |
| `00aa1f90` | EncounterCompleteEvent::Apply | confirmed | Virtual slot 1 override (the GameEvent base slot returns 0 without doing anything): when bit 0 of its flag argument is set it calls FUN_00769710 to obtain the … |
| `00aa1fd0` | EncounterCompleteEvent::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor: calls the destructor body FUN_00aa1f10 and, when bit 0 of the flags argument is set, calls _free on the object; returns the ob… |

**PlayerEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5510` | PlayerEvent::~PlayerEvent | confirmed | Destructor body restores PlayerEvent::vftable, then calls base event teardown FUN_007e7980 with this in ECX inside an SEH frame; twenty EH funclets call it. |
| `008a5570` | PlayerEvent::ScalarDeletingDestructor | confirmed | Scalar deleting destructor restores PlayerEvent::vftable, calls FUN_007e7980 with this in ECX, then calls _free(this) when param_2 bit 0 is set. |

**PlayerEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0093f1c0` | PlayerEventHandler::HandleEvent | confirmed | Branches on masks 0x8, 0x10, 0x80, 0x400, and 0x800; creates or updates Target records, compares target keys, and calls player-event and target callbacks. |

**PlayerRequestHeadShotEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf6840` | PlayerRequestHeadShotEvent::DtorBody | likely | Stores PlayerRequestHeadShotEvent::vftable into the object first, then calls 007e7980. The body has no member cleanup of its own. |

### `game`

**CTP::Game** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01353fe0` | OnGameEndDraw | confirmed | Calls 013508b0 with this and the text GAME ENDS IN A DRAW, then tail-jumps to virtual slot 116 (offset 0x1d0) of this with ECX set to this (bytes: push, call, … |
| `01354000` | OnGameEndAdjourned | confirmed | Calls 013508b0 with this and the text GAME ENDS IN ADJOURNMENT, then tail-jumps to virtual slot 116 (offset 0x1d0) with ECX set to this. |
| `013563c0` | ValidActionRepeatCheck | confirmed | Looks up param_2 in the tree at this+0x2c8 (01354b10, nil flag +0x1d); when the entry has its counter at +0x18 above 5, with matching fields +0x10 == param_3 a… |
| `013566e0` | OnGameLostByPlayer | confirmed | Calls 01353b80 with param_2 (the player id); when the result is non-zero (the id is not in that list) it logs Game::gameLostByPlayer: received when player was … |
| `01353b00` | ForEachEntryCallVtbl13 | likely | Asserts the array bounds at this+0x28 and this+0x2c, then for each dword entry from +0x28 up to +0x2c calls 01344ef0, and invokes virtual slot 13 (offset 0x34)… |

**EffectTag** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009dc030` | EffectTag::EffectTag | confirmed | Copy constructor for EffectTag: stores EffectTag::vftable and copies the source fields at param_2+8..+0x48, +0x50..+0x5c and the three bytes at +0x60..+0x62 in… |
| `009dc0d0` | EffectTag::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the destructor body FUN_0103da90 and frees the object when (param_2 and 1) is set. Slot 0 of EffectTag::vftable (dword at 0x0… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01030030` | RecordDtorBody44 | confirmed | Fastcall destructor body (ECX = record): stores the SoeUtil::IString<char> vftable at +0x44, and when the length at +0x50 is positive decrements the reference … |
| `01030160` | RecordCopyCtor44 | confirmed | Thiscall (ECX = destination record, param_2 = source record): copies the first 17 dwords (offsets 0x0 to 0x40) from the source, stores the IString vftable at +… |
| `01354310` | PushOutOfSync | confirmed | Logs Pushing OUT OF SYNC. through 013508b0 and writes #Pushing OUT OF SYNC to the game log via 013541f0; allocates 0x24 bytes; when the allocation succeeds it … |
| `013543a0` | SetOutOfSync | confirmed | Logs SET OUT OF SYNC through 013508b0 and writes #SET OUT OF SYNC via 013541f0; stores param_2 (byte) at this+0x14c; allocates 0x14 bytes, calls 012f9eb0 when … |
| `01354450` | LogLostPlayer | confirmed | Calls 013508b0 with this and the text LostPlayer #%d and the stack argument (cdecl pushes, add esp 0xc), then returns with ret 4. Virtual slot 111 shared by CW… |
| `01354470` | ReportGameError | confirmed | Asserts param_2 (exceptObj) is non-null at line 0x1d00; stamps the time with __time64 and _wctime64; builds three error lines from the match id (this+0x64) and… |
| `01356830` | VectorContainsIdVirtual126 | confirmed | Returns false for a null param_2; otherwise converts param_2 with 01340370 and scans the dword vector between this+0x2dc and this+0x2e0 for it, returning wheth… |
| `009dd830` | EffectTag_CopyCtorWrapper | likely | Thin wrapper that calls the EffectTag copy constructor FUN_009dc030 for the destination/argument it receives and returns the destination. |
| `00c7f280` | StringRecord::CopyConstruct | likely | Copies two leading fields, initializes an embedded IString from the source, then copies a byte and two trailing 32-bit fields. |
| `0102dd10` | Record::DeletingDtor | likely | Scalar deleting destructor (thiscall, flag byte): calls the shared record member destructor 008d1550 on this, then calls _free(this) when bit 0 of the flag is … |
| `0102f9e0` | GroupStateDtorBody | likely | Fastcall destructor body (ECX = group state object): drains the GroupMember list at +0x4 through 008faac0, drains the Invitation list head at +0x6f0 through 01… |
| `01030410` | RowTableReloadCaretText | likely | Thiscall (ECX = row table manager, param_2 = text blob object with data pointer at +0x4 and length at +0x8, param_3 = filter flag): first removes every row (00… |
| `0105fef0` | CopyStateFromSource | likely | Thiscall (this in EDI, its list object at this+4) with one stack argument, the source object. Clears the list at this+4 (00a020e0), appends a copy of each node… |
| `0105ff90` | SetFieldsAndListValue | likely | Thiscall with seven stack arguments (ret 0x1c). Clears the list at this+4 (00a020e0), pushes one 8-byte value made from the first two stack arguments into it (… |
| `01356120` | DwordVectorContainsId | likely | Asserts param_2 non-null (Game.cpp line 0x6e8 path), converts it with 01340370 to an id, scans the dword vector between this+0x220 and this+0x224 for that id, … |
| `013561c0` | DwordVectorEraseId | likely | Asserts param_2 non-null (Game.cpp line 0x6f4 path), converts it with 01340370, finds the matching dword in the vector between this+0x220 and this+0x224, and w… |
| `013568c0` | DwordVectorEraseIdAt2dc | likely | When param_2 is non-null, converts it with 01340370, finds the matching dword in the vector between this+0x2dc and this+0x2e0, and when found shifts later dwor… |

### `game.account`

**Account** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a90e0` | Account::DtorBody | confirmed | Fastcall (ECX=Account). Stores Account::vftable into the object; releases the owned object at +0x50 through its vtable slot 0 with argument 1 when non-null (mP… |
| `013a9a20` | Account::ScalarDeletingDtor | confirmed | Thiscall (ECX=Account, arg=flags byte). Calls the body 013a90e0 (Account::DtorBody); when bit 0 of the flags is set it frees the object with _free (00d82463); … |
| `013a35c0` | Account::ReattachPropertySet | likely | Thiscall (ECX = account; arg = property set). Asserts through 012f5a60 that the account's mPropertySet (+0x50) is non-null (Account.cpp line 0x138) and that th… |
| `013a3620` | Account::GetPropertySet | likely | Fastcall (ECX = account). Asserts through 012f5a60 that the mPropertySet dword at +0x50 is non-null (Account.cpp line 0x155) and returns it. |
| `013a3690` | Account::AttachPropertySet | likely | Thiscall (ECX = account; arg = property set). Asserts that mPropertySet (+0x50) is non-null (Account.cpp line 0x161) and that the argument is non-null (line 0x… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a7780` | ClassifyRoleTier | likely | Fastcall (ECX=owner). Does nothing unless owner+0x5c is 0 and owner+0x50 is non-zero. Reads ValueData key 0xfc5 (013111d0, type 7 required) and then tests, in … |
| `013a92f0` | RebuildBadgeTierCounts | likely | Fastcall (ECX=owner). Clears the tree at owner+0x74 (013a5330 on its root, header reset). Calls the owner's vtable slot 1 getter with key 0x103f (type 8 list) … |

### `game.claim`

**ClaimItem** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01060b90` | ClaimItem::ClaimItem | confirmed | Thiscall with eight parameters (this plus seven stack arguments). Stores Loggable::vftable, then ClaimItem::vftable at +0. Constructs the id list member at +4 … |
| `01060c60` | ClaimItem::ClaimItem | confirmed | Fastcall with the object in ECX (default constructor, no stack arguments). Stores Loggable::vftable, then ClaimItem::vftable at +0. Constructs the id list at +… |
| `01060d10` | ClaimItem::ClaimItem | confirmed | Thiscall with one stack argument, a source ClaimItem (ret 4). Same base sequence as 01060c60: Loggable then ClaimItem vtables, the id list at +4 (01060090), th… |
| `01060db0` | ClaimItem::~ClaimItem | confirmed | Fastcall with the object in ECX. Stores ClaimItem::vftable at +0 and Loggable::vftable at +0x158, runs the Array<GameItemEntry,8,1> destructor body 0105f910 on… |
| `01060e40` | ClaimItem_ScalarDeletingDtor | confirmed | Thiscall with a flag byte (ret 4). Calls the ClaimItem destructor body 01060db0 on this, then when bit 0 of the flag is set calls _free on this. Returns this. |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c205c0` | GuardedClaimManager_ClaimItemEquip | likely | Requires one argument and a true indexed guard, resolves TooltipData, then passes the argument to 00a75790, which resets claim-manager fields and calls 00a04f4… |

### `game.client`

**BaseClient** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008f0fe0` | BaseClient::InitializeEnvironment | confirmed | When initialization is permitted, creates or replaces the client environment object and updates the logging service; otherwise formats a duplicate-initializati… |
| `008f12d0` | BaseClient::OnProviderStatus | confirmed | Formats the provider name and status into a log/UI message, invokes a client callback, then forwards provider, status, and share-enabled values to 00a72020. |
| `008f13c0` | BaseClient::OnPublishStatus | confirmed | Formats the provider and publish status, selects Share Successful only for status one and Share Failed otherwise, displays that result, and invokes the client … |
| `008f14c0` | BaseClient::OnBrowseRequired | confirmed | Formats the browse URL and ID for logging, invokes the client callback, and forwards the ID and URL to 00a720a0. |

**CTP::Game** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013673e0` | CTP::Game::ClearOwnedNodes | likely | Virtual method in slot 91 of the CTP::Game vtable (also listed in CWGame and CWMainController). Walks the list at this+0x10c; for each inner bucket entry that … |

**CWAIController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01404240` | CWAIController::CWAIController | confirmed | Constructor: calls 014240f0 on the object, stores CWAIController::vftable (018e4048), zeroes the dwords at +0x8, +0xc, +0x10, +0x18, +0x1c, +0x28, sets +0x20 t… |
| `014042c0` | CWAIController::DtorBody | confirmed | Non-deleting destructor body: stores CWAIController::vftable, and if the pointer at +0x18 is non-null calls its vtable slot [0] with 1 (scalar delete) and clea… |
| `014044f0` | CWAIController::~CWAIController | confirmed | Scalar deleting destructor: calls 014042c0 (CWAIController::DtorBody) on the object, frees it through _free when bit 0 of the stack byte is set; returns this. |

**CWPlayerController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01402d30` | CWPlayerController::DtorBody | confirmed | Non-deleting destructor body: stores CWPlayerController::vftable, and when the pointer at +0x30 is non-null calls its vtable slot [0] with 1 (scalar delete) an… |
| `01402df0` | CWPlayerController::~CWPlayerController | confirmed | Scalar deleting destructor: calls 01402d30 (CWPlayerController::DtorBody) and frees the object through _free when bit 0 of the stack byte is set; returns this. |

**ClientAccountManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0070c8b0` | ClientAccountManagerSetState | confirmed | If the state changes, saves old state at +0x22274, writes the new state at +0x22270, and conditionally formats and logs the transition. |
| `00715e30` | ClientAccountManager::RemoveTrackedActorNode | likely | For a non-null node, passes the value at node+0x04 to 00985550; scans the chain headed at this+0x22290 via node+0x62b0 and unlinks a match with 00715d80; then … |

**ClientLoginManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00957530` | ClientLoginManager::BuildAccountCharacterDetails | likely | Calls 00769710 with no stack arguments to obtain the manager service; when its service pointer is present, initializes a temporary ten-slot AccountInfoCharacte… |

**ClientPcData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a1d6d0` | ClientPcData::Dtor | likely | Installs ClientPcData vftable and destroys profile, effect tags, character stats, lists, guild/name state, event queues, references, and embedded managers. |
| `00a1db70` | ClientPcData::GetField_0x46c | likely | Returns the dword stored at this+0x46c. |

**GameProcessor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ce1cb0` | GameProcessor::GetByte8 | likely | Virtual slot [4] (no stack arguments): returns the byte at this+8 (mov al,[ecx+8]; ret). |
| `00ce1e10` | GameProcessor::ReleaseOwnedMember | likely | Virtual slot [3] (no stack arguments): if vtbl+0x10 of this returns nonzero, calls FUN_010c2170, calls FUN_010c2610 and _free on the object at [this+0x14] when… |
| `00ce2810` | GameProcessor::LoadPreloadForKey | likely | Virtual slot [2] (two stack arguments): when the virtual slot-4 flag is clear it stores param_2 at this+0xc and param_3 at this+0x10, looks up a node by param_… |

**MoveAndSelectTargetControllerHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00934870` | MoveAndSelectTargetControllerHandler::SelectPlayer 🌐 | likely | Reads the candidate target and its position. When both target guards pass, constructs CommandPacketSelectPlayer and sends it only if the gateway pointer at thi… |

**PlayerEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b8160` | PlayerEvent::PlayerEvent | confirmed | Calls the base event constructor, installs PlayerEvent::vftable, and stores param_3 and param_4 at fields +0x48 and +0x4c. |

**PlayerMotionEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b82a0` | PlayerMotionEvent::PlayerMotionEvent | confirmed | Calls PlayerEvent base construction, installs PlayerMotionEvent::vftable, and stores the supplied dword at offset +0x50. |
| `008b8330` | PlayerMotionEvent::DtorBody | confirmed | Restores PlayerMotionEvent then PlayerEvent vtables, calls FUN_007e7980 with param_1 in ECX, restores ExceptionList, and returns. |
| `008b8400` | PlayerMotionEvent::ScalarDeletingDtor | confirmed | Calls PlayerMotionEvent cleanup and frees the object when the deleting flag bit is set. |

**ProfileManager<ClientPcProfile>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a197e0` | ClientPcProfileManager_Ctor | likely | Installs the ProfileManager vftable, zeros its linked-list and cache fields, constructs the ProfileTypeEntry hash list, and clears two trailing fields. |
| `00a19870` | ClientPcProfileManager_Dtor | likely | Restores the manager vftable, removes and frees list-head nodes, clears a field, and destroys its ProfileTypeEntry map. |
| `00a19bf0` | ClientPcProfileManager_ScalarDeletingDtor | likely | Calls the ProfileManager teardown body, conditionally frees this, and returns this when the deleting flag is clear. |

**(no class)** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0070c9f0` | HandleNextPrevBackdoorChord | confirmed | Checks the UI object's virtual input predicate; on true calls 0070c8b0 with state 6 and the chord message, then clears +0x53a50; on false calls it with state 0… |
| `00936d00` | WaitForWorldReady 🌐 | confirmed | Runs the world-ready polling loop, checks zone, actor, sky and minigame-processor readiness, reports stalls or disconnects, then finalizes loading, logs the ne… |
| `009386f0` | LoadZone | confirmed | Resets zone and player-position state and applies the destination transform. If this+0x8f is zero, calls WaitForWorldReady(1); otherwise loads Sky.xml through … |
| `0093b730` | ReportPlayerRequest 🌐 | confirmed | On successful player lookup, constructs ReportPacketReportPlayer, prepares up to ten report entries, conditionally serializes and sends through PacketTunneledC… |
| `0070c7d0` | LogAcceptCurrentDummy | likely | Only when state equals 0x13 sets a UI byte, formats and logs the current dummy index, then releases the temporary string; it does not submit or create a charac… |
| `0093af90` | UpdatePositionSnapshot | likely | Checks state and input masks, compares current and stored four-float vectors, updates position snapshots and timestamps, and calls movement callbacks. |
| `0093b340` | ProcessPositionInput | likely | Checks client state and input conditions, compares coordinate vectors, updates coordinates and flags, and invokes state callbacks. |
| `0093bba0` | HandleInputEvent | likely | Checks client, target, activity, and modifier state; calls the handler with coordinates and event kind; kind 8 additionally emits click-move and invokes a gate… |
| `0096aed0` | RefreshPlayerInfoAndInventoryData | likely | Thiscall. Stores param_2 at this+0x194. When the object at this+0x738 links back to this (+0xc8), it looks up the BaseClient.PlayerInfo entry through 00968840 … |
| `00a13df0` | ClientMap_FindNodeByKey | likely | Walks the client's keyed table at this+0x4a8 (5 buckets chosen by param_2 % 5, chain through node+0xd4) and returns the node whose key field at +0xd8 equals pa… |
| `00a14040` | ClientMap_FindNodeAndStoreField14 | likely | Finds the node with key param_2 in the client table at this+0x4a8 (5 buckets, chain +0xd4, key +0xd8) and writes param_3 into the dword at node+0x14, leaving t… |
| `00a14090` | ClientMap_NodeEligibleNoLimit | likely | Returns 1 only when the client table barrier at this+0x464 is zero, the node for key param_2 exists with +0x3d non-zero and +0x48 zero, and either this+0x490 i… |
| `00a140e0` | ClientMap_NodeEligibleWithBudget | likely | Looks up key param_2 in the client table at this+0x4a8 and returns 1 when the node exists with +0x3c non-zero, +0x48 zero, the +0x490/+0x4c gate satisfied and … |
| `00a14170` | ClientMap_LookupNestedThenGetValue | likely | Finds the node for key param_3 in the client table at this+0x4a8, then performs a second lookup by param_2 inside that node's own 4-bucket sub-table at node+0x… |
| `00a14560` | ClientList_FindArrayElementByID | likely | Scans the list at this+0x49c (next at +0xe0) and, for each element, walks the array described by element+0xd0 (data pointer in slot 0, count in slot 1, stride … |
| `00a147f0` | ClientList_DestroyAllNodes | likely | Walks the list at this+4/+8 (next link index 0xe9 into node+0x3a4, previous index 0xe8 at node+0x3a0) and for every element calls FUN_00a11c70 (bucket unlink f… |
| `00a14b50` | ClientMap_RemoveHeadAndClearCache | likely | Twenty-byte routine (bytes 568bf18d4e04e8f5ecffffc74670000000005ec3): calls the head-removal-and-free routine 00a13850 with this+4 as the list header and then … |
| `00a153f0` | ClientMap_NodeEligibleComposite | likely | Returns 1 only when the client barrier this+0x464 is zero, the node for param_2 exists with +0x3d non-zero, +0x48 zero and the +0x490/+0x4c gate satisfied, and… |
| `00a154d0` | ClientMap_NestedLookupThenTrack | likely | Finds the node for param_3 in the client table at this+0x4a8, looks param_2 up inside that node's inner 4-bucket map through FUN_00ae19c0 with the node as the … |
| `00a15530` | ClientMap_EntryMatchesCurrentOwner | likely | Resolves an id through FUN_00b0d810, reads its +0x30 field, gates on FUN_00a12580 and then on 00a154d0(rhs) returning an inner record whose +0x10 equals this+0… |
| `00a16670` | KeyedMap_InsertNodeAfter | likely | Inserts an element relative to an existing one in the same 5-bucket keyed table: first links the node into the ordered list through 00a138e0 (front when the re… |
| `00a19f40` | FindCachedKeyOrResolve | likely | Uses bucket this+0x35c2c+(key&0x1f)*4, walks nodes comparing key at +0x0c and following +0x10, returns the first dword of an exact node, or calls 00a19290 and … |
| `01404480` | CWCommandPairRelation | likely | Pure predicate on two classIDs (arg1, arg3) and two more values (arg2, arg4): returns 1 when (arg1, arg3) is one of the pairs 0x222e6 to 0x222e7, 0x222e7 to 0x… |

### `game.combat`

**CombatProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a29b40` | CombatProcessor::~CombatProcessor | likely | Stores CombatProcessor::vftable at +0 and returns. |
| `00a2a120` | CombatProcessor::CombatProcessor | likely | Stores CombatProcessor::vftable at +0 and the three constructor arguments at +8, +0xc and +0x10, then copies two time-source globals (DAT_01be6820/DAT_01be6824… |

**PlayerHitpointDeltaEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b81e0` | PlayerHitpointDeltaEvent::PlayerHitpointDeltaEvent | confirmed | Calls PlayerEvent base construction, installs PlayerHitpointDeltaEvent::vftable, and stores one dword, one byte, and two dwords at offsets +0x50 through +0x5c. |
| `008b8280` | PlayerHitpointDeltaEvent::ScalarDeletingDtor | confirmed | Calls the derived cleanup routine and frees the object when the deleting flag bit is set. |

**ProjectileParameters** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eb0a0` | ProjectileParametersCopyCtor | confirmed | Copies projectile scalars, initializes two Target members, copies two strings and remaining parameter fields. |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009058d0` | SetInWorldCombatFlag | confirmed | Thiscall. If param_2 differs from this+0x80: stores it there, computes the combined flag old=(this+0x80 && this+0x81) and new=(param_2 && this+0x81). When old … |
| `00905a50` | SetCombatIndicatorFlag | confirmed | Thiscall. Compares the combined flag (this+0x80 && this+0x81) before and after storing param_2 at this+0x81. If the combined state is unchanged and this+0x80 i… |
| `008c6470` | CombatTarget_EngageStep | likely | Walks the list at (this+500)+0xe18 until a node has field +0x10 equal to -0x76599480, then overwrites that loop value with FUN_00d83c90(). When the state gate … |
| `00969590` | AppendPendingRecordWithParams | likely | Thiscall with ECX = owner and 13 stack arguments. If param_8 is zero it asks the owner predicate (vtable +0x24) and 0076c590; otherwise, it takes a sequence nu… |
| `00a2a290` | IsStoredTimeAfterNow | likely | Compares the 64-bit value at this+0x18/+0x1c against the current value from FUN_007027f0 and returns 1 only when the stored value is strictly greater (high dwo… |
| `00a2a6c0` | StoreTimestampFromSource | likely | Walks the record list hanging off *(*(this+8)+500)+0x4c4 looking for a node whose +0x10 tag is -0x2dcd06d5 (the walk itself has no effect), then writes the cur… |
| `00a2e7d0` | CopyIdPairOrDefault | likely | Writes an 8-byte id pair to the output pointer: when the object at this+0x78 exists it copies that object's +0x18/+0x1c pair, otherwise it writes DAT_01be6870/… |

### `game.equipment`

**SoeUtil::HashList<EquipmentGroupItemDefinition,128,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010399a0` | HashList_EquipmentGroupItemDefinition_Construct | confirmed | Constructor: stores the HashListMap vftable at *this, zeroes +0x10, +8, +0xc, memsets 0x200 bytes at +0x14, zeroes +4, then stores the HashList<EquipmentGroupI… |
| `01039a20` | HashList_EquipmentGroupItemDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): calls 010398a0 on this, then _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeUtil::HashListMap<int,EquipmentGroupItemDefinition,128,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01039780` | HashListMap_EquipmentGroupItemDefinition_DtorBody | confirmed | Destructor body without a free: stores the HashListMap vftable at *this, then while the head at this+8 is non-null calls 008cc000 with ECX=this and the head as… |
| `01039860` | HashListMap_EquipmentGroupItemDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): stores the HashListMap vftable at *this, removes head nodes with 008cc000 until this+8 is null, then calls _free(this) only… |
| `010398a0` | HashListMap_EquipmentGroupItemDefinition_DtorBodySEH | confirmed | Destructor body inside an SEH frame: stores the HashListMap vftable at *this, removes head nodes with 008cc000 until this+8 is null; no free and no return valu… |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00797a70` | EquipmentSlotDefinitionParse | confirmed | Thiscall (ECX = record, param_2 = source text). Measures the text length, then reads its fields in order through 0072d900 and 0072d550 (token readers into the … |
| `00798490` | EquipmentSlotDefinitionFind | confirmed | Returns 0 for a NULL or empty name argument. Otherwise hashes the name with 00705bc0, looks the hash up with 00796d30, and when that lookup returns 0 formats a… |
| `007989a0` | EquipmentSlotDefinitionLoadData | confirmed | Thiscall, one stack argument. Builds the data path by assigning the argument to an IString (00402ce0) and appending EquipmentSlotDefinitions.txt (00402770), op… |
| `00797d70` | EquipmentSlotDefinitionCtor | likely | Fastcall (ECX = this). Zeroes the scalar fields (dwords 0..0xe, flag bytes at +0x24, +0x25, +0x26, +0x9c and +0x9d, dword at +0xa0 set to 99999999). Installs t… |
| `00798940` | EquipmentSlotDefinitionId | likely | One stack argument (ret 4). Calls 00798490 with it; returns the dword at the found record's +0x20 (slot id) or 0 when the lookup returns 0. |

### `game.factory`

**FactoryDefinitionManager** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01035d50` | FactoryDefinitionManager::ClearAllLists | confirmed | Virtual slot [2] (fastcall ECX = manager): drains six node lists by calling the node removers: list at +0xc with 010347e0, +0x1020 with 01034890, +0x3048 with … |
| `01038ff0` | FactoryDefinitionManager::~FactoryDefinitionManager | confirmed | Destructor body: stores FactoryDefinitionManager::vftable into *this, calls 01035d50 on this, calls 01038f00 on this+4 (table set teardown), then stores BaseDe… |
| `01039330` | FactoryDefinitionManager::FactoryDefinitionManager | confirmed | Constructor: stores BaseDefinitionManager::vftable, then FactoryDefinitionManager::vftable into *this, calls 01038e10 with ECX = this+4 (the table set), and re… |
| `01039390` | FactoryDefinitionManager::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (ret 4): calls 01038ff0 on this, then calls FreeMemory(this, 0) (00401c00) only when bit 0 of the flag argument is set; returns this… |
| `01038e10` | FactoryDefinitionManager_InitTableSet | likely | Constructs the table set at ECX (FactoryDefinitionManager passes this+4): calls the constructors 01038570, 010385f0, 01035fb0, 01036030, 010360b0, 01038670, 01… |
| `01038f00` | FactoryDefinitionManager_DestroyTableSet | likely | Destroys the table set at ECX in reverse construction order by calling the table destructor bodies 01037230, 01035ce0, 01035c70, 010371c0, 01035c00, 01037150, … |
| `01039070` | FactoryDefinitionManager::LoadDefinitionFiles | likely | Virtual slot 3 of FactoryDefinitionManager::vftable: forwards its two stack arguments (param_1, param_2) to eight routines in this order: 010372a0, 010374e0, 0… |

**FactoryPlot** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01059480` | FactoryPlot_RemoveFoundation | confirmed | If param_2 is nonzero, copies the 64-bit key at param_2+0xf0/+0xf4 into a local, looks it up with 01057b60, and when found copies that node's key words, remove… |
| `01059780` | FactoryPlot_AddFoundation | confirmed | Calls 009e0920 with a pointer to param_2 to obtain a node; if the node is nonzero stores param_4 at [node], param_2 at node+0xf0 and param_3 at node+0xf4, and … |

**FactoryProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e42a0` | FactoryProcessor::Ctor | confirmed | Installs processor vtable, initializes state and tooltip record, stores context, creates/retains two helpers and clears flags. |

**SoeUtil::HashList<FactoryBlueprintDefinition*,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010360b0` | HashList_FactoryBlueprintDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryBlueprintDefinition*,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket table a… |
| `010362f0` | HashList_FactoryBlueprintDefinitionPtr_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035b90 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashList<FactoryBlueprintDefinition,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01036030` | HashList_FactoryBlueprintDefinition_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryBlueprintDefinition,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket table at… |
| `010362d0` | HashList_FactoryBlueprintDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035b20 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashList<FactoryFoundationDefinition*,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01035fb0` | HashList_FactoryFoundationDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryFoundationDefinition*,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket table … |
| `010362b0` | HashList_FactoryFoundationDefinitionPtr_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035ab0 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashList<FactoryMisfortuneDefinition,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010361b0` | HashList_FactoryMisfortuneDefinition_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryMisfortuneDefinition,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket table a… |
| `01036330` | HashList_FactoryMisfortuneDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035c70 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashList<FactoryMisfortuneGroupDefinition,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01036230` | HashList_FactoryMisfortuneGroupDefinition_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryMisfortuneGroupDefinition,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket ta… |
| `01036350` | HashList_FactoryMisfortuneGroupDefinition_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035ce0 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashList<FactoryPlotDefinition*,1024,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01036130` | HashList_FactoryPlotDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): stores the HashListMap<FactoryPlotDefinition*,1024,-1> vtable, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket table at +0x… |
| `01036310` | HashList_FactoryPlotDefinitionPtr_ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = object, flag argument): calls the HashListMap destructor body 01035c00 on this, calls _free(this) when bit 0 of the flag is s… |

**SoeUtil::HashListMap<int,FactoryBlueprintDefinition*,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031670` | HashListMap_FactoryBlueprintDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryBlueprintDefinition*,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bu… |
| `01032cd0` | HashListMap_FactoryBlueprintDefinitionPtr_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032d10` | HashListMap_FactoryBlueprintDefinitionPtr_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01034f50` | HashListMap_FactoryBlueprintDefinitionPtr_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryBlueprintDefinition*,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, callin… |
| `010350d0` | HashListMap_FactoryBlueprintDefinitionPtr_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 010334e0, calls _free(this) when bit 0 of the flag is se… |
| `01035b90` | HashListMap_FactoryBlueprintDefinitionPtr_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 010334e0 on the head; call… |

**SoeUtil::HashListMap<int,FactoryBlueprintDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031600` | HashListMap_FactoryBlueprintDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryBlueprintDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry buc… |
| `01032c70` | HashListMap_FactoryBlueprintDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032cb0` | HashListMap_FactoryBlueprintDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01034f20` | HashListMap_FactoryBlueprintDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryBlueprintDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calling… |
| `01035090` | HashListMap_FactoryBlueprintDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034550, calls _free(this) when bit 0 of the flag is se… |
| `01035b20` | HashListMap_FactoryBlueprintDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034550 on the head; call… |

**SoeUtil::HashListMap<int,FactoryBlueprintRankDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010316d0` | HashListMap_FactoryBlueprintRankDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryBlueprintRankDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry… |
| `01032d60` | HashListMap_FactoryBlueprintRankDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032da0` | HashListMap_FactoryBlueprintRankDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `010365e0` | HashListMap_FactoryBlueprintRankDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryBlueprintRankDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, cal… |
| `01036cc0` | HashListMap_FactoryBlueprintRankDefinition_ScalarDeletingDt… | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034960, calls _free(this) when bit 0 of the flag is se… |
| `01037150` | HashListMap_FactoryBlueprintRankDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034960 on the head; call… |

**SoeUtil::HashListMap<int,FactoryFoundationDefinition*,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010315a0` | HashListMap_FactoryFoundationDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryFoundationDefinition*,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry b… |
| `01032c00` | HashListMap_FactoryFoundationDefinitionPtr_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032c40` | HashListMap_FactoryFoundationDefinitionPtr_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01034ef0` | HashListMap_FactoryFoundationDefinitionPtr_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryFoundationDefinition*,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calli… |
| `01035050` | HashListMap_FactoryFoundationDefinitionPtr_ScalarDeletingDt… | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01033470, calls _free(this) when bit 0 of the flag is se… |
| `01035ab0` | HashListMap_FactoryFoundationDefinitionPtr_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01033470 on the head; call… |

**SoeUtil::HashListMap<int,FactoryFoundationDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031540` | HashListMap_FactoryFoundationDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryFoundationDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bu… |
| `01032ba0` | HashListMap_FactoryFoundationDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032be0` | HashListMap_FactoryFoundationDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01036450` | HashListMap_FactoryFoundationDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryFoundationDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, callin… |
| `01036c80` | HashListMap_FactoryFoundationDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034890, calls _free(this) when bit 0 of the flag is se… |
| `010370e0` | HashListMap_FactoryFoundationDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034890 on the head; call… |

**SoeUtil::HashListMap<int,FactoryMisfortuneDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010317f0` | HashListMap_FactoryMisfortuneDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryMisfortuneDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bu… |
| `01032ee0` | HashListMap_FactoryMisfortuneDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032f20` | HashListMap_FactoryMisfortuneDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01034fd0` | HashListMap_FactoryMisfortuneDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryMisfortuneDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, callin… |
| `01035150` | HashListMap_FactoryMisfortuneDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034600, calls _free(this) when bit 0 of the flag is se… |
| `01035c70` | HashListMap_FactoryMisfortuneDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034600 on the head; call… |

**SoeUtil::HashListMap<int,FactoryMisfortuneGroupDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031850` | HashListMap_FactoryMisfortuneGroupDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryMisfortuneGroupDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-ent… |
| `01032f70` | HashListMap_FactoryMisfortuneGroupDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032fb0` | HashListMap_FactoryMisfortuneGroupDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01035000` | HashListMap_FactoryMisfortuneGroupDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryMisfortuneGroupDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, c… |
| `01035190` | HashListMap_FactoryMisfortuneGroupDefinition_ScalarDeleting… | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 010346b0, calls _free(this) when bit 0 of the flag is se… |
| `01035ce0` | HashListMap_FactoryMisfortuneGroupDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 010346b0 on the head; call… |

**SoeUtil::HashListMap<int,FactoryPlotDefinition*,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031730` | HashListMap_FactoryPlotDefinitionPtr_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryPlotDefinition*,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket … |
| `01032df0` | HashListMap_FactoryPlotDefinitionPtr_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032e30` | HashListMap_FactoryPlotDefinitionPtr_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01034fa0` | HashListMap_FactoryPlotDefinitionPtr_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryPlotDefinition*,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calling cal… |
| `01035110` | HashListMap_FactoryPlotDefinitionPtr_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01033280, calls _free(this) when bit 0 of the flag is se… |
| `01035c00` | HashListMap_FactoryPlotDefinitionPtr_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01033280 on the head; call… |

**SoeUtil::HashListMap<int,FactoryPlotDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010314e0` | HashListMap_FactoryPlotDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryPlotDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket t… |
| `01032b40` | HashListMap_FactoryPlotDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if that is non-null calls its vtable slot 1, else if DAT_01b… |
| `01032b80` | HashListMap_FactoryPlotDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 0x10. |
| `01036370` | HashListMap_FactoryPlotDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryPlotDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calling call… |
| `01036c40` | HashListMap_FactoryPlotDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 010347e0, calls _free(this) when bit 0 of the flag is se… |
| `01037070` | HashListMap_FactoryPlotDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 010347e0 on the head; call… |

**SoeUtil::HashListMap<int,FactoryRankDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01031790` | HashListMap_FactoryRankDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryRankDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket t… |
| `01032e80` | HashListMap_FactoryRankDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01032ec0` | HashListMap_FactoryRankDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `010366c0` | HashListMap_FactoryRankDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryRankDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calling call… |
| `01036d00` | HashListMap_FactoryRankDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034b50, calls _free(this) when bit 0 of the flag is se… |
| `010371c0` | HashListMap_FactoryRankDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034b50 on the head; call… |

**SoeUtil::HashListMap<int,FactoryToolDefinition,1024,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010318b0` | HashListMap_FactoryToolDefinition_Construct | confirmed | Fastcall ctor (ECX = this): installs the HashListMap<int,FactoryToolDefinition,1024,-1> vtable at +0, zeroes +0x10, +0x8, +0xc, memsets the 1024-entry bucket t… |
| `01033010` | HashListMap_FactoryToolDefinition_AllocNode | confirmed | Virtual slot [1] (no arguments): calls 006fb9c0, then gets the thread memory manager via 006fb860; if non-null calls its vtable slot 1, else if DAT_01bdf280 is… |
| `01033050` | HashListMap_FactoryToolDefinition_FreeNode | confirmed | Virtual slot [2] (one stack argument, ret 4): calls FreeMemory 00401c00 on the pointer argument with alignment 4. |
| `01036950` | HashListMap_FactoryToolDefinition_UnwindDtor | confirmed | Fastcall (ECX = table): stores the HashListMap<int,FactoryToolDefinition,1024,-1> vtable at +0, then loops while the list head (+0x8) is non-null, calling call… |
| `01036d40` | HashListMap_FactoryToolDefinition_ScalarDeletingDtor | confirmed | Thiscall (ECX = table, flag argument): stores the vtable at +0, unlinks every list node through callee 01034a30, calls _free(this) when bit 0 of the flag is se… |
| `01037230` | HashListMap_FactoryToolDefinition_DtorBody | confirmed | Fastcall (ECX = table, cookie-bearing 102-byte body): stores the vtable at +0, loops while the list head is non-null, calling callee 01034a30 on the head; call… |

**(no class)** (115)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00991050` | ValidateFactoryToolsDataSource | confirmed | Checks the supplied object against GuiDataSource and FactoryToolsDataSource RTTI descriptors, returns the check result, and formats a diagnostic when a non-nul… |
| `009911b0` | FindFactoryToolsDataSourceByNameHash | confirmed | Hashes the supplied name with 00705bc0, probes the 1000-bucket table at this+0xaea4, and follows the selected chain until node+8 equals the hash. It then RTTI-… |
| `009dfb20` | List_Dtor_ItemRequestEntry | confirmed | Destructor body of SoeUtil::List<ItemRequestEntry,-1>: stores the list vtable and empties it through FUN_009df460. |
| `009dfb60` | HashListMap_Dtor_BlueprintDataSource32 | confirmed | Destructor body of SoeUtil::HashListMap<int,BlueprintDataSourceInfo,32,-1>: stores the container vtable and repeatedly removes the head node with FUN_009dc2f0 … |
| `009dfb90` | List_Dtor_ClientItemConstPtr | confirmed | Destructor body of SoeUtil::List<ClientItem_const*,-1>: stores the list vtable and empties it through FUN_009df170. |
| `009e1590` | DumpFactoryData | confirmed | Prints factory plot definition, instance, category, grid size, bounds, foundation count, identifiers, positions and blueprint status. |
| `009e1b60` | LogPlacementNodeAtPosition | confirmed | Finds plot containing location, reports absence, resolves placement node, extracts type/key and logs values. |
| `009e4dc0` | FactoryMisfortuneMap_scalarDeletingDtor | confirmed | Scalar deleting destructor for the 64-bucket FactoryMisfortuneGuid map: stores the map vtable, drains the node chain with FUN_009df6c0 and frees this through _… |
| `009e5040` | FactoryMisfortuneMap_copyCtor | confirmed | Copy constructor for the 64-bucket FactoryMisfortuneGuid hash map: stores the HashListMap vtable, zeroes the count/head/tail fields, memsets the 64 inline buck… |
| `009e5080` | PlacementNodeMap_copyCtor | confirmed | Copy constructor for the SoeUtil::HashListMap<__int64,FactoryPlot::PlacementNode,1024,-1>: stores the vtable, clears count/head/tail plus the 1024 inline bucke… |
| `009e8590` | FoundationMap_copyCtor | confirmed | Copy constructor for SoeUtil::HashListMap<StrongType<unsigned __int64,SoeUtilSignatureFactoryFoundationGuid,0>,FactoryFoundation,64,-1>: stores the vtable, zer… |
| `010372a0` | LoadFactoryPlots | confirmed | Reads FactoryPlots.txt from the data folder: builds the path (adds a separator via 006fc030 when needed, appends the file name via 00402ff0), reads the text wi… |
| `010374e0` | LoadFactoryFoundations | confirmed | Reads FactoryFoundations.txt from the data folder (same path and line walk as LoadFactoryPlots): for each accepted line parses the first '^'-delimited field as… |
| `01037730` | LoadFactoryBlueprints | confirmed | Reads FactoryBlueprints.txt from the data folder (same path and line walk): for each accepted line parses the key, inserts the row through 01036530, parses the… |
| `010379b0` | LoadFactoryBlueprintRanks | confirmed | Reads FactoryBlueprintRanks.txt from the data folder (same path and line walk): for each accepted line parses the first '^'-delimited field with 007065d0, inse… |
| `01037bd0` | LoadFactoryTools | confirmed | Reads FactoryTools.txt from the data folder (same path and line walk): for each accepted line parses the first '^'-delimited field with 007065d0, calls 01058f2… |
| `01037ea0` | LoadFactoryRanks | confirmed | Reads FactoryRanks.txt from the data folder (same path and line walk): for each accepted line parses the first '^'-delimited field with 007065d0, inserts the r… |
| `010380c0` | LoadFactoryMisfortunes | confirmed | Reads FactoryMisfortunes.txt from the data folder (same path and line walk): for each accepted line parses the first '^'-delimited field with 007065d0, inserts… |
| `01058700` | FactoryStateMapsDtorBody | confirmed | Owner teardown with an EH frame: calls 01058690 with ECX=this+0x2148, then stores the CharacterGuid to FactoryMisfortune* map vftable at this+0x1130 and unlink… |
| `01058860` | FactoryStateMapsCtor | confirmed | Stores two data words (DAT_01cb6ec8, DAT_01cb6ecc) at [this] and [this+4]; constructs a HashList<FactoryPlot,64,-1> at this+8 via 010584e0; sets the FactoryMis… |
| `01058d00` | InitHashMemberBlock | confirmed | Zeroes [this], [this+4], sets [this+8] and [this+0xc], [this+0x10], [this+0x14], [this+0x18], [this+0x1c] from data words, byte +0x20 = 0, +0x21 = 1, +0x22 = 0… |
| `01058e00` | InitHashMemberBlockDtorBody | confirmed | Destructor body for the block built by 01058d00: stores HashListMap<int,int,10,-1> vftable at +0x9c and calls 009df2f0 with ECX=this+0x9c; stores HashListMap<i… |
| `01059590` | FactoryPlotDataBlock_Ctor | confirmed | Zeroes [this] and [this+8]; writes the FactoryFoundation guid map vftable at +4 with zeroed +0xc, +0x10 and +0x14 and 0x100 bytes at +0x18; writes the misfortu… |
| `010596b0` | FactoryPlotDataBlock_DtorBody | confirmed | Destructor body with EH frame: stores the int64 placement-node map vftable at +0x250 and calls 009df380 with ECX=this+0x250; calls 009dd8f0 with ECX=this+0x238… |
| `009d9b40` | LookupValueByKey | likely | If the object at this+0x2270 has a non-null pointer at +0x21c, resolves the argument through FUN_010329e0 and returns the first dword of the result; otherwise … |
| `009d9b70` | SetHousingGridSpacingFromDefinition | likely | Looks up the owner-key entry with FUN_01057a20 called on the map member at this+0x10 (the first argument and the owner key *(*(this+0x2270)+0x21c) are pushed b… |
| `009d9bf0` | ForwardKeyToMember_23a4 | likely | When the pointer at this+0x23a4 is non-null it tail-jumps to FUN_00c64b50 with ECX = *(this+0x23a4) and the wrapper's single 4-byte stack argument pushed as th… |
| `009d9c60` | GetValueForOwnerKey | likely | Pushes *(*(this+0x2270)+0x21c) as the argument, sets ECX = this+0x10 (the map member) and returns FUN_01057e50's result: a value field of the entry found for t… |
| `009d9c80` | LookupEntryForKey | likely | Calls FUN_01057a20(argument, *(this+0x2270)+0x21c) and ignores the result, i.e. a fire-and-forget lookup/touch of the entry for the owner's current key. |
| `009db860` | HasEntriesInBothTables | likely | Returns 0 when the pair (param_2,param_1) equals the sentinel globals DAT_01be6010/DAT_01be6014; otherwise calls FUN_00984960 with ECX = *(*(this+0x2270)+0xc0)… |
| `009db8d0` | GetActiveNpcTooltipInfo_22c0 | likely | Returns this+0x22c0 (the active NPC tooltip info record, whose effect list hangs off record+0x1c) only when the key pair at this+0x22b8/this+0x22bc differs fro… |
| `009db970` | Wrapper_KeyPairMapLookup_01057ab0 | likely | Sets ECX = this+0x10 (the list member) and forwards a by-value 8-byte key pair (the first two arguments) plus two out pointers to FUN_01057ab0; nothing else is… |
| `009dd3c0` | FindPairThenCall_01059480 | likely | Calls FUN_01057ab0(param_1,param_2,&local_8,&local_4) and, when both output words are non-zero, calls FUN_01059480(local_4); nothing is returned. |
| `009dd420` | PlotCollection_FindCopyAndSelect | likely | Iterates the node list headed by *(this+0x20), searching each node's embedded 1024-bucket sub-map for the key pair (param_2,param_3) at node+0x38/node+0x3c; on… |
| `009de470` | FactoryPlot_StampGridQuads | likely | For every node of the list at this+0x20 (next at node+0x1268) whose definition resolves through FUN_01032980, walks the definition's grid rectangle (8 floats a… |
| `009dea70` | FactoryPlot_StampGridQuadsByCell | likely | Variant of 009de470: for every plot node it additionally computes grid indices from the cell offsets (scale = 1/cellSize) and calls FUN_01059b80(row,col,1) per… |
| `009df1d0` | List_AddTailEffectTag | likely | Appends a copy of an EffectTag to a SoeUtil list: allocates a node through vtable slot 2, copy-constructs the EffectTag in it through FUN_009dd830, links the n… |
| `009dfba0` | Tree_DeleteAndZero_Alignment4 | likely | Destructor of a tree member at *this: zeroes the words at +8/+0xc, then when the root is non-null frees its two child subtrees through FUN_009dda80 with alignm… |
| `009dfca0` | List_ScalarDeletingDtor_ItemRequestEntry | likely | Scalar deleting destructor of SoeUtil::List<ItemRequestEntry,-1>: stores its own vtable, empties the list through FUN_009df460 and frees the object when (param… |
| `009dfcd0` | HashListMap_ScalarDeletingDtor_BlueprintDataSource32 | likely | Scalar deleting destructor of SoeUtil::HashListMap<int,BlueprintDataSourceInfo,32,-1>: stores its own vtable, removes all nodes with FUN_009dc2f0 and frees the… |
| `009dfd10` | List_ScalarDeletingDtor_ClientItemConstPtr | likely | Scalar deleting destructor of SoeUtil::List<ClientItem_const*,-1>: stores its own vtable, empties the list through FUN_009df170 and frees the object when (para… |
| `009e4000` | ZeroField14AndInitBoundedList | likely | Zeros field +0x14 and initializes the bounded list through 009e1d70. |
| `009e4510` | CopySixDwordsAndListNodes | likely | Copies six leading dwords; unless source and destination are the same, clears the destination list and appends a two-dword copy of each source node. |
| `009e4570` | BuildBlueprintDataSourceMap 🌐 | likely | Walks the source list under the client/context guards, builds two IString values per entry, and keeps entries whose strings match and whose id equals the selec… |
| `009e48e0` | ToolDefinitionMap_scalarDeletingDtor | likely | Scalar deleting destructor: runs the destructor body FUN_009e4140, which clears the instance vtable-relative node list, and when bit 0 of the flag byte argumen… |
| `009e4960` | FactoryMisfortuneMap_dtor | likely | Destructor body for a 64-bucket SoeUtil::HashListMap keyed by SoeUtilSignatureFactoryMisfortuneGuid: stores the map vtable and repeatedly frees the head node a… |
| `009e49f0` | FactoryMisfortuneMap_assign | likely | Copy/assign helper for a 64-bucket hash map: when destination and source differ it drains the destination chain with FUN_009df6c0 and then, walking the source … |
| `009e4a60` | HashListAssign_CopyChainB | likely | Copy/assign variant: clears the destination chain with FUN_009ddcf0 and re-links every source node reachable through the +4 next pointer by calling FUN_009e392… |
| `009e4e00` | HashListDrain_00 | likely | Drain loop over the chain headed at +8: while the head is non-null it calls FUN_009e0680 on it and re-reads the head field (the free routine is expected to unl… |
| `009e4e20` | HashListMap_Insert64 | likely | Inserts a node into a 64-bucket hash map: allocates through vtable slot 1, builds the node with FUN_009e3a70(key, value), links it after the tail at +0xc or as… |
| `009e4f10` | ToolMap_NodeCtor | likely | Node constructor for the 256-bucket tool map: runs FUN_009e3ad0 to construct the node value from the value argument and stores the key dword of the second argu… |
| `009e4f70` | PlacementRecord_copyCtor | likely | Copies the 17 dwords at +0 up to +0x40 of a factory placement record (index 3 is skipped), installs the SoeUtil::List<EffectTag> vtable at +0x44, zeroes the li… |
| `009e5290` | DwordKeyedMap_read 🌐 | likely | Drains the destination chain, reads a dword count, then checks the stream error flag at each loop head. A failed key read sets error and supplies key 0, but th… |
| `009e5370` | PlacementRecord_read 🌐 | likely | Deserializes the 17-dword placement record: two dwords into +0/+4, one dword into +8, three 8-byte pairs into +0x10, +0x18 and +0x20, two dwords into +0x28/+0x… |
| `009e6220` | HashListMap_Assign64 | likely | Assignment for a 64-bucket map: when destination and source differ it drains the destination chain with FUN_009e0680 and re-inserts every source node (walking … |
| `009e62e0` | ToolMap_Insert | likely | Insert into the 256-bucket tool map: allocates a node through vtable slot 1, builds it with 009e4f10(key pointer, value pointer), links it after the tail at +0… |
| `009e6390` | PlacementMap_NodeCtor | likely | Node constructor for the placement-keyed map: copies the value record with FUN_009e4f70 (which also installs the List<EffectTag> vtable) and stores the two key… |
| `009e6c50` | ToolMap_Assign | likely | Assignment for the 256-bucket tool map: drains the destination chain with FUN_009ddd60 and re-inserts every source node (walking the +0x4c link) with FUN_009e6… |
| `009e6ca0` | PlacementMap_Insert | likely | Insert into a 1024-bucket map keyed by two dwords: allocates a node via vtable slot 1, builds it with 009e6390(key, value), links it after the tail, computes t… |
| `009e6d60` | FactoryDataRecord_CopyCtor | likely | Copy constructor for the composite factory record: copies dwords 0..0x13 and the byte fields at +0x20..+0x22, installs the SoeUtil::List<EffectTag> vtable at +… |
| `009e7340` | PlacementMap_Assign | likely | Assignment for the placement map: drains the destination chain with FUN_009ddc40 and re-inserts every source node (walking the +0x60 link) with FUN_009e6ca0, p… |
| `009e7410` | FoundationRecord_copyCtorB | likely | Second copy constructor of the same foundation record: copies the leading dword, copies the composite member with FUN_009e6d60 at +8 and copies the six trailin… |
| `009e7720` | FoundationMap_NodeCtor | likely | Map node constructor for the foundation map: copies the value record with FUN_009e7410 and stores the two key dwords of the second argument at node+0x108 and n… |
| `009e79c0` | FactoryHandler_NpcTooltipReply 🌐 | likely | Parses the tooltip record and, on a clean parse, stores the null guid pair at client+0x22b0/+0x22b4. If cached +0x22b8/+0x22bc equals the parsed pair OR equals… |
| `009e7b80` | FoundationMap_Insert | likely | Inserts a node into the foundation map: allocates through vtable slot 1, builds the node with 009e7720(key pointer, value pointer) and then hands the node to F… |
| `009e83b0` | FoundationMap_Assign | likely | Assignment for the foundation map: drains the destination chain with FUN_009ddac0 and re-inserts every source node (walking the +0x100 link) with FUN_009e7b80,… |
| `009e8630` | Tooltip_RequestIfStale 🌐 | likely | If FUN_009db860 accepts the guid pair, caches it at client+0x22b8/+0x22bc. It requests when no tooltip is shown, the shown pair differs, or FUN_00707040 return… |
| `009e89d0` | PlotRecord_Assign | likely | In-place composite-record assignment: assigns maps at +4 and +0x118, copies dwords at +0x230..+0x23c, resets/refills the array beginning at +0x244, clears and … |
| `009e8aa0` | FactoryHandler_UpdateFoundation 🌐 | likely | Builds the FactoryPacketUpdateFoundation object (009d92e0), reads the payload with 009e7d20 (foundation record plus a trailing dword key), calls the shared fac… |
| `009e9060` | FactoryHandler_OpenBlueprintBrowserOnFoundation 🌐 | likely | Builds the FactoryPacketOpenBlueprintBrowserOnFoundation command object (009e4210), reads its record with 009e6620, finds the plot with FUN_01057c40 and, when … |
| `009e9280` | Tooltip_RefreshTick 🌐 | likely | Calls 008a67a0 and 01032950, then requires a nonzero 01032950 result, shown flag +0x23a0, non-null cached guid +0x22b8/+0x22bc, null requested guid +0x22b0/+0x… |
| `009e9520` | PlotRecord_copyCtor | likely | Copies the leading dword; initializes and copies the 64-bucket FactoryFoundationGuid map at +4 via 009e83b0 and the 64-bucket FactoryMisfortuneGuid map at +0x1… |
| `009e9780` | PlotTable_NodeCtor | likely | Map node constructor for the client plot table: copies the record value with FUN_009e9520 and stores the guid dword of the second argument at node+0x1270; retu… |
| `009e98b0` | FactoryHandler_UpdatePlot 🌐 | likely | Constructs the FactoryPacketUpdatePlot parser object (009d98a0), reads one plot record with 009e7880 and sets the result byte to 1 after a clean parse. It then… |
| `009e99b0` | PlotTable_Insert | likely | Inserts a plot node into the 64-bucket client table: allocates through vtable slot 1, builds the node with 009e9780(key pointer, value pointer), links it after… |
| `009e9af0` | PlotTable_Assign | likely | Assignment for the client plot table: drains the destination chain with FUN_009ddb70 and re-inserts every source node (walking the +0x1268 link) with FUN_009e9… |
| `009e9c50` | FactoryHandler_UpdateFactoryInfo 🌐 | likely | Constructs FactoryPacketUpdateFactoryInfo, parses the large record through 009e9a70 and copies it with 009e9be0. On a clean parse, if client+0x23a8 is nonzero,… |
| `00c23970` | RequestRecipeStart | likely | Reads an indexed argument, creates a CommandPacketRecipeStart carrying it, and wraps the serialized packet when the client command service is available. |
| `00c23fc0` | RequestRecipeReward | likely | Reads one recipe identifier, updates TooltipData through 00a79620, constructs FactoryPacketRequestRecipeReward, and wraps it when command service is active. |
| `01031a80` | NodeSetKey_0x68 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01088c20 on this, then copies the dword at *param_2 into this+0x68 and returns this. |
| `01031ae0` | NodeSetKey_0x38 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01088da0 on this, then copies the dword at *param_2 into this+0x38 and returns this. |
| `01031b60` | NodeSetKey_0x58 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01088640 on this, then copies the dword at *param_2 into this+0x58 and returns this. |
| `01031be0` | NodeSetKey_0x44 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01089020 on this, then copies the dword at *param_2 into this+0x44 and returns this. |
| `01031c60` | NodeSetKey_0x3c | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01089140 on this, then copies the dword at *param_2 into this+0x3c and returns this. |
| `01031cc0` | NodeSetKey_0x30 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 010888d0 on this, then copies the dword at *param_2 into this+0x30 and returns this. |
| `01031d20` | NodeSetKey_0x18 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01088a30 on this, then copies the dword at *param_2 into this+0x18 and returns this. |
| `01031d80` | NodeSetKey_0x50 | likely | Thiscall (ECX = node, one stack argument, ret 4): calls callee 01058f20 on this, then copies the dword at *param_2 into this+0x50 and returns this. |
| `01034780` | CopyRecordSetKey44 | likely | Thiscall (ECX = node, param_2 = key pointer, param_3 = source record): calls CopyRecord_3c (01033210) on this with the source record, stores the key dword at *… |
| `01057f30` | FactoryPlotList_SumField14 | likely | Walks the list headed at this+0x10 (next pointer at node+0x1268). Per node calls 01032980 with the node's first dword; zero logs 'Missing Plot Definition: %d' … |
| `010589d0` | InitMemberBlockAtPlus8 | likely | Writes 0 to [this], calls 01058d00 with ECX=this+8, then stores four data words DAT_01cb6ee8, DAT_01cb6eec, DAT_01cb6ef0, DAT_01cb6ef4 at this+0xe0, +0xe4, +0x… |
| `0105a200` | GridRandomPlaceTry50 | likely | Thiscall with five stack arguments and a box pointer. Returns at once when the grid is empty (w or h < 1), incrementing the global DAT_01cb6fe8. Otherwise trie… |
| `0105a4c0` | GridCellRecordAtPos | likely | Thiscall with a min-corner pointer (param_2), an output buffer (param_3) and a record (param_4, scale at +0x18, box at +0x20). Computes cell x=(param_2[0]-box.… |
| `0105a5e0` | GridRebuildFromNodeList | likely | Fastcall (ECX=this). Sets the count at +0x248 to 0; if w*h > 0 and the count is below it, calls 01059410(w*h-1) with ECX=this+0x240. If w > 0 and h > 0, walks … |
| `0105a670` | NodeRecordInit | likely | Fastcall (ECX=this). Stores DAT_01cb7060 at [this], DAT_01cb7064 at +4, zero at +8, DAT_01cb7048 at +0x10, DAT_01cb704c at +0x14, DAT_01cb7058 at +0x18, DAT_01… |
| `0105aa10` | ScalarDtorCallsBody_01058700 | likely | Scalar deleting destructor, thiscall with a flag byte: calls 01058700 with ECX=this, then frees this when bit 0 of the flag is set; returns this. |
| `01088540` | ParseFactoryBlueprintRow | likely | Thiscall with a text cursor (param_2) and a length (param_3), returns nothing (no return value). Parses one row of about eighteen fields: dwords into this+0x0,… |
| `013cec00` | RegisterFactory0x2e | likely | Calls 013ce130 and then 013ce230 with the key 0x2e and the factory 013ceb90. |
| `013cfd50` | RegisterFactory0x16a | likely | Calls 013ce130 and then 013ce230 with the key 0x16a and the factory 013cfcf0. |
| `013d0360` | RegisterFactory0x169 | likely | Calls 013ce130 and then 013ce230 with the key 0x169 and the factory 013d0300. |
| `013d08b0` | RegisterFactory0x18f | likely | Calls 013ce130 and then 013ce230 with the key 399 (0x18f) and the factory 013d0850. |
| `013d0d90` | RegisterFactory0x18d | likely | Calls 013ce130 and then 013ce230 with the key 0x18d and the factory 013d0d30. |
| `013d2960` | RegisterFactory0x194 | likely | Calls 013ce130 and then 013ce230 with the key 0x194 and the factory 013d28f0. |
| `013d2eb0` | RegisterFactory0x191 | likely | Calls 013ce130 and then 013ce230 with the key 0x191 and the factory 013d2e50. |
| `013d3230` | RegisterFactory0x57 | likely | Calls 013ce130 and then 013ce230 with the key 0x57 and the factory 013d31d0. |
| `013d3700` | RegisterFactory0x15e | likely | Calls 013ce130 and then 013ce230 with the key 0x15e and the factory 013d36a0. |
| `0150b410` | RegisterFactory0xc5 | likely | Pushes the address of 0150b390 and the key 0xc5, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150c1a0` | RegisterFactory0xe0 | likely | Pushes the address of 0150c120 and the key 0xe0, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150c880` | RegisterFactory0x1c7 | likely | Pushes the address of 0150c800 and the key 0x1c7, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150d180` | RegisterFactory0x13b | likely | Pushes the address of 0150d100 and the key 0x13b, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150d640` | RegisterFactory0xcf | likely | Pushes the address of 0150d5c0 and the key 0xcf, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150d9b0` | RegisterFactory0xba | likely | Pushes the address of 0150d930 and the key 0xba, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150dd80` | RegisterFactory0xbb | likely | Pushes the address of 0150dd00 and the key 0xbb, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150e1d0` | RegisterFactory0x114 | likely | Pushes the address of 0150e150 and the key 0x114, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150ea80` | RegisterFactory0xb7 | likely | Pushes the address of 0150ea00 and the key 0xb7, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |
| `0150eed0` | RegisterFactory0xd9 | likely | Pushes the address of 0150ee50 and the key 0xd9, calls 013ce130 (factory registration helper), passes its returned value in ECX to 013ce230, then returns. |

### `game.firsttimeevent`

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aab70` | HashListMap_FirstTimeEventEntry_Construct | confirmed | Constructor of the first-time-event definition map: installs the SoeUtil::HashListMap<int,ClientFirstTimeEventDefinitionManager::Entry,64,-1> vtable, zeroes +4… |

### `game.fte`

**BasePacket** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c89f90` | ValidateNamedFteTrigger | confirmed | Looks up a name, tests the result, returns 1 for a positive test, and formats either an invalid-name or not-triggered message when requested. |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00928bd0` | ClientFirstTimeEventEntryHashListCtor | likely | Calls 009250a0 to initialize a stack-local HashList<ClientFirstTimeEventDefinitionManager::Entry,64,32>, restores the exception chain, and returns its input ar… |
| `00c88b10` | FormatFirstTimeEventSummary | likely | Builds a status string from the queued event count and event names, then passes it to the owning object's virtual display callback. |

### `game.garage`

**GarageManager** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b86d40` | GarageManager::GarageManager | confirmed | Constructor of GarageManager: stores param_2 at +0x10, installs GarageManager::vftable, zeroes +0x14/+0x18/+0x20, sets the flag bytes at +0x1c and +0x24 to 0 a… |
| `00b86ff0` | GarageManager::ScalarDeletingDtor | confirmed | Vtable slot 0 of GarageManager: reinstalls GarageManager::vftable and frees the manager object when bit 0 of param_2 is set. |
| `00b877d0` | GarageManager::Start 🌐 | confirmed | Opens the vehicle garage: when the active flag at this+0x24 is clear it constructs and (if this+0x10->+0x280 is set) sends the config request via 00b86a80/00b8… |
| `00b86da0` | GarageManager::Stop | likely | If the active flag byte at this+0x24 is set, runs the shutdown sequence (FUN_009d2640, FUN_009d2670, FUN_008d2950, FUN_0094c8a0(1)), then obtains the player/HU… |
| `00b87a30` | GarageManager::SelectVehicle 🌐 | likely | Selects a vehicle: constructs the select-vehicle packet 00b86be0 with the two arguments stored at packet+0xc/+0x10, sends it via 00b87530 when this+0x10->+0x28… |

**VehiclePartProcessor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d2530` | VehiclePartProcessor::Ctor | confirmed | Installs the VehiclePartProcessor vtable, stores the supplied owner, initializes a helper object, clears the cached resource and state fields, and sets its act… |
| `009d2eb0` | VehiclePartProcessor::Dtor | confirmed | Restores the VehiclePartProcessor vtable, releases its cached resource at field 900 when present, then destroys its helper object. |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d2640` | ReleaseAndClearCachedResource | likely | If this+0xe10 is non-null, invokes virtual slot zero with deleting flag one and clears the cached pointer. |
| `009d2670` | StopAndClearInterfaceAtC | likely | If this+0xc is non-null, calls its virtual slot +0x18 with argument one, clears the pointer, and sets the byte at this+4. |

### `game.group`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a983b0` | GroupInviteOnResponse | confirmed | Thiscall with a bool accept flag. When the object at this+8+0x6e8 is nonzero, calls 008dcf30, 00a8b640 on it, sends via 00a98280 with flag 1 when this+8+0x280 … |

### `game.inspect`

**InspectProxy** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad7430` | InspectProxy::OnInspectSelect | confirmed | Calls target handler slot +0x10 with id +0x13f0; on success obtains an object through 0072f4d0 and invokes its slot +0x30. |
| `00ad7460` | InspectProxy::UpdateProfile | confirmed | Compares changed object ids, updates profile geometry and active state, and for ids 1 through 17 creates per-entry tracking records when referenced objects exi… |
| `00ad7660` | InspectProxy::ClearProfileEntries | confirmed | Iterates 17 pointer slots starting at +0x1444, releases active entries through 007e7f00, clears them and resets field +0x13f4. |
| `00ad7c10` | InspectProxy::OnTargetChanged | confirmed | Compares incoming object id with stored id +0x13ec; releases and clears matching target or releases and clears alternate target field +0x13f4. |
| `00ad7fc0` | InspectProxy::Dtor | confirmed | Restores InspectProxy vtables, releases target references and nested resources, destroys CharacterStat and profile arrays and strings, then frees owned compone… |
| `00ad8390` | InspectProxy::ScalarDeletingDtor | confirmed | Calls InspectProxy cleanup and frees the allocation when the deleting flag bit is set. |
| `00ad8880` | InspectProxy::InspectProxy | likely | Constructor for an InspectProxy actor handler: installs ActorHandler::vftable, BaseAttachmentGroupManagerHandler::vftable and both InspectProxy vtable pointers… |

### `game.instance`

**InstanceSubID** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01483170` | InstanceSubID::InstanceSubID | confirmed | Default constructor: stores the PersistentBase, PersistentComponent and InstanceSubID vtables in turn, then sets Series ID (+4) and Instance Number (+8) to 0. |
| `01483200` | InstanceSubID::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'InstanceSubID'. |
| `01483210` | InstanceSubID::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5bc. |
| `01483220` | InstanceSubID::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5bc. |
| `01483240` | InstanceSubID::InstanceSubID | confirmed | One-argument constructor: stores the PersistentBase, PersistentComponent and InstanceSubID vtables, sets Series ID (+4) to 0 and stores the argument as Instanc… |
| `014832b0` | InstanceSubID::InstanceSubID | confirmed | Two-argument constructor: stores the PersistentBase, PersistentComponent and InstanceSubID vtables, then Series ID (+4) = arg1 and Instance Number (+8) = arg2. |
| `01483330` | InstanceSubID::InstanceSubID | confirmed | Source-pointer constructor: stores the PersistentBase, PersistentComponent and InstanceSubID vtables, then copies the source's +8 into Series ID (+4) and its +… |
| `01483650` | InstanceSubID::~InstanceSubID | confirmed | Scalar deleting destructor (vtable slot 0): stores the InstanceSubID, PersistentComponent and PersistentBase vtables in turn, then frees the block with _free w… |
| `014836c0` | InstanceSubID::InstanceSubID | confirmed | Copy constructor: stores the PersistentBase, PersistentComponent and InstanceSubID vtables, then copies the source's +4 and +8 into Series ID and Instance Numb… |
| `014835d0` | InstanceSubID::Factory | likely | Allocates 0xc bytes with operator new (00d833a6), calls the InstanceSubID default constructor 01483170 on the block, and returns the object or 0. |

**InstanceSubIDFlags** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01482ca0` | InstanceSubIDFlags::InstanceSubIDFlags | confirmed | Default constructor of InstanceSubIDFlags: calls the InstanceSubID base constructor 01483170, then stores the InstanceSubIDFlags vtable and sets the flags dwor… |
| `01482d20` | InstanceSubIDFlags::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'InstanceSubIDFlags' as the type name. |
| `01482d30` | InstanceSubIDFlags::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5b8. |
| `01482d40` | InstanceSubIDFlags::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5b8. |
| `01482e40` | InstanceSubIDFlags::InstanceSubIDFlags | confirmed | Copy constructor: passes the source's +4 and +8 to InstanceSubID::InstanceSubID (014832b0), stores the InstanceSubIDFlags vtable and copies the source's +0xc f… |
| `01483110` | InstanceSubIDFlags::~InstanceSubIDFlags | confirmed | Scalar deleting destructor (vtable slot 0): stores the InstanceSubIDFlags vtable, calls the InstanceSubID destructor body 014833b0, and frees the block with _f… |
| `01482d60` | InstanceSubIDFlags::InstanceSubIDFlags | likely | Constructor with three arguments: calls 014832b0(arg1, arg2) to set Series ID (+4) and Instance Number (+8), stores the InstanceSubIDFlags vtable and sets the … |
| `01482dd0` | InstanceSubIDFlags::InstanceSubIDFlags | likely | Constructor with two arguments: calls 01483240(arg1), which zeroes Series ID (+4) and stores arg1 as Instance Number (+8); stores the InstanceSubIDFlags vtable… |
| `01482eb0` | InstanceSubIDFlags::InstanceSubIDFlags | likely | Constructor taking a source pointer and a flags word: calls 01483330(source), which copies the source's +8 and +0xc into Series ID and Instance Number; stores … |
| `01482f20` | InstanceSubIDFlags::~InstanceSubIDFlags | likely | Destructor body without free: stores the InstanceSubIDFlags vtable, then calls the InstanceSubID destructor body 014833b0, which resets the vtables to the base… |
| `01483090` | InstanceSubIDFlags::Factory | likely | Allocates 0x10 bytes with operator new (00d833a6), calls the InstanceSubIDFlags constructor 01482ca0 on the block when the allocation succeeds, and returns the… |

### `game.interaction`

**ClosestInteractionTargetDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bafe30` | ClosestInteractionTargetDataSource_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00bafd00 on ECX, then _free when the delete flag bit 0 is set, and returns the object. |
| `00bafb90` | ClosestInteractionTargetDataSource::GetField | likely | Virtual slot [4] of ClosestInteractionTargetDataSource: resolves the object for the cached pair through 00984960 and checks its state (virtual slot [5]); mode … |
| `00bafd00` | ClosestInteractionTargetDataSource_DtorBody | likely | Destructor body: stores ClosestInteractionTargetDataSource::vftable at +0, +0x30 and +0x34, then calls 0091bf60 (destroys the remaining sub-object; receiver no… |
| `00bafd80` | ClosestInteractionTargetDataSource_Ctor | likely | Constructor: calls 00922890 with the 'BaseClient.ClosestInteractionTargetData' name, stores the ClosestInteractionTargetDataSource vtables at +0, +0x30 and +0x… |

**InteractionProcessor** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a380d0` | InteractionProcessor::InteractionProcessor | confirmed | Ctor with SEH frame: installs InteractionProcessor::vftable, stores param_2 (owner) at +4, clears the byte at +8, stores the default pair DAT_01be68f8/DAT_01be… |
| `00a384c0` | InteractionProcessor::ClearTargetAndSendFreeNpc | confirmed | Virtual slot [1] of InteractionProcessor::vftable (binary vtable read). When the stored pair at this+0x10/+0x14 differs from DAT_01be68f8/DAT_01be68fc it build… |
| `00a381a0` | InteractionProcessor::~InteractionProcessor | likely | Sets InteractionProcessor::vftable, calls 00a37fe0 (ListInteractionData10DtorBody) on the list embedded at this+0x1c (raw bytes: lea ecx,[esi+0x1c] before the … |
| `00a383e0` | InteractionProcessor::SendInteractRequest 🌐 | likely | Constructs a CommandPacketInteractRequest through 008b6bc0 (command family ctor with sub-opcode 8), then calls 00925a70 when the owner's +0x280 field is non-ze… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a39000` | ResetPairAndSendFreeNpc | likely | Thiscall: sets this+0x10 and this+0x14 to the default pair DAT_01be68f8/DAT_01be68fc, builds a FreeInteractionNpc through 009d9a90 and, when the owner (this+4)… |
| `00a39090` | SendInteractionSelectThenReset 🌐 | likely | Thiscall: builds a CommandPacketInteractionSelect through 00a36f80, and when the owner (this+4) has +0x280 non-zero sends it through the tunnel builder 00a382b… |

### `game.items`

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae1a40` | CanEquipItemInSlot | likely | Looks up the requested equipment slot, checks the item's optional predicate, minimum profile value and item-category restriction, and returns a packed eligibil… |
| `00ae1ba0` | RecalculateProfileItemCache | likely | Sets a dirty/cache flag, clears two cached fields, then scans the linked profile items and global item definitions to select the matching entry with the smalle… |
| `00ae1c40` | GetCachedProfileItemValues | likely | If cache flag at this+0xc0 is clear, calls cache recalculation; then copies this+0xc4 and this+0xc8 into the two output pointers. |
| `00ae1d50` | DestroyProfileItemOwner | likely | Frees optional allocations at +0xcc and +0xd0, destroys the embedded ProfileItem map, then destroys the base object through 008ea780. |
| `00ae1f70` | ProfileItemOwnerInit | likely | Initializes base state and embedded ProfileItem map, zeros the listed owner fields, allocates two 8-byte helper objects, and stores their pointers. |
| `00ae2090` | InsertProfileItemAndRefreshCache | likely | Passes its arguments to the ProfileItem insertion helper and then recalculates the cached selection fields. |

### `game.npc`

**ClosestInteractionTargetDataSource** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00baf690` | ClosestInteractionTargetDataSource::FieldName | confirmed | Returns a static field-name string for the index in param_1: 0 gives 'NpcName', 1 gives 'NpcGuid', 2 gives 'NpcIcon', any other value gives '<error>'. |

### `game.objective`

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aaf50` | ThunkField2bea4_00c43140 | likely | 11-byte thunk: raw bytes 8b 89 a4 be 02 00 e9 e5 81 29 00 decode as mov ecx,[ecx+0x2bea4]; jmp FUN_00c43140 - loads the service pointer at this+0x2bea4 and tai… |
| `009aaf60` | GatedCall_00c43180 | likely | Calls FUN_00c43180 only when the nested field *(*(*(this+8)+0x3e0)+0x45c) equals 1; otherwise returns without calling. |
| `009aaf80` | ThunkField2bea4_00c431c0 | likely | 11-byte thunk: raw bytes 8b 89 a4 be 02 00 e9 35 82 29 00 decode as mov ecx,[ecx+0x2bea4]; jmp FUN_00c431c0 - re-targets the call onto the pointer stored at th… |
| `009ad510` | UpdateObjectiveRecordStatus | likely | Finds record by key, writes param_3 to its value, selects status 5, 6, or 2 from record and manager flags, then reformats display text. |
| `009ad5a0` | SetObjectiveRecordField44 | likely | Finds a keyed record, stores param_3 at +0x44, and reformats it when field +0x48 exceeds one. |
| `009ad620` | CountIncompleteObjectives | likely | Resolves current or keyed state and counts records with byte +0x28 clear and dword +0x2c not equal to one. |
| `009ad6a0` | CountObjectiveRowsWithFlag28 | likely | Resolves current or keyed state and counts objective records whose byte +0x28 is nonzero. |

### `game.observer`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013539e0` | AddObserverIfAbsent | confirmed | Scans the entry array between this+0x48 and this+0x4c; if an entry is non-zero and 01344e20 returns the argument id, it logs Skipping adding Observer and retur… |

### `game.pets`

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa270` | HashListMap_PetTrickInfo_Construct | confirmed | Constructor of the pet-trick map: installs the SoeUtil::HashListMap<int,PetTrickInfo,16,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 16 bucket pointers … |

### `game.pirates`

**ShipCombatZonePickupLocationConfig::PickupInstance** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a64ef0` | ShipCombatZonePickupLocationConfig::PickupInstance_CopyCtor | likely | Copy constructor of the pickup-location instance: copies the ten dwords at +0..+0x24 from param_2, installs SoeUtil::Array<ShipCombatZonePickupLocationConfig::… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a644e0` | ShipCombat_RefreshShipEffectObjects | likely | Clears paired effect handles, adjusts the effect-array capacity at +0x3f0 against the entity count +0x21c, and iterates that many entries from the entity list … |
| `00a64960` | Pirates_SetupShipCombatStage | likely | One-shot stage setup guarded by the flag +0x3ac: when the flag is clear it sets it, walks the entity list hanging off the interface (*(*(this+8)+0x10), linked … |
| `00a65030` | ShipCombatCannonConfig_ScalarDtor | likely | Scalar deleting destructor for a 0x4c-byte cannon-config element: runs the destructor body 00a62030 and, when bit 0 of param_2 is set, releases the object with… |
| `00a656b0` | ShipCombatCannonConfig_CopyCtor | likely | Copy constructor of a cannon-config element: copies the ten leading dwords from param_2, copy-constructs the IString member at +0x28 (installs SoeUtil::IString… |
| `00a668f0` | ProxiedShip_CtorFromSpawnData | likely | Constructor-like helper: runs the base initialiser 0106d500 under an SEH frame and copies the dword pointed to by param_2 into the +0x360 field of the object; … |
| `00a673f0` | FactoryResultRegister | likely | Calls the function at vtable slot +4 with ECX=this and no stack arguments. If its EAX result is nonzero, uses that result as ECX for 00a668f0 and passes param_… |
| `00a67470` | Wrapper_00a67470 | likely | Saves the incoming ECX receiver while establishing an SEH frame, calls destructor body 00a66780 with that receiver still in ECX and no stack argument, restores… |

### `game.player_event`

**PlayerEffectEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00957ef0` | PlayerEffectEvent::ScalarDeletingDtor | likely | Calls the PlayerEffectEvent destructor and frees the object when deletion flag bit 0 is set. |

**PlayerHitpointDeltaEvent** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae2130` | PlayerHitpointDeltaEventBaseDtor | confirmed | Writes PlayerHitpointDeltaEvent then PlayerEvent vtables and invokes the PlayerEvent base cleanup helper. |
| `00ae2350` | PlayerHitpointDeltaEvent::Apply | confirmed | When enabled and player context exists, formats the signed delta, emits a hitpoint change, checks current hitpoint pair against event values and invokes a seco… |

**PlayerHitpointEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae20d0` | PlayerHitpointEventBaseDtor | confirmed | Writes PlayerHitpointEvent then PlayerEvent vtables and invokes the PlayerEvent base cleanup helper. |
| `00ae2190` | PlayerHitpointEventCtor | confirmed | Constructs PlayerEvent base, copies six constructor values into event offsets +0x54 through +0x68, sets event vtable, and assigns an incrementing context count… |
| `00ae2260` | PlayerHitpointEvent::ScalarDeletingDtor | confirmed | Calls the event destructor body, optionally frees this on flag bit 0, and returns this otherwise. |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c56130` | CreatePlayerSoundEmitterEvent | likely | If ECX is non-null: looks up a key through 00709a70, writes the result to the out-pointer, and when it is non-null and its vtable slot [2] returns true, alloca… |
| `00c56220` | CreatePlayerModelMaterialEvent | likely | Same factory shape as CreatePlayerSoundEmitterEvent: key lookup through 007dd650, out-pointer write, vtable slot [2] predicate, operator new of 0xb0 bytes, the… |
| `00c56310` | CreatePlayerLightEmitterEvent | likely | Same factory shape: key lookup through 007dd5f0, out-pointer write, vtable slot [2] predicate, operator new of 0xb0 bytes, then constructs PlayerLightEmitterEv… |
| `00c56400` | CreatePlayerCameraEffectEvent | likely | Same factory shape: key lookup through 007dd6a0, out-pointer write, vtable slot [2] predicate, operator new of 0xb0 bytes, then constructs PlayerCameraEffectEv… |
| `00c564f0` | CreatePlayerRumbleEffectEvent | likely | Same factory shape: key lookup through 008beba0, out-pointer write, vtable slot [2] predicate, operator new of 0xb0 bytes, then constructs PlayerRumbleEffectEv… |
| `00c565e0` | CreatePlayerMeshEmitterEvent | likely | Same factory shape: key lookup through 007dd700, out-pointer write, vtable slot [2] predicate, operator new of 0xb0 bytes, then constructs PlayerMeshEmitterEve… |

### `game.poll`

**PollChoice** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442ba0` | PollChoice::PollChoice | confirmed | Fastcall constructor (this in ECX). Stores the PersistentBase, PersistentComponent and PollChoice vtables in that order (the last leaves PollChoice::vftable), … |
| `01442c80` | PollChoice::~PollChoice | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores PollChoice::vftable, frees the string buffer at +0x10 through _free when the capacity at +0x2… |
| `01442d90` | PollChoice::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Calls the destructor body 01442c80; when bit 0 of the argument is set calls _free on this; re… |
| `01442d10` | PollChoice::Create | likely | Factory with no arguments. Calls operator new with 0x30 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 01442ba… |

### `game.requirement`

**BaseRequirementManager<RequirementSet,RequirementGroup,BaseRequirementTemplate<int,int,int>_>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd910` | BaseRequirementManager::Ctor | confirmed | Writes BaseDefinitionManager, then IRequirementManager, then BaseRequirementManager<RequirementSet,RequirementGroup,BaseRequirementTemplate<int,int,int>> vftab… |
| `00bd05b0` | BaseRequirementManager::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls destructor body 00bd0250 on this, then calls FreeMemory(this, 0) only when bit 0 of the flag argument is set; returns this. |
| `00bce7c0` | CreateRequirementFromStream | likely | Virtual slot [4] (also ClientRequirementManager::vftable[4]). Stdcall with one stack argument, a buffer cursor whose dword [0] is the start and [8] the end. Ca… |
| `00bce870` | RequirementFromKeyInvokeSlot4 | likely | Virtual slot [5] (also ClientRequirementManager::vftable[5]). Stdcall with two stack arguments. Reads a key through 00bbf3e0 into a local, looks it up with 00b… |

**BaseRequirementTemplate<int,int,int>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbe410` | BaseRequirementTemplate::DtorBody | confirmed | Destructor body (bytes checked): stores the BaseRequirementTemplate<int,int,int> vftable (01825eac) at this+0, runs IString<char> destructor body 00722bf0 on t… |
| `00bbe820` | BaseRequirementTemplate::ScalarDeletingDtor | confirmed | Scalar deleting destructor (bytes checked, 32 bytes): calls the template destructor body 00bbe410 on ECX, then if bit 0 of the flag argument is set calls 00401… |
| `00bbf3e0` | BaseRequirementTemplate::ParseDelimitedFields | likely | Pseudocode (340 bytes; thiscall with a source cursor and a delimiter as stack args, ret 8). Constructs a 256-char fixed string (004029b0) for the scope and rea… |

**ClientRequirementManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd0c50` | ClientRequirementManager::DtorBody | confirmed | Destructor body: stores the ClientRequirementManager vftable at this+0, then calls the base destructor body 00bd0250 on this. Returns nothing. |
| `00bd0df0` | ClientRequirementManager::Ctor | confirmed | Constructor body: calls the base constructor 00bcd910, stores the ClientRequirementManager vftable at this+0, stores the argument at dword [0xD024], and when t… |
| `00bd0f40` | ClientRequirementManager::ScalarDeletingDtor | confirmed | Virtual slot 0. Stores the ClientRequirementManager vftable at this+0, calls the base destructor body 00bd0250, then calls FreeMemory(this, 0) only when bit 0 … |

**IRequirement** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbf840` | IRequirement::Ctor | confirmed | Bytes checked (90 bytes): saves ECX as this, calls 00bbe840 (the ExpressionNode<ValidationData> constructor) on this, then stores the IRequirement vftable (018… |
| `00bc19b0` | IRequirement::AppendFormattedId | likely | Pseudocode (29 bytes): calls the virtual at this vtable +0x20 (slot [8]) with no arguments to get a dword, then calls 00402ff0 (IStringAppendFormat, labelled c… |

**Requirements::ExpressionNode<ValidationData>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbe840` | ExpressionNodeValidationData::Ctor | confirmed | Constructor (bytes checked for the call): saves ECX as this, calls the ExpressionNodeBase constructor 00bbd3c0 on this, then stores the Requirements::Expressio… |

**Requirements::ExpressionNodeBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbe8a0` | ExpressionNodeBase::ScalarDeletingDtor | confirmed | Pseudocode (bytes not separately decoded): stores the Requirements::ExpressionNodeBase vftable (01825d74) then the SoeUtil::RefCounted vftable (017e7d08) into … |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementAlwaysFails,SoeUtil::RefObjectPool<ClientRequirementAlwaysFails,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd680` | PoolWrapper_ClientRequirementAlwaysFails::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls destructor body 00bcbcd0 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCountry,SoeUtil::RefObjectPool<ClientRequirementCheckCountry,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd620` | PoolWrapper_ClientRequirementCheckCountry::ScalarDeletingDt… | confirmed | Virtual slot 0. Calls destructor body 00bcbbb0 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCurrentProfile,SoeUtil::RefObjectPool<ClientRequirementCheckCurrentProfile,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd6c0` | PoolWrapper_ClientRequirementCheckCurrentProfile::ScalarDel… | confirmed | Virtual slot 0. Calls destructor body 00bcbd90 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLanguage,SoeUtil::RefObjectPool<ClientRequirementCheckLanguage,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd640` | PoolWrapper_ClientRequirementCheckLanguage::ScalarDeletingD… | confirmed | Virtual slot 0. Calls destructor body 00bcbc10 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank,SoeUtil::RefObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd660` | PoolWrapper_ClientRequirementCheckLastPlayedProfileOfTypeRa… | confirmed | Virtual slot 0. Calls destructor body 00bcbc70 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckNudge,SoeUtil::RefObjectPool<ClientRequirementCheckNudge,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd700` | PoolWrapper_ClientRequirementCheckNudge::ScalarDeletingDtor | confirmed | Virtual slot 0. Calls destructor body 00bcbe50 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckRenamePending,SoeUtil::RefObjectPool<ClientRequirementCheckRenamePending,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd6e0` | PoolWrapper_ClientRequirementCheckRenamePending::ScalarDele… | confirmed | Virtual slot 0. Calls destructor body 00bcbdf0 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckVipRank,SoeUtil::RefObjectPool<ClientRequirementCheckVipRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcd6a0` | PoolWrapper_ClientRequirementCheckVipRank::ScalarDeletingDt… | confirmed | Virtual slot 0. Calls destructor body 00bcbd30 on this, then calls _free(this) only when bit 0 of the flag argument is set; returns this. |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcfa70` | LoadRequirementsTxt | confirmed | Opens the text file made from the prefix argument and "Requirements.txt" (format "%sRequirements.txt"), then for each non-comment line (not starting with '#') … |
| `00bcfc80` | LoadRequirementGroupsTxt | confirmed | Opens the file made from the prefix and "RequirementGroups.txt" (format "%sRequirementGroups.txt"). For each non-comment line it parses two '^'-separated integ… |
| `00bd0fb0` | RequirementValidationDiffLog 🌐 | confirmed | For set id param_2 it finds the node in the 8192-bucket map at this+0xA058 (key at node+0x34) and, when the expression param_3 is non-null, compares a result f… |
| `00bbe730` | RequirementSetFirstFailure | likely | Bytes checked (190 bytes, thiscall with one stack arg obj; ECX = this). Clears obj+0x230 and drains the list at obj+0x234 (00bbe2c0). Phase 1: walks the node l… |
| `00bc19d0` | RequirementIdListAppendAnd | likely | Pseudocode (67 bytes; thiscall, ECX = owner, stack arg = string): walks the list at owner+0x10 (next link at +0xc). For each element it calls the virtual at vt… |
| `00bc1a20` | RequirementSetAppendText | likely | Pseudocode (274 bytes; thiscall, ECX = owner, stack arg = output string). Walks the list at owner+0x14 (next +4): for each element calls 00bc19d0 (RequirementI… |
| `00bcd750` | ExpressionTokenize | likely | Tokenizer over a NUL-terminated char string. Characters whose bit is set in the 256-bit table at this+0x483B end a token; for '&', '\|' and '^' a doubled chara… |
| `00bcf460` | ListDumpIdNameToFile | likely | Optionally opens a text file (FileOpen 00703e10 with the path argument, mode 2) and walks the linked list at this+0x321C -> +0xA04C, formatting each entry's in… |
| `00bd05d0` | RegisterCreator_Pool00bcd020 | likely | Registers a new creator under the 32-bit tag in param_2. Takes the pool from 00bcd020 when the flag dword [0x2202C] of this is nonzero, else 0. Allocates a 12-… |
| `00bd06a0` | RegisterCreator_Pool00bcd0e0 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd0e0 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd0770` | RegisterCreator_Pool00bcd1a0 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd1a0 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd0840` | RegisterCreator_Pool00bcd260 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd260 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd0910` | RegisterCreator_Pool00bcd320 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd320 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd09e0` | RegisterCreator_Pool00bcd3e0 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd3e0 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd0ab0` | RegisterCreator_Pool00bcd4a0 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd4a0 when [0x2202C] is nonzero, allocates a 12-byte creator, initialises it with… |
| `00bd0b80` | RegisterCreator_Pool00bcd560 | likely | Same body as RegisterCreator_Pool00bcd020 (197 bytes): takes the pool from 00bcd560 (KeyedLazyPoolGet, which builds the ClientRequirementCheckNudge pool wrappe… |

### `game.ruleset`

**ArgTree** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014872d0` | ArgTree::~ArgTree | confirmed | Destructor body: stores the ArgTree vtable, walks the map at +0x1c from its leftmost node with iterator increment 01485df0, deleting each non-null value (vtabl… |
| `01487420` | ArgTree::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'ArgTree'. |
| `01487430` | ArgTree::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5c8. |
| `01487440` | ArgTree::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5c8. |
| `01487680` | ArgTree::ArgTree | confirmed | Constructor: stores the PersistentBase, PersistentComponent and ArgTree vtables, sets +4 to the argument (when it is zero, takes the next value of the static c… |
| `01487750` | ArgTree::~ArgTree | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 014872d0, then frees the block with _free when bit 0 of the argument is set; returns the … |
| `01487770` | ArgTree::ArgTree | confirmed | Copy constructor (deep): stores the PersistentBase, PersistentComponent and ArgTree vtables, copies the source's +4, +8 and +0xc, constructs the child object a… |
| `01487e30` | ArgTree::Factory | likely | Allocates a 0x34-byte block with operator new (00d833a6), calls the ArgTree constructor 01487680 with arguments 0 and 0 on the block, and returns the object or… |

**BaseDefinitionManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbd360` | BaseDefinitionManager::~BaseDefinitionManager | confirmed | Restores the BaseDefinitionManager vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bb9b80` | BaseDefinitionManager::~BaseDefinitionManager | likely | Replaces the object's vtable with BaseDefinitionManager::vftable and returns; EH unwind funclets call it during cleanup. |

**BaseRequirementManager<RequirementSet,RequirementGroup,BaseRequirementTemplate<int,int,int>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc35c0` | ClearRequirementSetMap | confirmed | Virtual slot 2 of the requirement manager vtable: repeatedly takes the head of the RequirementSet map at this+0xa04c and removes it through 00bbde60 until the … |

**BaseRequirementManager<RequirementSet,RequirementGroup,BaseRequirementTemplate<int,int,int>_>::GameTechRequirementManager** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcc710` | GameTechRequirementManager_Ctor | confirmed | Constructor: runs the constructor 00bc7e50 on the receiver (it constructs the LogicalInfixExpressionTreeBuilder member at ECX), installs the GameTechRequiremen… |
| `00bcc8e0` | GameTechRequirementManager_DtorBody | confirmed | Destructor body: installs the GameTechRequirementManager vtable and runs the RequirementManager destructor body 00bcbf50 on the receiver. Does not free. |

**BaseRequirementTemplate<int,int,int>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc3410` | Ctor | likely | Constructor: stores the BaseRequirementTemplate<int,int,int> vtable, zeroes the word at +0x2c, runs three IString member initialisers (00722da0), zeroes the se… |

**CTP::Game** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01350a90` | CTP::Game::GetPlayArea | confirmed | Virtual slot [129] (thiscall: ECX = game, param_2 = area id, param_3 = environment): logs Game::getPlayArea %d through 013846b0 and dispatches the id to a play… |

**CWCommandObject_AddCostMod** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145a690` | CWCommandObject_AddCostMod::DtorBody | confirmed | Destructor body, fastcall ECX=this, no stack arguments. Stores CWCommandObject_AddCostMod::vftable (018ee594) at +0 and the same name's vftable (018ee558) at +… |
| `0145a700` | CWCommandObject_AddCostMod::GetTypeName | confirmed | Virtual slot [11] of the +8 vtable. Returns the address of the literal 'CWCommandObject_AddCostMod' (mov eax,018ee608; ret). |
| `0145a710` | CWCommandObject_AddCostMod::GetClassDword | confirmed | Virtual slot [12] of the +8 vtable. Loads the global dword DAT_01cbe55c into EAX and returns it (ret 0). |
| `0145a720` | CWCommandObject_AddCostMod::SetClassDword | confirmed | Virtual slot [13] of the +8 vtable, ret 4. Stores its stack argument into the global dword DAT_01cbe55c. |
| `0145a740` | CWCommandObject_AddCostMod::DoCommand | confirmed | Virtual slot [17] of the primary vtable 018ee594, fastcall, returns 1. Traces 'CWCommandObject_AddCostMod::doCommand %d' through 013508b0 with this+0x134 on th… |
| `0145a7c0` | CWCommandObject_AddCostMod::UndoCommand | confirmed | Virtual slot [19] of the primary vtable 018ee594, fastcall, returns 1. Same lookup and casts as DoCommand (01357a00, PlayElement to AttributeModifier, CTP::Gam… |
| `0145a8f0` | CWCommandObject_AddCostMod::ScalarDeletingDtor_Thunk8 | confirmed | Adjustor thunk for the +8 vtable slot [0]: subtracts 8 from ECX (83 e9 08) and jumps to 0145a900 (the scalar deleting destructor). It has no body of its own. |
| `0145a900` | CWCommandObject_AddCostMod::ScalarDeletingDtor | confirmed | Vtable slot [0] of the primary vtable 018ee594, thiscall with one flag argument. Calls the destructor body 0145a690 with ECX=this, then frees the object with _… |
| `0145a920` | CWCommandObject_AddCostMod::Ctor | confirmed | Default constructor, fastcall ECX=this, no stack arguments. Calls base init 01505280, stores CWCommandObject_AddCostMod::vftable (018ee594) at +0 and the vftab… |
| `0145a990` | CWCommandObject_AddCostMod::CtorWithArgs | confirmed | Constructor overload, thiscall with four stack arguments. Calls 015055e0 with the four arguments and the constant 1, stores both AddCostMod vtables (018ee594 a… |

**CWCommandObject_Destroy** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145a110` | CWCommandObject_Destroy::DtorBody | confirmed | Destructor body, fastcall with ECX=this, no stack arguments. Stores CWCommandObject_Destroy::vftable (018ee3cc) at +0 and the same name's vftable (018ee390) at… |
| `0145a180` | CWCommandObject_Destroy::GetTypeName | confirmed | Virtual slot [11] of the +8 vtable of CWCommandObject_Destroy. Returns the address of the literal 'CWCommandObject_Destroy' (mov eax,018ee440; ret). |
| `0145a190` | CWCommandObject_Destroy::GetClassDword | confirmed | Virtual slot [12] of the +8 vtable. Loads the global dword DAT_01cbe558 into EAX and returns it (ret 0). |
| `0145a1a0` | CWCommandObject_Destroy::SetClassDword | confirmed | Virtual slot [13] of the +8 vtable, ret 4. Stores its stack argument into the global dword DAT_01cbe558. |
| `0145a280` | CWCommandObject_Destroy::ScalarDeletingDtor_Thunk8 | confirmed | Adjustor thunk for the +8 vtable slot [0]: subtracts 8 from ECX (83 e9 08) and jumps to 0145a290 (the scalar deleting destructor). It has no body of its own. |
| `0145a290` | CWCommandObject_Destroy::ScalarDeletingDtor | confirmed | Vtable slot [0] of the primary vtable 018ee3cc, thiscall with one flag argument, ret 4. Calls the destructor body 0145a110 with ECX=this, then frees the object… |
| `0145a470` | CWCommandObject_Destroy::Ctor | confirmed | Default constructor, fastcall ECX=this, no stack arguments. Calls base init 0142b070, stores CWCommandObject_Destroy::vftable (018ee3cc) at +0 and the vftable … |
| `0145a4f0` | CWCommandObject_Destroy::CtorWithArg | confirmed | Constructor overload, thiscall with one stack argument. Calls base init 0142b150 with the argument, stores the Destroy vtables (018ee3cc at +0, 018ee390 at +8)… |

**CWGameCommand_SetupGame** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01459a20` | CWGameCommand_SetupGame::CtorDefault | confirmed | Default constructor, thiscall with no stack arguments, ret 0. Calls base init 014d8420 with ECX=this, stores CWGameCommand_SetupGame::vftable at +0, sets the d… |
| `01459ab0` | CWGameCommand_SetupGame::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal 'CWGameCommand_SetupGame' (mov eax,018ee25c; ret). |
| `01459ac0` | CWGameCommand_SetupGame::GetClassDword | confirmed | Virtual slot [12]. Loads the global dword DAT_01cbe554 into EAX and returns it (ret 0). |
| `01459ad0` | CWGameCommand_SetupGame::SetClassDword | confirmed | Virtual slot [13], ret 4. Stores its stack argument into the global dword DAT_01cbe554. |
| `01459af0` | CWGameCommand_SetupGame::CtorFromGame | confirmed | Thiscall with one stack argument, ret 4. Calls base init 014d84f0 with the argument, stores the vtable, dynamic_casts the argument from CTP::Game to CWGame (00… |
| `01459b90` | CWGameCommand_SetupGame::DtorBody | confirmed | Destructor body, fastcall ECX=this, no stack arguments. Stores CWGameCommand_SetupGame::vftable (018ee204) at +0 and calls the base destructor 014d8320 with EC… |
| `01459be0` | CWGameCommand_SetupGame::Clone | confirmed | Virtual slot [17], fastcall. Allocates 0x88 bytes with 00d833a6; when non-null constructs CtorFromGame(this+8) (01459af0) in the block; then calls its own slot… |
| `01459c60` | CWGameCommand_SetupGame::CopyFieldsTo | confirmed | Virtual slot [18], ret 4. dynamic_casts its argument from NetworkCommand to CWGameCommand_SetupGame (00d8d382); when the result is null it reports the assert '… |
| `01459e10` | CWGameCommand_SetupGame::ScalarDeletingDtor | confirmed | Vtable slot [0], thiscall with one flag argument, ret 4. Stores the vtable at +0, calls base dtor 014d8320 with ECX=this, then frees the object with _free (thu… |
| `01459e70` | CWGameCommand_SetupGame::ClientHandler | confirmed | Virtual slot [15], fastcall. Dynamic-casts this+8 from CTP::Game to CWGame (assert 'mg_game' at line 0x5c on null). If this+0x50 (command version) is nonzero a… |
| `0145a0e0` | CWGameCommand_SetupGame::Dump | confirmed | Virtual slot [19] of CWGameCommand_SetupGame::vftable, ret 4. Appends the 0x1a-char trace text 'Setup CWGame Game Command' plus newline to its output string ar… |

**CWMatchCommand_LaunchGame** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014599d0` | CWMatchCommand_LaunchGame::Dump | confirmed | Virtual slot [19] of CWMatchCommand_LaunchGame::vftable, ret 4. Appends the 0x1c-char trace text 'CWLaunch Game Match Command' plus newline to its output strin… |
| `01459a00` | CWMatchCommand_LaunchGame::DisplayName | confirmed | Virtual slot [20], ret 4. Loads the stack argument into ECX and appends the 0x1b-char literal 'CWLaunch Game Match Command' to it via 01241650 (std::string app… |

**ClientRequirementAlwaysFails** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4590` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementAlwaysFails vtable at +0. |
| `00bc4610` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4e60` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementAlwaysFails in place through 00bc4590 and returns the object pointer. |

**ClientRequirementCheckCountry** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4350` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckCountry vtable at +0. |
| `00bc43b0` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4d40` | PoolPopAndConstruct | likely | Pops a block from the pool through 00bc16c0; returns 0 when that yields null, otherwise constructs ClientRequirementCheckCountry in place through 00bc4350 and … |

**ClientRequirementCheckCurrentProfile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4740` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckCurrentProfile vtable at +0. |
| `00bc47b0` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4f20` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementCheckCurrentProfile in place through 00bc4740 and returns the object … |

**ClientRequirementCheckLanguage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4410` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckLanguage vtable at +0. |
| `00bc4470` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4da0` | PoolPopAndConstruct | likely | Pops a block from the pool through 00bc16c0; returns 0 when that yields null, otherwise constructs ClientRequirementCheckLanguage in place through 00bc4410 and… |

**ClientRequirementCheckLastPlayedProfileOfTypeRank** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc44d0` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckLastPlayedProfileOfTypeRank vtable at +0. |
| `00bc4530` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4e00` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementCheckLastPlayedProfileOfTypeRank in place through 00bc44d0 and return… |

**ClientRequirementCheckNudge** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc48e0` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckNudge vtable at +0. |
| `00bc4950` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4fe0` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementCheckNudge in place through 00bc48e0 and returns the object pointer. |

**ClientRequirementCheckRenamePending** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4810` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckRenamePending vtable at +0. |
| `00bc4880` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4f80` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementCheckRenamePending in place through 00bc4810 and returns the object p… |

**ClientRequirementCheckVipRank** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4670` | Ctor | confirmed | Constructor: calls the requirement-template base constructor 00bc3410 on ECX, then stores the ClientRequirementCheckVipRank vtable at +0. |
| `00bc46e0` | ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls the requirement-template destructor body 00bbe410 on this, then frees this with 00401c00 when bit 0 of the flag argument is s… |
| `00bc4ec0` | PoolPopAndConstruct | likely | Pops a block through 00bc1730; returns 0 when null, otherwise constructs ClientRequirementCheckVipRank in place through 00bc4670 and returns the object pointer. |

**ContentPackManager** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010463c0` | ContentPackManager::IsContentUnlocked | confirmed | Thiscall, one stack arg (int id; returns a byte). If id < 1 returns 1. Otherwise indexes the 128-bucket table at this+0x20 by (id & 0x7f), walks the chain (nex… |
| `01046410` | ContentPackManager::SetFlagById | confirmed | Thiscall, two stack args (uint id, char flag). Finds the node for id in the 128-bucket table at this+0x20 (bucket id & 0x7f, chain next at node+0x1c). If found… |
| `010469e0` | ContentPackManager::LoadDefinitions | confirmed | Thiscall. Builds the path: the directory string from 00725ee0 with a '/' appended when the last character is not a separator (006fc030), then "ContentPacks.txt… |
| `01046cf0` | ContentPackManager::DeleteDefinitions | confirmed | Fastcall, ECX = manager. Bytes: push esi; mov esi,ecx; lea ecx,[esi+0x14]; call 01046b70 (clears the ContentPack map at this+0x14); then logs to ContentPacks.l… |
| `01047b80` | ContentPackManager::DtorBody | confirmed | Fastcall, ECX = manager. Stores ContentPackManager::vftable (0187f460) at *this, calls 01046b70 (clears the content-pack nodes), logs "ContentPackManager::Dele… |
| `01047d50` | ContentPackManager::Ctor | confirmed | Thiscall, one stack arg (name C string pointer, may be null). Stores BaseDefinitionManager::vftable and then ContentPackManager::vftable (0187f460) at *this, s… |
| `01047e00` | ContentPackManager::ScalarDeletingDtor | confirmed | Thiscall with a flag byte param_2. Calls 01047b80 (ContentPackManager::DtorBody), then if bit 0 of the flag is set calls 00401c00 with alignment 0 on the objec… |
| `01047e20` | ContentPackManager::LoadDefinitionsFromBuffer | confirmed | Thiscall, one stack arg (text buffer, C string). Parses the buffer line by line with 006fc270. Lines starting with '#' are header rows: the cells are read with… |

**ExpressionNodeBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbad80` | ExpressionNodeBase::Cleanup | confirmed | Replaces the object's vtable with ExpressionNodeBase::vftable and then RefCounted::vftable during destruction. |
| `00bbadd0` | ExpressionNodeBase::Cleanup | confirmed | Replaces the object's vtable with ExpressionNodeBase::vftable and then RefCounted::vftable during destruction. |

**IRequirement** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbd2a0` | IRequirement::CallSlot14AndUpdateList | confirmed | Calls the receiver's virtual slot +0x14 with the supplied value, appends a temporary value through 00bbd210 on either branch, and returns the slot result or 0. |

**IRequirementManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb9b10` | IRequirementManager::IRequirementManager | confirmed | Writes BaseDefinitionManager::vftable and then IRequirementManager::vftable to the object, returning the receiver. |

**ProjectileParameters** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d00e0` | ProjectileParameters::ProjectileParameters | confirmed | Constructor of the ProjectileParameters record: installs ProjectileParameters::vftable, zeroes the field block +0x10..+0x1c, stores the global float uVar1 at +… |
| `008d0240` | ProjectileParameters::~ProjectileParameters | confirmed | Destructor of the ProjectileParameters record: rewrites ProjectileParameters::vftable, releases the two embedded SoeUtil::IString members at +0x94 and +0x84 (c… |
| `008d0310` | ProjectileParameters::ScalarDeletingDestructor | likely | MSVC scalar deleting destructor: runs the ProjectileParameters destructor body FUN_008d0240 and frees the object with _free when bit 0 of the flag argument is … |

**RequirementSet** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bba490` | RequirementSet::~RequirementSet | likely | Forwards the incoming receiver to RequirementSet cleanup at 010821b0 inside an exception frame, restores the exception-list pointer, and returns. |
| `00bbc690` | RequirementSet::~RequirementSet | likely | Runs the RequirementSet destructor body on the receiver, calls _free(this) only when flag bit 0 is set, and restores the SEH frame. |
| `01082040` | RequirementSetCtor | likely | Constructor body: stores the RequirementSet vftable at this+0x0, zeroes +0x4, +0x8, +0x14, +0x18, +0x1c, +0x20, +0x24 and +0x28, and stores the argument at +0x… |
| `010821b0` | RequirementSetDtorBody | likely | Destructor body (bytes: lea ecx,[esi+0x14]; store vftable; call 01082120; lea ecx,[esi+0x20]; jmp 01082120): stores the RequirementSet vftable at this+0x0, dra… |
| `010821d0` | RequirementSetScalarDeletingDtor | likely | Virtual slot 0 scalar deleting destructor: stores the RequirementSet vftable, drains two lists through 01082120, then frees this with FreeMemory (alignment 0) … |

**Requirements::BinaryExpressionNode<ValidationData>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc8500` | BinaryExpressionNode_DtorBody | confirmed | Destructor body: installs the BinaryExpressionNode vtable, then releases its two RefCounted operand references at +8 and +0xc (each refcount at +4 is decrement… |
| `00bc8e00` | BinaryExpressionNode_ScalarDeletingDtor | confirmed | Virtual slot 0 of the BinaryExpressionNode vtable: calls destructor body 00bc8500 on the receiver, calls 00401c00(this, 0) when flag bit 0 of the argument is s… |
| `00bc9320` | BinaryExpressionNode_Ctor | confirmed | Constructor: runs the ExpressionNode base constructor 00bbe840 under an SEH frame, installs the BinaryExpressionNode vtable, and stores its two stack arguments… |

**Requirements::ExpressionNodeBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbd3c0` | ExpressionNodeBase::ExpressionNodeBase | confirmed | Runs the RefCounted base constructor, installs the ExpressionNodeBase vtable, restores the SEH frame, and returns the receiver. |

**Requirements::ExpressionNodeGrouping<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9f90` | ExpressionNodeGrouping_Ctor | confirmed | Constructor: runs the UnaryExpressionNode constructor 00bc9390 on the receiver with the operand argument, then installs the ExpressionNodeGrouping vtable. Retu… |
| `00bc9ff0` | ExpressionNodeGrouping_Evaluate | confirmed | Virtual slot 3 (10 bytes, decoded: mov ecx,[ecx+8]; mov eax,[ecx]; mov eax,[eax+0xc]; jmp eax): tail-jumps to slot 3 of the operand at this+8 with the same sta… |
| `00bca050` | ExpressionNodeGrouping_ScalarDeletingDtor | confirmed | Virtual slot 0 of the Grouping vtable: calls the UnaryExpressionNode destructor body 00bc8590, calls 00401c00(this, 0) when flag bit 0 of the argument is set, … |
| `00bca0b0` | ExpressionNodeGrouping_AppendText | confirmed | Virtual slot 2 (thiscall, one stack argument): appends the literal DAT_01770580 ('(') through 00402770, calls slot 2 of the operand (this+8) with the output ar… |

**Requirements::ExpressionNodeLogicalAnd<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9930` | ExpressionNodeLogicalAnd_Ctor | confirmed | Constructor: runs the BinaryExpressionNode constructor 00bc9320 on the receiver with the two stack arguments (left, right), then installs the ExpressionNodeLog… |
| `00bc99a0` | ExpressionNodeLogicalAnd_Evaluate | confirmed | Virtual slot 3 (thiscall, three stack arguments, ret 0xc): calls slot 3 of the left operand (this+8) with the three arguments; only when that returns nonzero i… |
| `00bc9a40` | ExpressionNodeLogicalAnd_ScalarDeletingDtor | confirmed | Virtual slot 0 of the LogicalAnd vtable: calls the BinaryExpressionNode destructor body 00bc8500 on the receiver, calls 00401c00(this, 0) when flag bit 0 of th… |
| `00bc9aa0` | ExpressionNodeLogicalAnd_AppendText | confirmed | Virtual slot 2 (thiscall, one stack argument): calls slot 2 of the left operand (this+8) with the output argument, appends the literal DAT_01825f28 (' && ') th… |

**Requirements::ExpressionNodeLogicalNot<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9e20` | ExpressionNodeLogicalNot_Ctor | confirmed | Constructor: runs the UnaryExpressionNode constructor 00bc9390 on the receiver with the operand argument, then installs the ExpressionNodeLogicalNot vtable. Re… |
| `00bc9e80` | ExpressionNodeLogicalNot_Evaluate | confirmed | Virtual slot 3 (thiscall, three stack arguments, ret 0xc): calls slot 3 of the operand (this+8) with the arguments (p2, p4, p3), the last two swapped (disassem… |
| `00bc9f00` | ExpressionNodeLogicalNot_ScalarDeletingDtor | confirmed | Virtual slot 0 of the LogicalNot vtable: calls the UnaryExpressionNode destructor body 00bc8590, calls 00401c00(this, 0) when flag bit 0 of the argument is set… |
| `00bc9f60` | ExpressionNodeLogicalNot_AppendText | confirmed | Virtual slot 2 (thiscall, one stack argument): appends the literal DAT_018264d4 ('!') through 00402770 and then calls slot 2 of the operand (this+8) with the s… |

**Requirements::ExpressionNodeLogicalOr<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9ad0` | ExpressionNodeLogicalOr_Ctor | confirmed | Constructor: runs the BinaryExpressionNode constructor 00bc9320 on the receiver with the two stack arguments, then installs the ExpressionNodeLogicalOr vtable.… |
| `00bc9b40` | ExpressionNodeLogicalOr_Evaluate | confirmed | Virtual slot 3 (thiscall, three stack arguments, ret 0xc): calls slot 3 of the left operand (this+8) with the three arguments; when that returns zero it calls … |
| `00bc9be0` | ExpressionNodeLogicalOr_ScalarDeletingDtor | confirmed | Virtual slot 0 of the LogicalOr vtable: calls the BinaryExpressionNode destructor body 00bc8500, calls 00401c00(this, 0) when flag bit 0 of the argument is set… |
| `00bc9c40` | ExpressionNodeLogicalOr_AppendText | confirmed | Virtual slot 2 (thiscall, one stack argument): calls slot 2 of the left operand (this+8), appends the literal DAT_01825f30 (' \|\| ') through 00402770, then ca… |

**Requirements::ExpressionNodeLogicalXOr<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9c70` | ExpressionNodeLogicalXOr_Ctor | confirmed | Constructor: runs the BinaryExpressionNode constructor 00bc9320 on the receiver with the two stack arguments, then installs the ExpressionNodeLogicalXOr vtable… |
| `00bc9ce0` | ExpressionNodeLogicalXOr_Evaluate | confirmed | Virtual slot 3 (thiscall, three stack arguments, ret 0xc): evaluates the left operand (this+8) with the arguments (p2, p3, p4); the right operand (this+0xc) is… |
| `00bc9d90` | ExpressionNodeLogicalXOr_ScalarDeletingDtor | confirmed | Virtual slot 0 of the LogicalXOr vtable: calls the BinaryExpressionNode destructor body 00bc8500, calls 00401c00(this, 0) when flag bit 0 of the argument is se… |
| `00bc9df0` | ExpressionNodeLogicalXOr_AppendText | confirmed | Virtual slot 2 (thiscall, one stack argument): calls slot 2 of the left operand (this+8), appends the literal DAT_01826528 (' ^^ ') through 00402770, then call… |

**Requirements::InvalidExpressionNode<ValidationData>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc9730` | InvalidExpressionNode_Ctor | confirmed | Constructor: runs the ExpressionNode base constructor 00bbe840, installs the InvalidExpressionNode vtable, constructs an embedded StringFixed<8> at this+8 (008… |
| `00bc97c0` | InvalidExpressionNode_DtorBody | confirmed | Destructor body: installs the InvalidExpressionNode vtable, calls 00792630 on the embedded message member, then installs the ExpressionNodeBase and SoeUtil::Re… |
| `00bc9830` | InvalidExpressionNode_AppendMessage | confirmed | Virtual slot 2 (thiscall, one stack argument, ret 4): appends the message buffer pointer at this+0xc to the output string argument through 00402ff0 (IStringApp… |
| `00bc9850` | InvalidExpressionNode_ScalarDeletingDtor | confirmed | Virtual slot 0 of the InvalidExpressionNode vtable: calls destructor body 00bc97c0 on the receiver, calls 00401c00(this, 0) when flag bit 0 of the argument is … |

**Requirements::LogicalInfixExpressionTreeBuilder<ValidationData>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc4af0` | Ctor | confirmed | Constructor: stores the LogicalInfixExpressionTreeBuilder vtable at +0, zeroes the words at +4, +8, +0xc, stores the SoeUtil::List<Requirements::ExpressionNode… |

**Requirements::RequirementManager<ValidationData>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc7e50` | Ctor | confirmed | Constructor: calls 00bc4af0 (LogicalInfixExpressionTreeBuilder constructor) on ECX, stores the Requirements::RequirementManager<ValidationData> vtable at +0, t… |
| `00bcbf50` | RequirementManager_DtorBody | confirmed | Destructor body: installs the RequirementManager vtable, releases each reference-counted entry on the list whose head is the dword at this+0x4864 (refcount at … |
| `00bcc000` | RequirementManager_ScalarDeletingDtorSlot1 | confirmed | Virtual slot 1 of the RequirementManager vtable (not slot 0): calls the destructor body 00bcbf50 on the receiver, calls 00401c00(this, 0) when flag bit 0 of th… |

**Requirements::UnaryExpressionNode<ValidationData>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bc8590` | UnaryExpressionNode_DtorBody | confirmed | Destructor body: installs the UnaryExpressionNode vtable, releases its single RefCounted operand reference at +8 (refcount at +4 decremented under LOCK; at zer… |
| `00bc8e20` | UnaryExpressionNode_ScalarDeletingDtor | confirmed | Virtual slot 0 of the UnaryExpressionNode vtable: calls the unary destructor body 00bc8590, calls 00401c00(this, 0) when flag bit 0 of the argument is set, and… |
| `00bc9390` | UnaryExpressionNode_Ctor | confirmed | Constructor: runs the ExpressionNode base constructor 00bbe840 under an SEH frame, installs the UnaryExpressionNode vtable and stores its stack argument as the… |

**RuleSetDB** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0129f6d0` | RuleSetDB_LoadRuleIndex | likely | Asserts the id is not -1 (012f5a60, 'v != -1' at RuleSetDB.cpp line 0x67). Looks the id up in the tree at this+0x10; when the id is absent or its flag byte (01… |
| `0129f9e0` | RuleSetDB_EraseEntry | likely | Asserts the id is not -1 (012f5a60, 'v != -1' at RuleSetDB.cpp line 0xc2), opens the store through 0129f420, takes the holder for the id through 0129edb0, find… |
| `0129fab0` | RuleSetDB_RegisterEntry | likely | Asserts the id is not -1 (012f5a60, 'v != -1' at RuleSetDB.cpp line 0x101), opens the store through 0129f420, calls virtual slot 17 (+0x44) of the object in th… |
| `0129fbf0` | RuleSetDB_CollectKeys | likely | Asserts the id is not -1 (012f5a60, 'v != -1' at RuleSetDB.cpp line 0x16f), opens the store through 0129f420, then walks the holder for the id (0129f1b0) from … |
| `0129fe50` | RuleSetDB_SetIdAndLoad | likely | Stores the id in this+0 (the default constant 0x9f3 when the argument is -1), then calls RuleSetDB_LoadRuleIndex (0129f6d0) with the same this. |
| `0129fea0` | RuleSetDB_GetOrLoadRuleSet | likely | Asserts the second argument is not -1 (012f5a60, 'v != -1' at RuleSetDB.cpp line 0xd0), opens the store through 0129f420, then looks up the first-argument key … |
| `012a01a0` | RuleSetDB::SaveArchetype | likely | Asserts param != -1 (RuleSetDB.cpp line 0x120). Appends the id and a __time64 low word twice each through 013c23f0 as a header, walks trees obtained from 0129f… |

**RuleSetData** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c5920` | RuleSetData::RuleSetData | confirmed | Stores PersistentBase::vftable, then PersistentComponent::vftable, then RuleSetData::vftable into the object, and zeroes the words at +4 (RuleSetID) and +8 (ex… |
| `013c59b0` | RuleSetData::GetTypeName | confirmed | Returns the constant string 'RuleSetData' (vtable slot [11]). |
| `013c59f0` | RuleSetData::DtorBody | confirmed | Installs RuleSetData::vftable, and when the expression-tree pointer at +8 is non-null calls its deleting destructor through slot [0] with flag 1 and clears the… |
| `013c5a90` | RuleSetData::FormatDebug | confirmed | Writes '(RuleSetData' with 01385610, calls vtable +0x38 of the expression tree at +8 with (param_2, param_3), writes ') => %s' with 'true' or 'false' chosen fr… |
| `013c5c80` | RuleSetData::~RuleSetData | confirmed | Scalar deleting destructor: calls the RuleSetData destructor body 013c59f0, frees the object when bit 0 of the argument is set, and returns this. Vtable slot [… |
| `013c5a70` | RuleSetData::SetExpressionTree | likely | Stores its argument word at the object's +8 (the expression-tree pointer). Vtable slot [19]. |
| `013c5a80` | RuleSetData::GetExpressionTree | likely | Returns the word at the object's +8 (the expression-tree pointer). Vtable slot [20]. |
| `013c5b50` | RuleSetData::GetRuleSetID | likely | Returns the word at the object's +4. Vtable slot [17]. |
| `013c5b60` | RuleSetData::SetRuleSetID | likely | Stores its argument word at the object's +4 (RuleSetID). Vtable slot [18]. |

**SoeGems::ObjectFactory<IRequirement,int>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbd4d0` | ObjectFactory::InitRequirementFactory | confirmed | Installs the factory and requirement-to-pool map vtables, clears the 0x20000-byte pool map, initializes the embedded creator and pool maps on their subobjects,… |

**SoeGems::ObjectFactory<IRequirement,int>::BasePool** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb9e50` | BasePool::~BasePool | confirmed | Replaces the object's vtable with BasePool::vftable and returns; EH unwind funclets call this cleanup routine. |

**SoeGems::ObjectFactory<IRequirement,int>::ICreator** (47)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bb9e40` | ICreator::~ICreator | confirmed | Replaces the object's vtable with the ICreator base vtable and returns; EH unwind funclets call this cleanup routine. |
| `00bbb4b0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb520` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb590` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb600` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb670` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb6e0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb750` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb7c0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb830` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb8a0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb910` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb980` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbb9f0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbba60` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbbad0` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbbb40` | ICreator::InitRegisteredPool | confirmed | Writes the specialized creator vtable, stores the two supplied values at offsets 4 and 8, and returns. |
| `00bbbbe0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbc30` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbc80` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbcd0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbd20` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbd70` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbdc0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbe10` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbe60` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbeb0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbf00` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbf50` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbfa0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbbff0` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbc040` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbc090` | ICreator::Init | confirmed | Writes the ICreator vtable into the receiver and returns. |
| `00bbd6e0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd740` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd7a0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd800` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd860` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd8c0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd920` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd980` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbd9e0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbda40` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbdaa0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbdb00` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbdb60` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |
| `00bbdbc0` | ICreator::~ICreator | confirmed | Restores the generic ICreator vtable, frees the receiver with alignment 0 only when flag bit 0 is set, and restores the SEH frame. |

**SoeGems::ObjectFactory<IRequirement,int>::InternalRegister<SoeUtil::RefObjectPool<ClientRequirementCheckCurrentProfile,0>,ClientRequirementCheckCurrentProfile>::__l2::MyCreator** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bccdb0` | MyCreatorGetOrCreate_ClientRequirementCheckCurrentProfile | confirmed | Virtual slot 1 of the local creator class: returns the cached object at receiver+8 through 00bcb4e0 when present; otherwise allocates a 0x60-byte block, constr… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementAlwaysFails,SoeUtil::ObjectPool<ClientRequirementAlwaysFails,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbb10` | PoolWrapper_ClientRequirementAlwaysFails_ScalarDeletingDtor | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc5fa0, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementAlwaysFails,SoeUtil::RefObjectPool<ClientRequirementAlwaysFails,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb7e0` | PoolWrapper_ClientRequirementAlwaysFails_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementAlwaysFails, RefObjectPool> vtable at +0, then calls 00bc42e0 to initialize… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCountry,SoeUtil::RefObjectPool<ClientRequirementCheckCountry,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb630` | PoolWrapper_ClientRequirementCheckCountry_Ctor | confirmed | Constructor: installs the SoeGems::ObjectFactory<IRequirement,int>::BasePool vtable and then the PoolWrapper<ClientRequirementCheckCountry, RefObjectPool> vtab… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCurrentProfile,SoeUtil::ObjectPool<ClientRequirementCheckCurrentProfile,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbb50` | PoolWrapper_ClientRequirementCheckCurrentProfile_ScalarDele… | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc6060, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckCurrentProfile,SoeUtil::RefObjectPool<ClientRequirementCheckCurrentProfile,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb900` | PoolWrapper_ClientRequirementCheckCurrentProfile_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckCurrentProfile, RefObjectPool> vtable at +0, then calls 00bc42e0 to in… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLanguage,SoeUtil::ObjectPool<ClientRequirementCheckLanguage,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbad0` | PoolWrapper_ClientRequirementCheckLanguage_ScalarDeletingDt… | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc5ee0, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLanguage,SoeUtil::RefObjectPool<ClientRequirementCheckLanguage,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb6c0` | PoolWrapper_ClientRequirementCheckLanguage_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckLanguage, RefObjectPool> vtable at +0, then calls 00bc4270 to initiali… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank,SoeUtil::ObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbaf0` | PoolWrapper_ClientRequirementCheckLastPlayedProfileOfTypeRa… | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc5f40, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank,SoeUtil::RefObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb750` | PoolWrapper_ClientRequirementCheckLastPlayedProfileOfTypeRa… | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckLastPlayedProfileOfTypeRank, RefObjectPool> vtable at +0, then calls 0… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckNudge,SoeUtil::ObjectPool<ClientRequirementCheckNudge,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbb90` | PoolWrapper_ClientRequirementCheckNudge_ScalarDeletingDtor | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc6120, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckNudge,SoeUtil::RefObjectPool<ClientRequirementCheckNudge,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcba20` | PoolWrapper_ClientRequirementCheckNudge_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckNudge, RefObjectPool> vtable at +0, then calls 00bc42e0 to initialize … |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckRenamePending,SoeUtil::ObjectPool<ClientRequirementCheckRenamePending,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbb70` | PoolWrapper_ClientRequirementCheckRenamePending_ScalarDelet… | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc60c0, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckRenamePending,SoeUtil::RefObjectPool<ClientRequirementCheckRenamePending,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb990` | PoolWrapper_ClientRequirementCheckRenamePending_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckRenamePending, RefObjectPool> vtable at +0, then calls 00bc42e0 to ini… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckVipRank,SoeUtil::ObjectPool<ClientRequirementCheckVipRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcbb30` | PoolWrapper_ClientRequirementCheckVipRank_ScalarDeletingDtor | confirmed | Virtual slot 0 (30 bytes): calls the BasePool destructor body 00bc6000, then _free(receiver) when flag bit 0 is set; returns the receiver. Scalar deleting dest… |

**SoeGems::ObjectFactory<IRequirement,int>::PoolWrapper<ClientRequirementCheckVipRank,SoeUtil::RefObjectPool<ClientRequirementCheckVipRank,0>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcb870` | PoolWrapper_ClientRequirementCheckVipRank_Ctor | confirmed | Constructor: installs the BasePool vtable and then the PoolWrapper<ClientRequirementCheckVipRank, RefObjectPool> vtable at +0, then calls 00bc42e0 to initializ… |

**SoeUtil::RefCounted** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bbd0f0` | RefCounted::~RefCounted | confirmed | Restores the ExpressionNodeBase and RefCounted vtables; if flag bit 0 is set, frees the receiver with alignment 0, then restores the SEH frame. |
| `00bbd160` | RefCounted::~RefCounted | confirmed | Restores the IRequirement, ExpressionNodeBase, and RefCounted vtables in sequence, then returns. |
| `00bbd2f0` | RefCounted::~RefCounted | confirmed | Restores the IRequirement, ExpressionNodeBase, and RefCounted vtables; if flag bit 0 is set, frees the receiver with alignment 0, then restores the SEH frame. |

**SoeUtil::RefObjectPool<ClientRequirementAlwaysFails,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcab20` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckCountry,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcaac0` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6690 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckCurrentProfile,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcab60` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckLanguage,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcaae0` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6690 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckLastPlayedProfileOfTypeRank,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcab00` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckNudge,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcaba0` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckRenamePending,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcab80` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SoeUtil::RefObjectPool<ClientRequirementCheckVipRank,0>::Node** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bcab40` | RefObjectPoolNodeRecycle | confirmed | Virtual slot 1 of the pool node: calls the node's own slot 0 with argument 0 (destroy without freeing), then calls 00bc6970 with ECX set to the owning pool at … |

**SumExpression** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0148dcd0` | SumExpression::SumExpression | confirmed | ECX = this, no stack args, returns this. Calls base constructor 0149adc0 with ECX = this, stores SumExpression::vftable at [this], zeroes [this+0x24] and [this… |
| `0148dd50` | SumExpression::TypeName | confirmed | No arguments. Returns the address of the literal SumExpression (mov eax,0x18f6c78; ret). |
| `0148dd90` | SumExpression::DtorBody | confirmed | __fastcall, this in ECX. Stores SumExpression::vftable at [this], then calls base destructor 0149ae80 with ECX = this, and returns. It frees nothing and destro… |
| `0148de00` | SumExpression::FormatDebugTyped | confirmed | Virtual slot 15, two stack args (the second is the stream; the first is not read by the pseudocode), ret 8, returns the bool in al. Writes '(gvt: sum' through … |
| `0148dd60` | SumExpression::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5dc (mov eax,[0x1cbe5dc]; ret). |
| `0148dd70` | SumExpression::SetStaticDword | likely | One stack arg, ret 4. Stores the argument into the dword global DAT_01cbe5dc. |

**TargetQueryNode** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01489270` | TargetQueryNode::TargetQueryNode | confirmed | ECX = this, no stack args, returns this. Calls base constructor 01494290 with ECX = this, stores TargetQueryNode::vftable at [this], then zeroes [this+0x1c], t… |
| `01489300` | TargetQueryNode::TypeName | confirmed | No arguments. Returns the address of the literal TargetQueryNode (mov eax,0x18f5f0c; ret). |
| `01489340` | TargetQueryNode::DtorBody | confirmed | __fastcall, this in ECX, no stack args. Stores TargetQueryNode::vftable at [this]. If [this+0x24] is non-null it calls that object's vtbl[0] with flag 1 and cl… |
| `01489790` | TargetQueryNode::ScalarDeletingDtor | confirmed | Thiscall, one stack arg (flag byte), ret 4. Calls 01489340 (TargetQueryNode dtor body) on this. If bit 0 of the flag is set, frees this through the free thunk … |
| `01489950` | TargetQueryNode::CheckTarget | confirmed | Virtual slot 23, three stack args (candidate, arg2, trace sink), ret 0xc, returns a bool. Asserts the candidate is non-null (TargetQueryNode.cpp line 0xa0). Tr… |
| `01489ff0` | TargetQueryNode::FindValidPlayerCards | confirmed | Virtual slot 20, three stack args, ret 0xc, returns 1. Traces 'Finding valid playercards.' to the trace sink. Calls 01381380 and 01347e40 to get a 4-byte-eleme… |
| `0148d960` | TargetQueryNode::EvaluateTarget2 | confirmed | Virtual slot 22, one stack arg (trace sink), ret 4, returns a card pointer or 0. Copies two words of the Target2 child at +0x28 and of the Target1 child at +0x… |
| `0148db70` | TargetQueryNode::EvaluateTarget1 | confirmed | Virtual slot 21, one stack arg (trace sink), ret 4, returns a card pointer or 0. Copies two words of the Target1 child at +0x24 (013f7b10 on the child at +0xc … |
| `01489310` | TargetQueryNode::GetStaticDword | likely | No arguments. Returns the dword global DAT_01cbe5d8 (mov eax,[0x1cbe5d8]; ret). |
| `01489320` | TargetQueryNode::SetStaticDword | likely | One stack arg, ret 4. Stores the argument into the dword global DAT_01cbe5d8. |
| `014893d0` | TargetQueryNode::SetAltPlayArea | likely | Thiscall, one stack arg (value pointer), ret 4. If the argument is null, asserts value at TargetQueryNode.cpp line 0x2a through 012f5a60. Allocates 0xc bytes (… |
| `01489710` | TargetQueryNode::Factory | likely | No args. Calls 00d833a6 (operator new) with 0x34. When non-null it calls 01489270 (TargetQueryNode constructor) with ECX = the block and returns that value; ot… |
| `01489770` | TargetQueryNode::RegisterFactory | likely | No args. Pushes factory 01489710 and the constant 0x4b (75), calls 013ce130 (the caller does not pop its arguments), then passes the returned registry in ECX t… |

**ValueNode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014882c0` | ValueNode::ValueNode | confirmed | Constructor: calls the expression-tree node base constructor 0149ab30, stores the ValueNode vtable, runs the child initialiser 012fa910 for the embedded child … |
| `01488350` | ValueNode::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'ValueNode'. |
| `01488360` | ValueNode::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5d4. |
| `01488370` | ValueNode::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5d4. |
| `01488390` | ValueNode::~ValueNode | confirmed | Destructor body without free: stores the ValueNode vtable, zeroes the words at +0x1c, +0x20, +0x24 and +0x28, runs 01300cd0, then calls the expression-tree nod… |
| `01488770` | ValueNode::GetTarget | confirmed | Two stack args (ret 8); ECX is not used as this. Traces 'ValueNode::getTarget(%d)' with arg1 to the sink arg2 (013846b0). Then calls 01381380 with ECX = arg2 a… |
| `014889c0` | ValueNode::ScalarDeletingDtor | confirmed | Thiscall, one stack arg (flag byte), ret 4. Calls 01488390 (the ValueNode destructor body) on this. If bit 0 of the flag is set, frees this through the free th… |
| `014889e0` | ValueNode::FormatDebugTyped | confirmed | Virtual slot 15, two stack args (stream, context pair), ret 8. Writes '(gvt:ValueNode' to the stream through 01385610. If this+0x24 equals 0x42, traces 'False … |
| `01488940` | ValueNode::Factory | likely | No args. Calls 00d833a6 (operator new) with 0x2c. When the result is non-null it calls 014882c0 with ECX = the block and returns that call's value; otherwise r… |
| `014889a0` | ValueNode::RegisterFactory | likely | No args. Pushes the factory address 01488940 and the constant 0x28, calls 013ce130 (the caller does not pop its arguments), then passes the returned registry i… |

**WhileExpression** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01487ec0` | WhileExpression::WhileExpression | confirmed | Constructor: calls the expression base constructor 0149adc0, then stores the WhileExpression vtable. |
| `01487f40` | WhileExpression::TypeName | confirmed | Virtual (vtable slot 11) returning the literal 'WhileExpression'. |
| `01487f50` | WhileExpression::GetClassStatic | confirmed | Virtual (vtable slot 12) returning the per-class static dword DAT_01cbe5d0. |
| `01487f60` | WhileExpression::SetClassStatic | confirmed | Virtual (vtable slot 13) storing its argument in the per-class static dword DAT_01cbe5d0. |
| `01487f80` | WhileExpression::~WhileExpression | confirmed | Destructor body without free: stores the WhileExpression vtable, then calls the expression base destructor body 0149ae80. |
| `01487fd0` | WhileExpression::TraceFalse | confirmed | Virtual (vtable slot 15): writes the trace text '(gvt: while' and ') => false' to its argument through 01385610 and 01384ee0, and returns 0. |
| `01488110` | WhileExpression::~WhileExpression | confirmed | Scalar deleting destructor (vtable slot 0): stores the WhileExpression vtable, calls the expression base destructor body 0149ae80, and frees the block with _fr… |
| `01488170` | WhileExpression::Evaluate | confirmed | Virtual (vtable slot 14). Traces '(While', then repeats while the condition child at +0x1c (its slot 14 with the trace stream) returns true, for at most 1000 i… |
| `01488090` | WhileExpression::Factory | likely | Allocates 0x24 bytes with operator new (00d833a6), calls the WhileExpression constructor 01487ec0 on the block when the allocation succeeds, and returns the ob… |

**(no class)** (59)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01301590` | ValueData_EvalExpressionIntoSelf | confirmed | Thiscall (ECX = ValueData). Requires the type tag at +4 to be 5 (the assert 'mTypeID == kExpressionTreeTypeID' otherwise). Constructs a temporary ValueData, ca… |
| `008adff0` | Wrapper_00adac80 | likely | Sets up an SEH frame and calls FUN_00adac80 with incoming ECX unchanged; restores ExceptionList and returns. The callee releases reference-counted IString memb… |
| `00bc9610` | TokenKindFromOperatorText | likely | Assigns the text argument to the string member at this+8 (00894b10 with the length argument when it is >= 0, otherwise 00402ce0 for a NUL-terminated source), t… |
| `00bc98c0` | PoolRecord36_Init | likely | Initializes a pooled 36-byte record: zeroes the dwords at +0 and +4, constructs a StringFixed<4> at +8 through 00bc92c0, and sets the dword at +0x20 to 1; retu… |
| `00bc99f0` | BinaryExpressionNode_DtorCleanup | likely | Unwind-time cleanup wrapper: installs an SEH frame, calls the BinaryExpressionNode destructor body 00bc8500 on ECX (the receiver), then restores the frame and … |
| `00bc9b90` | BinaryExpressionNode_DtorCleanup | likely | Unwind-time cleanup wrapper, same code as 00bc99f0: installs an SEH frame, calls the BinaryExpressionNode destructor body 00bc8500 on ECX, then restores the fr… |
| `00bc9d40` | BinaryExpressionNode_DtorCleanup | likely | Unwind-time cleanup wrapper, same code as 00bc99f0: installs an SEH frame, calls the BinaryExpressionNode destructor body 00bc8500 on ECX, then restores the fr… |
| `00bc9eb0` | UnaryExpressionNode_DtorCleanup | likely | Unwind-time cleanup wrapper: installs an SEH frame, calls the UnaryExpressionNode destructor body 00bc8590 on ECX (the receiver), then restores the frame and r… |
| `00bca000` | UnaryExpressionNode_DtorCleanup | likely | Unwind-time cleanup wrapper, same code as 00bc9eb0: installs an SEH frame, calls the UnaryExpressionNode destructor body 00bc8590 on ECX, then restores the fra… |
| `00bca270` | PoolAllocRecord36 | likely | Allocates one element through 00bca0e0 from the pool object whose address the code loads into ECX; when the allocation succeeds it initializes the element with… |
| `00bca2d0` | ExprReduceLogicalAnd | likely | Reduces the two most recent operand entries of the list at this+0x482c (head +0x482c, tail +0x4830, count +0x4834): pops the tail twice, calling slot 3 of the … |
| `00bca410` | ExprReduceLogicalOr | likely | Same reduction as 00bca2d0 for the \|\| operator: pops two operand entries from the list at this+0x482c, releases them through slot 3 of the object at this+0x4… |
| `00bca550` | ExprReduceLogicalXOr | likely | Same reduction as 00bca2d0 for the ^^ operator: pops two operand entries from the list at this+0x482c, releases them through slot 3 of the object at this+0x482… |
| `00bca690` | ExprReduceLogicalNot | likely | Reduces the most recent operand entry of the list at this+0x482c: pops the tail, releases it through slot 3 of the object at this+0x4828, allocates a 12-byte n… |
| `00bca790` | ExprReduceGrouping | likely | Reduces the most recent operand entry of the list at this+0x482c into a grouping node: pops the tail, releases it through slot 3 of the object at this+0x4828, … |
| `00bca900` | PoolRecord36_Destroy | likely | Destroys a 36-byte pool element (stack argument): runs the IStringFixed<char,4> destructor body 00bc9150 on element+8, then returns the element to the pool thr… |
| `00bcae30` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xa801bea8: finds or creates the 0x18-byte PoolWrapper for LastPlayedProfileOfTypeRank (00bc5600), inserts it through 00bc1090 and… |
| `00bcaef0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xac01bea8: finds or creates the 0x18-byte PoolWrapper for ClientRequirementAlwaysFails (00bc5b10), inserts it through 00bc11b0 an… |
| `00bcafb0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb001bea8: finds or creates the 0x18-byte PoolWrapper for ClientRequirementCheckVipRank (00bc5bc0), inserts it through 00bc12d0 a… |
| `00bcb070` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb401bea8: finds or creates the 0x18-byte PoolWrapper for ClientRequirementCheckCurrentProfile (00bc5c70), inserts it through 00b… |
| `00bcb130` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb801bea8: finds or creates the 0x18-byte PoolWrapper for ClientRequirementCheckRenamePending (00bc5d20), inserts it through 00bc… |
| `00bcb1f0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xbc01bea8: finds or creates the 0x18-byte PoolWrapper for ClientRequirementCheckNudge (00bc5dd0), inserts it through 00bc1630 and… |
| `00bcb2b0` | GuardedPoolNodeInit | likely | Calls the guard 00bc38b0 (SEH-framed wrapper around 00fc45f0 and 00bc17a0); when the guard returns nonzero it constructs the pool node for ClientRequirementChe… |
| `00bcb320` | GuardedPoolNodeInit | likely | Guard 00bc38b0 then pool node construction through 00bc6790 for ClientRequirementCheckLanguage; returns the node or 0. |
| `00bcb390` | GuardedPoolNodeInit | likely | Guard 00bc3930 (SEH-framed wrapper around 00fc45f0 and 00bc16c0) then pool node construction through 00bc68c0 for ClientRequirementCheckLastPlayedProfileOfType… |
| `00bcb400` | GuardedPoolNodeInit | likely | Guard 00bc3930 then pool node construction through 00bc6a70 for ClientRequirementAlwaysFails; returns the node or 0. |
| `00bcb470` | GuardedPoolNodeInit | likely | Guard 00bc3930 then pool node construction through 00bc6ba0 for ClientRequirementCheckVipRank; returns the node or 0. |
| `00bcb4e0` | GuardedPoolNodeInit | likely | Guard 00bc3930 then pool node construction through 00bc6cd0 for ClientRequirementCheckCurrentProfile; returns the node or 0. |
| `00bcb550` | GuardedPoolNodeInit | likely | Guard 00bc3930 then pool node construction through 00bc6e00 for ClientRequirementCheckRenamePending; returns the node or 0. |
| `00bcb5c0` | GuardedPoolNodeInit | likely | Guard 00bc3930 then pool node construction through 00bc6f30 for ClientRequirementCheckNudge; returns the node or 0. |
| `00bcbeb0` | ExprListPairCtor | likely | Calls 00bc7600 on the receiver (speculatively the two-expression-list construction: 00bc49b0 then 00bc2630 with no visible arguments) and returns the receiver … |
| `00bcbf00` | ExprListsDtorCleanup | likely | Unwind-time cleanup wrapper: calls 00bc7660 on a cookie-tagged receiver (the expression-list object at ECX), which destroys the List<IRequirement*,5> through 0… |
| `00bcc040` | ExprFeedToken | likely | Skips leading spaces in the text argument; when characters remain it allocates a token record (00bca270), classifies the text (00bc9610 with the length), and r… |
| `00bcc090` | RegisterRequirementCreatorCountry | likely | When the flag dword at this+0x2202c is nonzero it obtains the Country keyed pool (00bcacb0); allocates a 0xc-byte node, constructs its ICreator registration th… |
| `00bcc160` | RegisterRequirementCreatorLanguage | likely | Creator registration for the Language check: obtains the Language keyed pool (00bcad70) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initia… |
| `00bcc230` | RegisterRequirementCreatorLastPlayedProfileOfTypeRank | likely | Creator registration for the LastPlayedProfileOfTypeRank check: obtains its keyed pool (00bcae30) when the flag at this+0x2202c is set, allocates a 0xc-byte no… |
| `00bcc300` | RegisterRequirementCreatorAlwaysFails | likely | Creator registration for the AlwaysFails check: obtains its keyed pool (00bcaef0) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initializes … |
| `00bcc3d0` | RegisterRequirementCreatorVipRank | likely | Creator registration for the VipRank check: obtains its keyed pool (00bcafb0) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initializes it t… |
| `00bcc4a0` | RegisterRequirementCreatorCurrentProfile | likely | Creator registration for the CurrentProfile check: obtains its keyed pool (00bcb070) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initializ… |
| `00bcc570` | RegisterRequirementCreatorRenamePending | likely | Creator registration for the RenamePending check: obtains its keyed pool (00bcb130) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initialize… |
| `00bcc640` | RegisterRequirementCreatorNudge | likely | Creator registration for the Nudge check: obtains its keyed pool (00bcb1f0) when the flag at this+0x2202c is set, allocates a 0xc-byte node, initializes it thr… |
| `00bcc800` | GameTechLookupByName | likely | Looks up a requirement by name: first runs the string-keyed bucket search 00bcc7b0; when that fails it parses the name through 0072d5d0 and searches the 1024-b… |
| `00bcd020` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xa001bea8 (same list lookup as 00bcacb0): when absent it allocates 0xa8 bytes, constructs the RefObjectPool-backed Country pool w… |
| `00bcd0e0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xa401bea8: when absent constructs the 0xa8-byte RefObjectPool-backed Language pool wrapper through 00bcb6c0 and inserts it throug… |
| `00bcd1a0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xa801bea8: when absent constructs the 0xa8-byte RefObjectPool-backed LastPlayedProfileOfTypeRank pool wrapper through 00bcb750 an… |
| `00bcd260` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xac01bea8: when absent constructs the 0xa8-byte RefObjectPool-backed AlwaysFails pool wrapper through 00bcb7e0 and inserts it; re… |
| `00bcd320` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb001bea8: when absent constructs the 0xa8-byte RefObjectPool-backed VipRank pool wrapper through 00bcb870 and inserts it; return… |
| `00bcd3e0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb401bea8: when absent constructs the 0xa8-byte RefObjectPool-backed CurrentProfile pool wrapper through 00bcb900 and inserts it;… |
| `00bcd4a0` | KeyedLazyPoolGet | likely | Keyed lazy accessor with key 0xb801bea8: when absent constructs the 0xa8-byte RefObjectPool-backed RenamePending pool wrapper through 00bcb990 and inserts it; … |
| `01046650` | NegateIsUnlocked_01046650 | likely | One stack arg (ret 4); bytes 8b 44 24 04 50 e8 .. f6 d8 1b c0 83 c0 01 c2 04 00 = push arg; call 010463c0 (ECX is passed through, not set here); neg al; sbb ea… |
| `01360820` | GameActionSetEntry | likely | Thiscall with two stack arguments (ECX = object): asserts that the first argument is non-null (string action, Game.cpp line 0x1962 through 012f5a60) and the se… |
| `013c5c00` | CreateRuleSetData | likely | Allocates 0xc bytes with 00d833a6; when the allocation succeeds it constructs the object with 013c5920 and returns it, otherwise it returns 0. |
| `01459d80` | AllocAndConstruct_0x88 | likely | Allocates 0x88 bytes with 00d833a6 (operator new, cookie/EH frame present). When the block is non-null calls the default constructor 01459a20 on it and returns… |
| `01459df0` | RegisterFactory_222e7 | likely | Pushes factory 01459d80 and id 0x222e7 as stack arguments and calls 013ce130 (no caller cleanup); passes the returned registry in ECX to 013ce230 with no stack… |
| `0145a5f0` | AllocAndConstruct_0x124 | likely | Allocates 0x124 bytes with 00d833a6 (EH frame present). When the block is non-null calls the default constructor 0145a470 on it and returns the block plus 8 (t… |
| `0145a670` | RegisterFactory_222fb | likely | Pushes factory 0145a5f0 and id 0x222fb and calls 013ce130 (no caller cleanup); passes the returned registry in ECX to 013ce230 with no stack arguments. |
| `0145aa20` | AllocAndConstruct_0x144 | likely | Allocates 0x144 bytes with 00d833a6 (EH frame present). When the block is non-null calls the default constructor 0145a920 on it and returns the block plus 8; r… |
| `0145aaa0` | RegisterFactory_222fe | likely | Pushes factory 0145aa20 and id 0x222fe and calls 013ce130 (no caller cleanup); passes the returned registry in ECX to 013ce230 with no stack arguments. |
| `014878e0` | ArgTreeSetChildNode | likely | Sets the child node stored under an int key in the ArgTree map at +0x1c: asserts the child argument is non-null (ArgTree.cpp line 0xae, 'childNode'), looks up … |

### `game.sort`

**SearchCriteria** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c73a0` | SearchCriteria::MatchSortable | likely | Thiscall with one stack arg (sortable object, asserted non-null with SortService.cpp line 0x1e6). Gets a ValueData from the sortable through its vtable slot 2 … |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012c8a20` | CastSortablesToArchetypes | likely | Thiscall on a result dword vector with one stack arg: a source vector of Sortable pointers. Clears the result, then for each element calls 00d8d382 (__RTDynami… |
| `012c8b50` | FilterSortablesByCriteria | likely | Stdcall with three stack args (source sortables, output list, criteria vector), ret 0xc. Copies the source into a local list. For each criterion in the criteri… |
| `012c9b40` | SortListByCriteriaReverse | likely | Stdcall with three stack args, returns 1. Walks the criteria vector from the last element to the first, asserting each is non-null (SortService.cpp, criterion)… |

### `game.statemachine`

**ActionPlayedState** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0147e830` | ActionPlayedState::getWhatCardSelected | confirmed | Thiscall with no arguments. Passes the literal 'ActionPlayedState::getWhatCardSelected' to 0145c4d0 and 013508b0. If the dword at this+0x100 is zero and the by… |
| `0147e910` | ActionPlayedState::getWhatCardSelected2 | confirmed | Same shape as 0147e830 with literal 'ActionPlayedState::getWhatCardSelected2': tests the dword at this+0x104 and the byte at this+0x161, sets the byte at this+… |
| `0147ef00` | ActionPlayedState::DestructBody | confirmed | Stores ActionPlayedState::vftable into [this]; when the pointer at this+0x174 is non-null, frees it with _free; zeroes the dwords at this+0x174, +0x178 and +0x… |
| `0147efc0` | ActionPlayedState::GetClassName | confirmed | Returns the address of the literal 'ActionPlayedState' (mov eax,018f5074; ret). No arguments and no object access. |
| `0147f000` | ActionPlayedState::~ActionPlayedState | confirmed | Scalar deleting destructor: calls the dtor body 0147ef00 with this, then calls _free (thunk 00d82463) on this when bit 0 of its stack byte argument is set; ret… |
| `0147f3f0` | ActionPlayedState::ActionPlayedState | confirmed | Fastcall no-argument constructor: calls 0147c980 (CancellableStateMachineState constructor) with this, stores ActionPlayedState::vftable at [this], constructs … |
| `0147f560` | ActionPlayedState::ActionPlayedStateArg | confirmed | Thiscall constructor with one stack argument: calls base constructor 0147c9f0 with that argument, stores ActionPlayedState::vftable at [this], constructs two m… |
| `01482820` | ActionPlayedState::getWhatCardSelected2WithText | confirmed | Virtual (ActionPlayedState slot 55). If mTargetID2 (+0x104) is zero and AllowGetTarget (+0x161) is set: sets mNeedTarget2 (+0x10d) to 1, mCurrentState (+0x114)… |
| `01482a50` | ActionPlayedState::getWhatCardSelectedWithText | confirmed | Virtual (ActionPlayedState slot 52). If mTargetID (+0x100) is zero and AllowGetTarget (+0x161) is set: sets mNeedTarget (+0x10c) to 1, mCurrentState (+0x114) t… |
| `0147e790` | ActionPlayedState::ForwardSlot61_SetFlag198 | likely | Thiscall with no arguments. If this+4 (mPreviousState) is non-null, calls its vtable slot 61 (offset 0xf4) and stores whether the result equals 1 as a byte at … |
| `0147e7c0` | ActionPlayedState::GetDword_f8_slot50 | likely | Thiscall getter with no arguments: returns the dword at this+0xf8 (mov eax,[ecx+0xf8]; ret). |
| `0147e7d0` | ActionPlayedState::GetDword_f8_slot46 | likely | Thiscall getter with no arguments: returns the dword at this+0xf8 (mov eax,[ecx+0xf8]; ret). Same body as 0147e7c0. |
| `0147e7e0` | ActionPlayedState::GetDword_fc_slot47 | likely | Thiscall getter with no arguments: returns the dword at this+0xfc (mov eax,[ecx+0xfc]; ret). |
| `0147e7f0` | ActionPlayedState::ForwardSlot62OrReturnFc | likely | Thiscall with one byte stack argument (ret 4). If this+4 is non-null and the argument equals 1, overwrites the stack argument with 0 and tail-jumps to slot 62 … |
| `0147e820` | ActionPlayedState::GetDword_108_slot48 | likely | Thiscall getter with no arguments: returns the dword at this+0x108 (mov eax,[ecx+0x108]; ret). |
| `0147edd0` | ActionPlayedState::HandleModeCommand | likely | Thiscall with two stack arguments (id, mode). Returns 2 unless this+0x114 equals 0xe, this+0x120 is non-zero and the id equals 01344e20(). Then: mode 1 sets th… |
| `0147efd0` | ActionPlayedState::GetGlobal_01cbe5b4 | likely | Returns the current value of global DAT_01cbe5b4 (mov eax,[01cbe5b4]; ret). No this access. |
| `0147efe0` | ActionPlayedState::SetGlobal_01cbe5b4 | likely | Stores its single stack argument into global DAT_01cbe5b4 and returns with ret 4. No this access. |
| `0147f020` | ActionPlayedState::HandleYesNoChoice | likely | Thiscall with three stack arguments (id, mode, object). Returns 2 unless this+0x114 equals 0xe, this+0x120 is non-zero and the id equals 01344e20(). It builds … |

**AlternateMultiActionState** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0147cb00` | AlternateMultiActionState::AlternateMultiActionState | confirmed | Thiscall no-argument constructor: calls base constructor 0146ad00 with ECX = this, stores AlternateMultiActionState::vftable at [this], constructs the member o… |
| `0147cb90` | AlternateMultiActionState::GetClassName | confirmed | Returns the address of the literal 'AlternateMultiActionState' (mov eax; ret). No arguments and no object access. |
| `0147cbd0` | AlternateMultiActionState::AlternateMultiActionStateArg | confirmed | Thiscall constructor with one stack argument (ret 4): calls base constructor 0146ae30 with that argument and ECX = this, stores AlternateMultiActionState::vfta… |
| `0147cc40` | AlternateMultiActionState::DestructBody | confirmed | Stores AlternateMultiActionState::vftable into [this], calls 01401ed0 with ECX = this+0x178, then calls the base destructor body 0146ab40 with ECX = this. Does… |
| `0147ccb0` | AlternateMultiActionState::buttonPressed | confirmed | Thiscall handler with two stack arguments (ret 8). Traces the literal 'AlternateMultiActionState::buttonPressed' via 0145c4d0 and 013508b0. When the first argu… |
| `0147cd30` | AlternateMultiActionState::leaveState | confirmed | Thiscall with no arguments: passes the literal 'AlternateMultiActionState::leaveState' to 0145c4d0, calls 013508b0 on the result, calls 013f39c0 and returns 1. |
| `0147cd60` | AlternateMultiActionState::DeserializeState | confirmed | Thiscall reader with one stack argument: calls this->vtable[4](arg, &arg) (offset 0x10) and returns 0 when false; calls base reader 0146c3e0(arg) and returns 0… |
| `0147cdc0` | AlternateMultiActionState::SerializeStateWithTrace | confirmed | Thiscall writer with one stack argument: if the argument is 0 uses vtable slot 1 else slot 2 with it to obtain the writer; passes 'Starting AlternateMultiActio… |
| `0147ced0` | AlternateMultiActionState::~AlternateMultiActionState | confirmed | Scalar deleting destructor: calls the dtor body 0147cc40 with this, then calls _free (thunk 00d82463) on this when bit 0 of its stack byte argument is set; ret… |
| `0147cef0` | AlternateMultiActionState::nextState | confirmed | Virtual step (slot 73 of AlternateMultiActionState::vftable), no arguments, fastcall with ECX = this. Traces 'AlternateMultiActionState::nextState' via 0145c4d… |
| `0147dd30` | AlternateMultiActionState::restoreState | confirmed | Virtual step (slot 17 of AlternateMultiActionState::vftable), no arguments, ECX = this. Traces 'AlternateMultiActionState::restoreState' via 0145c4d0 and 01350… |
| `0147e270` | AlternateMultiActionState::enterState | confirmed | Virtual step (slot 14 of AlternateMultiActionState::vftable), no arguments, ECX = this. Calls slot 19 (offset 0x4c) of this, then reads keyed entries of the re… |
| `0147cba0` | AlternateMultiActionState::GetGlobal_01cbe5b0 | likely | Returns the current value of global DAT_01cbe5b0 (mov eax,[01cbe5b0]; ret). No this access. |
| `0147cbb0` | AlternateMultiActionState::SetGlobal_01cbe5b0 | likely | Stores its single stack argument into global DAT_01cbe5b0 and returns with ret 4. No this access. |

**CWActionPlayedState** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144ffc0` | CWActionPlayedState::GetClassName | confirmed | Returns the address of the literal CWActionPlayedState (mov eax, literal; ret), no arguments. |
| `01450000` | CWActionPlayedState::MoveCardBackToOrigin | confirmed | Thiscall with one stack argument (ret 4): traces the literal CWActionPlayedState::moveCardBackToOrigin with the argument via 0145c4d0 and 013508b0; if the argu… |
| `014505f0` | CWActionPlayedState::~CWActionPlayedState | confirmed | Scalar deleting destructor shape: stores CWActionPlayedState::vftable into [this], calls base body FUN_0147ef00 with this in ECX, then calls _free(this) only w… |
| `01450650` | CWActionPlayedState::HandleRandomReturn | confirmed | Fastcall (this in ECX, no stack arguments; 1140 bytes): traces the call; returns after 0147c810 when 01348640() is true; when [this+0xf4] is nonzero it handles… |
| `01450ad0` | CWActionPlayedState::CWActionPlayedState | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_0147f3f0 on this, stores CWActionPlayedState::vftable into [this], sets the dword at this+0x19c to 0… |
| `01450b50` | CWActionPlayedState::CWActionPlayedStateArg | confirmed | Thiscall ctor taking one stack argument: passes it to base ctor FUN_0147f560, stores CWActionPlayedState::vftable into [this], zeroes this+0x19c and the byte a… |
| `01450c60` | CWActionPlayedState::UpdateState | confirmed | Fastcall (this in ECX, no stack arguments; 6772 bytes): traces CWActionPlayedState::updateState; reads the card id at [this+0xf8] (validated via 01357a00 and a… |
| `0144ff50` | CWActionPlayedState::DestructBody | likely | Fastcall (this in ECX): stores CWActionPlayedState::vftable into [this], then calls the base body FUN_0147ef00 with this in ECX; takes no flag argument and doe… |
| `0144ffd0` | CWActionPlayedState::GetGlobal_01cbe4f8 | likely | Returns the current value of global DAT_01cbe4f8 (mov eax,[01cbe4f8]; ret); no this access. |
| `0144ffe0` | CWActionPlayedState::SetGlobal_01cbe4f8 | likely | Stores its single stack argument into global DAT_01cbe4f8 (mov eax,[esp+4]; mov [01cbe4f8],eax) and returns with ret 4; no this access. |
| `01450570` | CWActionPlayedState::SerializeStateWithTrace | likely | Thiscall with one stack argument (ret 4): if the argument is 0 obtains a stream from virtual slot [1]() else from slot [2](arg); traces Starting CWActionPlayed… |

**CWDoublePassState** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144f820` | CWDoublePassState::GetClassName | confirmed | Returns the address of the literal CWDoublePassState (mov eax, literal; ret), no arguments. |
| `0144f860` | CWDoublePassState::ActionPlayed | confirmed | Thiscall with three stack arguments (ret 12): calls 01321eb0 and 01321ea0 on param_4, logs CWDoublePassState::actionPlayed with the three values and the result… |
| `0144f8f0` | CWDoublePassState::CardPlayed | confirmed | Thiscall with two stack arguments (ret 8): logs CWDoublePassState::cardPlayed with the two values via 0145c4d0 and 013508b0, runs gate 0145c550 and 013f8020; i… |
| `0144fa20` | CWDoublePassState::RestoreStatePartTwo | confirmed | Logs the literal FRDOUBLEPASS::RESTORESTATEPARTTWO via 0145c4d0 and 013508b0, then tail-jumps (e9) to 01474040 with no further work here. |
| `0144fb10` | CWDoublePassState::~CWDoublePassState | confirmed | Scalar deleting destructor shape: stores CWDoublePassState::vftable into [this], calls base body FUN_01474c60 with this in ECX, then calls _free(this) only whe… |
| `0144fbb0` | CWDoublePassState::CWDoublePassState | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_01475230 on this, stores CWDoublePassState::vftable into [this], assigns the 17-char literal CWDoubl… |
| `0144fc20` | CWDoublePassState::CWDoublePassStateArg | confirmed | Thiscall ctor taking one stack argument (ret 4): passes it to base ctor FUN_01475300, stores CWDoublePassState::vftable into [this], assigns the 17-char litera… |
| `0144f7b0` | CWDoublePassState::DestructBody | likely | Fastcall (this in ECX): stores CWDoublePassState::vftable into [this], then calls the base body FUN_01474c60 with this in ECX; takes no flag argument and does … |
| `0144f830` | CWDoublePassState::GetGlobal_01cbe4f4 | likely | Returns the current value of global DAT_01cbe4f4 (mov eax,[01cbe4f4]; ret); no this access. |
| `0144f840` | CWDoublePassState::SetGlobal_01cbe4f4 | likely | Stores its single stack argument into global DAT_01cbe4f4 (mov eax,[esp+4]; mov [01cbe4f4],eax) and returns with ret 4; no this access. |
| `0144fa90` | CWDoublePassState::SerializeStateWithTrace | likely | Thiscall with one stack argument (ret 4): obtains a state object from virtual slot [1]() when the argument is 0, else virtual slot [2](arg); calls trace helper… |
| `0144fd20` | CWDoublePassState::RebuildOrderFromActive | likely | Virtual slot [75] of CWDoublePassState: searches the vector returned by 01347e40 for the value returned by 013f8020 and then appends that vector's elements to … |

**CWDrawCardState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144ec60` | CWDrawCardState::GetClassName | confirmed | Returns the address of the literal CWDrawCardState (mov eax,018ec724; ret), no arguments. |
| `0144ed30` | CWDrawCardState::~CWDrawCardState | confirmed | Scalar deleting destructor shape: stores CWDrawCardState::vftable into [this], calls the base body FUN_014713a0 with this in ECX, then calls _free(this) only w… |
| `0144ed90` | CWDrawCardState::CWDrawCardState | confirmed | Fastcall ctor with no stack arguments: calls base ctor FUN_01471b20 on this, stores CWDrawCardState::vftable into [this], assigns the 15-char literal CWDrawCar… |
| `0144ee00` | CWDrawCardState::CWDrawCardStateArg | confirmed | Thiscall ctor taking one stack argument (ret 4): passes it to base ctor FUN_01471bd0, stores CWDrawCardState::vftable into [this], assigns the 15-char literal … |
| `0144ebf0` | CWDrawCardState::DestructBody | likely | Stores CWDrawCardState::vftable into [this], then calls the base body FUN_014713a0 with this in ECX; takes no flag argument and does not free memory; returns v… |
| `0144ec70` | CWDrawCardState::GetGlobal_01cbe4f0 | likely | Returns the current value of global DAT_01cbe4f0 (mov eax,[01cbe4f0]; ret); no this access. |
| `0144ec80` | CWDrawCardState::SetGlobal_01cbe4f0 | likely | Stores its single stack argument into global DAT_01cbe4f0 (mov eax,[esp+4]; mov [01cbe4f0],eax) and returns with ret 4; no this access. |
| `0144ece0` | CWDrawCardState::SerializeStateWithTrace | likely | Thiscall with one stack argument (ret 4): obtains a state object from virtual slot [1]() when the argument is 0, else virtual slot [2](arg); calls trace helper… |
| `0144ef30` | CWDrawCardState::nextState | likely | Fastcall (this in ECX, no stack args): logs the literal CWDrawCardState::nextState, then branches on [this+0x108]: value 1 runs a setup block (sets the value t… |

**CWHandleResponseState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144e860` | CWHandleResponseState::TypeName | confirmed | Virtual slot [11]: returns the address of the literal CWHandleResponseState (mov eax,imm; ret). No arguments. |
| `0144e870` | CWHandleResponseState::ReadGlobal01cbe4ec | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4ec. No arguments. |
| `0144e880` | CWHandleResponseState::WriteGlobal01cbe4ec | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4ec and returns (ret 4). |
| `0144e8a0` | CWHandleResponseState::GetPushClass | confirmed | Virtual slot [73] (one class-id argument, ECX=this): traces CWHandleResponseState::getPushClass with the id; for id 0x16d allocates 0x128 bytes (00d833a6; retu… |
| `0144e990` | CWHandleResponseState::Read | confirmed | Virtual slot [9] (Read, one stream argument, returns the success byte): calls header read slot [4] with the stream and an out word; on success calls base read … |
| `0144e9d0` | CWHandleResponseState::Write | confirmed | Virtual slot [10] (Write, one stream argument): when the argument is 0 obtains a stream from slot [1] (+4); otherwise calls slot [2] (+8) with it; writes the S… |
| `0144ea20` | CWHandleResponseState::~CWHandleResponseState | confirmed | Scalar deleting destructor shape: stores CWHandleResponseState::vftable into [this], calls base body FUN_01412e60 with this in ECX, then calls _free(this) only… |
| `0144ea80` | CWHandleResponseState::CWHandleResponseState | confirmed | Thiscall ctor with no arguments: runs base ctor FUN_014136f0 on this, stores CWHandleResponseState::vftable into [this], assigns the 0x15-char literal CWHandle… |
| `0144eaf0` | CWHandleResponseState::CWHandleResponseStateArg | confirmed | Thiscall ctor taking one stack argument (ret 4): passes that argument to base ctor FUN_01413830, stores CWHandleResponseState::vftable into [this], assigns the… |

**CWMultiActionState** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144e400` | CWMultiActionState::CWMultiActionState | confirmed | Default constructor (ECX=this): calls the MultiActionState default-constructor body 0146ad00 under an EH frame, stores CWMultiActionState::vftable at +0, retur… |
| `0144e480` | CWMultiActionState::TypeName | confirmed | Virtual slot [11]: returns the address of the literal CWMultiActionState (mov eax,imm; ret). No arguments. |
| `0144e490` | CWMultiActionState::ReadGlobal01cbe4e8 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4e8. No arguments. |
| `0144e4a0` | CWMultiActionState::WriteGlobal01cbe4e8 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4e8 and returns (ret 4). |
| `0144e4c0` | CWMultiActionState::CWMultiActionState | confirmed | Overload constructor with one stack argument (ECX=this): calls MultiActionState::Ctor 0146ae30 with the argument, then stores CWMultiActionState::vftable at +0… |
| `0144e520` | CWMultiActionState::~CWMultiActionState | confirmed | Destructor body (ECX=this): stores CWMultiActionState::vftable at +0 and calls the MultiActionState destructor body 0146ab40 with a cookie argument. It does no… |
| `0144e570` | CWMultiActionState::Read | confirmed | Virtual slot [9] (Read, one stream argument, bool): calls header read slot [4] with the stream and an out word and returns false when it fails; calls base read… |
| `0144e5c0` | CWMultiActionState::Write | confirmed | Virtual slot [10] (Write, one stream argument): when the argument is 0 obtains a stream from slot [1] (+4); otherwise calls slot [2] (+8) with it; writes the S… |
| `0144e6a0` | CWMultiActionState::ScalarDeletingDtor | confirmed | Vtable slot [0] of CWMultiActionState: stores the vtable at +0, calls the MultiActionState destructor body 0146ab40, and when bit 0 of the flag argument is set… |
| `0144e610` | CWMultiActionState::Create | likely | Factory: allocates 0x178 bytes with operator new (00d833a6); when the allocation is non-null constructs the state with 0144e400 and returns it, otherwise retur… |
| `0144e680` | CWMultiActionState::RegisterFactory | likely | Registers the factory 0144e610 under classID 0x222f8: pushes the factory address and 0x222f8 and calls 013ce130, then moves its eax result into ECX and calls 0… |

**CWProcessEventEmissionState** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144d390` | CWProcessEventEmissionState::~CWProcessEventEmissionState | confirmed | Destructor body (ECX=this): stores CWProcessEventEmissionState::vftable at +0 and calls the base destructor 01464a50. It does not free the object. |
| `0144d400` | CWProcessEventEmissionState::TypeName | confirmed | Virtual slot [11]: returns the address of the literal CWProcessEventEmissionState (mov eax,imm; ret). No arguments. |
| `0144d410` | CWProcessEventEmissionState::ReadGlobal01cbe4e4 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4e4. No arguments. |
| `0144d420` | CWProcessEventEmissionState::WriteGlobal01cbe4e4 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4e4 and returns (ret 4). |
| `0144d440` | CWProcessEventEmissionState::KeepGoing | confirmed | Virtual slot [72] (no arguments): traces CWProcessEventEmissionState::keepGoing through 0145c4d0 and 013508b0, then calls 0145c710 with no arguments and return… |
| `0144d7c0` | CWProcessEventEmissionState::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] with the stream and an out word; on success calls the base read 01465430 on the stream; on success calls bo… |
| `0144d800` | CWProcessEventEmissionState::Write | confirmed | Virtual slot [10] (Write, one stream argument): when the argument is 0 obtains a stream from slot [1] (+4); otherwise calls slot [2] (+8) with it; writes the S… |
| `0144d850` | CWProcessEventEmissionState::ScalarDeletingDtor | confirmed | Vtable slot [0] of CWProcessEventEmissionState: stores the vtable at +0, calls base destructor 01464a50, and when bit 0 of the flag argument is set frees the o… |
| `0144d9b0` | CWProcessEventEmissionState::CWProcessEventEmissionState | confirmed | Default constructor (ECX=this): calls base default constructor 01464d40 under an EH frame, stores CWProcessEventEmissionState::vftable at +0, assigns the 27-ch… |
| `0144da20` | CWProcessEventEmissionState::CWProcessEventEmissionState | confirmed | Overload constructor with one stack argument (ECX=this): calls base constructor 01464e30 with the argument, stores CWProcessEventEmissionState::vftable at +0, … |
| `0144db20` | CWProcessEventEmissionState::LeaveState | confirmed | Virtual slot [18] (no arguments, returns a byte): traces CWProcessEventEmissionState::leaveState() through 0145c4d0 and 013508b0; collects the list returned by… |
| `0144dd60` | CWProcessEventEmissionState::NextState | confirmed | Virtual slot [73] (no arguments, returns a dword): on the first call (byte +0x130 clear) calls virtual slot 0x170 of the 0145c4d0 result with the event word at… |
| `0144d460` | CWProcessEventEmissionState::TestActionPreconditions | likely | Thiscall (ECX=this, two stack arguments: an object pointer and a value): traces the text CWProcessEventEmissionState::testActionPreconitions with three %d slot… |
| `0144da90` | CWProcessEventEmissionState::Create | likely | Factory: allocates 0x148 bytes with operator new (00d833a6); when the allocation is non-null constructs the state with 0144d9b0 and returns it, otherwise retur… |
| `0144db00` | CWProcessEventEmissionState::RegisterFactory | likely | Registers the factory 0144da90 under classID 0x222ef: pushes the factory address and 0x222ef and calls 013ce130, then moves its eax result into ECX and calls 0… |

**CWProcessEventEmissionStateForActivePlayer** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015163f0` | CWProcessEventEmissionStateForActivePlayer::Dtor | confirmed | Stores the CWProcessEventEmissionStateForActivePlayer vtable into *this, calls base destructor 015102e0, and returns without freeing the object. |
| `01516460` | CWProcessEventEmissionStateForActivePlayer::TypeName | confirmed | Returns the address of the literal class name CWProcessEventEmissionStateForActivePlayer (mov eax, imm32; ret). |
| `01516470` | CWProcessEventEmissionStateForActivePlayer::ReadGlobal01cbe… | confirmed | Returns the dword in global DAT_01cbeb40 (mov eax, [01cbeb40]; ret) and takes no arguments. |
| `01516480` | CWProcessEventEmissionStateForActivePlayer::WriteGlobal01cb… | confirmed | Stores its stack argument into DAT_01cbeb40 and returns with ret 4. |
| `015164a0` | CWProcessEventEmissionStateForActivePlayer::KeepGoing | confirmed | Emits the keepGoing trace text through 0145c4d0 and 013508b0 (the trace output is gated by flags in the trace object), then tail-jumps to 0145c710 with ECX set… |
| `015164c0` | CWProcessEventEmissionStateForActivePlayer::TestActionPreco… | confirmed | Traces the precondition test, reads the player id through param_2 virtual slot 10, calls param_2 virtual slot 55 with param_3 and a local buffer, traces the re… |
| `015168b0` | CWProcessEventEmissionStateForActivePlayer::ScalarDeletingD… | confirmed | Virtual slot 0: stores the CW vtable into *this, calls base destructor 015102e0, frees the object with _free (00d82463) when bit 0 of param_2 is set, and retur… |
| `01516ad0` | CWProcessEventEmissionStateForActivePlayer::Ctor | confirmed | Calls base constructor 015106b0, stores the CW vtable into *this, then assigns the class-name literal (count 0x2a) to the std::string at this+0x1c. |
| `01516b40` | CWProcessEventEmissionStateForActivePlayer::CtorWithArg | confirmed | Calls base constructor 015107a0 with its stack argument, stores the CW vtable into *this, then assigns the class-name literal (count 0x2a) to the std::string a… |
| `01516c40` | CWProcessEventEmissionStateForActivePlayer::LeaveState | confirmed | Virtual slot 18: traces leaveState() through 013508b0; calls virtual slot 131 with event 0x222f6 on each element of the list returned by mMachine virtual slot … |
| `01516e80` | CWProcessEventEmissionStateForActivePlayer::NextState | confirmed | Virtual slot 73. First call (byte this+0x130 clear): traces the init line, calls mMachine virtual slot 92 (0x170) with the event id at this+0xfc, calls this sl… |
| `01516bb0` | CWProcessEventEmissionStateForActivePlayer::Factory | likely | Allocates 0x148 bytes with operator new (00d833a6) and, when the allocation succeeds, default-constructs the object with 01516ad0; returns the object or 0. |

**CWStartOfGameState** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144c120` | CWStartOfGameState::ButtonPressed | confirmed | Virtual slot [40] (two int arguments, ret 8): traces CWStartOfGameState::buttonPressed with the two arguments, then the text Got button pressed not during mull… |
| `0144c160` | CWStartOfGameState::EnterState | confirmed | Virtual slot [14] (no arguments): calls vtable slot [19] (+0x4c) on this, traces CWStartOfGameState::enterState through 0145c4d0 and 013508b0, stores 1 at the … |
| `0144c1a0` | CWStartOfGameState::RestoreState | confirmed | Virtual slot [17] (no arguments): traces CWStartOfGameState::restoreState through 0145c4d0 and 013508b0, then tail-jumps to vtable slot [15] (UpdateState) with… |
| `0144c1c0` | CWStartOfGameState::LeaveState | confirmed | Virtual slot [18] (no arguments, returns 1): traces CWStartOfGameState::leaveState through 0145c4d0 and 013508b0, calls 013f39c0 with ECX set to the word at th… |
| `0144c300` | CWStartOfGameState::Write | confirmed | Virtual slot [10] (Write, one stream argument): calls header write slot [2] with the stream, writes the Starting text through no-op 013c26e0, calls 0145cdc0 on… |
| `0144cfe0` | CWStartOfGameState::SetFlagAtF8AndUpdate | confirmed | Virtual slot [43] (one argument, ret 4): calls 0129d050 with ECX=this+0xf8 and the address of its argument, stores 1 through the returned pointer, then calls v… |
| `0144d010` | CWStartOfGameState::Read | confirmed | Virtual slot [9] (void, one stream argument): calls header read slot [4] with an out word, tracing false at cwstartofgamestate.cpp line 0x166 on failure; calls… |
| `0144d110` | CWStartOfGameState::~CWStartOfGameState | confirmed | Destructor body (ECX=this): stores CWStartOfGameState::vftable at +0, calls 0129d560 under a cookie, then calls the StateMachineState base destructor 0145d080.… |
| `0144d1a0` | CWStartOfGameState::TypeName | confirmed | Virtual slot [11]: returns the address of the literal CWStartOfGameState (mov eax,imm; ret). No arguments. |
| `0144d1b0` | CWStartOfGameState::ReadGlobal01cbe4e0 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4e0. No arguments. |
| `0144d1c0` | CWStartOfGameState::WriteGlobal01cbe4e0 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4e0 and returns (ret 4). |
| `0144d1e0` | CWStartOfGameState::CWStartOfGameState | confirmed | Default constructor (ECX=this): calls the StateMachineState default constructor 0145d8c0 under an EH frame, stores CWStartOfGameState::vftable at +0, zeroes th… |
| `0144d260` | CWStartOfGameState::ScalarDeletingDtor | confirmed | Vtable slot [0] of CWStartOfGameState: calls the destructor body 0144d110 with ECX=this, and when bit 0 of the flag argument is set frees the object through _f… |
| `0144d280` | CWStartOfGameState::CWStartOfGameState | confirmed | Overload constructor with one stack argument (ECX=this): calls the StateMachineState constructor 0145d9f0 with the argument, stores CWStartOfGameState::vftable… |
| `0144c250` | CWStartOfGameState::AllNodeFlagsSet | likely | Thiscall (ECX=this): walks the list whose head pointer is at this+0xfc; returns 0 as soon as a node's flag byte (+0x10) is zero, and for each flagged node call… |
| `0144d300` | CWStartOfGameState::Create | likely | Factory: allocates 0x104 bytes with operator new (00d833a6); when the allocation is non-null constructs the state with 0144d1e0 and returns it, otherwise retur… |
| `0144d370` | CWStartOfGameState::RegisterFactory | likely | Registers the factory 0144d300 under classID 0x222ea: pushes the factory address and 0x222ea and calls 013ce130, then moves its eax result into ECX and calls 0… |

**CWStateMachine** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144b950` | CWStateMachine::CWStateMachine | confirmed | Constructor (ECX=this): calls base constructor 013f6300 under an EH frame, stores CWStateMachine::vftable at +0, returns this. |
| `0144b9d0` | CWStateMachine::TypeName | confirmed | Virtual slot [11]: returns the address of the literal CWStateMachine (mov eax,imm; ret). No arguments. |
| `0144b9e0` | CWStateMachine::ReadGlobal01cbe4dc | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4dc. No arguments. |
| `0144b9f0` | CWStateMachine::WriteGlobal01cbe4dc | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4dc and returns (ret 4). |
| `0144ba10` | CWStateMachine::~CWStateMachine | confirmed | Destructor body (ECX=this): stores CWStateMachine::vftable at +0, then calls the base destructor 013f5c50 with a cookie argument. It does not free the object. |
| `0144ba80` | CWStateMachine::CreateState0x15c | confirmed | Virtual slot [16]: allocates 0x15c bytes with operator new (00d833a6); when the allocation is non-null calls 0144eaf0 with its argument and returns that result… |
| `0144baf0` | CWStateMachine::CreateState0x148_a | confirmed | Virtual slot [17]: allocates 0x148 bytes with operator new (00d833a6); when the allocation is non-null calls 0144da20 with its argument and returns that result… |
| `0144bb60` | CWStateMachine::CreateState0x148_b | confirmed | Virtual slot [18]: allocates 0x148 bytes with operator new (00d833a6); when the allocation is non-null calls 01516b40 with its argument and returns that result… |
| `0144bbd0` | CWStateMachine::CreateState0x1a4 | confirmed | Virtual slot [19]: allocates 0x1a4 bytes with operator new (00d833a6); when the allocation is non-null calls 01450b50 with its argument and returns that result… |
| `0144bc40` | CWStateMachine::CreateState0x12c | confirmed | Virtual slot [20]: allocates 300 (0x12c) bytes with operator new (00d833a6); when the allocation is non-null calls 0144fc20 with its argument and returns that … |
| `0144bcb0` | CWStateMachine::ReadState | confirmed | Virtual slot [9] (void, two arguments): calls header read slot [4] with the stream and an out word; when that returns false traces false at cwstatemachine.cpp … |
| `0144bd20` | CWStateMachine::WriteState | confirmed | Virtual slot [10] (one argument): when the argument is 0 obtains a stream from slot [1] (+4); otherwise calls slot [2] (+8) with the argument and uses its resu… |
| `0144be00` | CWStateMachine::ScalarDeletingDtor | confirmed | Vtable slot [0] of CWStateMachine: stores the vtable at +0, calls base destructor 013f5c50, and when bit 0 of the flag argument is set frees the object through… |
| `0144bd70` | CWStateMachine::Create | likely | Factory: allocates 0x1bc bytes with operator new (00d833a6); when the allocation is non-null constructs the object with 0144b950 and returns it, otherwise retu… |
| `0144bde0` | CWStateMachine::RegisterFactory | likely | Registers the factory 0144bd70 under classID 0x222e6: pushes the factory address and 0x222e6 and calls 013ce130, then moves its eax result into ECX and calls 0… |
| `0144be60` | CWStateMachine::RunStateStack | likely | Virtual slot [15] (two arguments): loops while the result code is 1 (013f3680) or 3 (013f3680 then 013efc60); when the state stack at +0xc..+0x10 is empty it c… |

**CancellableStateMachineState** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0147c520` | CancellableStateMachineState::DestructBody | confirmed | Stores CancellableStateMachineState::vftable into [this], calls the base destructor body 0145d080 with this, and returns void. It does not free the object or d… |
| `0147c590` | CancellableStateMachineState::GetClassName | confirmed | Returns the address of the literal 'CancellableStateMachineState' (mov eax,018f4978; ret). No arguments and no object access. |
| `0147c850` | CancellableStateMachineState::DeserializeState | confirmed | Thiscall reader with one stack argument (the stream object, ret 4). Calls this->vtable[4](stream, &tmp) (offset 0x10) and returns 0 when false. Otherwise calls… |
| `0147c8b0` | CancellableStateMachineState::SerializeStateWithTrace | confirmed | Thiscall writer with one stack argument. If the argument is 0 it obtains the writer from vtable slot 1 (offset 4); otherwise from slot 2 (offset 8) with the ar… |
| `0147c920` | CancellableStateMachineState::~CancellableStateMachineState | confirmed | Scalar deleting destructor: stores CancellableStateMachineState::vftable into [this], calls base destructor body 0145d080 with this, then calls _free (thunk 00… |
| `0147c980` | CancellableStateMachineState::CancellableStateMachineState | confirmed | Thiscall no-argument constructor: calls base constructor 0145d8c0 with ECX = this, stores CancellableStateMachineState::vftable at [this], assigns the 28-chara… |
| `0147c9f0` | CancellableStateMachineState::CancellableStateMachineStateA… | confirmed | Thiscall constructor with one stack argument (ret 4): calls base constructor 0145d9f0 with that argument and ECX = this, stores CancellableStateMachineState::v… |
| `0147c5a0` | CancellableStateMachineState::GetGlobal_01cbe5ac | likely | Returns the current value of global DAT_01cbe5ac (mov eax,[01cbe5ac]; ret). No this access. |
| `0147c5b0` | CancellableStateMachineState::SetGlobal_01cbe5ac | likely | Stores its single stack argument into global DAT_01cbe5ac (mov eax,[esp+4]; mov [01cbe5ac],eax) and returns with ret 4. No this access. |
| `0147c5d0` | CancellableStateMachineState::ForwardToPreviousState_Slot47 | likely | If the dword at this+4 (mPreviousState) is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x1f. Then, with ECX … |
| `0147c620` | CancellableStateMachineState::ForwardToPreviousState_Slot46 | likely | Same shape as 0147c5d0: if this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x33; then tail-jumps throu… |
| `0147c650` | CancellableStateMachineState::ForwardToPreviousState_Slot48 | likely | Same shape as 0147c5d0: if this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x39; then tail-jumps throu… |
| `0147c680` | CancellableStateMachineState::ForwardToPreviousState_Slot30 | likely | If this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x3f. Then loads the vtable slot at offset 0x78 (sl… |
| `0147c6b0` | CancellableStateMachineState::ForwardToPreviousState_Slot61 | likely | Bare tail forwarder with no null check: mov ecx,[ecx+4]; mov eax,[ecx]; mov edx,[eax+0xf4]; jmp edx. It jumps to slot 61 (offset 0xf4) of the object at this+4. |
| `0147c6c0` | CancellableStateMachineState::ForwardToPreviousState_Slot34 | likely | If this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x4a; then tail-jumps through offset 0x88 (slot 34)… |
| `0147c6f0` | CancellableStateMachineState::ForwardToPreviousState_Slot35 | likely | If this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x55; then loads offset 0x8c (slot 35) of the objec… |
| `0147c720` | CancellableStateMachineState::ForwardToPreviousState_Slot36 | likely | If this+4 is zero, calls 012f5a60 with 'mPreviousState', the CancellableStateMachineState.cpp path and line 0x62; then loads offset 0x90 (slot 36) of the objec… |
| `0147c750` | CancellableStateMachineState::InitHelperObject0x94 | likely | Thiscall base virtual (slot 19). Allocates 0x94 bytes via 00d833a6 and, when non-null, constructs the object with 01385940 (ECX = new object); stores the resul… |
| `0147c800` | CancellableStateMachineState::SetCancelStrategy | likely | Thiscall setter with one stack argument (ret 4): stores the argument into the dword at this+0xf4 and returns. |
| `0147c810` | CancellableStateMachineState::PropagateCancelStrategy | likely | Thiscall with no arguments: if the dword at this+0xf4 equals 2, calls vtable slot 45 (offset 0xb4) of the object at this+4 with 2; then if the dword at this+0x… |

**ChainedActionState** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0147a920` | ChainedActionState::Enter | confirmed | Thiscall, no stack args. Pushes the literal Entering ChainedActionState, calls 0145c4d0 with ECX=this (it asserts mMachine at +8 and returns the machine's +4 f… |
| `0147aff0` | ChainedActionState::TypeName | confirmed | No arguments. Returns a pointer to the literal ChainedActionState (mov eax,018f4298; ret). |
| `0147c030` | ChainedActionState::updateState | confirmed | Thiscall step with no arguments that switches on the dword at this+0xf8. Case 1 passes the literal 'ChainedActionState::updateState - InitialChainedActionState… |
| `0147a960` | ChainedActionState::ForwardToSlot15 | likely | Loads the vtable of ECX and tail-jumps through its slot 15 (offset 0x3c) with ECX unchanged. Bytes: mov eax,[ecx]; mov edx,[eax+3ch]; jmp edx. |
| `0147acf0` | ChainedActionState::SerializeState | likely | Thiscall with one stack argument, ret 4. Gets the output object from vtable slot 1 when the argument is 0, otherwise from slot 2 with the argument. Writes fram… |
| `0147b000` | ChainedActionState::ReadGlobal01cbe5a8 | likely | No arguments. Returns the 32-bit global at 01cbe5a8 (mov eax,[01cbe5a8]; ret). The global is uninitialised .data in the file. |
| `0147b010` | ChainedActionState::WriteGlobal01cbe5a8 | likely | One stack argument, ret 4. Stores the argument to the global at 01cbe5a8. |
| `0147c3a0` | ChainedActionState::DeserializeState | likely | Thiscall reader with one stack argument (ret 4). Calls this->vtable[4](arg, &tmp) and returns 0 when that is false. Otherwise calls base reader 0147c850(arg), … |

**ControllerDialog** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afee70` | ControllerDialog::DispatchHotKeyToggle | confirmed | Fastcall, vtable slot 50 (+0xc8). Walks a list headed at this+0xc8 (element at node+0x00, next at node+0x04). For an element with type 0 (+0x18 == 0) and flag … |
| `00affb10` | ControllerDialog::~ControllerDialog | confirmed | Fastcall dtor body (ECX=this, ret 0): stores ControllerDialog::vftable at +0x00, sets the unwind state, calls 00769710, then 00984960 to find an entity, and se… |
| `00affc80` | ControllerDialog::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (thiscall, one flag byte arg): calls dtor body 00affb10 on this, then calls _free(this) (00d82463) when the flag bit 0 is set. Retur… |
| `00affeb0` | ControllerDialog::ControllerDialog | confirmed | Thiscall ctor (ECX=this) with five stack args, returns this. Calls base 0094f6c0 with the forwarded args, stores ControllerDialog::vftable at +0x00, sets the t… |

**ControllerDraw** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afcff0` | ControllerDraw::ControllerDraw | confirmed | Thiscall ctor, 3 stack args forwarded to base 00afe470, returns this. Stores ControllerDraw::vftable at +0x00, clears the byte at +0x3b0, installs SoeUtil::Arr… |
| `00afd200` | ControllerDraw::~ControllerDraw | confirmed | Fastcall dtor body (ECX=this): restores ControllerDraw::vftable at +0x00 and Array vftable at +0x3b4, zeroes the size at +0x3bc, calls 00401c00(data, 4) to fre… |
| `00afd290` | ControllerDraw::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (ret 4, flag byte arg): calls dtor body 00afd200 on this, then calls _free(this) (00d82463) when flag bit 0 is set. Returns this. |

**ControllerRegular** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b00110` | ControllerRegular::~ControllerRegular | confirmed | Fastcall dtor body (ECX=this, ret 0): stores ControllerRegular::vftable at +0x00, sets the unwind state to -1, calls 0094fab0 on this (base teardown), then ret… |
| `00b00340` | ControllerRegular::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (thiscall, one flag byte arg): stores ControllerRegular::vftable, sets the unwind state to -1, calls 0094fab0 on this, then calls _f… |

**ControllerTactileTarget** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afe290` | ControllerTactileTarget::~ControllerTactileTarget | confirmed | Fastcall dtor body (ECX=this, ret 0): stores ControllerTactileTarget::vftable at +0x00, releases the IString members at +0x368 and +0x378 by decrementing their… |
| `00afe470` | ControllerTactileTarget::ControllerTactileTarget | confirmed | Thiscall ctor (ECX=this), three stack args forwarded to base 00950530, returns this (ret 0x14). Stores ControllerTactileTarget::vftable at +0x00, calls 00afe13… |
| `00afe5b0` | ControllerTactileTarget::ScalarDeletingDtor | confirmed | Scalar deleting dtor shape (thiscall, one flag byte arg, ret 4): calls the dtor body 00afe290 on this, then calls _free(this) (00d82463) when the flag bit 0 is… |
| `00afebe0` | ControllerTactileTarget::FormatTrackerDebugText | confirmed | Thiscall with no args, vtable slot 42 (+0xa8). On first use sets a static guard bit (DAT_01be8e78 bit 0), initialises the static IString DAT_01be8e68 (vtable S… |

**GameTurn** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013f8030` | GameTurn::SetActivePlayer | confirmed | Virtual slot 18 of GameTurn::vftable: stores the new player pointer at this+8 when it changes; with a trace (setActivePlayer %d) gated by a stub that returns 0… |
| `013f8180` | GameTurn::GetCurrentBroadPhaseID | confirmed | Virtual slot 15 of GameTurn::vftable: asserts the dword at this+0xc is zero (GameTurn.cpp line 0x24f, message Invalid phase in GameTurn::getCurrentBroadPhaseID… |
| `013f7fa0` | GameTurn::CreateStateMachine | likely | Virtual slot 14 of GameTurn::vftable: allocates 0x1bc bytes with operator new, constructs a StateMachine through 013f6300 when the allocation succeeds, then ca… |

**GetTargetState** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141baf0` | GetTargetState::TypeName | confirmed | Virtual slot 11: returns the literal 'GetTargetState' (mov eax,018e68f0; ret). |
| `0141bb30` | GetTargetState::DeletingDtor | confirmed | Scalar deleting destructor: calls the dtor body 0141b9f0, then frees this via _free when flag bit 0 of the stack arg is set; returns this. |
| `0141bb90` | GetTargetState::GetTargetState | confirmed | Default constructor: calls base ctor 0147c980, stores GetTargetState::vftable, zeroes triples at +0x100, +0x114 and +0x12c, assigns 'GetTargetState' to +0x1c a… |
| `0141bb00` | GetTargetState::GetStaticDword | likely | Virtual slot 12: returns the global dword DAT_01cbe26c. |
| `0141bb10` | GetTargetState::SetStaticDword | likely | Virtual slot 13: stores its stack arg into the global DAT_01cbe26c (ret 4). |
| `0141bfa0` | GetTargetState::GetTargetState_Arg | likely | Constructor with one stack arg passed to base ctor 0147c9f0: same field initialisation as GetTargetState::GetTargetState, then stores 01347a90() into +0x124 an… |
| `0141c1c0` | GetTargetState::GetTargetState_Arg2 | likely | Constructor with two stack args: base ctor 0147c9f0(arg1); same field initialisation, sets +0xf8=1 and +0x121=0, calls 0135cfe0(arg2), sets +0x10c=0 and +0x120… |
| `0141c3e0` | GetTargetState::GetTargetState_Arg3 | likely | Constructor with a stack argument and a byte argument: base ctor 0147c9f0(arg1); field init as the others; sets +0xf8=1, +0x120=1, stores the byte arg at +0x12… |
| `0141c700` | GetTargetState::EnterWaitDialog | likely | Thiscall, one stack arg. Builds the 'Get Target' wait dialog: '#title#' and '#owner#' lookups (01402480 and 01402400), trace 'GetTargetState::WaitDialogText: %… |
| `0141e260` | GetTargetState::GetTargetState_Arg4 | likely | Constructor with arg2 passed to base 0147c9f0, arg3 to 005f1e5c, and arg4 (a pointer) whose dwords +4 and +8 are copied into +0x58 and +0x5c and whose subobjec… |
| `0141ed60` | GetTargetState::GetTargetState_Arg5 | likely | Constructor variant with arg2 to base 0147c9f0, arg3 to 0135cfe0, arg4 pointer used through 01402400 and 0145c4d0; same field initialisation and 'cancel' strin… |

**MultiActionState** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0146a3d0` | MultiActionState::RestoreState | confirmed | Virtual slot [17] of MultiActionState::vftable (ECX=this; the same body is also slot [17] of CWMultiActionState::vftable at 018ec294). Traces 'MultiActionState… |
| `0146aac0` | MultiActionState::ButtonPressed | confirmed | Virtual slot [40] of MultiActionState::vftable (also slot [40] of CWMultiActionState::vftable). Traces 'MultiActionState::buttonPressed' through FUN_0145c4d0 a… |
| `0146ab00` | MultiActionState::ButtonPressedWithString | confirmed | Virtual slot [41] of MultiActionState::vftable (also slot [41] of CWMultiActionState::vftable). Traces 'MultiActionState::buttonPressedWithString' through FUN_… |
| `0146ab40` | MultiActionState::Dtor | confirmed | Destructor body. Stores MultiActionState::vftable at +0, and when the word at +0x168 is non-null calls its slot 0 with argument 1 and clears it. Calls FUN_0130… |
| `0146ac50` | MultiActionState::GetTypeName | confirmed | Virtual slot [11] of MultiActionState::vftable. Bytes b8 9c 14 8f 01 c3: returns the address of the literal 'MultiActionState' (018f149c). |
| `0146ace0` | MultiActionState::ScalarDeletingDtor | confirmed | Vtable slot [0] of MultiActionState::vftable. Bytes 56 8b f1 e8 ... f6 44 24 08 01 74 09 56 e8 ... 83 c4 04 8b c6 5e c2 04 00: calls the destructor body FUN_01… |
| `0146ae30` | MultiActionState::Ctor | confirmed | Constructor overload taking one argument (__thiscall). Forwards it to base init FUN_0147c9f0, then stores MultiActionState::vftable, calls FUN_012fa910, zeroes… |
| `0146aff0` | MultiActionState::GetCommandObjectList | confirmed | Virtual slot [75] of MultiActionState::vftable (also slot [75] of AlternateMultiActionState::vftable 018f4a3c); ECX=this, args (param_2, param_3 flag, param_4)… |
| `0146c3e0` | MultiActionState::ReadFieldsSlot9 | likely | Virtual slot [9] of MultiActionState::vftable (ECX=this, argument param_2 and a version value in EBX). Calls slot [4] (+0x10) of this with param_2 and an out p… |

**MultiPlayerState** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014695c0` | MultiPlayerState::GetTypeName | confirmed | Virtual slot [11] of MultiPlayerState::vftable. Bytes b8 b8 0c 8f 01 c3: returns the address of the literal 'MultiPlayerState' (018f0cb8). |
| `01469600` | MultiPlayerState::DoEffect | confirmed | Traces 'MultiPlayerState::doEffect' through FUN_0145c4d0 and FUN_013508b0, then runs a sequence of effect calls (FUN_01385940, FUN_01386b70 on [this+0xE8], FUN… |
| `014698a0` | MultiPlayerState::UpdateState | confirmed | Virtual slot [15] of MultiPlayerState::vftable (ECX=this). Switches on the state word at +0x104: states 0 and 2 trace 'invalidState %d' and return 2; any other… |
| `014699d0` | MultiPlayerState::ScalarDeletingDtor | confirmed | Vtable slot [0] of MultiPlayerState::vftable: calls the destructor body FUN_01469510 with ECX=this; when bit 0 of the flag argument is set, frees the object wi… |
| `014699f0` | MultiPlayerState::Ctor | confirmed | Thiscall with one stack argument. Calls base init FUN_0147c9f0 with that argument, stores MultiPlayerState::vftable, calls FUN_012fa910, zeroes the list words … |
| `01469ab0` | MultiPlayerState::CtorDefault | confirmed | Thiscall with no stack argument (ECX=this). Calls base init FUN_0147c980, stores MultiPlayerState::vftable, calls FUN_012fa910, zeroes +0x10C, +0x110, +0x114 a… |
| `01469bf0` | MultiPlayerState::StartWithReturnValues | confirmed | Virtual slot [14] of MultiPlayerState::vftable (ECX=this). Calls virtual slot [19] (+0x4c) of this first. Gets the returnValueMap object via FUN_013f37e0 (asse… |

**PhaseState** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014690b0` | PhaseState::RestoreState | confirmed | Virtual slot [17] of PhaseState::vftable. Calls FUN_0145c4d0 with the format '%s::restoreState' and the name text (inline buffer at +0x20 when the size word at… |
| `014690e0` | PhaseState::LeaveState | confirmed | Virtual slot [18] of PhaseState::vftable. Calls FUN_0145c4d0 with the format '%s::leaveState' and the same inline-or-pointer name text, passes the result to FU… |
| `01469120` | PhaseState::Ctor | confirmed | Constructor body. Calls FUN_0145d8c0 first (base state-machine init; its strings include 'Basic StateMachineState'), then stores PhaseState::vftable at +0, zer… |
| `014691a0` | PhaseState::CtorWithMachine | confirmed | Constructor overload taking one stack argument (ret 4) that it forwards to FUN_0145d9f0 (base init; its strings include 'mMachine'). Then stores PhaseState::vf… |

**StateMachine** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013f10f0` | StateMachine::DispatchUserInput | confirmed | Virtual slot 14 (+0x38) of StateMachine: calls 01347ab0 first and returns 1 when its byte result is nonzero. Otherwise it switches on the UserInput type (param… |
| `013f3b20` | StateMachine::HandleStateResult | confirmed | Virtual slot 15 (+0x3c) of StateMachine (ECX = this, one stack argument = state result code): loops while the code is 1 (calls 013f3680, pop and resume) or 3 (… |
| `013f5c50` | StateMachine::~StateMachine | confirmed | Teardown body of StateMachine: stores StateMachine::vftable at entry, then for two owned-pointer vectors (header words +0xc/+0x10 and +0x34/+0x38) calls each n… |
| `013ef6b0` | StateMachine::CreateState_v16 | likely | Virtual slot 16 (+0x40) of StateMachine: allocates 0x15c bytes with operator new (00d833a6); when non-null it constructs an object with 01413830, passing this … |
| `013ef720` | StateMachine::CreateState_v17 | likely | Virtual slot 17 (+0x44) of StateMachine: allocates 0x148 bytes with operator new (00d833a6); when non-null it constructs an object with 01464e30 passing this a… |
| `013ef790` | StateMachine::CreateState_v18 | likely | Virtual slot 18 (+0x48) of StateMachine: allocates 0x148 bytes with operator new (00d833a6); when non-null it constructs an object with 015107a0 passing this a… |
| `013ef800` | StateMachine::CreateState_v19 | likely | Virtual slot 19 (+0x4c) of StateMachine: allocates 0x19c bytes with operator new (00d833a6); when non-null it constructs an object with 0147f560 passing this a… |
| `013ef870` | StateMachine::CreateState_v20 | likely | Virtual slot 20 (+0x50) of StateMachine: allocates 300 bytes (0x12c) with operator new (00d833a6); when non-null it constructs an object with 01475300 passing … |
| `013ef900` | StateMachine::GetActivePlayer | likely | Calls 01347e10 (a getter of the object reached through the current game/turn), then 013f8020 on that result; if 013f8020 returns 0 it asserts 'activePlayer' (S… |
| `013f3530` | StateMachine::PopState | likely | Fastcall (ECX = this): when the state stack at +0xc/+0x10 holds more than one state it pops the top (end -= 4), sets the current state at +0x18 to the new top,… |

**StateMachineState** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145c4d0` | StateMachineState::MachineForwardA | confirmed | Fastcall with ECX=this. Asserts this+8 (mMachine) non-null (StateMachineState.cpp line 0x4c, through 012f5a60), then tail-jumps to 013ef8e0 with ECX=mMachine; … |
| `0145c520` | StateMachineState::SetMachine | confirmed | Virtual slot [20] shared by the state vtables, thiscall with one stack argument, ret 4. Asserts the argument non-null ('state', StateMachineState.cpp line 0x5c… |
| `0145c5b0` | StateMachineState::GetDwordAt0xE8 | confirmed | Virtual slot [71] shared by the state vtables, returns the dword at this+0xE8 (mov eax,[ecx+0xe8]; ret). |
| `0145c7a0` | StateMachineState::DragStoppedDefault | confirmed | Virtual slot [24] shared by ten state vtables. Thiscall, returns 0. Passes 'Got drag stopped, no need for it.' to 013ef8e0 and sends the result to 01353a90 (tr… |
| `0145c7e0` | StateMachineState::InvalidPlayCard | confirmed | Virtual slot [21] shared by twelve state vtables. Thiscall, returns 0. Traces 'Card attempted to be played in an invalid state?' through 013ef8e0 and 01353a90,… |
| `0145c840` | StateMachineState::InvalidDragCard | confirmed | Virtual slot [23] shared by the state vtables. Thiscall, returns false (uVar2 & 0xffffff00). Traces 'Card attempted to be drug in an invalid state.' through 01… |
| `0145c890` | StateMachineState::InvalidPlayAction | confirmed | Virtual slot [25] shared by the state vtables. Thiscall, returns 0. Traces 'Action attempted to be played in an invalid state.' through 013ef8e0 and 01353a90, … |
| `0145c8e0` | StateMachineState::InvalidSelectCard | confirmed | Virtual slot [26] shared by the state vtables. Thiscall, returns 0. Traces 'Card attempted to be selected in an invalid state.' through 013ef8e0 and 01353a90, … |
| `0145c930` | StateMachineState::InvalidDeselectCard | confirmed | Virtual slot [27] shared by the state vtables. Thiscall, returns 0. Traces 'Card attempted to be deselected in an invalid state.' through 013ef8e0 and 01353a90… |
| `0145c990` | StateMachineState::InvalidSynchPoint | confirmed | Virtual slot [44] shared by the state vtables. Thiscall, returns 0. Traces 'Synch point reached in an invalid state.' through 013ef8e0 and 01353a90, then calls… |
| `0145ca30` | StateMachineState::ChainSlot62 | confirmed | Virtual slot [62] shared by the state vtables. Thiscall, returns a dword. When the child pointer at this+4 is null it returns the dword at this+0xE0; otherwise… |
| `0145cae0` | StateMachineState::GetDwordAt0x10 | confirmed | Virtual slot [59] shared by the state vtables. Thiscall, returns the dword at this+0x10. |
| `0145caf0` | StateMachineState::GetDwordAt0x14 | confirmed | Virtual slot [60] shared by the state vtables. Thiscall, returns the dword at this+0x14. |
| `0145cb00` | StateMachineState::SetChildAt0x4 | confirmed | Virtual slot [63] shared by the state vtables. Thiscall, stores its stack argument at this+4. |
| `0145cb20` | StateMachineState::GetByteAt0xF0 | confirmed | Virtual slot [33] shared by the state vtables. Thiscall, returns the byte at this+0xF0. |
| `0145d080` | StateMachineState::DtorBody | confirmed | Destructor body, fastcall ECX=this. Stores StateMachineState::vftable at +0, zeroes +0x8 and +0xE8, frees the buffers of the std::string members at +0x1C, +0x3… |
| `0145d200` | StateMachineState::GetTypeName | confirmed | Virtual slot [11] of StateMachineState::vftable. Returns the address of the literal 'StateMachineState' (mov eax,018ef540; ret). |
| `0145d210` | StateMachineState::GetClassDword | confirmed | Virtual slot [12] of StateMachineState::vftable. Loads the global dword DAT_01cbe564 and returns it. |
| `0145d220` | StateMachineState::SetClassDword | confirmed | Virtual slot [13] of StateMachineState::vftable, ret 4. Stores its stack argument into the global dword DAT_01cbe564. |
| `0145d830` | StateMachineState::ScalarDeletingDtor | confirmed | Vtable slot [0] of StateMachineState::vftable, thiscall with one flag argument, ret 4. Calls the destructor body 0145d080 with ECX=this, then frees the object … |
| `0145d9f0` | StateMachineState::CtorWithMachine | confirmed | Constructor overload, thiscall with one stack argument (the machine). Performs the same base initialisation as 0145d8c0, stores the argument at +0x8 (mMachine)… |
| `0145dd20` | StateMachineState::SetPhaseDialogForPlayer | confirmed | Virtual slot [37] shared by the state vtables, thiscall with six stack arguments (player id, PhaseName string, PhaseDialog object at +0x54, Button1 and Button2… |
| `0145ebb0` | StateMachineState::SubstituteActorTextAndSend | confirmed | Virtual slot 38 of StateMachineState; the vtables of 11 state subclasses share it. Seven stack arguments (ret 0x1c). Asserts mPhase (this+0x18) is valid, copie… |
| `0145c550` | StateMachineState::MachineForwardB | likely | Fastcall with ECX=this. Asserts this+8 (mMachine) non-null (line 0x62, through 012f5a60), then tail-jumps to 013ef8f0 with ECX=mMachine; stack arguments pass t… |
| `0145c580` | StateMachineState::MachineForwardC | likely | Fastcall with ECX=this. Asserts this+8 (mMachine) non-null (line 0x68, through 012f5a60), then tail-jumps to 013ef900 with ECX=mMachine. |
| `0145c710` | StateMachineState::Slot72Gate | likely | Virtual slot [72] shared by the state vtables including RevealState. Thiscall, returns 1 when 01348640 and 013f7e60 both return 0, otherwise 0; forwards to 013… |
| `0145cb30` | StateMachineState::SendIfPredicate | likely | Virtual slot [39] shared by twelve state vtables. Thiscall, no return value. Forwards to 013ef8f0; when 013f8020 returns nonzero it allocates 0x14 bytes (00d83… |
| `0145cc50` | StateMachineState::AllocAt0xE8 | likely | Virtual slot [19] of CWStartOfGameState, StateMachineState and PhaseState, thiscall with no stack arguments, ret 0. Allocates 0x94 bytes (00d833a6); when non-n… |
| `0145db60` | StateMachineState::Slot66DispatchToSlot37 | likely | Virtual slot [66] shared by the state vtables, thiscall with one stack argument (the player id). Copies the std::string at +0x3C (PhaseName), +0x8C (Button1) a… |
| `0145f700` | StateMachineState::Factory | likely | Allocates 0xf4 bytes with operator new (00d833a6). A null allocation returns 0; otherwise it runs the base constructor 0145d8c0 on the block and returns its re… |

**WhileState** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145ad80` | WhileState::TailCallSlot15 | confirmed | Virtual slot [17] of WhileState::vftable. Bytes 8b 01 8b 50 3c ff e2: loads the vtable from ECX and jumps to its slot [15] with the same this. It has no body o… |
| `0145aec0` | WhileState::SendCloseReveal | confirmed | Thiscall with one stack byte argument (target-gotten flag). Traces 'WhileState::sendCloseReveal' and iterates the 4-byte reveal ids from this+0x130 to this+0x1… |
| `0145b170` | WhileState::DtorBody | confirmed | Destructor body, fastcall ECX=this. Stores WhileState::vftable at +0, frees the buffers at +0x130 (zeroes +0x130..+0x138) and +0x118 (zeroes +0x118..+0x120) wi… |
| `0145b260` | WhileState::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal 'WhileState' (mov eax,018eea20; ret). |
| `0145b270` | WhileState::GetClassDword | confirmed | Virtual slot [12]. Loads the global dword DAT_01cbe560 and returns it (ret 0). |
| `0145b280` | WhileState::SetClassDword | confirmed | Virtual slot [13], ret 4. Stores its stack argument into the global dword DAT_01cbe560. |
| `0145b2a0` | WhileState::ScalarDeletingDtor | confirmed | Vtable slot [0] of WhileState::vftable, thiscall with one flag argument. Calls the destructor body 0145b170 with ECX=this, then frees the object with _free (th… |
| `0145b390` | WhileState::CtorWithMachine | confirmed | Constructor overload, thiscall with one stack argument. Calls base init 0147c9f0 with the argument, stores WhileState::vftable at +0, calls 012fa910 twice, zer… |
| `0145b500` | WhileState::CheckReturnValue | confirmed | Thiscall with ECX=this, returns 1 (0 when the return-value map 013f37e0 is missing). Sets the target-gotten flag (+0x128) when 013836c0 reports a target. Picks… |
| `0145c100` | WhileState::UpdateState | confirmed | Virtual slot [15] of WhileState::vftable, fastcall ECX=this, returns a dword. Switches on the state word at +0xF8: 1 traces 'InitialWhileState', calls 0145ab00… |

**(no class)** (33)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00afeb30` | PointerTrackDebugFormat | confirmed | Fastcall (ECX=this), returns the dword at this+0x04 (the buffer of the IString at +0x00). Chooses Moving and Still values from flag +0x79 (Moving is +0x74 when… |
| `013f3910` | Vec4_DeleteAllOwned_0x24 | likely | Fastcall (ECX = this): for each element of the pointer vector at +0x24/+0x28 that is non-null it calls its scalar deleting destructor (slot 0 with flag 1), the… |
| `013f5a00` | StateStackPushNonNull | likely | Asserts its stack argument is non-null (StateMachine.cpp line 0x2c3), then pushes that 4-byte value onto the vector at this+0x20 via 013f5550 and returns true … |
| `013f7f70` | GameTurnGetCurrentStateMachine | likely | Getter with a null check: asserts this+0x18 is non-null (GameTurn.cpp line 0x1b5, expression mCurrentStateMachine) and returns the dword at this+0x18. |
| `0141c6e0` | RegisterFactory_Id250 | likely | Registers 0141c670 as the factory for classID 0xfa (250): calls 013ce130(0xfa, factory), then 013ce230 on the returned entry. |
| `0144eb60` | CreateCWHandleResponseState | likely | Pushes 0x15c and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_0144ea80 with it in ECX and returns that object,… |
| `0144ebd0` | RegisterFactory0x222ee | likely | Pushes the key 0x222ee and the address of FUN_0144eb60, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; the two pushed values stay … |
| `0144ee70` | CreateCWDrawCardState | likely | Pushes 0x128 and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_0144ed90 with it in ECX and returns that object,… |
| `0144eee0` | RegisterFactory0x222f9 | likely | Pushes the key 0x222f9 and the address of FUN_0144ee70, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `0144fc90` | CreateCWDoublePassState | likely | Pushes 300 and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_0144fbb0 with it in ECX and returns that object, o… |
| `0144fd00` | RegisterFactory0x222f7 | likely | Pushes the key 0x222f7 and the address of FUN_0144fc90, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `01450bd0` | CreateCWActionPlayedState | likely | Pushes 0x1a4 and calls allocator FUN_00d833a6 (one stack argument); if the pointer is non-null, calls ctor FUN_01450ad0 with it in ECX and returns that object,… |
| `01450c40` | RegisterFactory0x222eb | likely | Pushes the key 0x222eb and the address of FUN_01450bd0, calls FUN_013ce130, moves its return value into ECX and calls FUN_013ce230; same registration shape as … |
| `0145b470` | AllocAndConstruct_0x13c | likely | Allocates 0x13c bytes with 00d833a6 (EH frame present). When the block is non-null calls the default constructor 0145b2c0 on it and returns the object; returns… |
| `0145b4e0` | RegisterFactory_196 | likely | Pushes factory 0145b470 and id 0x196 and calls 013ce130 (no caller cleanup); passes the returned registry in ECX to 013ce230 with no stack arguments. |
| `0145f770` | RegisterFactory_0xea | likely | Static initializer: calls 013ce130 with classID 0xea and factory 0145f700, then calls 013ce230 on the returned registry slot to store 0145f700 under 0xea. |
| `01469290` | RegisterPhaseStateFactory | likely | Pushes the factory FUN_01469220 and the id 0x176 as stack arguments and calls FUN_013ce130 (no caller cleanup after the call); then passes the returned EAX as … |
| `01469b60` | CreateMultiPlayerState | likely | Allocates 0x128 bytes through FUN_00d833a6 under a cookie and EH frame. Returns 0 when the allocation is null; otherwise passes the block in ECX to MultiPlayer… |
| `01469bd0` | RegisterMultiPlayerStateFactory | likely | Pushes the factory FUN_01469b60 and the id 0x103 as stack arguments and calls FUN_013ce130 (no caller cleanup after the call); then passes the returned EAX as … |
| `0146af60` | CreateMultiActionState | likely | Allocates 0x178 bytes through FUN_00d833a6 under a cookie and EH frame. Returns 0 when the allocation is null; otherwise calls MultiActionState::CtorDefault (0… |
| `0146afd0` | RegisterMultiActionStateFactory | likely | Pushes the factory FUN_0146af60 and the id 0xFE as stack arguments and calls FUN_013ce130 (no caller cleanup after the call); then passes the returned EAX as E… |
| `01477e10` | AllocAndConstruct_01477c80 | likely | Calls 00d833a6(300) (operator new). If the result is non-null, calls 01477c80 on it and returns the constructed pointer; otherwise returns 0. |
| `01477e80` | RegisterCreator_Id0x102 | likely | Pushes 0x102 and the address 01477e10, calls 013ce130, moves its return value (EAX) into ECX and calls 013ce230 with no further pushed arguments, then returns. |
| `0147a890` | AllocAndConstruct_0147a780 | likely | Calls 00d833a6(0x108) (operator new). If non-null it calls 0147a780 on the block and returns the constructed pointer; otherwise returns 0. |
| `0147a900` | RegisterCreator_Id0x13d | likely | Pushes 0x13d and the address 0147a890, calls 013ce130, moves its return value into ECX and calls 013ce230 with no further pushed arguments, then returns. |
| `0147b3a0` | AllocAndConstruct_0147b050 | likely | Calls 00d833a6(0x148) (operator new). If non-null it calls 0147b050 on the block and returns the constructed pointer; otherwise returns 0. |
| `0147b410` | RegisterCreator_Id0x100 | likely | Pushes 0x100 and the address 0147b3a0, calls 013ce130, moves its return value into ECX and calls 013ce230 with no further pushed arguments, then returns. |
| `0147ca70` | CreateCancellableStateMachineState | likely | Allocates 0xf8 bytes via 00d833a6; when the pointer is non-null, constructs the object with 0147c980 and returns it, otherwise returns 0. |
| `0147cae0` | RegisterFactory0xf5 | likely | Pushes the creator 0147ca70 and the key 0xf5, calls 013ce130 with them, moves its return value into ECX and calls 013ce230; the two pushed values stay on the s… |
| `0147ce40` | CreateAlternateMultiActionState | likely | Allocates 0x194 bytes via 00d833a6; when the pointer is non-null, constructs the object with 0147cb00 and returns it, otherwise returns 0. |
| `0147ceb0` | RegisterFactory0x113 | likely | Pushes the creator 0147ce40 and the key 0x113, calls 013ce130, moves its return value into ECX and calls 013ce230 with the two pushed values still on the stack… |
| `0147f6d0` | CreateActionPlayedState | likely | Allocates 0x19c bytes via 00d833a6; when the pointer is non-null, constructs the object with 0147f3f0 and returns it, otherwise returns 0. |
| `0147f740` | RegisterFactory0xfb | likely | Pushes the creator 0147f6d0 and the key 0xfb, calls 013ce130 with them, moves its return value into ECX and calls 013ce230 with the two pushed values still on … |

### `game.time`

**ClientGameTimeManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd1bf0` | ClientGameTimeManager::Ctor | confirmed | Constructor body: runs base constructor 010BD190, stores the ClientGameTimeManager vftable at this+0, stores the argument at dword [0x1E] (+0x78), calls the pe… |
| `00bd13c0` | ClientGameTimeManager::HoursFromTimeField | likely | Virtual slot [1] of ClientGameTimeManager. Calls 00769710, reads the dword at global+4 -> +0x778 -> +0x14; when that value is negative returns 010BD030(), othe… |
| `00bd1b40` | ClientGameTimeManager::PeriodicTimeReport 🌐 | likely | Virtual slot [10] of ClientGameTimeManager (no arguments). When this+8 minus this+0x10 is greater than 5 it builds a PacketGameTimeSync with 00bd1600 and, when… |

**GameEventManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eada0` | GameEventManagerCtor | confirmed | Installs manager vtable, allocates and constructs a sequencer and second helper, and stores both pointers. |
| `008eae50` | GameEventManagerDtor | confirmed | Restores manager vtable, deletes two owned helper objects when non-null, and clears both pointers. |
| `008eae90` | GameEventManager::ScalarDeletingDtor | confirmed | Deletes both owned helpers and frees manager if flag bit zero is set, otherwise returns pointer. |

**GameTimeGameEventSequencer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008eabe0` | GameTimeGameEventSequencerCtor | confirmed | Constructs sequencer base, installs vtable, stores manager pointer, allocates a GameTimeTimeline and stores it at +4. |
| `008eacf0` | GameTimeGameEventSequencerDtor | confirmed | Installs sequencer vtable and invokes owned cleanup helper under exception frame. |
| `008ead40` | GameTimeGameEventSequencer::ScalarDeletingDtor | confirmed | Runs sequencer cleanup, frees object if flag bit zero is set, otherwise returns pointer. |
| `008eac70` | GameTimeGameEventSequencer::UpdateTimelineFromCurrentTime | likely | If the optional time source at +0xa0 exists, calls its first virtual method, converts the result to float, wraps it using the period at timeline +0x53c and the… |

**GameTimeManager** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010bd010` | GameTimeManager::SecondsOfDay | confirmed | Fastcall virtual with this in ECX, no stack arguments. Returns the field at +0x34 plus (+0x30 plus +0x2c times 60) times 60, that is seconds since midnight fro… |
| `010bd190` | GameTimeManager::Ctor | confirmed | Fastcall constructor body with this in ECX. Stores GameTimeManager::vftable into [this]. Sets +0x10 and +0x18 to DAT_01cb9000, +0x14 and +0x1c to DAT_01cb9004.… |
| `010bd050` | GameTimeManager::TimeSlotToSeconds | likely | Virtual with one stack dword argument (ret 4). Switch on the argument: -5 returns 0x5460 (21600), -4 returns 0x57e4 (22500), -3 returns 0x10b30 (68400), -2 ret… |
| `010bd0b0` | GameTimeManager::TimeSlotToSecondsAlt | likely | Virtual with one stack dword argument (ret 4). Switch on the argument: -5 returns 0x5b68, -4 returns 0x10eb4, -3 returns 0x11238, -2 returns 0x57e4, -1 returns… |
| `010bd270` | GameTimeManager::SetFromUnixTime | likely | Thiscall virtual with a 64-bit time (two stack dwords) and a flag byte (ret 0xc). If the frozen flag byte at +0x64 is zero: converts the time to calendar field… |
| `010bd380` | GameTimeManager::AdvanceClock | likely | Virtual with this in ECX (slot [8]). If the frozen flag at +0x64 is set, zeroes +0x5c and returns. Otherwise computes seconds = (+0x5c times the scale +0x60) d… |
| `010bd460` | GameTimeManager::Tick | likely | Fastcall virtual, this in ECX (slot [4]). Takes the millisecond tick from 007027f0 and subtracts the stored tick at +0x70/+0x74, saturating the difference at 0… |
| `010bd500` | GameTimeManager::SetTimeOfDay | likely | Thiscall virtual (slot [5]) with three stack dwords: hour (first argument), minute (second) and second (third). Copies the seven-dword block at +0x3c into a lo… |

**GameTimeTimeline** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ec850` | GameTimeTimeline::DtorBody | confirmed | Fastcall (ECX = object). Stores GameTimeTimeline::vftable, drains the list through 007ec6b0, stores SoeUtil::List<GameTimeEventInfo,-1>::vftable at +0x548, dra… |
| `007ec950` | GameTimeTimeline::ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the GameTimeTimeline destructor body 007ec850, then calls _free on this when bit 0 of the flag is set. Returns this. |
| `008eab60` | GameTimeTimelineCtor | confirmed | Initializes timeline base, installs its vtable and zeros embedded GameTimeEventInfo list fields. |

**SoeUtil::List<GameTimeEventInfo,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ec7a0` | GameTimeEventListDtorBody | confirmed | Fastcall (ECX = object). Stores SoeUtil::List<GameTimeEventInfo,-1>::vftable at the object, then calls 007ec6b0 (ListDrainRelease) on it. Returns nothing. |
| `007ec820` | GameTimeEventListScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Stores SoeUtil::List<GameTimeEventInfo,-1>::vftable, drains the list through 007ec6b0, and calls _free on this when bit 0 of the… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ec610` | QueueGameTimeEventTriple | likely | Three stack args (param_1, param_2, param_3) with ECX passed through from the caller. Packs them into a local three-dword record (in the order param_1, param_2… |
| `010bd110` | TimeOfDayDeltaSeconds | likely | Takes two stack dwords (ret 8). Reduces each value modulo 86400 with a multiply-shift division (constant 0x3dd1baf9 and 0x15180), then returns the second minus… |
| `010bd160` | TimeOfDayInWindow | likely | Thiscall with the clock object in ECX (its vtable slot 0 gives the current second of day) and two stack arguments, start and end (ret 8). Returns 1 when the cu… |

### `game.titles`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0097c390` | PlayerTitleTemplateExpand | likely | Reads param_4[0]: when it is zero it returns 0097c280 (PlayerNameTemplateExpand). Otherwise it switches on param_4[1]: kinds 1 and 3 get the template handle fo… |
| `00b49cd0` | TitleManagerDtor | likely | Destructor of the title manager object: releases the reference-counted pointer at [this+0x9c] (virtual slot 0x28 of the owner object with argument 0, then FUN_… |
| `00b4a190` | ReadTitleEntryPair | likely | Parses a byte range into a title entry: reads the shared 16-bit + 32-bit pair with FUN_00b49e00 and then reads a pair of values into (param_1+0xc) and (param_1… |

### `game.value`

**Engine::MatchView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143d160` | Engine::MatchView::MatchView | confirmed | Constructor (fastcall, returns this): calls FUN_013aeff0 on this (base constructor, not in this shard), stores Engine::MatchView::vftable into *this and return… |
| `0143d1c0` | Engine::MatchView::~MatchView | confirmed | Destructor body with EH frame: stores Engine::MatchView::vftable, sets the EH frame local and calls FUN_013af0b0 (base destructor) on this; returns. No free. C… |
| `0143d210` | Engine::MatchView::~MatchView | confirmed | Scalar deleting destructor (virtual slot 0): stores Engine::MatchView::vftable, calls FUN_013af0b0 on this, then if param_2 bit 0 is set calls _free on this; r… |

**Engine::PlayerView** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143d050` | Engine::PlayerView::PlayerView | confirmed | Constructor (fastcall, returns this): calls FUN_0143d270 (PlayerViewBase constructor) on this, stores Engine::PlayerView::vftable into *this, returns this. |
| `0143d0b0` | Engine::PlayerView::~PlayerView | confirmed | Destructor body with EH frame: stores Engine::PlayerView::vftable, then calls FUN_0143d2d0 (PlayerViewBase destructor body) on this and returns. No free. Calle… |
| `0143d100` | Engine::PlayerView::~PlayerView | confirmed | Scalar deleting destructor (virtual slot 0): stores Engine::PlayerView::vftable, calls FUN_0143d2d0 on this, then if param_2 bit 0 is set calls _free on this; … |

**Engine::PlayerViewBase** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143d270` | Engine::PlayerViewBase::PlayerViewBase | confirmed | Constructor (fastcall, returns this): calls FUN_0143cfd0 (SortableView constructor) on this, then stores Engine::PlayerViewBase::vftable into *this and returns… |
| `0143d2d0` | Engine::PlayerViewBase::~PlayerViewBase | confirmed | Destructor body: stores Engine::PlayerViewBase::vftable into *this, then calls FUN_0143c1e0 (SortableView destructor body) on this and returns. No free. Called… |
| `0143d350` | Engine::PlayerViewBase::~PlayerViewBase | confirmed | Scalar deleting destructor (virtual slot 0): stores Engine::PlayerViewBase::vftable, calls FUN_0143c1e0 (SortableView destructor body) on this, then if param_2… |
| `0143d730` | Engine::PlayerViewBase::EmitPlayerFields | confirmed | Virtual slot 5 shared by PlayerView and PlayerViewBase (named for the base class). Writes a key/value list: AccountID (int from FUN_013a3480), GroupID (param_3… |
| `0143ddb0` | Engine::PlayerViewBase::EmitStatsFields | confirmed | Virtual slot 1 shared by PlayerView and PlayerViewBase (named for the base class). First calls FUN_0143c390 (the SortableView slot-1 emission, called directly)… |
| `0143d580` | Engine::PlayerViewBase::EmitPlayer | likely | Virtual slot 4 shared by PlayerView and PlayerViewBase (named for the base class). When param_3 is non-zero: writes the 'Player' key through FUN_0130df10 (6-ch… |

**Engine::SortableView** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143c390` | Engine::SortableView::EmitCardTextFields | confirmed | Virtual slot 1 shared by Engine::SortableView, Engine::ArchetypeViewBase, Engine::ArchetypeView and Engine::FRTWArchetypeView (named for the base class). Walks… |
| `0143cdb0` | Engine::SortableView::GetPropertyName | confirmed | Virtual slot 3 shared by SortableView, ArchetypeViewBase, ArchetypeView, MatchViewBase, MatchView, PlayerView, PlayerViewBase and FRTWArchetypeView (named for … |

**ValueData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01240e20` | ValueData_GetStringPtr | confirmed | Asserts through 012f5a60 that the type tag at +4 equals 3 (kStringTypeID); otherwise returns 0. Returns the C-string pointer of the std::string at +8: its heap… |
| `012c6e70` | ValueData::SetStringValue | likely | Thiscall (ECX = object whose +8 field is a std::string pointer; param_2 = C string). Asserts (012f5a60, line 0x1bc) that param_2 is non-null with the name "str… |

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143bbd0` | MakeStringValue | confirmed | Two arguments (param_1 = output, param_2 = source std::string). Asserts through FUN_012f5a60 with 'stringValue' and the ValueData.h path (line 0x1bc) if the st… |
| `0123c310` | ValueData_GetStringPayload | likely | Returns the 32-bit payload at +8 when the type id at +4 is 3, otherwise asserts mTypeID == kStringTypeID through FUN_012f5a60 (line 0x123 of ValueData.h) and r… |
| `0123c340` | ValueData_GetIntegerPayload | likely | Returns the 32-bit payload at +8 when the type id at +4 is 2, otherwise asserts mTypeID == kIntegerTypeID through FUN_012f5a60 (line 0x129) and returns 0 if th… |
| `0123c370` | ValueData_GetBooleanPayload | likely | Returns the byte payload at +8 when the type id at +4 is 1, otherwise asserts mTypeID == kBooleanTypeID through FUN_012f5a60 and returns 0 when the type still … |
| `0123c3a0` | ValueData_GetIntegerListPayload | likely | Returns the payload pointer at +8 when the type id at +4 is 6, otherwise asserts mTypeID == kIntegerListTypeID through FUN_012f5a60 and returns 0 when the type… |
| `0123c3d0` | ValueData_GetIntIntMapPayload | likely | Returns the payload pointer at +8 when the type id at +4 is 13 (0xd), otherwise asserts mTypeID == kIntIntMapTypeID through FUN_012f5a60 and returns 0 when the… |
| `0123c400` | ValueData_GetStringListPayload | likely | Returns the payload pointer at +8 when the type id at +4 is 7, otherwise asserts mTypeID == kStringListTypeID through FUN_012f5a60 and returns 0 when the type … |
| `0123c430` | ValueData_GetValueDataListPayload | likely | Returns the payload pointer at +8 when the type id at +4 is 8, otherwise asserts mTypeID == kValueDataListTypeID through FUN_012f5a60 and returns 0 when the ty… |
| `01259b80` | ValueData_ReadStringAlt | likely | Same protocol as 012599f0 with a char* source: calls vtable slot [1] of the object at param_2+4 with (param_3, &local); on zero assigns the C string at param_2… |
| `012fe9b0` | ValueData_CompareOp | likely | Thiscall with the left ValueData in ECX, the right operand as stack arg 1 and an opcode as stack arg 2. Switches on the opcode (cases 1 to 0xe) and compares th… |
| `01300d70` | ValueData_SetInteger | likely | Thiscall with one dword argument: if the type tag at +4 is not 2, calls ValueData_SetType (01300680) with type 2; then stores the argument at +8. Returns void. |
| `01300d90` | ValueData_SetBool | likely | Thiscall with one byte argument: if the type tag at +4 is not 1, calls ValueData_SetType (01300680) with type 1; then stores the byte argument at +8. Returns v… |
| `01300f50` | ValueData_SetType14Empty | likely | Fastcall (ECX = ValueData): when the type tag at +4 is not 0xe, builds an empty tree temporary through TreeCtorAllocHeader (01300450), calls ValueData_SetType … |
| `01300ff0` | ValueData_SetString | likely | Thiscall with a std::string argument: if the type tag at +4 is not 3, calls ValueData_SetType (01300680) with type 3; then assigns the argument into the std::s… |
| `0143bca0` | PropertyIdToName | likely | One stack argument (int id). Returns a fixed string literal for about 70 ids through range-split comparisons and switch tables (e.g. 1 -> 'Title', 2 -> 'Keywor… |

### `game.wall_of_data`

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00add410` | WallOfData_RecordEvent6Pair | likely | When event 6 is enabled (00add220) appends the two argument dwords as one 8-byte element to the array at +0x194/+0x198 (count)/+0x19c (capacity) of this object. |
| `00add5e0` | WallOfData_Initialize | likely | Clears 32 12-byte event rows, stores the session pointer at +0x180, sets +0x184 to 5, and constructs the 64-bit array member at +0x190 (raw x86 loads this+0x19… |
| `00add7a0` | WallOfData_LoadEventTypes | likely | Builds the file name from the argument plus WallOfDataEventTypes.txt and loads it (FUN_00fbf4a0); while a returned buffer lasts it tokenises each line (FUN_006… |
| `00ade5e0` | WallOfData_EmitClickMove 🌐 | likely | When event 7 is enabled constructs a click-move packet (00adce20), copies the four-dword position block from the argument into the packet, sends it with 00addf… |
| `00ade730` | WallOfData_EmitWalletBalance 🌐 | likely | When event 29 (0x1d) is enabled constructs a wallet-balance packet (00adcf80), stores the two value arguments at +0xc/+0x10, sends it with 00ade210 when the cl… |

### `net.interaction`

**PlayClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092efe0` | PlayClient::RequestInteraction 🌐 | confirmed | Applies state and target eligibility checks, blocks with the pending GAQ invitation warning on one failed check, scans the target's item list, rate-limits repe… |

### `net.player_update`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092f460` | BaseClient::DispatchBasePlayerUpdate 🌐 | confirmed | Reads an update header/opcode from the supplied byte span, switches across subtype cases, invokes subtype readers, and runs successful client update paths incl… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0092e5d0` | SendFlourishUpdate | confirmed | If the client is not in solo mode, constructs a PlayerUpdatePacketFlourish, reads two words through the interface at param_1+200+0x54, and passes them to sende… |
| `0092e670` | SendJumpUpdate | confirmed | Copies the current four-word position, records clock+0x32 at client offsets +0x868/+0x86c, and in connected mode builds a jump update with position, interface-… |
| `0092e860` | SendPositionUpdateFlag2 | confirmed | Copies the current four-word position and clock+0x32 into client bookkeeping; unless solo, constructs PlayerUpdatePacketUpdatePosition with interface-derived v… |
| `0092ea40` | SendPositionUpdateFlag2WithModeBit | confirmed | Copies the current four-word position and clock+0x32 into client bookkeeping; in connected mode builds and sends PlayerUpdatePacketUpdatePosition with four fie… |
| `0092ebf0` | SendPositionUpdateFlag0x82 | confirmed | Copies the current four-word position and clock+0x32 into client bookkeeping; in connected mode builds and sends PlayerUpdatePacketUpdatePosition with fields a… |
| `0092eda0` | SendPositionUpdateFlag0x42 | confirmed | Copies the current four-word position and clock+0x32 into client bookkeeping; in connected mode builds and sends PlayerUpdatePacketUpdatePosition with fields a… |

## Source shards

[[S0144 007079d0-0070c9f0|S0144]] (6), [[S0147 007140c0-00715ea0|S0147]] (1), [[S0171|S0171]] (2), [[S0172 00798490-0079d890|S0172]] (4), [[S0186|S0186]] (13), [[S0220 008a3ce0-008a8500|S0220]] (2), [[S0221 008a86b0-008b1d80|S0221]] (4), [[S0223 008b7790-008bcf40|S0223]] (7), [[S0225 008c52a0-008ca980|S0225]] (1), [[S0226 008ca9e0-008d08b0|S0226]] (3), [[S0232 008e82e0-008ec1d0|S0232]] (9), [[S0233 008ec2a0-008f0cc0|S0233]] (2), [[S0234 008f0d00-008f50c0|S0234]] (5), [[S0238|S0238]] (2), [[S0240 009105e0-009161d0|S0240]] (1), [[S0244 00925fe0-0092b110|S0244]] (1), [[S0245 0092b170-0092f380|S0245]] (7), [[S0246 0092f460-0092f460|S0246]] (1), [[S0247 00934740-00939200|S0247]] (3), [[S0248 009392c0-0093d740|S0248]] (4), [[S0249 0093daf0-00942c60|S0249]] (2), [[S0254 00954ec0-0095a710|S0254]] (20), [[S0256 009606e0-00965340|S0256]] (1), [[S0257|S0257]] (1), [[S0258 00969a10-0096de80|S0258]] (2), [[S0261 00976740-0097c910|S0261]] (1), [[S0265 0098ec00-00995660|S0265]] (2), [[S0269 009a6cf0-009aceb0|S0269]] (3), [[S0270 009acf20-009b2140|S0270]] (4), [[S0276 009cdf80-009d3fe0|S0276]] (4), [[S0278 009d89c0-009e00c0|S0278]] (25), [[S0279 009e0340-009e4510|S0279]] (6), [[S0280 009e4570-009ea840|S0280]] (44), [[S0288 00a13190-00a18a10|S0288]] (16), [[S0289 00a18a90-00a1dd30|S0289]] (15), [[S0292 00a29620-00a2fe20|S0292]] (7), [[S0294 00a35150-00a39a80|S0294]] (8), [[S0302 00a62280-00a67470|S0302]] (10), [[S0312 00a97450-00a9d360|S0312]] (1), [[S0313 00a9d3c0-00aa24e0|S0313]] (4), [[S0323 00ad3270-00ad83b0|S0323]] (13), [[S0324 00ad8880-00adee10|S0324]] (10), [[S0325 00adee60-00ae4e70|S0325]] (14), [[S0331 00afb7a0-00b005c0|S0331]] (43), [[S0344 00b44fa0-00b4b360|S0344]] (2), [[S0356 00b80ac0-00b84ce0|S0356]] (1), [[S0357 00b84e90-00b8a5f0|S0357]] (7), [[S0365 00bac0d0-00bb1110|S0365]] (7), [[S0366 00bb11b0-00bb7710|S0366]] (1), [[S0367 00bb7790-00bbdbc0|S0367]] (63), [[S0368|S0368]] (25), [[S0369|S0369]] (42), [[S0370|S0370]] (127), [[S0371|S0371]] (46), [[S0380 00bf5ff0-00bf9410|S0380]] (2), [[S0382 00bfcfe0-00c056a0|S0382]] (1), [[S0384 00c0dcd0-00c171d0|S0384]] (1), [[S0386 00c1f190-00c26aa0|S0386]] (6), [[S0395|S0395]] (8), [[S0402 00c7a610-00c7f590|S0402]] (3), [[S0404 00c84f00-00c89db0|S0404]] (5), [[S0405 00c89e70-00c8f7c0|S0405]] (1), [[S0421 00ce0340-00ce4c30|S0421]] (3), [[S0425 00cf69c0-00cfc480|S0425]] (2), [[S0566 0102b8a0-01030cb0|S0566]] (27), [[S0567|S0567]] (118), [[S0568|S0568]] (12), [[S0571 01044480-0104a2b0|S0571]] (9), [[S0574|S0574]] (22), [[S0575|S0575]] (7), [[S0580 0107d0c0-010829c0|S0580]] (3), [[S0581|S0581]] (1), [[S0592|S0592]] (12), [[S0654 012393a0-0123f010|S0654]] (7), [[S0655|S0655]] (1), [[S0660|S0660]] (7), [[S0673|S0673]] (7), [[S0674|S0674]] (1), [[S0677|S0677]] (2), [[S0680|S0680]] (2), [[S0681|S0681]] (5), [[S0691|S0691]] (16), [[S0705|S0705]] (2), [[S0706|S0706]] (21), [[S0708|S0708]] (1), [[S0710|S0710]] (2), [[S0721|S0721]] (4), [[S0722|S0722]] (10), [[S0728|S0728]] (14), [[S0730|S0730]] (9), [[S0736|S0736]] (60), [[S0737|S0737]] (11), [[S0739|S0739]] (20), [[S0744|S0744]] (31), [[S0750|S0750]] (18), [[S0751|S0751]] (5), [[S0753|S0753]] (72), [[S0754|S0754]] (50), [[S0756|S0756]] (96), [[S0757|S0757]] (8), [[S0759 014690b0-0146c830|S0759]] (39), [[S0762|S0762]] (6), [[S0763|S0763]] (66), [[S0764|S0764]] (51), [[S0765|S0765]] (31), [[S0789|S0789]] (13), [[S0791|S0791]] (15), [[S0809 0066ca4d-0147b010|S0809]] (6)
