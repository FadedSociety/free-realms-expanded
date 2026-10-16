# System: net.packet.soccer

76 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (52)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b73110` | `BaseSoccerPacket::~BaseSoccerPacket` | BaseSoccerPacket | Shared cleanup for every soccer packet temp; it is the most-called destructor in the shard (77 callers, almost all MSVC unwind funclets that clean up packet temporaries). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73160` | `BaseSoccerPacket::ScalarDeletingDtor` | BaseSoccerPacket | Vtable slot 0 of BaseSoccerPacket; the deleting half of the family-76 packet destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b731d0` | `SoccerPacketDtor_00b731d0` |  | One of the per-class destructors MSVC emitted for family-76 packet temporaries destroyed by the dispatcher FUN_00b80ac0 and its unwind funclets; the body cannot distinguish which derived class it was emitted for. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73220` | `SoccerPacketDtor_00b73220` |  | Destroyer used by the sub-14 UpdatePlayerState sender FUN_00cdc5c0 (which also constructs 00b746b0), so this copy is most plausibly SoccerPacketUpdatePlayerState's destructor; the body itself does not name a class. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73270` | `SoccerPacketDtor_00b73270` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b732c0` | `SoccerPacketDtor_00b732c0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73310` | `SoccerPacketDtor_00b73310` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b733c0` | `SoccerPacketDtor_00b733c0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73410` | `SoccerPacketDtor_00b73410` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73460` | `SoccerPacketDtor_00b73460 → SoccerPacketRegisterPlayer::~SoccerPacketRegisterPl…` |  | Destroyer called by FUN_00b83710, the same client path that constructs SoccerPacketRegisterPlayer (ctor 00b74b30), so this copy is most plausibly that class's destructor; the body itself does not name a class. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b734b0` | `SoccerPacketDtor_00b734b0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b735b0` | `SoccerPacketDtor_00b735b0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73600` | `SoccerPacketDtor_00b73600` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73650` | `SoccerPacketDtor_00b73650` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73740` | `SoccerPacketDtor_00b73740 → SoccerPacketFreeBall::~SoccerPacketFreeBall` |  | Destroyer called by FUN_00cdc710, the same client path that constructs SoccerPacketFreeBall (ctor 00b75080), so this copy is most plausibly that class's destructor; the body itself does not name a class. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b737f0` | `SoccerPacketDtor_00b737f0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73840` | `SoccerPacketBallBounce::Ctor` | SoccerPacketBallBounce | Constructs the temporary packet object used by the client family-76 receive dispatcher for sub 0x19; the dispatcher parses its body and applies the received bounce notice, including the selected ball sound/effect action. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b738a0` | `SoccerPacketDtor_00b738a0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73950` | `BaseSoccerPacket::~BaseSoccerPacket` | BaseSoccerPacket | Shared base packet destructor body used by the family-76 dispatcher and an EH cleanup path; this body does not identify a derived packet class. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b739a0` | `SoccerPacketDtor_00b739a0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b739f0` | `SoccerPacketDtor_00b739f0` |  | Per-class destructor emitted for one family-76 packet temporary destroyed in the dispatcher path; no derived class is identifiable from the body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74640` | `SoccerPacketStatRollInfo::ScalarDeletingDtor` | SoccerPacketStatRollInfo | Vtable slot 0 of SoccerPacketStatRollInfo; deleting half of the sub-17 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b746b0` | `SoccerPacketUpdatePlayerState::Ctor` | SoccerPacketUpdatePlayerState | Builds the player-state message; the vault note maps sub 14 as the client's action channel (62-byte payload: u64 guid, u8 state code, u8 aux, u32 word, three f32[4] vectors). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74710` | `SoccerPacketUpdatePlayerState::ScalarDeletingDtor` | SoccerPacketUpdatePlayerState | Vtable slot 0 of SoccerPacketUpdatePlayerState; deleting half of the sub-14 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b747e0` | `SoccerPacketUpdatePlayerStats::ScalarDeletingDtor` | SoccerPacketUpdatePlayerStats | Vtable slot 0 of SoccerPacketUpdatePlayerStats; deleting half of the sub-15 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74850` | `SoccerPacketUpdatePlayerPos::Ctor` | SoccerPacketUpdatePlayerPos | Builds the player-position update (sub 1, the most common soccer message in the capture corpus); sender FUN_00cdc4a0 constructs it for the client. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b748b0` | `SoccerPacketUpdatePlayerPos::ScalarDeletingDtor` | SoccerPacketUpdatePlayerPos | Vtable slot 0 of SoccerPacketUpdatePlayerPos; deleting half of the sub-1 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74920` | `SoccerPacketUpdatePlayerControls::Ctor` | SoccerPacketUpdatePlayerControls | Builds the player-controls message; the vault note maps sub 16 as a 20-byte payload (u64 guid, two direction floats, u32 control bitfield) sent by the client's input path FUN_00afb4a0. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74980` | `SoccerPacketUpdatePlayerControls::ScalarDeletingDtor` | SoccerPacketUpdatePlayerControls | Vtable slot 0 of SoccerPacketUpdatePlayerControls; deleting half of the sub-16 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b749f0` | `SoccerPacketGoalMade::ScalarDeletingDtor` | SoccerPacketGoalMade | Vtable slot 0 of SoccerPacketGoalMade; deleting half of the sub-2 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74ac0` | `SoccerPacketSpecialAnimTime::ScalarDeletingDtor` | SoccerPacketSpecialAnimTime | Vtable slot 0 of SoccerPacketSpecialAnimTime; deleting half of the sub-22 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74b90` | `SoccerPacketRegisterPlayer::ScalarDeletingDtor` | SoccerPacketRegisterPlayer | Vtable slot 0 of SoccerPacketRegisterPlayer; deleting half of the sub-3 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74c60` | `SoccerPacketPlayerVariableStats::ScalarDeletingDtor` | SoccerPacketPlayerVariableStats | Vtable slot 0 of SoccerPacketPlayerVariableStats; deleting half of the sub-19 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74cd0` | `SoccerPacketUpdateSoccerBall::Ctor` | SoccerPacketUpdateSoccerBall | Builds the ball-state update (sub 4); the client path FUN_00cdc9a0 constructs it and the capture corpus round-trips 83 sub-4 records outside this shard. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74d30` | `SoccerPacketUpdateSoccerBall::ScalarDeletingDtor` | SoccerPacketUpdateSoccerBall | Vtable slot 0 of SoccerPacketUpdateSoccerBall; deleting half of the sub-4 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74da0` | `SoccerPacketHighArcBall::ScalarDeletingDtor` | SoccerPacketHighArcBall | Vtable slot 0 of SoccerPacketHighArcBall; deleting half of the sub-0x12 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74e70` | `SoccerPacketSetSoccerPlayerOneTimeStats::ScalarDeletingDtor` | SoccerPacketSetSoccerPlayerOneTimeStats | Vtable slot 0 of SoccerPacketSetSoccerPlayerOneTimeStats; deleting half of the sub-21 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74ee0` | `SoccerPacketAcquirePickUp::Ctor` | SoccerPacketAcquirePickUp | Builds the pick-up-acquired message (sub 0xa); the client path FUN_00b83ae0 constructs it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74f40` | `SoccerPacketAcquirePickUp::ScalarDeletingDtor` | SoccerPacketAcquirePickUp | Vtable slot 0 of SoccerPacketAcquirePickUp; deleting half of the sub-0xa packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75010` | `SoccerPacketAcquireBall::ScalarDeletingDtor` | SoccerPacketAcquireBall | Vtable slot 0 of SoccerPacketAcquireBall; deleting half of the sub-11 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75080` | `SoccerPacketFreeBall::Ctor` | SoccerPacketFreeBall | Builds the ball-released message (sub 12); the client path FUN_00cdc710 constructs it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b750e0` | `SoccerPacketFreeBall::ScalarDeletingDtor` | SoccerPacketFreeBall | Vtable slot 0 of SoccerPacketFreeBall; deleting half of the sub-12 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75150` | `SoccerPacketImpulseBall::ScalarDeletingDtor` | SoccerPacketImpulseBall | Vtable slot 0 of SoccerPacketImpulseBall; deleting half of the sub-13 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b751c0` | `SoccerPacketBallBounce::ScalarDeletingDtor` | SoccerPacketBallBounce | Vtable slot 0 of SoccerPacketBallBounce; deleting half of the sub-0x19 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75230` | `SoccerPacketUpdateGameTime::ScalarDeletingDtor` | SoccerPacketUpdateGameTime | Vtable slot 0 of SoccerPacketUpdateGameTime; deleting half of the sub-0x17 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b752a0` | `SoccerPacketUpdateGameState::Ctor` | SoccerPacketUpdateGameState | Builds the game-state message (sub 5); the client path FUN_00b839e0 constructs it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75300` | `SoccerPacketUpdateGameState::ScalarDeletingDtor` | SoccerPacketUpdateGameState | Vtable slot 0 of SoccerPacketUpdateGameState; deleting half of the sub-5 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b753d0` | `SoccerPacketSetPlayerTeam::ScalarDeletingDtor` | SoccerPacketSetPlayerTeam | Vtable slot 0 of SoccerPacketSetPlayerTeam; deleting half of the sub-6 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75fe0` | `SoccerPacketSpawnPickUp::ScalarDeletingDtor` | SoccerPacketSpawnPickUp | Vtable slot 0 of SoccerPacketSpawnPickUp; deleting half of the sub-9 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77ca0` | `SoccerPacketSendAIInfo::ScalarDeletingDtor` | SoccerPacketSendAIInfo | Vtable slot 0 of SoccerPacketSendAIInfo; deleting half of the sub-24 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77cc0` | `SoccerPacketSetClientConfig::ScalarDeletingDtor` | SoccerPacketSetClientConfig | Vtable slot 0 of SoccerPacketSetClientConfig; deleting half of the sub-7 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77d50` | `SoccerPacketTeamColours::ScalarDeletingDtor` | SoccerPacketTeamColours | Vtable slot 0 of SoccerPacketTeamColours; deleting half of the sub-8 packet's destructor pair. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## 🟢 likely (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b73500` | `SoccerPacketUpdateSoccerBall::~SoccerPacketUpdateSoccerBall` |  | The family-76 dispatcher case for this packet invokes this cleanup after parsing/applying the temporary; the derived-type pairing comes from that caller and its EH cleanup path, not from this body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b736a0` | `SoccerPacketAcquirePickUp::~SoccerPacketAcquirePickUp` |  | The family-76 dispatcher case for this packet invokes this cleanup after parsing/applying the temporary; the derived-type pairing comes from that caller and its EH cleanup path, not from this body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b736f0` | `SoccerPacketAcquireBall::~SoccerPacketAcquireBall` |  | The family-76 dispatcher case for this packet invokes this cleanup after parsing/applying the temporary; the derived-type pairing comes from that caller and its EH cleanup path, not from this body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73790` | `SoccerPacketImpulseBall::Ctor ⚠downgrade` | SoccerPacketImpulseBall | Builds the kick/pass impulse message; the vault note maps sub 13 as SoccerPacketImpulseBall with a 36-byte payload (u32, f32[4], f32, f32, u32, f32) and no player field. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74b30` | `SoccerPacketRegisterPlayer::Ctor ⚠downgrade` | SoccerPacketRegisterPlayer | Builds the player-registration message (sub 3); the client path FUN_00b83710 constructs it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76d50` | `SoccerPacketSendAIInfo::~SoccerPacketSendAIInfo` | SoccerPacketSendAIInfo | The class's ~1 function; the pairing rests on the scalar deleting dtor FUN_00b77ca0 (slot 0 of SoccerPacketSendAIInfo per the binary vtable) calling exactly this body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76e40` | `SoccerPacketSetClientConfig::~SoccerPacketSetClientConfig` | SoccerPacketSetClientConfig | The class's real destructor that releases the configuration string; the scalar deleting dtor FUN_00b77cc0 (slot 0 of SoccerPacketSetClientConfig) calls it. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76ed0` | `SoccerPacketTeamColours::~SoccerPacketTeamColours` | SoccerPacketTeamColours | The class's ~1 function; the pairing rests on the scalar deleting dtor FUN_00b77d50 (slot 0 of SoccerPacketTeamColours) calling exactly this body. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## ⚪ unknown (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b73040` | `— ⚠reject` | BaseSoccerPacket | Vtable-installing base constructor used by the family-76 dispatcher FUN_00b80ac0 for the shared temp object before a derived packet constructor is applied. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b730a0` | `— ⚠reject` | BaseSoccerPacket | Shared family-76 constructor called by all 23 derived soccer packet constructors; the vault note pins it as the base family-76 ctor. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73360` | `— ⚠reject` | SoccerPacketGoalMade | Builds the goal-made notice sent by the soccer score path; sub id 2 names the message on the wire. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b73550` | `— ⚠reject` | SoccerPacketHighArcBall | Builds the high-arc ball trajectory notice (sub 0x12); the payload is filled by the kick/ball code that uses this class. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b738f0` | `— ⚠reject` | SoccerPacketUpdateGameTime | Builds the game-clock update (sub 0x17); the capture corpus shows sub 23 messages on the wire. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b745e0` | `— ⚠reject` | SoccerPacketStatRollInfo | Builds the stat-roll info message (sub 17); the capture corpus round-trips 17-byte sub-17 records outside this shard. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74780` | `— ⚠reject` | SoccerPacketUpdatePlayerStats | Builds the player-statistics update message (sub 15). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74a60` | `— ⚠reject` | SoccerPacketSpecialAnimTime | Builds the special-animation timing message (sub 22). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74c00` | `— ⚠reject` | SoccerPacketPlayerVariableStats | Builds the player variable-statistics message (sub 19). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74e10` | `— ⚠reject` | SoccerPacketSetSoccerPlayerOneTimeStats | Builds the one-time player-stat message (sub 21). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b74fb0` | `— ⚠reject` | SoccerPacketAcquireBall | Builds the ball-acquired message (sub 11). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75370` | `— ⚠reject` | SoccerPacketSetPlayerTeam | Builds the team-assignment message (sub 6), typically server-authored in a match flow. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b75f80` | `— ⚠reject` | SoccerPacketSpawnPickUp | Builds the pick-up spawn message (sub 9); the vault capture round-trips 8 sub-9 records outside this shard. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b76dc0` | `— ⚠reject` | SoccerPacketSetClientConfig | Builds the client-configuration message (sub 7) that carries a string payload; the embedded IString is why this class has a real destructor (FUN_00b76e40) instead of the trivial vtable reset. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77c40` | `— ⚠reject` | SoccerPacketSendAIInfo | Builds the AI-info message (sub 24) sent by the AI-controlled player path. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00b77ce0` | `— ⚠reject` | SoccerPacketTeamColours | Builds the team-colours message (sub 8), typically server-authored when a match assigns teams. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b73840` | `SoccerPacketBallBounce::Ctor` | recv |  | 25 |  |
| `00b746b0` | `SoccerPacketUpdatePlayerState::Ctor` | send |  | 14 | Hypothesis (note-derived): the original server read the state byte as the client's kick/pass action request and had to reject payload-guids that differ from the authenticated sender; the reader FUN_00b794b0/sender FUN_00cdc5c0 and the state-name table live outside this shard. |
| `00b74850` | `SoccerPacketUpdatePlayerPos::Ctor` | send |  | 1 |  |
| `00b74920` | `SoccerPacketUpdatePlayerControls::Ctor` | send |  | 16 | Hypothesis (note-derived): the original server rejected payload-guids that differ from the authenticated sender and treated the direction floats plus bitfield as advisory input, never as authoritative movement. |
| `00b74cd0` | `SoccerPacketUpdateSoccerBall::Ctor` | send |  | 4 |  |
| `00b74ee0` | `SoccerPacketAcquirePickUp::Ctor` | send |  | 10 |  |
| `00b75080` | `SoccerPacketFreeBall::Ctor` | send |  | 12 |  |
| `00b752a0` | `SoccerPacketUpdateGameState::Ctor` | send |  | 5 |  |
| `00b73790` | `SoccerPacketImpulseBall::Ctor ⚠downgrade` | send |  | 13 | Hypothesis (note-derived): an impulse names no player, so the original server had to bind it to the authenticated session and the current possession; the impulse floats are attacker-controlled, so a server slice must bound velocity and ignore impulses from players without the ball. The reader side is FUN_00b79580/FUN_00b7b840 (outside this shard). |
| `00b74b30` | `SoccerPacketRegisterPlayer::Ctor ⚠downgrade` | send |  | 3 | Hypothesis: a server used sub 3 to attach the authenticated session to a soccer participant slot; the note's reviewed capture slice also shows a player guid in this family. |
| `00b73040` | `— ⚠reject` | send |  | 76/0 |  |
| `00b730a0` | `— ⚠reject` | send |  | 76/2 |  |
| `00b73360` | `— ⚠reject` | send |  | 2 | Hypothesis: a server that scores a goal emits this sub-2 message with the scored-against team/player fields; the payload layout is read by the separately-labelled goal handling code, so the server contract depends on that reader. |
| `00b73550` | `— ⚠reject` | send |  | 18 | Hypothesis: the high-arc variant of a ball impulse is a server-authoritative trajectory notice; without a captured sub-0x12 record its payload meaning stays unproven. |
| `00b738f0` | `— ⚠reject` | send |  | 23 |  |
| `00b745e0` | `— ⚠reject` | send |  | 17 |  |
| `00b74780` | `— ⚠reject` | send |  | 15 |  |
| `00b74a60` | `— ⚠reject` | send |  | 22 |  |
| `00b74c00` | `— ⚠reject` | send |  | 19 |  |
| `00b74e10` | `— ⚠reject` | send |  | 21 |  |
| `00b74fb0` | `— ⚠reject` | send |  | 11 |  |
| `00b75370` | `— ⚠reject` | send |  | 6 |  |
| `00b75f80` | `— ⚠reject` | send |  | 9 |  |
| `00b76dc0` | `— ⚠reject` | send |  | 7 |  |
| `00b77c40` | `— ⚠reject` | send |  | 24 |  |
| `00b77ce0` | `— ⚠reject` | send |  | 8 |  |
