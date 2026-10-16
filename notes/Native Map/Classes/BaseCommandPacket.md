# Class: BaseCommandPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2820` | `BaseCommandPacket::BaseCommandPacket` | ✅ | ctor | Command family constructor taking sub-opcode and an extra word: installs BasePacket::vftable, stores 0x1a (26) in the packet id slot param_1[1], installs BaseCommandPacket::vftable, then stores the two arguments in para… |
| `008a2890` | `BaseCommandPacket::~BaseCommandPacket` | ✅ | dtor | Destructor body: restores BaseCommandPacket::vftable then BasePacket::vftable; 133 EH unwind funclets call it, making it the shared cleanup body for command packet locals across the client. |
| `008a28e0` | `BaseCommandPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `009d9af0` | `BaseCommandPacket::~BaseCommandPacket` | 🟢 | dtor | Trivial destructor body: rewrites the vtables to BaseCommandPacket::vftable then BasePacket::vftable. Called by four EH unwind funclets; it is also the address-adjacent destructor candidate for FreeInteractionNpc (not p… |
| `009f58f0` | `BaseCommandPacket::DtorBody` | ✅ | dtor | Destructor body without free: stores BaseCommandPacket::vftable then BasePacket::vftable into *this and returns. |
| `009f5990` | `BaseCommandPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor used as PacketDialogResponse::vftable slot [0]: stores BaseCommandPacket::vftable then BasePacket::vftable into *this, sets the EH state to -1, and calls _free(this) through thunk 00d82463 onl… |
| `00a0a4a0` | `BaseCommandPacket::~BaseCommandPacket` | ✅ | dtor | Calls FUN_0070a790 with receiver this+0x1c, then rewrites the vtable to BaseCommandPacket::vftable and BasePacket::vftable. |
| `00a103b0` | `BaseCommandPacket_Dtor` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable then BasePacket::vftable; no fields released. |
| `00a12250` | `BaseCommandPacket_ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: stores BaseCommandPacket::vftable then BasePacket::vftable and frees the object with _free when bit 0 of param_2 is set. |
| `00a36c00` | `BaseCommandPacket::~BaseCommandPacket` | ✅ | dtor | Bytes: prologue installs an MSVC SEH frame (push -1, fs:[0] save, /GS cookie xor), then stores BaseCommandPacket::vftable into *this and afterwards BasePacket::vftable, restores fs:[0] and returns. Body frees nothing an… |
| `00a4f510` | `BaseCommandPacket::BaseCommandPacket ⚠downgrade` | 🟢 | ctor | Constructor of the generic command packet: stores BasePacket::vftable, writes opcode 0x1a (26) into field +4, stores BaseCommandPacket::vftable and stores the two caller arguments (command id/argument) into fields +8 an… |
| `00a57ad0` | `BaseCommandPacket_DtorWithString` | ✅ | dtor | Releases the embedded shared string buffer and resets the object through BaseCommandPacket then BasePacket vtables. |
| `00a98620` | `BaseCommandPacket::BaseCommandPacketInit` | ✅ | ctor | Stores BasePacket::vftable, sets the word at +4 to 0x1a, stores BaseCommandPacket::vftable, and zeroes the words at +8 and +12. No callees. |
| `00a988e0` | `BaseCommandPacket::BaseDtorChain` | 🟢 | dtor | Stores BaseCommandPacket::vftable and then BasePacket::vftable into the object and returns; no member cleanup. |
| `00c09fd0` | `BaseCommandPacket_DtorBody` | ✅ | dtor | Base-class destructor body: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable; reached from the unwind funclet 015f4600 of the quest-abandon send path. |
| `00c0a800` | `BaseCommandPacket_DtorBody_00c0a800 ⚠downgrade` | 🟢 | dtor | Second copy of the BaseCommandPacket destructor body in this translation unit: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable; reached from the unwind funclet 015f4748 of the merchant clear-i… |
| `00c14980` | `BaseCommandPacketDtorNoMembers_00c14980` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns (no member releases). |
| `00c14a30` | `BaseCommandPacketDtorNoMembers_00c14a30` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14ae0` | `BaseCommandPacketDtorNoMembers_00c14ae0` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14ba0` | `BaseCommandPacketDtorNoMembers_00c14ba0` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c14cc0` | `BaseCommandPacketDtorNoMembers_00c14cc0` | 🟢 | dtor | Destructor body: stores BaseCommandPacket::vftable and then BasePacket::vftable at *this and returns. |
| `00c1be10` | `BaseCommandPacket::EmbeddedMemberCleanupA` | ✅ | dtor | Runs the embedded-member cleanup helper under an exception frame, restores BaseCommandPacket and BasePacket vptrs, restores the exception list and returns. |
| `00c1be70` | `BaseCommandPacket::EmbeddedMemberCleanupB` | ✅ | dtor | Runs the embedded-member cleanup helper under an exception frame, restores BaseCommandPacket and BasePacket vptrs, restores the exception list and returns. |
| `00c1bed0` | `— ⚠reject` | ⚪ | dtor | Restores an embedded IString vptr, decrements its shared buffer reference count and releases the buffer at zero, then restores packet base vptrs. |
| `00c1bf60` | `— ⚠reject` | ⚪ | dtor | Restores an embedded IString vptr, decrements its shared buffer reference count and releases the buffer at zero, then restores packet base vptrs. |
