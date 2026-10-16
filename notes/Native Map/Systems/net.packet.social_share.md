# System: net.packet.social_share

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a5040` | `BaseSocialSharePacket::BaseSocialSharePacket` | BaseSocialSharePacket | Base constructor for opcode-205 social-share response objects; its only caller 009028b0 is the client response parser, which dispatches provider/social-share results. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a50a0` | `BaseSocialSharePacket::~BaseSocialSharePacket` | BaseSocialSharePacket | Destructor of the social-share packet class; the two in-body vtable restores name the class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `008a50f0` | `BaseSocialSharePacket::ScalarDeletingDestructor` | BaseSocialSharePacket | Compiler-generated delete wrapper for the social-share class. | [[Native Map/Shards/S0220 008a3ce0-008a8500|S0220]] |
| `00ade960` | `BaseSocialSharePacket::BaseSocialSharePacket` | BaseSocialSharePacket | Common opcode-205 header constructor used by both request and response packet classes; callers include 00ade9d0, 00adea80, 00adf140 and 00adf280. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00ade9d0` | `SocialSharePacketGetProviderStatusRequest::SocialSharePacketGetProviderStatusRe…` | SocialSharePacketGetProviderStatusReque… | Builds the social-share provider-status request that 00ae0ea0 fills and sends through 00ae0c40 when the client is connected. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adea30` | `BaseSocialSharePacket::~BaseSocialSharePacket` | BaseSocialSharePacket | Base cleanup for social-share packet objects (this translation-unit copy of the destructor, matching the note-documented copy at 008a50a0). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adea80` | `SocialSharePacketPublishResponse::SocialSharePacketPublishResponse` | SocialSharePacketPublishResponse | Builds the social-share publish-response message that 00ae0520 parses with 00adfa90 and matches against the tracked message map. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adeae0` | `BaseSocialSharePacket::~BaseSocialSharePacket` | BaseSocialSharePacket | Base cleanup for social-share packet objects (duplicate translation-unit copy of 00adea30). | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00aded30` | `SocialSharePacketGetProviderStatusRequest::ScalarDeletingDtor` | SocialSharePacketGetProviderStatusReque… | Heap-delete path for provider-status requests. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adeda0` | `SocialSharePacketPublishResponse::ScalarDeletingDtor` | SocialSharePacketPublishResponse | Heap-delete path for publish-response messages. | [[Native Map/Shards/S0324 00ad8880-00adee10|S0324]] |
| `00adf140` | `SocialSharePacketGetProviderStatusResponse::SocialSharePacketGetProviderStatusR…` | SocialSharePacketGetProviderStatusRespo… | Builds the object that the receive-side handler 00ae03d0 fills from the social-share provider-status response body. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adf1c0` | `SocialSharePacketGetProviderStatusResponse::~SocialSharePacketGetProviderStatus…` | SocialSharePacketGetProviderStatusRespo… | Cleanup of the provider-status response object, called from its deleting destructor 00adf490 and from the receive path 00ae03d0. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adf300` | `SocialSharePacketPublishRequest::~SocialSharePacketPublishRequest` | SocialSharePacketPublishRequest | Cleanup of the publish-request object, called from its deleting destructor 00adf4b0 and from the request-build path 00ae0f60. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adf490` | `SocialSharePacketGetProviderStatusResponse::ScalarDeletingDtor` | SocialSharePacketGetProviderStatusRespo… | Heap-delete path for the provider-status response objects held by the receive handler 00ae03d0. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00adf4b0` | `SocialSharePacketPublishRequest::ScalarDeletingDtor` | SocialSharePacketPublishRequest | Heap-delete path for the publish-request objects built by the send path 00ae0f60. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009028b0` | `BaseClient::DispatchSocialShareSubOp → DispatchSocialShareResponseSubOp` | BaseClient | Routes response sub-opcodes 2 and 4 of the social-share family to the provider-status handler (00ae03d0) and the share-publish handler (00ae0520). Vtable slot 79 is shared with PlayClient; named for BaseClient. | [[Native Map/Shards/S0237 008ff3d0-00904e70|S0237]] |
| `00adfa90` | `ReadSocialSharePublishResponseFields` |  | Used by the social-share response dispatch path 00ae0520. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae02c0` | `WriteProviderPair` |  | Serializer helper called by the get-status and publish packet encoders. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae06f0` | `WriteProviderStatusResponse` |  | Packet serializer used by the provider status response encoder. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adf280` | `SocialSharePacketPublishRequest::SocialSharePacketPublishRequest ? ⚠downgrade` | SocialSharePacketPublishRequest | Builds the outgoing social-share publish request that the game server is expected to act on. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00adf970` | `— ⚠reject` |  | Called while decoding the provider-status response in 00ae03d0. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0790` | `— ⚠reject` |  | Serializer for the social-share publish request packet. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0c40` | `— ⚠reject` |  | Send-side encoder for the provider status operation. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |
| `00ae0d50` | `— ⚠reject` |  | Send-side encoder for a social-share publish request. | [[Native Map/Shards/S0325 00adee60-00ae4e70|S0325]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `008a5040` | `BaseSocialSharePacket::BaseSocialSharePacket` | recv |  | 205 | Hypothesis: the peer sends opcode-205 social-share/provider responses parsed by 009028b0. This client path does not show a client share request or original server posting behavior. |
| `00adf140` | `SocialSharePacketGetProviderStatusResponse::SocialSharePacketGetProviderStatusR…` | recv |  | 2 |  |
| `009028b0` | `BaseClient::DispatchSocialShareSubOp → DispatchSocialShareResponseSubOp` | recv |  | 2/4 |  |
| `00ae02c0` | `WriteProviderPair` | send |  | 2 |  |
| `00adf280` | `SocialSharePacketPublishRequest::SocialSharePacketPublishRequest ? ⚠downgrade` | send |  | 3 |  |
| `00ae0c40` | `— ⚠reject` | send |  | 2 |  |
