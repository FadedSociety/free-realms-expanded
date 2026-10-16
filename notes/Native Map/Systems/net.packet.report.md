# System: net.packet.report

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5650` | `BaseReportPacket::BaseReportPacket` | BaseReportPacket | Base constructor of the opcode-68 report family; callers are 008dd550 and 00aae4f0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a56c0` | `BaseReportPacket::~BaseReportPacket` | BaseReportPacket | Destructor of the report packet class; the two in-body vtable restores name the class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5710` | `BaseReportPacket::ScalarDeletingDestructor` | BaseReportPacket | Compiler-generated delete wrapper for the report packet class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008d1e10` | `BaseReportPacket::~BaseReportPacket` | BaseReportPacket | Base teardown of the report packet family, destroying its ten embedded report records and one string; its caller 008dd5e0 is the family's slot-0 scalar deleting destructor. | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `00aae4f0` | `ReportPacketReportHouse::ctor` | ReportPacketReportHouse | Client-side report message for a house; send wrapper 00ac71a0 constructs and sends it. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00ab0a40` | `ReportPacketReportHouseScalarDeletingDtor` | ReportPacketReportHouse | Scalar deleting destructor for ReportPacketReportHouse. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008d1cf0` | `Ctor_TwoStringsAt8And18` |  | Element constructor of the ten-record inline array BaseReportPacket destroys through its vector destructor 008d1e10 (element size 0x30, same two-string layout as the array element destructor 008d1d60); its only caller i… | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `008d1d60` | `Dtor_TwoStringsAt8And18` |  | Element destructor of the inline report array inside BaseReportPacket: the packet destructor 008d1e10 passes exactly this function to the CRT vector-destructor iterator (element size 0x30, count 10). | [[Native Map/Shards/S0227 008d0920-008d5c60|S0227]] |
| `00aae580` | `ReportPacketReportHouse::~ReportPacketReportHouse` | ReportPacketReportHouse | Destruction of the 68/3 report packet; send wrapper 00ac71a0 pairs it with the constructor 00aae4f0. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00ac19a0` | `SendReportPacketReportHouse` | ReportPacketReportHouse | Send helper for the report request that SendReportHouse builds from the selected scoreboard row with the constructor FUN_00aae4f0. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac71a0` | `SendReportHouse` |  | Submits a report (moderation complaint) about the house row currently selected in the browser. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a5650` | `BaseReportPacket::BaseReportPacket` | send |  | 68 | Hypothesis: the peer accepts client report-family messages. Client report builders 008dd550 and 00aae4f0 establish the send path; this constructor does not reveal server review, moderation or telemetry handling. |
| `00aae4f0` | `ReportPacketReportHouse::ctor` | send |  | 3 | Hypothesis: the client reports a house (or its content) to the moderation service with two strings plus two ids and sends 68/3; the original server had to record the report. The two id fields and the string roles are not recovered. |
