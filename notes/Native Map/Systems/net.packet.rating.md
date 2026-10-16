# System: net.packet.rating

24 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00aa5ca0` | `BaseRatingPacket::BaseRatingPacket` | BaseRatingPacket | Constructor of the rating packet base with family opcode 174 and a caller-supplied field. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa5d10` | `BaseRatingPacket::~BaseRatingPacket` | BaseRatingPacket | Destructor body of the rating packet base; reached from 28 EH unwind funclets. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa5d60` | `BaseRatingPacket::ScalarDeletingDtor` | BaseRatingPacket | Scalar deleting destructor for BaseRatingPacket. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aadab0` | `RatingPacketRequestFeatured::ctor` | RatingPacketRequestFeatured | Client-side request for the featured rating candidates; send wrapper 00ac62e0 fills the string with the literal RatingSystem.HousingId before sending. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aadbb0` | `RatingPacketAddCandidate::ctor` | RatingPacketAddCandidate | Client-side message nominating a rating candidate; send wrapper 00ac5250 fills the strings, one of which is RatingSystem.HousingId. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aade40` | `RatingPacketRemoveCandidate::ctor` | RatingPacketRemoveCandidate | Client-side message removing a rating candidate; send wrapper 00ac5b90 constructs and sends it. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aadf80` | `RatingPacketAddVote::ctor` | RatingPacketAddVote | Client-side message casting a rating vote; send wrapper 00ac6a00 constructs and sends it. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae140` | `RatingPacketSearchRequest::ctor` | RatingPacketSearchRequest | Client-side rating search request; send wrapper 00ac7260 constructs and sends it. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae280` | `RatingPacketCandidateInfoRequest::ctor` | RatingPacketCandidateInfoRequest | Client-side request for one candidate's details; callers 00ac2a10 and 00ac36c0 construct and send it. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00abab40` | `BaseRatingPacket::Dtor` | BaseRatingPacket | Destructor body of a rating packet that carries a string-keyed map and an IString member (the RatingLookupHousingIdAndSend path, neighbour caller 00ac9340). | [[Native Map/Shards/S0318 00ab73e0-00abcb30|S0318]] |

## 🟢 likely (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00aadb20` | `RatingPacketRequestFeatured::~RatingPacketRequestFeatured` | RatingPacketRequestFeatured | Destruction of the featured-request packet; the send wrapper 00ac62e0 pairs it with the constructor 00aadab0. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aadc90` | `RatingPacketAddCandidate::~RatingPacketAddCandidate` | RatingPacketAddCandidate | Destruction of the 174/6 candidate packet; the send wrapper 00ac5250 pairs it with the constructor 00aadbb0. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aadec0` | `RatingPacketRemoveCandidate::~RatingPacketRemoveCandidate` | RatingPacketRemoveCandidate | Destruction of the 174/7 packet; the send wrapper 00ac5b90 pairs it with the constructor 00aade40. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae020` | `RatingPacketAddVote::~RatingPacketAddVote` | RatingPacketAddVote | Destruction of the 174/8 packet; the send wrapper 00ac6a00 pairs it with the constructor 00aadf80. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae1c0` | `RatingPacketSearchRequest::~RatingPacketSearchRequest` | RatingPacketSearchRequest | Destruction of the 174/12 packet; the send wrapper 00ac7260 pairs it with the constructor 00aae140. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aae320` | `RatingPacketCandidateInfoRequest::~RatingPacketCandidateInfoRequest` | RatingPacketCandidateInfoRequest | Destruction of the 174/16 packet; the send wrappers 00ac2a10/00ac36c0 pair it with the constructor 00aae280. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00ac62e0` | `SendRatingRequestFeatured` |  | Asks the rating service for the featured entries of the housing category (the featured-house list of the house rating feature). | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |
| `00ac6a00` | `SendRatingAddVote` |  | Sends the player's vote and optional comment for the house currently selected in the browser to the rating service. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |
| `00ac7df0` | `SendRatingClientDataRequest` |  | Transport wrapper for the rating ClientDataRequest writer. Caller 00ac9340 constructs RatingPacketClientDataRequest for the Housing category; the fixture saver at 00ac8790 uses the separate 00ac7ce0 helper. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |
| `00ac9340` | `RatingLookupHousingIdAndSend` |  | Builds a housing-keyed rating-system client-data request. This identifies the client request path, not the server-side rating semantics. | [[Native Map/Shards/S0321 00ac8340-00acd3f0|S0321]] |
| `00c857b0` | `BuildRatingEntryKeys` |  | Builds keyed rating-entry lookup data for its caller. | [[Native Map/Shards/S0404 00c84f00-00c89db0|S0404]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ac5b90` | `SendRatingRemoveCandidate ?` |  | Builds a rating remove-candidate packet from the current house guid and Housing category; the trigger for this call is not established. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ac5250` | `— ⚠reject` |  | Builds a rating add-candidate packet for the current house and, when the house flag at +0x68 is set, also builds a housing lock-toggle packet. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |
| `00ac7260` | `— ⚠reject` |  | Rate-limits and sends a rating search request for the caller's non-null search string. | [[Native Map/Shards/S0320 00ac2a10-00ac7f40|S0320]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00aadab0` | `RatingPacketRequestFeatured::ctor` | send |  | 20 | Hypothesis: the client asks the rating service (identified by the string RatingSystem.HousingId) for the featured house list and the original server had to answer with the featured candidates; the reply packet class is not present in this shard, so the response shape stays unknown. |
| `00aadbb0` | `RatingPacketAddCandidate::ctor` | send |  | 6 | Hypothesis: the client nominates a house for rating by sending 174/6 with candidate identifiers; the original server had to validate the candidate and add it to the featured pool. The seven string roles (name, owner, description, ...) are not recovered. |
| `00aade40` | `RatingPacketRemoveCandidate::ctor` | send |  | 7 | Hypothesis: the client asks the server to drop a nominated candidate (174/7 with two identifying strings) and the original server had to remove it from the rating pool and update the candidate list. |
| `00aadf80` | `RatingPacketAddVote::ctor` | send |  | 8 | Hypothesis: the client submits a vote (174/8 with four identifying strings) and the original server had to record the vote, enforce one vote per player/house and update the featured scores. |
| `00aae140` | `RatingPacketSearchRequest::ctor` | send |  | 12 | Hypothesis: the client searches the rating database with two strings (e.g. a query and the RatingSystem.HousingId system id) and the original server had to answer with matching candidate entries; the reply class is not in this shard. |
| `00aae280` | `RatingPacketCandidateInfoRequest::ctor` | send |  | 16 | Hypothesis: the client asks for the information page of one rating candidate (174/16 carrying identifying strings) and the original server had to answer with the candidate's details; the reply class is not in this shard. |
| `00ac9340` | `RatingLookupHousingIdAndSend` | send | PacketTunneledClientWorldPacket |  |  |
| `00ac5250` | `— ⚠reject` | send | BaseHousingPacket |  |  |
