# System: net.packet.reference_data

17 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a44f0` | `ReferenceDataPacket::ReferenceDataPacket` | ReferenceDataPacket | Base constructor for opcode-44 reference data; dispatcher 00928d00 calls it while parsing incoming reference-data messages. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4550` | `ReferenceDataPacket::ReferenceDataPacketSub` | ReferenceDataPacket | Sub-opcode constructor for opcode-44 reference data; parser 00928d00 constructs item-class, item-category and profile-data packet objects through this family. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a45c0` | `ReferenceDataPacket::~ReferenceDataPacket` | ReferenceDataPacket | Destructor of the reference-data packet base class; the two vtable restores inside the body name the class without relying on callers. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4610` | `ReferenceDataPacket::ScalarDeletingDestructor` | ReferenceDataPacket | Compiler-generated delete wrapper for the reference-data packet class; no direct callers, it is reached through the class vtable. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4680` | `ReferenceDataPacketItemClassDefinitions::ReferenceDataPacketItemClassDefinitions` | ReferenceDataPacketItemClassDefinitions | Constructs the item-class definitions object while dispatcher 00928d00 parses reference-data sub-opcode 1; the catalogue class identifies packet 44/1. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a4740` | `ReferenceDataPacketItemCategoryDefinitions::ReferenceDataPacketItemCategoryDefi…` | ReferenceDataPacketItemCategoryDefiniti… | Constructs the item-category definitions object while dispatcher 00928d00 parses reference-data sub-opcode 2; the catalogue class identifies packet 44/2. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008b7cb0` | `ReferenceDataPacketItemClassDefinitions::ScalarDeletingDtor` | ReferenceDataPacketItemClassDefinitions | Releases this reference-data packet allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b7d20` | `ReferenceDataPacketItemCategoryDefinitions::ScalarDeletingDtor` | ReferenceDataPacketItemCategoryDefiniti… | Releases this reference-data packet allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00928d00` | `PlayClient::DispatchReferenceData` | PlayClient | Applies incoming reference-data subtype records to definitions and associated UI/data sources. | [[Native Map/Shards/S0244 00925fe0-0092b110|S0244]] |
| `00a39bb0` | `ReferenceDataPacketClientProfileData::ReferenceDataPacketClientProfileData` | ReferenceDataPacketClientProfileData | Builds the packet object that wraps the ClientProfileData table of the reference-data family; the community catalogue names opcode 44 sub-opcode 3 ReferenceDataPacketClientProfileData. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a39d90` | `ReferenceDataPacketClientProfileData::ScalarDeletingDestructor` | ReferenceDataPacketClientProfileData | Heap-delete path of the reference-data client-profile packet, i.e. the slot-0 entry of ReferenceDataPacketClientProfileData::vftable. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3a900` | `ReferenceDataPacketClientProfileData_Parse` | ReferenceDataPacketClientProfileData | Handler body the PlayClient reference-data dispatcher 00928d00 runs for sub-opcode 3, so this is where an inbound ClientProfileData table replaces the client's stored copy. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a46f0` | `ReferenceDataPacketItemClassDefinitions::~ReferenceDataPacketItemClassDefinitio…` | ReferenceDataPacketItemClassDefinitions | Teardown of the item class definitions packet; the body proves the ReferenceDataPacket part of the class, the derived class binding comes from the dispatcher that built it with sub-opcode 1. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a47b0` | `ReferenceDataPacketItemCategoryDefinitions::~ReferenceDataPacketItemCategoryDef…` | ReferenceDataPacketItemCategoryDefiniti… | Teardown of the item category definitions packet; the derived class binding comes from the dispatcher that built it with sub-opcode 2. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00a39c20` | `ReferenceDataPacketClientProfileData::~ReferenceDataPacketClientProfileData` | ReferenceDataPacketClientProfileData | Cleanup routine for the reference-data client-profile packet: it is called from an EH unwind funclet (Unwind@015b2f20, next to the 015b2f38 handler used by the 00a3a900 receive path), so it is the destructor MSVC emitte… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3a2b0` | `ClientProfileData_ReadFromReader` | ClientProfileData | Record deserialiser for the reference-data client profile table: 00a3a580 calls it once per inserted node, so this is where the byte layout of one ClientProfileData record is defined. | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |
| `00a3a8a0` | `ParseClientProfileDataBlob` |  | Payload-level entry of the reference-data client-profile path: it performs the same two-step (header read, table load) sequence that the packet receive path 00a3a900 performs, but takes the owner object instead of a pac… | [[Native Map/Shards/S0295 00a39b00-00a3f350|S0295]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a44f0` | `ReferenceDataPacket::ReferenceDataPacket` | recv | ReferenceDataPacket | 44 | Hypothesis: the original peer sends opcode-44 reference data consumed by client dispatcher 00928d00. The dispatcher and subtype constructors 008a4550/008a4680/008a4740 establish the client parsing path; server-side table generation is unknown. |
| `008a4550` | `ReferenceDataPacket::ReferenceDataPacketSub` | recv | ReferenceDataPacket | 44 | Hypothesis: the peer sends opcode-44 subtypes parsed by 00928d00. Item-class/category subtype names match packet-catalogue entries; how the original server built those tables is unknown. |
| `008a4680` | `ReferenceDataPacketItemClassDefinitions::ReferenceDataPacketItemClassDefinitions` | recv | ReferenceDataPacketItemClassDefinitions |  | Hypothesis: the peer sends reference-data packet ReferenceDataPacketItemClassDefinitions (44/1), which 00928d00 parses into client definition state. Server-side table generation is unknown. |
| `008a4740` | `ReferenceDataPacketItemCategoryDefinitions::ReferenceDataPacketItemCategoryDefi…` | recv | ReferenceDataPacketItemCategoryDefinitions |  | Hypothesis: the peer sends reference-data packet ReferenceDataPacketItemCategoryDefinitions (44/2), which 00928d00 parses into client definition state. Server-side table generation is unknown. |
| `00928d00` | `PlayClient::DispatchReferenceData` | recv | ReferenceDataPacket |  |  |
| `00a39bb0` | `ReferenceDataPacketClientProfileData::ReferenceDataPacketClientProfileData` | recv | ReferenceDataPacketClientProfileData |  | Hypothesis: the original server pushed the client profile table (sub-opcode 3 of opcode 44) to the client after login or zone-in, next to the item-class and item-category tables; this wrapper only stores a pointer, so the inference depends on the loader 00a3a580 and the PlayClient dispatcher 00928d00. |
| `00a3a900` | `ReferenceDataPacketClientProfileData_Parse` | recv | ReferenceDataPacketClientProfileData |  | Hypothesis: opcode 44 sub-opcode 3 carried a two-value header plus the counted profile table, and the client replaced its whole table on receipt; this depends on the dispatcher 00928d00, the loader 00a3a580 and the record reader 00a3a2b0. |
| `00a3a2b0` | `ClientProfileData_ReadFromReader` | recv | ReferenceDataPacketClientProfileData |  | Hypothesis: the original server serialised a client profile table as a count-prefixed array of records whose per-record layout is the eleven dwords plus two booleans plus nested row list read here; the inference depends on the same layout being used by the loader 00a3a580 and by the send side that built opcode 44 sub-opcode 3. |
