# System: net.minigame.soccer

10 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | SoccerProcessor | Receive-side dispatcher for the family-76 soccer packets on the client. The vault note maps sub 13 to the kick/pass impulse and sub 16 to the player controls. | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00cdc4a0` | `SoccerPlayer::SendUpdatePlayerPos` | SoccerPlayer | Sends the SoccerPlayer position-update packet through the shared gateway wrapper. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc5c0` | `SoccerPlayer::SendUpdatePlayerState` | SoccerPlayer | Sends the player's family-76 state update when the gateway is available. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc710` | `SoccerPlayer::SendFreeBall` | SoccerPlayer | Submits the soccer player's free-ball packet from the current ball state. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc810` | `SoccerPlayer::SendUpdatePlayerControls` | SoccerPlayer | Sends changed soccer-player controls and updates the local control path. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cdc9a0` | `SoccerPlayer::SendSoccerBallUpdate` | SoccerPlayer | Sends a soccer-ball state packet from the SoccerPlayer path. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |
| `00cde9b0` | `SoccerBall::ApplyOrSendImpulse` | SoccerBall | Applies the selected ball operation and conditionally sends its family-76 impulse packet. | [[Native Map/Shards/S0420 00cdb370-00ce0140|S0420]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b83380` | `— ⚠reject` |  | Thiscall with two stack arguments (record pointer, flag 1), ret 8. Creates a PacketWriter on the stack (FUN_008d4a40 with 0x10000000), serializes the record at the first argument through FUN_00b7fc40 (FUN_008fb810 on +0… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b83490` | `— ⚠reject` |  | Same tunnel template as TunnelSerializeRecordA, with a 0x54-byte record at the first argument. Serializer FUN_00b7fcf0 writes 4-byte fields from +0x0c to +0x30 directly, then calls helpers for +0x34 through +0x54 (FUN_0… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |
| `00b835a0` | `— ⚠reject` |  | Same tunnel template as TunnelSerializeRecordA, thiscall with a record pointer and flag 1, ret 8. The record (first argument, moved to ECX) is serialized by FUN_00b7ffd0, which writes the byte at +0x0c, the dword at +0x… | [[Native Map/Shards/S0356 00b80ac0-00b84ce0|S0356]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 1 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 2 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 3 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 4 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 5 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 6 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 7 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 8 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 9 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 10 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 11 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 12 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 13 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 14 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 15 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 16 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 17 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 18 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 19 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 21 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 22 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 23 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 24 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00b80ac0` | `SoccerProcessor::DispatchFamily76Packet` | recv |  | 25 | Hypothesis: the original server sends family-76 soccer sub packets to the client, and this switch is the client decoder for them. Depends on the sub-13 and sub-16 layouts in the vault note and on the sub-id constructors listed in open_questions. |
| `00cdc4a0` | `SoccerPlayer::SendUpdatePlayerPos` | send | PacketTunneledClientPacket |  |  |
| `00cdc5c0` | `SoccerPlayer::SendUpdatePlayerState` | send | PacketTunneledClientPacket |  |  |
| `00cdc710` | `SoccerPlayer::SendFreeBall` | send | PacketTunneledClientPacket |  |  |
| `00cdc810` | `SoccerPlayer::SendUpdatePlayerControls` | send | PacketTunneledClientPacket |  |  |
| `00cdc9a0` | `SoccerPlayer::SendSoccerBallUpdate` | send | PacketTunneledClientPacket |  |  |
| `00cde9b0` | `SoccerBall::ApplyOrSendImpulse` | send | PacketTunneledClientPacket |  |  |
