# Class: InteractionProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a380d0` | `InteractionProcessor::InteractionProcessor` | ✅ | ctor | Ctor with SEH frame: installs InteractionProcessor::vftable, stores param_2 (owner) at +4, clears the byte at +8, stores the default pair DAT_01be68f8/DAT_01be68fc at +0x10/+0x14, zeroes the holder at +0x18, constructs … |
| `00a381a0` | `InteractionProcessor::~InteractionProcessor ⚠downgrade` | 🟢 | dtor | Sets InteractionProcessor::vftable, calls 00a37fe0 (ListInteractionData10DtorBody) on the list embedded at this+0x1c (raw bytes: lea ecx,[esi+0x1c] before the call), then when the holder at this+0x18 (param_1[6]) is non… |
| `00a383e0` | `InteractionProcessor::SendInteractRequest` | 🟢 | method | Constructs a CommandPacketInteractRequest through 008b6bc0 (command family ctor with sub-opcode 8), then calls 00925a70 when the owner's +0x280 field is non-zero. The owner is this+4. Returns void. |
| `00a38470` | `— ⚠reject` | ⚪ | virtual | Virtual slot [0] of InteractionProcessor::vftable (binary vtable read). Loads the owner (this+4), then the player object at owner+0x178. When the player's pair at +0x10/+0x14 equals DAT_01be68f8/DAT_01be68fc, or the pen… |
| `00a384c0` | `InteractionProcessor::ClearTargetAndSendFreeNpc` | ✅ | virtual | Virtual slot [1] of InteractionProcessor::vftable (binary vtable read). When the stored pair at this+0x10/+0x14 differs from DAT_01be68f8/DAT_01be68fc it builds a FreeInteractionNpc through 009d9a90 (accepted label, com… |
