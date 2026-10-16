# System: net.packet.promotions

25 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a05a50` | `PromotionalBundleDataPacket::PromotionalBundleDataPacket` | PromotionalBundleDataPacket | Builds the client-side object that carries one promotional bundle message (sub-opcode 2 of the promotional family); the promotions handler deserialises into it before caching the bundle. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05ab0` | `PromotionalBundleDataPacket::~PromotionalBundleDataPacket` | PromotionalBundleDataPacket | Destroy a promotional bundle packet object; reached only from an exception-unwind funclet, the normal marker for a destructor. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a05b10` | `PromotionalBundleDataPacket::ScalarDeletingDtor` | PromotionalBundleDataPacket | Frees a heap-allocated promotional bundle packet; slot 0 of the class vtable is the deleting destructor. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08d70` | `ParseKeyCodeRedemptionNotification` |  | Consumes a key-code redemption notification (FRTCG loot card codes) delivered by the promotional message dispatcher 00a08e60 for subtype 1. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a07850` | `Stream_Read2Shorts` |  | Header/value reader used by the promotional packet parsers (00a089d0, 00a08d70, 00a088f0, 00a08cf0) to pull the leading 16-bit pair of a message body. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a079e0` | `Stream_Read3IntsPlusBlob` |  | Reader for one array element of the bundle record: three scalar fields plus a variable-length string (the element string sits at +0xc per the element copy 00a07dc0). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07ab0` | `Element50_Construct` |  | Element construction used when the record array appends a new entry (00a08420). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07b20` | `Element50_InitDefault` |  | Element construction for newly added slots of the record array (00a08380). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07dc0` | `Element50_CopyConstruct` |  | Element copy used when the record array grows (00a081b0). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07eb0` | `Stream_ReadArrayOf20` |  | Array field reader of the promotional bundle record (the icon-entry array at +0x34 of that structure, per 00a08aa0). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a081b0` | `Array50_GrowAndCopy` |  | Reallocation routine of the record array class. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08380` | `Array50_Resize` |  | Element default construction when the record array grows (00a08940). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08420` | `Array50_Append` |  | Element append used while materialising the redemption result list (00a08520). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08940` | `Stream_ReadArrayOf50` |  | Array field reader for the redemption/bundle record (the 0x50-byte array member at +0x98 of that structure, per 00a08aa0). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08aa0` | `Stream_ReadTwoItemArrays` |  | Item-list reader shared by the key-code redemption payload parsers 00a08d70 and 00a08cf0. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a088f0` | `Stream_ReadRedeemOptionRecord ? ⚠rename` |  | Reader for the redemption-option message body; it has no recorded callers but matches the parse path used for promo option records. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a072c0` | `— ⚠reject` |  | Field reader used while parsing a promotional bundle record (00a07eb0 reads one such pair per element). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07370` | `— ⚠reject` |  | Field reader used for the triple-int part of a bundle entry (00a07f40 reads one triple per map entry). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07e40` | `— ⚠reject` |  | Deleting destructor for the 0x50-byte record element/record holder whose only non-trivial member is the fixed string at the object start. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a07f40` | `— ⚠reject` |  | Parser for the keyed sub-list of a promotional record (used by the record reader 00a082e0 and by the top-level bundle parse). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08120` | `— ⚠reject` |  | Element teardown when a record array loses entries (00a08940). | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a082e0` | `— ⚠reject` |  | Record reader used by the redemption-option parsers 00a088f0/00a089d0 for a sub-record that carries a keyed list. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a089d0` | `— ⚠reject` |  | Body reader of the promotional bundle message; the bundle deserialiser 00a08b10 calls it twice, first to obtain the bundle id and again to fill the cached node. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08b10` | `— ⚠reject` |  | Caches one promotional bundle definition received from the server into the manager; called by the promotional message dispatcher 00a08e60 for subtype 2. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |
| `00a08e60` | `— ⚠reject` |  | Routes the subtype-1 and subtype-2 payloads from the promotional packet family to their respective parsers. | [[Native Map/Shards/S0286 00a05a50-00a0c390|S0286]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a08e60` | `— ⚠reject` | recv |  | 1 | Hypothesis: the original server sends the promotional-family receive payloads whose subtype values are 1 and 2; this function routes them to key-code and bundle parsers. Outer opcode 114 is established by caller 009471a0, and subtype meanings depend on 00a05910/00a05a50 constructors plus 00a08d70/00a08b10 handlers; no server-side rewards or policy are established. |
| `00a08e60` | `— ⚠reject` | recv |  | 2 | Hypothesis: the original server sends the promotional-family receive payloads whose subtype values are 1 and 2; this function routes them to key-code and bundle parsers. Outer opcode 114 is established by caller 009471a0, and subtype meanings depend on 00a05910/00a05a50 constructors plus 00a08d70/00a08b10 handlers; no server-side rewards or policy are established. |
