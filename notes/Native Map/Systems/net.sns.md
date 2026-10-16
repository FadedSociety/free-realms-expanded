# System: net.sns

71 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (34)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5710` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket` | BaseSnsIntegrationPacket | Base destructor of the SNS packet family. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5760` | `BaseSnsIntegrationPacket::ScalarDeletingDtor` | BaseSnsIntegrationPacket | Deleting destructor of the SNS packet base. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5880` | `SnsIntegrationMaxLevelReached::SnsIntegrationMaxLevelReached` | SnsIntegrationMaxLevelReached | Creates the subtype-5 object decoded by handler 00bb8d90; that handler reads the message, builds a Weibo XML request, and posts through the separate SNS HTTP path. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5be0` | `SnsIntegrationFirstTimeLogin::ScalarDeletingDtor` | SnsIntegrationFirstTimeLogin | Deleting destructor of the first-time-login SNS packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5c50` | `SnsIntegrationMaxLevelReached::ScalarDeletingDtor` | SnsIntegrationMaxLevelReached | Deleting destructor of the max-level SNS packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5cd0` | `SnsIntegrationProcessor::SnsIntegrationProcessor` | SnsIntegrationProcessor | Creates the client-side SNS processor in the bootstrap loader 0092b170; it drives the packet handlers and the HTTP posts to the SnsServer. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5d50` | `SnsIntegrationProcessor::ScalarDeletingDtor` | SnsIntegrationProcessor | Deleting destructor of the SNS processor. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6120` | `SnsIntegrationSecureSignatureResponse::~SnsIntegrationSecureSignatureResponse` | SnsIntegrationSecureSignatureResponse | Destroys the signature-response packet and its IString member; called by the scalar deleting destructor 00bb6470 (at SnsIntegrationSecureSignatureResponse::vftable slot 0), by the send path 00bb8100 and from the unwind … | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb61b0` | `SnsIntegrationSecureSignatureRequest::SnsIntegrationSecureSignatureRequest` | SnsIntegrationSecureSignatureRequest | Concrete SNS packet 0xc5/6 requesting a signature string; built by 00bb84f0 (SendSecureSignatureRequest). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6220` | `SnsIntegrationSecureSignatureRequest::~SnsIntegrationSecureSignatureRequest` | SnsIntegrationSecureSignatureRequest | Destroys the request packet and its signature string; the class identity comes from its only scalar-deleting-destructor caller 00bb6490, which sits at SnsIntegrationSecureSignatureRequest::vftable slot 0. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6320` | `SnsIntegrationGuildCreated::~SnsIntegrationGuildCreated` | SnsIntegrationGuildCreated | Destroys the guild-created packet and its string member; called by both deserialisers and by the scalar deleting destructor (00bb64b0, at SnsIntegrationGuildCreated::vftable slot 0). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6470` | `SnsIntegrationSecureSignatureResponse::ScalarDeletingDtor` | SnsIntegrationSecureSignatureResponse | Deleting destructor of the signature-response packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6490` | `SnsIntegrationSecureSignatureRequest::ScalarDeletingDtor` | SnsIntegrationSecureSignatureRequest | Deleting destructor of the signature-request packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb64b0` | `SnsIntegrationGuildCreated::ScalarDeletingDtor` | SnsIntegrationGuildCreated | Deleting destructor of the guild-created packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6ba0` | `PlayerTitleSNSUpdatePacket::PlayerTitleSNSUpdatePacket` | PlayerTitleSNSUpdatePacket | Constructs the 152/subtype-7 packet with an integer list; handler 00bb85d0 fills that list from 'titles'/'title' data and passes the packet to the sender 00bb8380. The server-side meaning remains a hypothesis. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6c10` | `PlayerTitleSNSUpdatePacket::~PlayerTitleSNSUpdatePacket` | PlayerTitleSNSUpdatePacket | Destroys the title packet and its id list; called by the scalar deleting destructor and from the unwind funclet 015e8be1. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6c80` | `PlayerTitleSNSUpdatePacket::ScalarDeletingDtor` | PlayerTitleSNSUpdatePacket | Deleting destructor of the title packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb74d0` | `Dtor_TwoStringsAndList` |  | Tears down the parsed-message object used by the reply handlers 00bb7db0/00bb7f50/00bb8000; the matching constructor is 00bb7670 and the copy constructor 00bb7710. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7670` | `Ctor_TwoStringsAndList` |  | Initializes the message object used by reply handling; destructor is 00bb74d0 and copy-assignment helper is 00bb7710. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7790` | `SnsIntegrationProcessor::HandleInspectReply` | SnsIntegrationProcessor | Updates the SNS inspect proxy from a successful response body. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb8220` | `SnsIntegrationProcessor::OnUiCancelShare` | SnsIntegrationProcessor | Cancels one pending SNS share after a UI cancellation callback. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb84f0` | `SnsIntegrationProcessor::SendSecureSignatureRequest` | SnsIntegrationProcessor | Starts the secure-signature exchange for a pending SNS message. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb85d0` | `SnsIntegrationProcessor::HandleAboutMeReply` | SnsIntegrationProcessor | Applies an About Me response and publishes the parsed title list through the client packet path. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb8b30` | `SnsIntegrationProcessor::HandleFirstTimeLoginPacket` | SnsIntegrationProcessor | Starts an SNS post for the first-time-login event after translation succeeds. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb8d90` | `SnsIntegrationProcessor::HandleJobMaxLevelReachedPacket` | SnsIntegrationProcessor | Starts an SNS post when the job-max-level event arrives and translation succeeds. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb92c0` | `SnsIntegrationProcessor::StartInspectRequest` | SnsIntegrationProcessor | Starts the SNS user-inspect request and tracks its eventual reply. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb94c0` | `SnsIntegrationProcessor::StartAboutMeRequest` | SnsIntegrationProcessor | Starts the SNS About Me request and tracks its eventual reply. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9650` | `SnsIntegrationProcessor::OnPostResult` | SnsIntegrationProcessor | Dispatches external SNS post results to the matching response handler. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00d2e0e0` | `OnVideoUploadAuthenticate` | YouTube::FRYouTubeUploaderHandler | Forwards the YouTube upload authenticate event to the handler callback stored in the uploader handler. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e100` | `OnVideoUploadPost` | YouTube::FRYouTubeUploaderHandler | Forwards the YouTube upload post event to the handler callback. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e130` | `OnVideoUploadProgress` | YouTube::FRYouTubeUploaderHandler | Forwards the YouTube upload progress event to the handler callback. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2e170` | `Ctor` | YouTube::FRYouTubeUploaderHandler | Constructs the YouTube uploader handler; the pointer at +4 is the callback that the OnVideoUpload* slots invoke. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `01080ce0` | `SnsBuildMultipartPost` | SnsIntegrationCommon | Builds and posts the multipart SNS message request for SnsIntegrationCommon::SnsSendMessage (tracking id and URL from the send record), logging the send to SnsIntegration.log. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081c60` | `SnsIntegrationCommonDtorBody` | SnsIntegrationCommon | Destructor body of SnsIntegrationCommon (SNS integration base), which derives from TinyHttp::HttpManagerHandler; called by the SnsIntegrationProcessor destructors. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## 🟢 likely (23)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5830` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_2` | BaseSnsIntegrationPacket | Base cleanup for the derived SNS packet destructors whose unwind chain points at this copy rather than 00bb5710. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb58e0` | `BaseSnsIntegrationPacket::~BaseSnsIntegrationPacket_3` | BaseSnsIntegrationPacket | Base cleanup for another derived SNS packet destructor chain. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5db0` | `SnsIntegrationProcessor::ThrottleRemaining` | SnsIntegrationProcessor | Gate used by SnsSendMessage (00bb6ca0) before posting; when it returns non-zero the message is dropped with the log line about an expired throttle. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb63b0` | `ReadChildElementList` |  | Repeated-element reader used by the SNS reply handlers (only caller 00bb85d0, HandleAboutMeReply) to collect children such as the titles list. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6560` | `ReadU16U16U32Into_4_8_c` |  | Field decoder shared by the SNS packet body readers (00bb65f0/00bb6690/00bb7080). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb65f0` | `ReadU16U16U32StringAndPair` |  | Body reader used by the SNS response path (only caller 00bb8100, HandleSecureSignatureResponsePacket), so this is the shape of the 0xc5/1 response payload. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6690` | `ReadU16U16U32AndString` |  | Body reader used by the two guild-created deserialisers 00bb7380/00bb7410; it decodes a short record plus one string. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7080` | `ReadU16U16U32AndU32` |  | Body reader used by the two max-level handlers (00bb8d90 calls it right after constructing SnsIntegrationMaxLevelReached), so it decodes the 0xc5/5 payload. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7380` | `DeserializeSnsGuildCreated` |  | Turns one guild-created record into the packet object; no caller is recorded yet, so it is either a recv entry point or a test helper. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7410` | `DeserializeSnsGuildCreated_2` |  | Second copy of the guild-created decoder, kept separate by the linker. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7590` | `WriteU16U16U32` |  | Field writer of the SNS packet build path; its only caller 00bb8260 sends the resulting tunneled gateway packet. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb7710` | `CopyAssign_TwoStringsAndList` |  | Copy-assignment helper for the message object; the only recorded caller is secure-signature response handler 00bb8100. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb90c0` | `StartPostAppWeiboRequest ⚠downgrade` |  | Submits one translated SNS post body to the secure-signature request flow. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9250` | `CopyPendingRecordAndSend → SnsQueuePendingStringRequest_Kind0` |  | Queues a copied record for the secure-signature path. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9280` | `CopyTypedPendingRecordAndSend → SnsQueuePendingStringRequest_Kind3` |  | Queues a type-3 record for the secure-signature path. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9930` | `SnsIntegrationProcessor::DispatchBaseSnsIntegrationPacket ⚠downgrade` | SnsIntegrationProcessor | Routes two BaseSnsIntegrationPacket subtypes to their body handlers. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `0107fc90` | `HashListMap64Ctor` | SoeUtil::HashListMap<int,SnsIntegration… | Constructor of the 64-bucket SoeUtil::HashListMap keyed by int for SnsIntegrationCommon::MessageTrackingNode (class name from RTTI). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081a40` | `HashListMapSnsDtorBody` | SoeUtil::HashListMap<int,SnsIntegration… | Destructor body of the SNS message-tracking hash list (container cleanup). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081bb0` | `HashListMapSnsScalarDeletingDtor` | SoeUtil::HashListMap<int,SnsIntegration… | Scalar deleting destructor (vtable slot 0) of the SNS message-tracking hash list. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081bf0` | `HashListMapSnsDtorBody` | SoeUtil::HashListMap<int,SnsIntegration… | Destructor body of the SNS message-tracking hash list (container cleanup). | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081d70` | `HashListSnsCtor` | SoeUtil::HashList<SnsIntegrationCommon:… | Constructor of the 64-bucket HashList for SnsIntegrationCommon message-tracking nodes. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081df0` | `HashListSnsScalarDeletingDtor` | SoeUtil::HashList<SnsIntegrationCommon:… | Scalar deleting destructor (vtable slot 0) of the SNS message-tracking HashList. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081f70` | `SnsIntegrationCommonScalarDeletingDtor` | SnsIntegrationCommon | Scalar deleting destructor (vtable slot 0) of SnsIntegrationCommon. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5fd0` | `SnsFindInterfaceAA264C6F ? ⚠rename` |  | Interface probe run by the SNS handlers 00bb8b30/00bb8d90/00bb92c0/00bb94c0 before they post; the hash key identifies one of the client's registered interfaces. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00d2e190` | `DeleteIfFlagBit0 ?` | YouTube::FRYouTubeUploaderHandler | Virtual slot 4 of FRYouTubeUploaderHandler: stores the vptr at +0, frees this through _free when bit 0 of the byte argument is set, and returns this (ret 4). It does not call a separate dtor body. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `0107fc80` | `ReleaseMember4IfNonNull ? ⚠rename` | SnsIntegrationCommon | Virtual slot 5 (shared by SnsIntegrationProcessor and SnsIntegrationCommon): if the pointer at this+0x4 is non-null it tail-jumps to 00d31890 with that pointer in ECX, else returns (bytes: mov ecx,[ecx+4]; test; je; jmp… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081880` | `CounterAssignAndIncrement ?` |  | Fastcall on this (bytes: lea esi,[ecx+0x18]; push esi; add ecx,0x1c; call 00bb7eb0): calls 00bb7eb0 with receiver this+0x1c and the address this+0x18 pushed; when the returned pointer is non-null, stores the current dwo… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `01081a00` | `HashListClearViaRemoveSns ? ⚠rename` |  | Loop: while the list head at this+0x8 is non-null, calls 00bb8000 on the head; empties the list through that removal routine. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## ⚪ unknown (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb5640` | `— ⚠reject` | BaseSnsIntegrationPacket | Base constructor of the SOE/The9 social-network (SNS) packet family that this client build still contains. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb56a0` | `— ⚠reject` | BaseSnsIntegrationPacket | Shared header initialiser for the concrete SNS packets (subtypes 1,2,3,5,6 and the PlayerTitle variant outside this shard). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb57d0` | `— ⚠reject` | SnsIntegrationFirstTimeLogin | Concrete SNS packet 0xc5/2, built by the handler 00bb8b30 (SnsIntegrationProcessor::HandleFirstTimeLoginPacket). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb5970` | `— ⚠reject` | SnsIntegrationProcessor | Base-class construction of the SNS processor (the concrete constructor 00bb5cd0 completes it). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb60b0` | `— ⚠reject` | SnsIntegrationSecureSignatureResponse | Concrete SNS packet 0xc5/1 carrying the signature string; built by 00bb8100 (HandleSecureSignatureResponsePacket). | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb62b0` | `— ⚠reject` | SnsIntegrationGuildCreated | Concrete SNS packet 0xc5/3; constructed on the stack by the two deserialisers 00bb7380/00bb7410. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb6ca0` | `— ⚠reject` | SnsIntegrationProcessor | The single HTTP poster of the SNS integration: everything the SnsServer sees (Weibo-style posts, image uploads, signature/help-window queries) goes through here; the hardcoded endpoint reveals this is the Chinese (The9)… | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb8100` | `— ⚠reject` | SnsIntegrationProcessor | Completes a pending SNS signature exchange after a response arrives. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `01081e10` | `— ⚠reject` | SnsIntegrationCommon | Constructor body of SnsIntegrationCommon: sets up the HTTP manager member, the message-tracking lists and the string member for the SNS integration. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00bb5880` | `SnsIntegrationMaxLevelReached::SnsIntegrationMaxLevelReached` | recv |  | 5 | Hypothesis: the remote peer sends SNS subtype 5 to the client as a max-level notification; client parsing and the later Weibo POST are visible in 00bb8d90 and 00bb84f0, but the peer-side contract and meaning are not independently recovered. |
| `00bb61b0` | `SnsIntegrationSecureSignatureRequest::SnsIntegrationSecureSignatureRequest` | send |  | 6 | Hypothesis: the client asks the server for a signature through packet 0xc5/6 and the response comes back as 0xc5/1 (ctor 00bb60b0); the pairing is inferred from the two constructors together, since the emulators have no 0xc5 row. |
| `00bb6ba0` | `PlayerTitleSNSUpdatePacket::PlayerTitleSNSUpdatePacket` | send | BasePlayerTitlePacket | 7 | Hypothesis: the receiver accepts a title-id list under BasePlayerTitlePacket subtype 7 and applies it to player-title state; depends on constructor 00bb6ba0, handler 00bb85d0 and sender 00bb8380, with no catalogue row for subtype 7. |
| `00bb85d0` | `SnsIntegrationProcessor::HandleAboutMeReply` | send | BasePlayerTitlePacket |  |  |
| `00bb9930` | `SnsIntegrationProcessor::DispatchBaseSnsIntegrationPacket ⚠downgrade` | recv |  | 1 | Hypothesis: a compatible server must send these two SNS subtypes for the client to reach the response and first-login handlers. This depends on the 197 base-opcode initializer at 00bb56a0/00bb5640 and this dispatcher; it does not establish original server-side logic. |
| `00bb9930` | `SnsIntegrationProcessor::DispatchBaseSnsIntegrationPacket ⚠downgrade` | recv |  | 2 | Hypothesis: a compatible server must send these two SNS subtypes for the client to reach the response and first-login handlers. This depends on the 197 base-opcode initializer at 00bb56a0/00bb5640 and this dispatcher; it does not establish original server-side logic. |
| `00bb5640` | `— ⚠reject` | send |  | 0xc5 | Hypothesis: opcode 197 is the SNS integration family of this specific client build; the community emulators (Sanctuary/FreeRealms-Legacy) contain no row for it, so the server-side handler is unknown and the opcode attribution rests only on this constructor pair (00bb5640/00bb56a0). |
| `00bb56a0` | `— ⚠reject` | send |  | 0xc5 | Hypothesis: as in 00bb5640, opcode 0xc5 is the SNS family and the caller passes the concrete sub-type (2 = first-time login, 5 = max level reached, 1 = secure signature response, 6 = secure signature request, 3 = guild created). |
| `00bb57d0` | `— ⚠reject` | send |  | 2 | Hypothesis: the client tells the server that this is the user's first login through SNS packet 0xc5 subtype 2 so the server can push the account-link flow; the base opcode 0xc5 is set inside 00bb56a0 and has no emulator row, so the contract is inferred from this constructor and its handler 00bb8b30. |
| `00bb60b0` | `— ⚠reject` | send |  | 1 | Hypothesis: the client answers a signature request with packet 0xc5/1 carrying an IString; the counterpart SnsIntegrationSecureSignatureRequest (0xc5/6, ctor 00bb61b0) is what the server pushes, so the two constructors together define the request/response pair. |
| `00bb62b0` | `— ⚠reject` | send |  | 3 | Hypothesis: guild creation is announced to the SNS backend with packet 0xc5/3 carrying the guild name string; the contract is inferred from this constructor plus the deserialisers 00bb7380/00bb7410 having no other caller in the map yet. |
