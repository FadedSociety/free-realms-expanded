# System: net.packet.combat

42 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a29780` | `BaseCombatPacket::BaseCombatPacket` | BaseCombatPacket | Initializes the family-32 receive object used by dispatcher 00a2c370 before it reads the header. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a297e0` | `BaseCombatPacket::BaseCombatPacketSubOpcode` | BaseCombatPacket | Shared sub-opcode constructor used by concrete family-32 packet classes in both send and receive paths. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29850` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | Destructor body for combat packet objects destroyed during exception unwinding. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a298a0` | `BaseCombatPacket::ScalarDeletingDtor` | BaseCombatPacket | MSVC scalar-deleting destructor for combat packet objects. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29910` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29960` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a299b0` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29a00` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29a50` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29aa0` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29af0` | `BaseCombatPacket::~BaseCombatPacket` | BaseCombatPacket | One of several identical destructor emissions used by an EH cleanup funclet of the combat packet handlers. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29c20` | `CombatPacketAutoAttackTarget::CombatPacketAutoAttackTarget` | CombatPacketAutoAttackTarget | Constructor of the client's auto-attack-target packet (family 32 / sub-opcode 1), built on the send path in 00a2c7d0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29cf0` | `CombatPacketSingleAttackTarget::CombatPacketSingleAttackTarget` | CombatPacketSingleAttackTarget | Constructor of the client's single-attack-target packet (family 32 / sub-opcode 3), built on the send path in 00a2c880. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29dc0` | `CombatPacketAttackTargetDamage::CombatPacketAttackTargetDamage` | CombatPacketAttackTargetDamage | Constructor of the damage-result packet (family 32 / sub-opcode 4) that the receive handler 00a2bff0 fills from the wire. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29e90` | `CombatPacketAttackAttackerMissed::CombatPacketAttackAttackerMissed` | CombatPacketAttackAttackerMissed | Constructor of the miss-result packet (family 32 / sub-opcode 5) filled by the receive handler 00a2b360. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a030` | `CombatPacketEnableBossDisplay::CombatPacketEnableBossDisplay` | CombatPacketEnableBossDisplay | Constructor of the boss-display toggle packet (family 32 / sub-opcode 9) filled by the receive handler 00a2b910. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a470` | `CombatPacketAttackProcessed::CombatPacketAttackProcessed` | CombatPacketAttackProcessed | Constructor of the attack-processed packet (family 32 / sub-opcode 7) filled by the receive handler 00a2ba40. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` |  | Receive dispatcher for wire family 32 (BaseCombatPacket = 0x20 per the base constructor): it routes each combat result frame to the handler that parses and applies it. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |

## 🟢 likely (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a29c80` | `CombatPacketAutoAttackTarget::ScalarDeletingDtor` | CombatPacketAutoAttackTarget | MSVC scalar-deleting destructor of the 32/1 auto-attack-target packet built by the constructor 00a29c20. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29d50` | `CombatPacketSingleAttackTarget::ScalarDeletingDtor` | CombatPacketSingleAttackTarget | MSVC scalar-deleting destructor of the 32/3 single-attack-target packet built by the constructor 00a29cf0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29e20` | `CombatPacketAttackTargetDamage::ScalarDeletingDtor` | CombatPacketAttackTargetDamage | MSVC scalar-deleting destructor of the 32/4 damage-result packet built by the constructor 00a29dc0 and filled by the handler 00a2bff0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29ef0` | `CombatPacketAttackAttackerMissed::ScalarDeletingDtor` | CombatPacketAttackAttackerMissed | MSVC scalar-deleting destructor of the 32/5 miss-result packet built by the constructor 00a29e90 and filled by the handler 00a2b360. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a29fc0` | `CombatPacketAttackTargetDodged::ScalarDeletingDtor` | CombatPacketAttackTargetDodged | MSVC scalar-deleting destructor of the 32/6 dodge-result packet built by the constructor 00a29f60 and filled by the handler 00a2b580. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a090` | `CombatPacketEnableBossDisplay::ScalarDeletingDtor` | CombatPacketEnableBossDisplay | MSVC scalar-deleting destructor of the 32/9 boss-display packet built by the constructor 00a2a030 and filled by the handler 00a2b910. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a4d0` | `CombatPacketAttackProcessed::ScalarDeletingDtor` | CombatPacketAttackProcessed | MSVC scalar-deleting destructor of the 32/7 attack-processed packet built by the constructor 00a2a470 and filled by the handler 00a2ba40. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a740` | `ReadInt16Pair` |  | Header reader shared by the family-32 code: the dispatcher 00a2c370 uses it to obtain the family and sub-opcode, and each handler's payload reader calls it first. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a7a0` | `ReadCombatPayloadTwoIds_Missed` |  | Payload reader for the miss result: the only caller is the 32/5 handler 00a2b360, whose class is CombatPacketAttackAttackerMissed. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a820` | `ReadCombatPayloadTwoIds_Dodged` |  | Payload reader for the dodge result: the only caller is the 32/6 handler 00a2b580, whose class is CombatPacketAttackTargetDodged. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a8a0` | `ReadCombatPayloadIdAndBool_BossDisplay` |  | Payload reader for the boss-display toggle: the only caller is the 32/9 handler 00a2b910, whose class is CombatPacketEnableBossDisplay. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a910` | `ReadCombatPayloadProcessed` |  | Payload reader for the attack-processed result: the only caller is the 32/7 handler 00a2ba40, whose class is CombatPacketAttackProcessed. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2aab0` | `ReadCombatPayloadDamage` |  | Payload reader for the damage result: the only caller is the 32/4 handler 00a2bff0, whose class is CombatPacketAttackTargetDamage. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2b360` | `HandleCombatPacketAttackAttackerMissed` |  | Client receive handler for the miss result: it is one of the five handlers called by the family-32 dispatcher 00a2c370 (case 5). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2b910` | `HandleCombatPacketEnableBossDisplay` |  | Client receive handler for the boss-display toggle: one of the five handlers called by the family-32 dispatcher 00a2c370 (case 9). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c2d0` | `WritePacketFamilyAndSubOpcode` |  | Serialises the family and sub-opcode shorts of a packet object into an outgoing tunneled client packet; both callers are combat send wrappers 00a2c550 and 00a2c670. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c7d0` | `ClientAutoAttackTarget` |  | Client action that builds the 32/1 auto-attack request and sends it inside PacketTunneledClientPacket through 00a2c550; reached from 008c6470 and target-name handlers 00a9e790/00a9e8f0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c880` | `ClientSingleAttackTarget` |  | Client action that builds the 32/3 single-attack request and sends it inside PacketTunneledClientPacket through 00a2c670; reached from 008c6470 and target-name handlers 00a9e790/00a9e8f0. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a29f60` | `— ⚠reject` | CombatPacketAttackTargetDodged | Constructor of the dodge-result packet (family 32 / sub-opcode 6) filled by the receive handler 00a2b580. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2b580` | `— ⚠reject` |  | Client receive handler for the dodge result: one of the five handlers called by the family-32 dispatcher 00a2c370 (case 6). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2ba40` | `— ⚠reject` |  | Client receive handler for the attack-processed result: one of the five handlers called by the family-32 dispatcher 00a2c370 (case 7). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2bff0` | `— ⚠reject` |  | Client receive handler for the landed-hit result: one of the five handlers called by the family-32 dispatcher 00a2c370 (case 4). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c550` | `— ⚠reject` |  | Send wrapper used by the auto-attack action 00a2c7d0; its only caller builds CombatPacketAutoAttackTarget, while this wrapper sends the outer tunnel packet. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2c670` | `— ⚠reject` |  | Send wrapper used by the single-attack action 00a2c880; its only caller builds CombatPacketSingleAttackTarget, while this wrapper sends the outer tunnel packet. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a29780` | `BaseCombatPacket::BaseCombatPacket` | recv |  | 32 |  |
| `00a29c20` | `CombatPacketAutoAttackTarget::CombatPacketAutoAttackTarget` | send |  | 1 | Hypothesis: the original server treated 32/1 as the player's auto-attack request against the carried target id and answered with a 32/5, 32/6, 32/4 or 32/7 result frame; the inference depends on the sibling result packets and their handlers in this shard. |
| `00a29cf0` | `CombatPacketSingleAttackTarget::CombatPacketSingleAttackTarget` | send |  | 3 | Hypothesis: the original server read 32/3 as a single-target attack request (one chosen target rather than the auto-attack sweep) and answered with the same result frames as 32/1; depends on the shared result handlers in this shard. |
| `00a29dc0` | `CombatPacketAttackTargetDamage::CombatPacketAttackTargetDamage` | recv |  | 4 | Hypothesis: the original server emitted 32/4 as the landed-hit result naming attacker and target ids plus a damage value and a flag; the client's reaction (health write, damage feedback, deselection on death) is visible in 00a2bff0. |
| `00a29e90` | `CombatPacketAttackAttackerMissed::CombatPacketAttackAttackerMissed` | recv |  | 5 | Hypothesis: the original server sent 32/5 when an attack missed, carrying the two actor ids, and the client only displayed the MISS! feedback; depends on 00a2b360 and its payload reader 00a2a7a0. |
| `00a2a030` | `CombatPacketEnableBossDisplay::CombatPacketEnableBossDisplay` | recv |  | 9 | Hypothesis: the original server used 32/9 to switch an actor into (or out of) boss-frame display mode, which matches the client setting bit 7 of the actor byte at +0x142 in 00a2b910. |
| `00a2a470` | `CombatPacketAttackProcessed::CombatPacketAttackProcessed` | recv |  | 7 | Hypothesis: the original server emitted the widest combat result frame (32/7) as the authoritative resolution of one attack, carrying both actor ids, several counters, two flag bytes and two further dwords; the client applied it in 00a2ba40. |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` | recv |  | 4 | Hypothesis: the original server emitted these five combat result frames (sub-opcodes 4, 5, 6, 7, 9 of family 32) after resolving an attack, and the client never emitted them itself; the inference depends on the sibling constructors and the two client send paths 00a2c7d0/00a2c880 in this shard. |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` | recv |  | 5 | Hypothesis: the original server emitted these five combat result frames (sub-opcodes 4, 5, 6, 7, 9 of family 32) after resolving an attack, and the client never emitted them itself; the inference depends on the sibling constructors and the two client send paths 00a2c7d0/00a2c880 in this shard. |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` | recv |  | 6 | Hypothesis: the original server emitted these five combat result frames (sub-opcodes 4, 5, 6, 7, 9 of family 32) after resolving an attack, and the client never emitted them itself; the inference depends on the sibling constructors and the two client send paths 00a2c7d0/00a2c880 in this shard. |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` | recv |  | 7 | Hypothesis: the original server emitted these five combat result frames (sub-opcodes 4, 5, 6, 7, 9 of family 32) after resolving an attack, and the client never emitted them itself; the inference depends on the sibling constructors and the two client send paths 00a2c7d0/00a2c880 in this shard. |
| `00a2c370` | `DispatchBaseCombatPacket → CombatProcessor::DispatchBaseCombatPacket` | recv |  | 9 | Hypothesis: the original server emitted these five combat result frames (sub-opcodes 4, 5, 6, 7, 9 of family 32) after resolving an attack, and the client never emitted them itself; the inference depends on the sibling constructors and the two client send paths 00a2c7d0/00a2c880 in this shard. |
| `00a2c2d0` | `WritePacketFamilyAndSubOpcode` | send | PacketTunneledClientPacket |  |  |
| `00a2c7d0` | `ClientAutoAttackTarget` | send | PacketTunneledClientPacket |  | Hypothesis: the original server answered inner frame 32/1 with a 32/5, 32/6, 32/4 or 32/7 result frame; depends on the receive dispatcher 00a2c370 and its handlers in this shard. |
| `00a2c880` | `ClientSingleAttackTarget` | send | PacketTunneledClientPacket |  | Hypothesis: the original server answered inner frame 32/3 with one of the result frames 32/4, 32/5, 32/6 or 32/7 handled by 00a2c370; the exact server logic is unknown. |
| `00a29f60` | `— ⚠reject` | recv |  | 6 | Hypothesis: the original server sent 32/6 when the target dodged, carrying the two actor ids; the client updated the target's displayed health from the read values and showed dodge feedback. |
| `00a2c550` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
| `00a2c670` | `— ⚠reject` | send | PacketTunneledClientPacket |  |  |
