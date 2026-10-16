# System: net.packet.loyalty

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5190` | `BaseLoyaltyRewardPacket::BaseLoyaltyRewardPacket` | BaseLoyaltyRewardPacket | Base constructor for opcode-173 loyalty messages; parser 0090e7b0 constructs sub-opcode 0 through 008a52c0. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5200` | `BaseLoyaltyRewardPacket::~BaseLoyaltyRewardPacket` | BaseLoyaltyRewardPacket | Destructor of the loyalty reward packet class; the two in-body vtable restores name the class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5250` | `BaseLoyaltyRewardPacket::ScalarDeletingDestructor` | BaseLoyaltyRewardPacket | Compiler-generated delete wrapper for the loyalty reward class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a52c0` | `SetLoyaltyRewardLogoutInformation::SetLoyaltyRewardLogoutInformation` | SetLoyaltyRewardLogoutInformation | Constructs opcode-173/sub-0 while parser 0090e7b0 reads two dwords and stores them at +0xc20/+0xc24. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5330` | `SetLoyaltyRewardLogoutInformation::~SetLoyaltyRewardLogoutInformation` | SetLoyaltyRewardLogoutInformation | Destructor of the logout-information packet; the three in-body vtable restores name the full class chain. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a5390` | `SetLoyaltyRewardLogoutInformation::ScalarDeletingDestructor` | SetLoyaltyRewardLogoutInformation | Compiler-generated delete wrapper for the logout-information class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a5190` | `BaseLoyaltyRewardPacket::BaseLoyaltyRewardPacket` | recv |  | 173 | Hypothesis: the peer sends family-173 messages parsed by 0090e7b0. This constructor and parser establish the client wire shape only; original reward calculation is not shown. |
| `008a52c0` | `SetLoyaltyRewardLogoutInformation::SetLoyaltyRewardLogoutInformation` | recv |  | 0 | Hypothesis: the peer supplies the two values parsed by 0090e7b0 in family-173/sub-0. Their original-server meaning is unknown. |
