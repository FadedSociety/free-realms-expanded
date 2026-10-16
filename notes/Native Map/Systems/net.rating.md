# System: net.rating

27 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ab0980` | `RatingPacketRequestFeaturedScalarDeletingDtor` | RatingPacketRequestFeatured | Scalar deleting destructor for RatingPacketRequestFeatured. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab09a0` | `RatingPacketAddCandidateScalarDeletingDtor` | RatingPacketAddCandidate | Scalar deleting destructor for RatingPacketAddCandidate. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab09c0` | `RatingPacketRemoveCandidateScalarDeletingDtor` | RatingPacketRemoveCandidate | Scalar deleting destructor for RatingPacketRemoveCandidate. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab09e0` | `RatingPacketAddVoteScalarDeletingDtor` | RatingPacketAddVote | Scalar deleting destructor for RatingPacketAddVote. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab0a00` | `RatingPacketSearchRequestScalarDeletingDtor` | RatingPacketSearchRequest | Scalar deleting destructor for RatingPacketSearchRequest. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00ab0a20` | `RatingPacketCandidateInfoRequestScalarDeletingDtor` | RatingPacketCandidateInfoRequest | Scalar deleting destructor for RatingPacketCandidateInfoRequest. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |
| `00abd470` | `RatingPacketClientDataRequest::RatingPacketClientDataRequest` | RatingPacketClientDataRequest | Constructs the rating client-data request; caller 00ac9340 is the lookup/send path. Base family comes from BaseRatingPacket constructor. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abd4f0` | `RatingPacketClientDataRequest::ScalarDeletingDtor` | RatingPacketClientDataRequest | Scalar deleting destructor identified by binary slot and destructor body. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00c86670` | `RatingPacketDataReplyCtor` | RatingPacketDataReply | Initializes a rating data reply packet object. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c866f0` | `RatingPacketDataReplyScalarDeletingDtor` | RatingPacketDataReply | Implements scalar deleting destruction for RatingPacketDataReply. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c86710` | `RatingPacketSearchReplyCtor` | RatingPacketSearchReply | Initializes a rating search reply packet object. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c86790` | `BaseRatingPacketWithListDtor` | BaseRatingPacket | Cleans up a rating packet containing a rating-data list. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c86830` | `RatingPacketCandidateInfoReplyScalarDeletingDtor` | RatingPacketCandidateInfoReply | Implements scalar deleting destruction for the candidate-info reply packet. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c868e0` | `RatingPacketSearchReplyScalarDeletingDtor` | RatingPacketSearchReply | Implements scalar deleting destruction for the search reply packet. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00abcfa0` | `RatingPacketAddCandidate_SerializeBody` |  | Add-candidate role is inferred from direct sender 00ac1540; this body establishes the field writes only. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abd070` | `RatingPacketAddVote_SerializeBody` |  | Add-vote role comes from direct sender 00ac1890; this function establishes field sequence and widths only. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abd280` | `SerializeLinkedRatingEntries → RatingPacketClientDataRequest_SerializeEntryList` |  | Rating context comes from direct caller 00abf6f0, the rating client-data body writer. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00abf6f0` | `RatingPacketClientDataRequest_SerializeBody` |  | Serialises the packet built by 00abd470 and sent through the wrapper 00ac7df0, i.e. the rating client-data request the housing rating path emits. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac1540` | `SendRatingPacketAddCandidate` | RatingPacketAddCandidate | Send helper for the rating add-candidate request that SendRatingAddCandidate builds with the constructor FUN_00aadbb0. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac1650` | `SendRatingPacketRemoveCandidate` | RatingPacketRemoveCandidate | Send helper for the rating remove-candidate request built by SendRatingRemoveCandidate with the constructor FUN_00aade40. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac1770` | `SendRatingPacketRequestFeatured` | RatingPacketRequestFeatured | Send helper for the featured-houses request built by SendRatingRequestFeatured with the constructor FUN_00aadab0. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac1890` | `SendRatingPacketAddVote` | RatingPacketAddVote | Send helper for the rating vote that SendRatingAddVote builds with the constructor FUN_00aadf80. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00ac1ae0` | `SendRatingPacketSearchRequest` | RatingPacketSearchRequest | Send helper for RatingPacketSearchRequest, which its caller (00ac7260) builds with FUN_00aae140 after the rate-limit check against this+0x2878/+0x287c. | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00c865a0` | `ReadRatingDataReply` |  | Decodes a rating data reply body. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## ⚪ unknown (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00abfe70` | `— ⚠reject` | RatingPacketCandidateInfoRequest | Send helper the subtype-39 and subtype-45 handlers use when the answered list belongs to the local player (caller label: constructs RatingPacketCandidateInfoRequest and sends it through FUN_00abfe70). | [[Native Map/Shards/S0319 00abcb90-00ac2280|S0319]] |
| `00c864f0` | `— ⚠reject` |  | Decodes a rating candidate reply body. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |
| `00c869e0` | `— ⚠reject` | BaseRatingPacket | Dispatches incoming rating-family packets to packet-specific parsing and client state updates. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00abd470` | `RatingPacketClientDataRequest::RatingPacketClientDataRequest` | send |  | 3 |  |
| `00c869e0` | `— ⚠reject` | recv |  | 5 | Hypothesis: the original server sends rating data, candidate ratings, housing vote/search results, featured-housing updates, and a rating-server version response. This depends on the client dispatch cases, RatingPacketDataReply and RatingPacketCandidateInfoReply constructors, and the HousingProcessor search-reply call; it does not establish server-side ranking or persistence rules. |
| `00c869e0` | `— ⚠reject` | recv |  | 17 | Hypothesis: the original server sends rating data, candidate ratings, housing vote/search results, featured-housing updates, and a rating-server version response. This depends on the client dispatch cases, RatingPacketDataReply and RatingPacketCandidateInfoReply constructors, and the HousingProcessor search-reply call; it does not establish server-side ranking or persistence rules. |
| `00c869e0` | `— ⚠reject` | recv |  | 18 | Hypothesis: the original server sends rating data, candidate ratings, housing vote/search results, featured-housing updates, and a rating-server version response. This depends on the client dispatch cases, RatingPacketDataReply and RatingPacketCandidateInfoReply constructors, and the HousingProcessor search-reply call; it does not establish server-side ranking or persistence rules. |
| `00c869e0` | `— ⚠reject` | recv |  | 22 | Hypothesis: the original server sends rating data, candidate ratings, housing vote/search results, featured-housing updates, and a rating-server version response. This depends on the client dispatch cases, RatingPacketDataReply and RatingPacketCandidateInfoReply constructors, and the HousingProcessor search-reply call; it does not establish server-side ranking or persistence rules. |
| `00c869e0` | `— ⚠reject` | recv |  | 23 | Hypothesis: the original server sends rating data, candidate ratings, housing vote/search results, featured-housing updates, and a rating-server version response. This depends on the client dispatch cases, RatingPacketDataReply and RatingPacketCandidateInfoReply constructors, and the HousingProcessor search-reply call; it does not establish server-side ranking or persistence rules. |
