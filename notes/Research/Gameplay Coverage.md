# Gameplay coverage

Updated after [[Gameplay Milestone]]. Evidence, implementation and native acceptance are separate claims.

| System/content | Evidence located | Behavior understood | Rust implementation | Isolation checks | Actual client |
| --- | --- | --- | --- | --- | --- |
| Login, one character/zone, movement | Supplied emulator and real capture | Supported flow | Yes; 46 private templates remain | Yes | Login/walk/restart verified |
| Inventory instances and profiles | 1,386 instances, 16 profiles; exact private spans | Supported fields | Typed state/serialization | Yes, private byte oracle | Lists and restored quantities visible |
| Equipment | Supplied/community serializers | Equip/unequip shapes and validation | Yes | Yes | Equip instance 2/profile 1/slot 4 and restore verified; native unequip appearance not accepted yet |
| Wallet and rewards | Packet shapes and definitions | Atomic local grant/reward-once | Yes | Yes | 25 + 10 coins; quantity 9 to 10; restored |
| Item categories | Community 71 categories/74 associations | 44/2 exact serializer match | Yes | Yes, 2,675-byte oracle | Stuff populates and category browsing works |
| Inventory search | Native UI and partial analysis | Cause of access violation unknown | Native client dependency | Not resolved | Crashed once; not accepted |
| Item use/effects | Definition/ability IDs | Incomplete native request mapping | Server-authorized consume only | Consume checked | Not verified |
| Original quest journal | 25 exact records and locale text | Serialization and one proven counter | Preserved/patched supported counter | Yes | Original presentation survives; original completion not verified |
| Original quest 1617 | Petty Yeti objective, required 20, item 1843 reward | Enemy identity unresolved | Matching disabled until evidenced | Wrong-enemy rejection checked | Not playable |
| Reconstructed quest 900001 | Explicit local design, original Robgoblin/item records | Accept/three kills/reward once | Yes | Yes | Accepted, completed and restored |
| Reconstructed combat | Original Robgoblin model and packet shapes | Local 100 HP/25-hit rules | Damage/range/cooldown/death/respawn | Yes | Hits, defeats, despawn verified; no AI/retaliation |
| Reconstructed checkpoint activity | Explicit local design and real movement | Ordered course/timing/reward/cooldown | Yes | Yes | 18.456s, 10 coins, record restored; timeout no reward |
| Original soccer | Original arena/config/team/ball/goal captures | Partial lifecycle | Not yet | Capture CRC/schema checks only | Not launched against Rust |
| Original racing/derby | Original partial traces | Partial; racing has four sequence gaps | Not yet | Trace analysis only | Not launched against Rust |
| Other original minigames | 783 MiniGameData references and UI/assets | Unknown coverage | No | No | No |
| Jobs, crafting, gathering, collections | Some surviving presentation/data | Not measured | No gameplay restoration | No | No |
| Housing, pets/mounts, social, travel, multiplayer | Partial data/UI only | Not measured | No gameplay restoration | No | No |
| Replacement Rust client | Native client/assets as reference | Research only | No renderer | No | Original Windows client required |

Private input counts are not completion counts. No original full game playthrough or broad content coverage is claimed.
