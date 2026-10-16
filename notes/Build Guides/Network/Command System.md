---
type: build-guide
section: Network
updated: 2026-10-12
---

# Command System

Part of [[Build Guides]] › Network. Related: [[Gateway and Transport]], [[User Data and Storage]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

2873 accepted functions: 1492 confirmed, 812 likely, 407 speculative, 162 unknown. Network-linked (`wire`): 81.

Subsystems: `net.command` (2293), `net.synchronization` (132), `net.event_command` (110), `net.factory` (86), `net.game_command` (79), `net.arch_command` (66), `net.archetype` (40), `net.broker` (36), `ui.broker` (31)

_Large or library guide: only confirmed and likely functions are listed (2304 of 2873)._

### `net.arch_command`

**ArchCommand** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e70b0` | ArchCommand::ArchCommand | confirmed | Calls 0140c090 (base ctor) first, then sets vftable to ArchCommand and zeroes the AccountID field at +4. Returns the object. |
| `014e7130` | ArchCommand::GetTypeName | confirmed | Virtual slot [11] of ArchCommand. Returns the constant string 'ArchCommand'. |
| `014e7170` | ArchCommand::~ArchCommand | confirmed | Stores the ArchCommand vftable into this, then calls 0140c160 (base teardown that resets the NetworkCommand, PersistentComponent and PersistentBase vftables). … |
| `014e71f0` | ArchCommand::Clone | confirmed | Virtual slot [17] of ArchCommand. Allocates 8 bytes via 00d833a6; if non-null it constructs an ArchCommand with 014e70b0, then calls vtable slot [18] on this w… |
| `014e7260` | ArchCommand::CloneInto | confirmed | Virtual slot [18] of ArchCommand. Runtime-casts param_2 (NetworkCommand RTTI) to ArchCommand; on null it asserts 'clone' at archcommand.cpp line 0x2b. Then cal… |
| `014e73d0` | ArchCommand::ScalarDeletingDtor | confirmed | Virtual slot [0] of ArchCommand. Stores the ArchCommand vftable, calls base teardown 0140c160, then frees the object through _free when bit 0 of the flag argum… |
| `014e7430` | ArchCommand::PrintSummary | confirmed | Virtual slot [19] of ArchCommand. Writes 'Arch Command' (13 bytes) through 01241650, then formats 'AccountID: %d' from this+4 with sprintf into a 1000-byte sta… |
| `014e74c0` | ArchCommand::PrintLabel | confirmed | Virtual slot [20] of ArchCommand. Writes the 12-byte string 'Arch Command' through 01241650 and returns. |
| `014e72b0` | ArchCommand::Read | likely | Virtual slot [9] of ArchCommand. Calls the slot-4 virtual on the argument, reads one integer into this+4 with 013c24d0 (readInteger), then calls the slot-5 vir… |

**ArchCommand_AddArchetypesToDB** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e68f0` | ArchCommand_AddArchetypesToDB::PrintSummary | confirmed | Virtual slot [19] of ArchCommand_AddArchetypesToDB. Writes 'Account Command Add Archetypes To DB' through 01241650 (37 bytes), then calls base PrintSummary 014… |
| `014e6920` | ArchCommand_AddArchetypesToDB::PrintLabel | confirmed | Virtual slot [20] of ArchCommand_AddArchetypesToDB. Writes the 36-byte string 'Account Command Add Archetypes To DB' through 01241650 and returns. |
| `014e6ac0` | ArchCommand_AddArchetypesToDB::ArchCommand_AddArchetypesToDB | confirmed | Calls base constructor 014e70b0 first, then sets the vftable to ArchCommand_AddArchetypesToDB and zeroes the fields at +0xc, +0x10 and +0x14. Returns the objec… |
| `014e6b40` | ArchCommand_AddArchetypesToDB::GetTypeName | confirmed | Virtual slot [11] of ArchCommand_AddArchetypesToDB. Returns the constant string 'ArchCommand_AddArchetypesToDB'. |
| `014e6c70` | ArchCommand_AddArchetypesToDB::~ArchCommand_AddArchetypesTo… | confirmed | Stores the ArchCommand_AddArchetypesToDB vftable, destroys the range at +0xc..+0x10 through 012a9f10 and 012327d0, frees the list storage at +0xc through _free… |
| `014e6d30` | ArchCommand_AddArchetypesToDB::ScalarDeletingDtor | confirmed | Virtual slot [0] of ArchCommand_AddArchetypesToDB. Calls the body destructor 014e6c70, then frees the object through _free when bit 0 of the flag argument is s… |
| `014e6d50` | ArchCommand_AddArchetypesToDB::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to ArchCommand_AddArchetypesToDB (NetworkCommand source RTTI); on null it asserts 'clone' at archcommandaddar… |
| `014e68b0` | ArchCommand_AddArchetypesToDB::Write | likely | Virtual slot [10] of ArchCommand_AddArchetypesToDB. Calls the slot-2 virtual, calls base Write 014e72f0, calls 012fcdc0 on this+8, then calls the slot-3 virtua… |

**ArchCommand_DeleteArchetype** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e6280` | ArchCommand_DeleteArchetype::ArchCommand_DeleteArchetype | confirmed | Calls base constructor 014e70b0 first, then sets the vftable to ArchCommand_DeleteArchetype and zeroes the fields at +8 and +0xc. Returns the object. |
| `014e6300` | ArchCommand_DeleteArchetype::GetTypeName | confirmed | Virtual slot [11] of ArchCommand_DeleteArchetype. Returns the constant string 'ArchCommand_DeleteArchetype'. |
| `014e6340` | ArchCommand_DeleteArchetype::~ArchCommand_DeleteArchetype | confirmed | Stores the ArchCommand_DeleteArchetype vftable into this, then calls base destructor 014e7170. Does not free the object. |
| `014e6430` | ArchCommand_DeleteArchetype::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to ArchCommand_DeleteArchetype (NetworkCommand source RTTI); on null it asserts 'clone' at archcommanddeletea… |
| `014e65c0` | ArchCommand_DeleteArchetype::ScalarDeletingDtor | confirmed | Virtual slot [0] of ArchCommand_DeleteArchetype. Stores the ArchCommand_DeleteArchetype vftable, calls base destructor 014e7170, then frees the object through … |
| `014e6620` | ArchCommand_DeleteArchetype::PrintSummary | confirmed | Virtual slot [19] of ArchCommand_DeleteArchetype. Writes 'Account Command Delete Archetype' through 01241650, calls base PrintSummary 014e7430, then writes 'Ar… |
| `014e66e0` | ArchCommand_DeleteArchetype::PrintLabel | confirmed | Virtual slot [20] of ArchCommand_DeleteArchetype. Writes the 32-byte string 'Account Command Delete Archetype' through 01241650 and returns. |
| `014e6490` | ArchCommand_DeleteArchetype::Read | likely | Virtual slot [9] of ArchCommand_DeleteArchetype. Calls the slot-4 virtual on the archive with a pointer to a local holding this, calls base Read 014e72b0, read… |
| `014e6500` | ArchCommand_DeleteArchetype::Write | likely | Virtual slot [10] of ArchCommand_DeleteArchetype. Calls the slot-2 virtual, calls base Write 014e72f0, appends the integer at +0xc and then the integer at +8 t… |

**ArchCommand_GetArchetypeID** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e5d70` | ArchCommand_GetArchetypeID::ArchCommand_GetArchetypeID | confirmed | Calls base constructor 014e70b0 on the object first, then sets its vftable to ArchCommand_GetArchetypeID and zeroes the fields at +8 and +0xc. Returns the obje… |
| `014e5df0` | ArchCommand_GetArchetypeID::GetTypeName | confirmed | Virtual slot [11] of ArchCommand_GetArchetypeID. Returns the constant string 'ArchCommand_GetArchetypeID'. |
| `014e5e30` | ArchCommand_GetArchetypeID::~ArchCommand_GetArchetypeID | confirmed | Stores the ArchCommand_GetArchetypeID vftable into this, then calls base destructor 014e7170. It does not free the object. |
| `014e5fb0` | ArchCommand_GetArchetypeID::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to ArchCommand_GetArchetypeID (NetworkCommand source RTTI); on null it asserts 'clone' at archcommandgetarche… |
| `014e6140` | ArchCommand_GetArchetypeID::ScalarDeletingDtor | confirmed | Virtual slot [0] of ArchCommand_GetArchetypeID. Stores the ArchCommand_GetArchetypeID vftable, calls base destructor 014e7170, then frees the object through _f… |
| `014e61a0` | ArchCommand_GetArchetypeID::PrintSummary | confirmed | Virtual slot [19] of ArchCommand_GetArchetypeID. Writes the header 'Account Command Get ArchetypeID' through 01241650, calls base PrintSummary 014e7430, then w… |
| `014e6260` | ArchCommand_GetArchetypeID::PrintLabel | confirmed | Virtual slot [20] of ArchCommand_GetArchetypeID. Writes the 31-byte string 'Account Command Get ArchetypeID' through 01241650 and returns. |
| `014e6010` | ArchCommand_GetArchetypeID::Read | likely | Virtual slot [9] of ArchCommand_GetArchetypeID. Calls the slot-4 virtual on the archive with a pointer to a local holding this, calls base Read 014e72b0, reads… |
| `014e6080` | ArchCommand_GetArchetypeID::Write | likely | Virtual slot [10] of ArchCommand_GetArchetypeID. Calls the slot-2 virtual, calls base Write 014e72f0, appends the integer at +0xc and then the integer at +8 th… |

**ArchCommand_SetEnabledArchetypes** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e5c20` | ArchCommand_SetEnabledArchetypes::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 (NetworkCommand RTTI) to ArchCommand_SetEnabledArchetypes; if the cast is null it asserts with message 'clone… |
| `014e5d10` | ArchCommand_SetEnabledArchetypes::Read | likely | Virtual slot [9] of ArchCommand_SetEnabledArchetypes. Calls the slot-4 virtual on the archive with a pointer to this, calls base Read 014e72b0, then reads the … |

### `net.archetype`

**ArchCommand_SetEnabledArchetypes** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e59c0` | ArchCommand_SetEnabledArchetypes::~ArchCommand_SetEnabledAr… | confirmed | Fastcall destructor body (non-deleting). Stores the SetEnabledArchetypes vftable into *this; if the pointer at +0xc is non-null calls _free on it; zeroes +0xc,… |
| `014e5a50` | ArchCommand_SetEnabledArchetypes::GetTypeName | confirmed | Returns the address of the constant string "ArchCommand_SetEnabledArchetypes" (mov eax, imm32; ret). |
| `014e5a90` | ArchCommand_SetEnabledArchetypes::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls base write 014e72f0 with the buffer; calls 012fbec0 with this+8 (the enable… |
| `014e5ad0` | ArchCommand_SetEnabledArchetypes::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e59c0 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e5af0` | ArchCommand_SetEnabledArchetypes::DebugPrint | confirmed | Virtual (slot 19) with one stack argument (used as the output for 014e7430). Writes "Arch Command Set Enabled Archetypes " (len 0x24) through 01241650, then ca… |
| `014e5b20` | ArchCommand_SetEnabledArchetypes::PrintTag | confirmed | Virtual (slot 20). Calls 01241650 with the fixed text "Arch Command Set Enabled Archetypes" (len 0x23) and returns nothing. |
| `014e5b40` | ArchCommand_SetEnabledArchetypes::ArchCommand_SetEnabledArc… | confirmed | Fastcall default constructor. Calls base init 014e70b0 with this, stores the SetEnabledArchetypes vftable at +0, zeroes the dwords at +0xc, +0x10 and +0x14, re… |
| `014e5a60` | ArchCommand_SetEnabledArchetypes::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8d4 (mov eax,[01cbe8d4]; ret). No this pointer is read. |
| `014e5a70` | ArchCommand_SetEnabledArchetypes::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8d4 (mov [01cbe8d4],eax; ret 4). No this pointer is read. |
| `014e5bb0` | ArchCommand_SetEnabledArchetypes::Clone | likely | Fastcall virtual (slot 17). Allocates 0x18 bytes with operator new (00d833a6); if non-null constructs the new object with the default constructor 014e5b40 and … |

**ArchCommand_StoreArchetype** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e53f0` | ArchCommand_StoreArchetype::ArchCommand_StoreArchetype | confirmed | Fastcall default constructor. Calls base init 014e70b0 with this, stores the StoreArchetype vftable at +0, zeroes the dwords at +8 and +0xc, returns this. |
| `014e5470` | ArchCommand_StoreArchetype::GetTypeName | confirmed | Returns the address of the constant string "ArchCommand_StoreArchetype" (mov eax, imm32; ret). |
| `014e54b0` | ArchCommand_StoreArchetype::~ArchCommand_StoreArchetype | confirmed | Fastcall destructor body (non-deleting). Stores the StoreArchetype vftable into *this; if the pointer at +8 is non-null calls its scalar deleting destructor (v… |
| `014e5610` | ArchCommand_StoreArchetype::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to ArchCommand_StoreArchetype (00d8d382); if null calls 012f5a60… |
| `014e56b0` | ArchCommand_StoreArchetype::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls base write 014e72f0; writes the integer at +0xc with 013c23f0; if the point… |
| `014e57a0` | ArchCommand_StoreArchetype::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e54b0 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e57c0` | ArchCommand_StoreArchetype::Read | confirmed | Thiscall virtual (slot 9 override). Calls virtual slot 4 (vtable +0x10) with (param_2, &local) where local starts as this; if nonzero calls base read 014e72b0 … |
| `014e5830` | ArchCommand_StoreArchetype::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Account Command Store Archetype " (len 0x20), base text routine 014e7430(param_2), then prints "ArchID: %d" wi… |
| `014e5900` | ArchCommand_StoreArchetype::PrintTag | confirmed | Virtual (slot 20). Calls 01241650 with the fixed text "Account Command Store Archetype" (len 0x1f) and returns nothing. |
| `014e5480` | ArchCommand_StoreArchetype::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8d0 (mov eax,[01cbe8d0]; ret). No this pointer is read. |
| `014e5490` | ArchCommand_StoreArchetype::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8d0 (mov [01cbe8d0],eax; ret 4). No this pointer is read. |
| `014e55a0` | ArchCommand_StoreArchetype::Clone | likely | Fastcall virtual (slot 17). Allocates 0x10 bytes with operator new (00d833a6); if non-null constructs the new object with 014e53f0 (default ctor) and returns i… |
| `014e5720` | ArchCommand_StoreArchetype::Create | likely | Non-member function. Allocates 0x10 bytes with operator new (00d833a6); if non-null calls 014e53f0 (default ctor) with the block in ECX and returns its EAX; if… |

**ArchCommand_ValidateUsername** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e4f20` | ArchCommand_ValidateUsername::Read | confirmed | Thiscall virtual (slot 9 override). Calls virtual slot 4 (vtable +0x10) with (param_2, &local) where local = this; if nonzero calls 014e72b0 with param_2 (base… |
| `014e4fa0` | ArchCommand_ValidateUsername::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls base write 014e72f0 with the buffer; writes the integer at +0x24 with 013c2… |
| `014e4ff0` | ArchCommand_ValidateUsername::ArchCommand_ValidateUsername | confirmed | Fastcall default constructor. Calls base init 014e70b0 with this, stores the ValidateUsername vftable at +0, sets the string member at +8 to empty (size dword … |
| `014e5080` | ArchCommand_ValidateUsername::GetTypeName | confirmed | Returns the address of the constant string "ArchCommand_ValidateUsername" (mov eax, imm32; ret). |
| `014e50c0` | ArchCommand_ValidateUsername::~ArchCommand_ValidateUsername | confirmed | Fastcall destructor body (non-deleting). Stores the ValidateUsername vftable into *this; if the capacity dword at +0x20 is above 15 calls _free on the pointer … |
| `014e5230` | ArchCommand_ValidateUsername::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e50c0 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e5250` | ArchCommand_ValidateUsername::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to ArchCommand_ValidateUsername (00d8d382); if null calls 012f5a… |
| `014e52c0` | ArchCommand_ValidateUsername::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Arch Command Validate Username " (len 0x1f), then base text routine 014e7430(param_2). Prints the username: us… |
| `014e53d0` | ArchCommand_ValidateUsername::PrintTag | confirmed | Virtual (slot 20). Calls 01241650 with the fixed text "Arch Command Validate Username" (len 0x1e) and returns nothing. |
| `014e4e80` | ArchCommand_ValidateUsername::Execute | likely | Fastcall virtual (slot 15, returns 1). Calls 012d46a0, then allocates 0x14 bytes with operator new (00d833a6); if non-null calls 012f9eb0 to build the object. … |
| `014e5090` | ArchCommand_ValidateUsername::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8cc (mov eax,[01cbe8cc]; ret). No this pointer is read. |
| `014e50a0` | ArchCommand_ValidateUsername::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8cc (mov [01cbe8cc],eax; ret 4). No this pointer is read. |
| `014e5140` | ArchCommand_ValidateUsername::Clone | likely | Fastcall virtual (slot 17). Allocates 0x2c bytes with operator new (00d833a6); if non-null constructs the new object with 014e4ff0 (default ctor) and returns i… |
| `014e51b0` | ArchCommand_ValidateUsername::Create | likely | Non-member function. Allocates 0x2c bytes with operator new (00d833a6); if non-null calls 014e4ff0 (default ctor) with the block in ECX and returns its EAX; if… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e5210` | RegisterFactory0x1c4 | likely | Pushes the address of 014e51b0 and key 0x1c4 (decimal 452), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair… |
| `014e5780` | RegisterFactory0x1c1 | likely | Pushes the address of 014e5720 and key 0x1c1 (decimal 449), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair… |

### `net.broker`

**AdminBrokerServerVersionPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beee40` | AdminBrokerServerVersionPacket::Dtor | confirmed | Releases the owned version string reference and restores BaseAdminBrokerPacket then BasePacket vtables. |
| `00bef070` | AdminBrokerServerVersionPacket::ScalarDeletingDtor | confirmed | Calls the server-version packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**BaseAdminBrokerPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bee720` | BaseAdminBrokerPacket::CtorWithSubtype 🌐 | confirmed | Installs BasePacket and BaseAdminBrokerPacket vtables, writes 0x84 to field 1, and stores the supplied subtype in field 2. |
| `00bee790` | BaseAdminBrokerPacket::Dtor | confirmed | Restores BaseAdminBrokerPacket and BasePacket vtables during destruction. |
| `00bee7e0` | BaseAdminBrokerPacket::ScalarDeletingDtor | confirmed | Restores BaseAdminBrokerPacket and BasePacket vtables, frees the object if flag bit 0 is set, and otherwise returns its pointer. |

**BaseBrokerPacket** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bee5f0` | BaseBrokerPacket::CtorWithSubtype 🌐 | confirmed | Installs BasePacket and BaseBrokerPacket vtables, writes 0x82 to field 1, and stores the supplied subtype in field 2. |
| `00bee660` | BaseBrokerPacket::Dtor | confirmed | Restores BaseBrokerPacket and then BasePacket vtables during destruction; no owned payload fields are released here. |
| `00bee6b0` | BaseBrokerPacket::ScalarDeletingDtor | confirmed | Restores BaseBrokerPacket and BasePacket vtables, frees the object if flag bit 0 is set, and otherwise returns its pointer. |
| `00bee8c0` | BaseBrokerPacket::DtorException | confirmed | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |
| `00bee970` | BaseBrokerPacket::DtorException | confirmed | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |
| `00bee9c0` | BaseBrokerPacket::DtorException | confirmed | Restores BaseBrokerPacket and BasePacket vtables during exception cleanup. |

**BrokerErrorPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beef40` | BrokerErrorPacket::Dtor | confirmed | Releases the owned shared string and restores BaseBrokerPacket then BasePacket vtables. |
| `00bef090` | BrokerErrorPacket::ScalarDeletingDtor | confirmed | Calls the broker error packet destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**BrokerInformationPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beeac0` | BrokerInformationPacket::ScalarDeletingDtor | confirmed | Restores the broker base vtables, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `00bee850` | BrokerInformationPacket::Ctor 🌐 | likely | Constructs subtype 12 of BaseBrokerPacket, installs BrokerInformationPacket vtable, and clears four payload fields. |

**BrokerMyItemAddedPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bee910` | BrokerMyItemAddedPacket::Ctor 🌐 | confirmed | Constructs subtype 9 of BaseBrokerPacket and installs BrokerMyItemAddedPacket vtable. |
| `00beeb30` | BrokerMyItemAddedPacket::ScalarDeletingDtor | confirmed | Restores the broker base vtables, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**BrokerRemoveItemFromListsPacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00beec10` | BrokerRemoveItemFromListsPacket::ScalarDeletingDtor | confirmed | Restores the broker base vtables, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `00beeba0` | BrokerRemoveItemFromListsPacket::Ctor 🌐 | likely | Constructs broker subtype 11, installs the concrete vtable, and copies two default payload values from globals. |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bef790` | BrokerErrorPacket_Parse 🌐 | confirmed | Constructs a broker error packet, reads a string key, logs a missing key, performs broker proxy callbacks, and destroys temporary string state. |
| `00befcf0` | BaseBrokerPacket_Dispatch 🌐 | confirmed | Reads a broker subtype from the payload and dispatches subtypes 8, 9, 10, 0xb, 0xc, 0x10 and 0x11 to handlers or version logging. |
| `00beece0` | BrokerProxyCtor | likely | Initializes six proxy fields, allocates a 0x2b0-byte control object, constructs it when allocation succeeds, and releases its reference. |
| `00bef5c0` | BrokerItemPacket_ReadFields | likely | Reads coordinate-like pairs, scalar values and a boolean into fixed object offsets, calling shared pair readers and setting reader failure on truncation. |
| `00bef900` | BrokerInformationPacket_Parse 🌐 | likely | Constructs BrokerInformationPacket, decodes four 32-bit fields from a bounded reader, stores them in packet offsets, and invokes a proxy callback. |
| `00bef9f0` | BrokerRemoveItemFromListsPacket_Parse | likely | Constructs a remove-item packet, reads two 32-bit values, and invokes two client proxy callbacks when the payload range is valid. |
| `00befb40` | ReadBrokerItemPacket | likely | Reads a prefix and broker item fields from the bounded reader, returning true when decoding succeeded and the requested range is valid. |
| `00befbc0` | BrokerMyItemAddedPacket_Parse | likely | Allocates a broker item, parses its bounded fields, then dispatches an item-added callback when the payload is valid. |

### `net.command`

**AccountCommand** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01407af0` | AccountCommand::AccountCommand | confirmed | Thiscall (ECX = this). Calls base constructor 0140c090 on this, stores AccountCommand::vftable into this, zeroes the dword at this+4 (the AccountID field, see … |
| `01407b70` | AccountCommand::GetClassName | confirmed | Virtual slot 11 (+0x2c) of AccountCommand::vftable. Returns the address of the literal 'AccountCommand' (bytes b8 imm32 / c3: mov eax, imm32; ret). |
| `01407bb0` | AccountCommand::DtorBody | confirmed | Stores AccountCommand::vftable into this, then calls base destructor body 0140c160 on this and returns. Frame setup and the /GS cookie check are present but ca… |
| `01407ca0` | AccountCommand::CloneInto | confirmed | Virtual slot 18 (+0x48) of AccountCommand::vftable. Casts its argument (NetworkCommand*) to AccountCommand through 00d8d382; asserts the cast succeeded ('clone… |
| `01407e10` | AccountCommand::~AccountCommand | confirmed | Scalar deleting destructor (vtable slot 0 of AccountCommand::vftable): stores AccountCommand::vftable into this, calls base destructor body 0140c160 on this, t… |
| `01407e70` | AccountCommand::Dump | confirmed | Virtual slot 19 (+0x4c) of AccountCommand::vftable. Appends 'Account Command' plus newline (16 characters) to its output argument through 01241650, calls 0140c… |
| `01407f00` | AccountCommand::AppendTypeName | confirmed | Virtual slot 20 (+0x50) of AccountCommand::vftable. Appends the 15-character literal 'Account Command' through 01241650 (std::string append) on the receiver, w… |
| `01407b80` | AccountCommand::GetBaseSharedValue | likely | Virtual slot 12 (+0x30) of AccountCommand::vftable. Returns the dword at the global DAT_01cbe22c (bytes a1 imm32 / c3: mov eax,[DAT_01cbe22c]; ret). This is a … |
| `01407b90` | AccountCommand::SetBaseSharedValue | likely | Virtual slot 13 (+0x34) of AccountCommand::vftable. Stores its stack argument into the global dword DAT_01cbe22c and returns with ret 4 (bytes 8b 44 24 04 / a3… |
| `01407c30` | AccountCommand::CreateAndCallSlot18 | likely | Virtual slot 17 (+0x44) of AccountCommand::vftable. Allocates 8 bytes with operator new (00d833a6); when non-null it constructs an AccountCommand in the block … |
| `01407d90` | AccountCommand::Create | likely | Allocates 8 bytes with operator new (00d833a6). Returns 0 when the allocation fails; otherwise constructs an AccountCommand in the block through 01407af0 and r… |

**AccountCommand_AddAchievement** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ef620` | AccountCommand_AddAchievement::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 01909cd4 at [0]; zeroes the dword at +8; returns this. |
| `014ef6a0` | AccountCommand_AddAchievement::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_AddAchievement" (mov eax,imm; ret). |
| `014ef6e0` | AccountCommand_AddAchievement::Dtor | confirmed | Stores vtable 01909cd4 at [0]; if the pointer at +8 is non-null calls its vtable slot [0] with flag 1 and nulls +8; calls base destructor 01407bb0 on this. Pla… |
| `014ef850` | AccountCommand_AddAchievement::Clone | confirmed | Allocates 0xc bytes with operator new (00d833a6); if non-null constructs the object with 014ef620; then calls this->vtable[+0x48] (slot [18]) with the new obje… |
| `014ef8c0` | AccountCommand_AddAchievement::CloneInto | confirmed | Virtual slot [18]: dynamic-casts param_2 to AccountCommand_AddAchievement (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandAddAchi… |
| `014efab0` | AccountCommand_AddAchievement::ScalarDeletingDtor | confirmed | Calls the destructor body 014ef6e0 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014efb20` | AccountCommand_AddAchievement::Dump | confirmed | Virtual slot [19]: appends "Add Achievement Account Command\n" via 01241650 and calls 01407e70 (base dump) with the stack argument. |
| `014efb50` | AccountCommand_AddAchievement::AppendTypeName | confirmed | Appends the literal "Add Achievement Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ef930` | AccountCommand_AddAchievement::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 01407d30 (base step); if the child at +8 is null writes integer 1 (013c23f0) else writes 0 and calls… |
| `014efad0` | AccountCommand_AddAchievement::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with &param_2; calls 01407cf0 (base step); calls 014ef990 with param_2 and this+8 (optional child read); the… |

**AccountCommand_BulkBriefIntroduceAccount** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ef020` | AccountCommand_BulkBriefIntroduceAccount::Dump | confirmed | Virtual slot [19]: appends "Bulk Introduce Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "Accounts: %d of them\n… |
| `014ef0c0` | AccountCommand_BulkBriefIntroduceAccount::AppendTypeName | confirmed | Appends the 38-byte literal "Bulk Introduce Account Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ef0e0` | AccountCommand_BulkBriefIntroduceAccount::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 01909c4c at [0]; zeroes the dwords at +0xc, +0x10, +0x14, +0x1c, +0x20, +0x24, +0x2c, +0x30 and +0x34; r… |
| `014ef180` | AccountCommand_BulkBriefIntroduceAccount::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_BulkBriefIntroduceAccount" (mov eax,imm; ret). |
| `014ef1c0` | AccountCommand_BulkBriefIntroduceAccount::Dtor | confirmed | Stores vtable 01909c4c at [0]; destroys and frees the container at +0x2c..+0x34 (012327d0 on the range, then _free on the begin pointer) and zeroes it; the con… |
| `014ef2a0` | AccountCommand_BulkBriefIntroduceAccount::Clone | confirmed | Allocates 0x38 bytes with operator new (00d833a6); if non-null constructs the object with 014ef0e0; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014ef390` | AccountCommand_BulkBriefIntroduceAccount::ScalarDeletingDtor | confirmed | Calls the destructor body 014ef1c0 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014eefa0` | AccountCommand_BulkBriefIntroduceAccount::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 01407d30 (base step), 012fbec0 with this+8, 012fcdc0 with this+0x28 and 012f… |

**AccountCommand_BulkIntroduceAccount** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ee2f0` | AccountCommand_BulkIntroduceAccount::Dump | confirmed | Virtual slot [19]: appends "Bulk Introduce Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "Accounts: %d of them\n… |
| `014ee390` | AccountCommand_BulkIntroduceAccount::AppendTypeName | confirmed | Appends the 38-byte literal "Bulk Introduce Account Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ee3b0` | AccountCommand_BulkIntroduceAccount::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 01909b0c at [0]; zeroes dwords at +0xc, +0x10, +0x14, +0x1c, +0x20, +0x24, +0x2c, +0x30, +0x34, +0x3c, +… |
| `014ee470` | AccountCommand_BulkIntroduceAccount::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_BulkIntroduceAccount" (mov eax,imm; ret). |
| `014ee680` | AccountCommand_BulkIntroduceAccount::Clone | confirmed | Allocates 0x58 bytes with operator new (00d833a6); if non-null constructs the object with 014ee3b0; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014ee770` | AccountCommand_BulkIntroduceAccount::ScalarDeletingDtor | confirmed | Calls the destructor body 014ee4b0 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014ee790` | AccountCommand_BulkIntroduceAccount::CloneInto | confirmed | Virtual slot [18] (433 bytes): dynamic-casts param_2 to AccountCommand_BulkIntroduceAccount (00d8d382); on null prints the "clone" trace via 012f5a60 (file Acc… |
| `014edd80` | AccountCommand_BulkIntroduceAccount::BuildAndPushEvents | likely | Virtual slot [15] (1231 bytes): calls 012dfbc0 and traces 'accountService' when it returns 0 (line 0x30); walks parallel element lists of this: an int list at … |
| `014ee250` | AccountCommand_BulkIntroduceAccount::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 01407d30 (base step); then calls 012fbec0 with this+8, 012fcdc0 with this+0x… |

**AccountCommand_ChangePassword** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ed8b0` | AccountCommand_ChangePassword::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 0190990c at [0]; initialises the string at +8 empty (size dword at +0x1c = 0, capacity dword at +0x20 = … |
| `014ed940` | AccountCommand_ChangePassword::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_ChangePassword" (mov eax,imm; ret). |
| `014ed980` | AccountCommand_ChangePassword::Dtor | confirmed | Stores vtable 0190990c at [0]; frees the heap pointer at +0xc when the capacity at +0x20 exceeds 0xf, then sets +0x20 = 0xf, +0x1c = 0 and the byte at +0xc = 0… |
| `014eda00` | AccountCommand_ChangePassword::Clone | confirmed | Allocates 0x24 bytes with operator new (00d833a6); if non-null constructs the object with 014ed8b0; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014edaf0` | AccountCommand_ChangePassword::ScalarDeletingDtor | confirmed | Calls the destructor body 014ed980 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014edc60` | AccountCommand_ChangePassword::CloneInto | confirmed | Virtual slot [18]: dynamic-casts param_2 to AccountCommand_ChangePassword (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandChangeP… |
| `014edcc0` | AccountCommand_ChangePassword::Dump | confirmed | Virtual slot [19]: appends "Change Password Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "Password: %s\n" forma… |
| `014edd60` | AccountCommand_ChangePassword::AppendTypeName | confirmed | Appends the 31-byte literal "Change Password Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ed810` | AccountCommand_ChangePassword::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with &param_2; calls 01407cf0 (base step) on param_2; reads one string into this+8 with 013c28d0 (SerialRead… |
| `014ed860` | AccountCommand_ChangePassword::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 01407d30 (base step), SerialAppendString on this+8 (013c2860), a no-op trace… |

**AccountCommand_GetAccountInfo** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ed230` | AccountCommand_GetAccountInfo::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 0190975c at [0]; allocates 0x10 bytes (00d833a6) and, when non-null, constructs them with 01312930 and s… |
| `014ed2e0` | AccountCommand_GetAccountInfo::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_GetAccountInfo" (mov eax,imm; ret). |
| `014ed320` | AccountCommand_GetAccountInfo::Dtor | confirmed | Stores vtable 0190975c at [0]; if the pointer at +8 is non-null calls its vtable slot [0] with flag 1 (deleting destructor) and nulls +8; calls base destructor… |
| `014ed480` | AccountCommand_GetAccountInfo::Clone | confirmed | Allocates 0x10 bytes with operator new (00d833a6); if non-null constructs the object with 014ed230; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014ed4f0` | AccountCommand_GetAccountInfo::CloneInto | confirmed | Virtual slot [18]: dynamic-casts param_2 to AccountCommand_GetAccountInfo (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandGetAcco… |
| `014ed670` | AccountCommand_GetAccountInfo::ScalarDeletingDtor | confirmed | Calls the destructor body 014ed320 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014ed700` | AccountCommand_GetAccountInfo::Dump | confirmed | Virtual slot [19]: appends "Get Account Info Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "Properties: %d of th… |
| `014ed7d0` | AccountCommand_GetAccountInfo::AppendTypeName | confirmed | Appends the 32-byte literal "Get Account Info Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ed390` | AccountCommand_GetAccountInfo::CallPropertySet | likely | Thiscall with ECX=this: when the pointer at this+8 is null, traces 'mPropertySet' via 012f5a60 (file AccountCommandGetAccountInfo.cpp line 0x1c); then calls 01… |
| `014ed550` | AccountCommand_GetAccountInfo::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 01407d30 (base step), a no-op trace, then writes the presence flag: integer … |
| `014ed690` | AccountCommand_GetAccountInfo::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with &param_2 (a stack copy); calls 01407cf0 (base step); reads the property object into +8 with 012faf70(pa… |

**AccountCommand_GetAccountsByPartialName** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ecbb0` | AccountCommand_GetAccountsByPartialName::Dump | confirmed | Virtual slot [19]: appends "Get Account By Partial Name Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "Partial A… |
| `014ecc90` | AccountCommand_GetAccountsByPartialName::AppendTypeName | confirmed | Appends the 43-byte literal "Get Account By Partial Name Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014eccb0` | AccountCommand_GetAccountsByPartialName::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 019096d4 at [0]; zeroes the dwords at +0xc, +0x10, +0x14, +0x1c, +0x20, +0x24, +0x2c, +0x30, +0x34; sets… |
| `014ecd60` | AccountCommand_GetAccountsByPartialName::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_GetAccountsByPartialName" (mov eax,imm; ret). |
| `014ecda0` | AccountCommand_GetAccountsByPartialName::Dtor | confirmed | Stores vtable 019096d4 at [0]; frees the string heap buffer at +0x3c when the capacity at +0x50 exceeds 0xf, then resets that string; for the container at +0x2… |
| `014ecea0` | AccountCommand_GetAccountsByPartialName::Clone | confirmed | Allocates 0x58 bytes with operator new (00d833a6); if non-null constructs the object with 014eccb0; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014ecf90` | AccountCommand_GetAccountsByPartialName::ScalarDeletingDtor | confirmed | Calls the destructor body 014ecda0 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014ecfb0` | AccountCommand_GetAccountsByPartialName::CloneInto | confirmed | Virtual slot [18]: dynamic-casts param_2 to AccountCommand_GetAccountsByPartialName (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountComm… |
| `014ec9d0` | AccountCommand_GetAccountsByPartialName::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 01407d30 (base step), then SerialAppendString on this+0x38 (013c2860) and Se… |
| `014ed060` | AccountCommand_GetAccountsByPartialName::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with a 12-byte local; calls 01407cf0 (base step); reads the string at +0x38 (013c28d0), the dword at +0x54 (… |

**AccountCommand_IntroduceAccount** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ec370` | AccountCommand_IntroduceAccount::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 019094b4 at [0]; initialises the string at +8 empty (size dword at +0x1c = 0, capacity at +0x20 = 0xf) a… |
| `014ec440` | AccountCommand_IntroduceAccount::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_IntroduceAccount" (mov eax,imm; ret). |
| `014ec480` | AccountCommand_IntroduceAccount::Dtor | confirmed | Stores vtable 019094b4 at [0]; if the pointer at +0x44 is non-null calls its vtable slot [0] with flag 1 (deleting destructor) and nulls +0x44; frees the heap … |
| `014ec540` | AccountCommand_IntroduceAccount::Clone | confirmed | Allocates 0x48 bytes with operator new (00d833a6); if non-null constructs the object with 014ec370; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014ec630` | AccountCommand_IntroduceAccount::ScalarDeletingDtor | confirmed | Calls the destructor body 014ec480 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014ec700` | AccountCommand_IntroduceAccount::CloneInto | confirmed | Virtual slot [18]: dynamic-casts param_2 to AccountCommand_IntroduceAccount (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandIntro… |
| `014ec780` | AccountCommand_IntroduceAccount::Dump | confirmed | Virtual slot [19]: appends "Introduce Account Command\n" via 01241650; calls 01407e70 (base dump) with the stack argument; appends "AccountName: %s\n" from the… |
| `014ec8d0` | AccountCommand_IntroduceAccount::AppendTypeName | confirmed | Appends the 33-byte literal "Introduce Account Account Command" via 01241650 (pseudocode shows no argument; receiver not decoded from bytes). |
| `014ec090` | AccountCommand_IntroduceAccount::CallPropertySet | likely | Thiscall with ECX=this: when the pointer at this+0x44 is null, traces 'mPropertySet' via 012f5a60 (file AccountCommandIntroduceAccount.cpp line 0x3d); then cal… |
| `014ec0d0` | AccountCommand_IntroduceAccount::BuildAndPushEvent | likely | Virtual slot [15]: traces 'mPropertySet' when this+0x44 is null (line 0x48), calls 012dfbc0 and traces 'accountService' when it returns 0 (line 0x4a), calls 01… |
| `014ec210` | AccountCommand_IntroduceAccount::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 013c26e0 (no-op, trace text not emitted), 01407d30 (base step), then SerialAppendString on this+8 (0… |
| `014ec2e0` | AccountCommand_IntroduceAccount::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with &this; calls 01407cf0 (base step); reads a string into this+8 (013c28d0); reads the property object via… |

**AccountCommand_ModifyRestrictions** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014eb990` | AccountCommand_ModifyRestrictions::Dtor | confirmed | Stores vtable 0190924c at [0]; frees the pointer at +0x10 when non-null (00d82463); zeroes +0x10, +0x14 and +0x18; calls base destructor 01407bb0 on this. Plai… |
| `014eba20` | AccountCommand_ModifyRestrictions::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_ModifyRestrictions" (6-byte function: mov eax,imm; ret). |
| `014ebbf0` | AccountCommand_ModifyRestrictions::ScalarDeletingDtor | confirmed | Calls the destructor body 014eb990 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014ebc10` | AccountCommand_ModifyRestrictions::Dump | confirmed | Virtual slot [19]: appends "Modify Restrictions Account Command\n" via 01241650, calls 01407e70 (base dump) with the stack argument, appends "Account ID: %d\n"… |
| `014ebce0` | AccountCommand_ModifyRestrictions::AppendTypeName | confirmed | Appends the 35-byte literal "Modify Restrictions Account Command" via 01241650 (receiver not decoded from the bytes). |
| `014ebd00` | AccountCommand_ModifyRestrictions::ctor | confirmed | Calls base constructor 01407af0 on this, stores vtable 0190924c at [0], zeroes the dwords at +0x10, +0x14 and +0x18, and returns this. |
| `014ebd60` | AccountCommand_ModifyRestrictions::Clone | confirmed | Allocates 0x1c bytes with operator new (00d833a6); if non-null constructs the object with 014ebd00; then calls this->vtable[+0x48] (slot [18], 014ebdd0) with t… |
| `014ebdd0` | AccountCommand_ModifyRestrictions::CloneInto | confirmed | Dynamic-casts param_2 to AccountCommand_ModifyRestrictions (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandModifyRestrictions.cpp… |
| `014eba60` | AccountCommand_ModifyRestrictions::Write | likely | Virtual slot [10]: calls vtable slot [2] on param_2; calls 01407d30 (base step) with ECX=this; writes this+8 with 013c23f0 (SerialAppendInteger); zeroes a 5-wo… |
| `014ebed0` | AccountCommand_ModifyRestrictions::Read | likely | Virtual slot [9]: calls vtable slot [4] on param_2 with a 12-byte local; calls 01407cf0 (base step); reads an integer into this+8 (013c24d0); reads the raw len… |

**AccountCommand_RecordSinglePlayerGame** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01408130` | AccountCommand_RecordSinglePlayerGame::DtorBody | confirmed | Stores AccountCommand_RecordSinglePlayerGame::vftable into this, frees the heap buffer at this+0x20 when the string capacity at this+0x34 is greater than 15, r… |
| `014081d0` | AccountCommand_RecordSinglePlayerGame::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the address of the literal 'AccountCommand_RecordSinglePlayerGame' (mov eax, imm32; ret). |
| `01408210` | AccountCommand_RecordSinglePlayerGame::~AccountCommand_Reco… | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01408130 on this, then calls _free(this) when bit 0 of the second argument is set; return… |
| `01408230` | AccountCommand_RecordSinglePlayerGame::AccountCommand_Recor… | confirmed | Thiscall (ECX = this). Calls base constructor 01407af0 on this, stores AccountCommand_RecordSinglePlayerGame::vftable, zeroes the five dwords at this+0x8 throu… |
| `01408340` | AccountCommand_RecordSinglePlayerGame::CloneInto | confirmed | Virtual slot 18 (+0x48). Casts its argument (NetworkCommand*) to AccountCommand_RecordSinglePlayerGame through 00d8d382; asserts the cast ('clone', AccountComm… |
| `014083c0` | AccountCommand_RecordSinglePlayerGame::Dump | confirmed | Virtual slot 19 (+0x4c). Appends 'Record Single Player Game Command' and then calls 01407e70 (base dump). Appends the labelled fields: 'Activity Type: ' with '… |
| `01408590` | AccountCommand_RecordSinglePlayerGame::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Appends the 33-character literal 'Record Single Player Game Command' through 01241650 (std::string append) to its output argument, the… |
| `014081e0` | AccountCommand_RecordSinglePlayerGame::GetSharedValue | likely | Virtual slot 12 (+0x30). Returns the dword at the global DAT_01cbe230 (mov eax,[DAT_01cbe230]; ret). |
| `014081f0` | AccountCommand_RecordSinglePlayerGame::SetSharedValue | likely | Virtual slot 13 (+0x34). Stores its stack argument into the global dword DAT_01cbe230 and returns. |
| `014082d0` | AccountCommand_RecordSinglePlayerGame::CreateAndCallSlot18 | likely | Virtual slot 17 (+0x44). Allocates 0x38 bytes with operator new (00d833a6); when non-null it constructs the command in the block through 01408230 and keeps the… |
| `014085b0` | AccountCommand_RecordSinglePlayerGame::Create | likely | Allocates 0x38 bytes with operator new (00d833a6). Returns 0 when the allocation fails; otherwise constructs an AccountCommand_RecordSinglePlayerGame in the bl… |

**AccountCommand_RedeemCode** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014eb430` | AccountCommand_RedeemCode::Read | confirmed | Calls vtable slot [4] (+0x10) on param_2; if true calls 01407cf0(param_2); reads the string at this+8 with 013c28d0 (SerialReadString, receiver = the stream, s… |
| `014eb510` | AccountCommand_RedeemCode::ctor | confirmed | Calls base constructor 01407af0 on this; stores vtable 01909144 at [0]; sets the string at +8 empty (size dword at +0x1c = 0, capacity dword at +0x20 = 0xf); z… |
| `014eb5a0` | AccountCommand_RedeemCode::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_RedeemCode" (mov eax,imm; ret). |
| `014eb5e0` | AccountCommand_RedeemCode::Dtor | confirmed | Stores vtable 01909144 at [0]; if the capacity dword at +0x20 exceeds 0xf it frees the pointer at +0xc (00d82463); sets +0x20 to 0xf, +0x1c to 0 and the byte a… |
| `014eb660` | AccountCommand_RedeemCode::Clone | confirmed | Allocates 0x2c bytes with operator new (00d833a6); if non-null constructs the object with 014eb510; then calls this->vtable[+0x48] (slot [18]) with the new obj… |
| `014eb750` | AccountCommand_RedeemCode::ScalarDeletingDtor | confirmed | Calls the destructor body 014eb5e0 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014eb790` | AccountCommand_RedeemCode::CloneInto | confirmed | Dynamic-casts param_2 to AccountCommand_RedeemCode (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandRedeemCode.cpp line 0x4e). Cal… |
| `014eb800` | AccountCommand_RedeemCode::Dump | confirmed | Virtual slot [19]: appends "Redeem Code Account Command\n" via 01241650, calls 01407e70 (base dump) with the stack argument, appends "Code: %s\n" formatted fro… |
| `014eb920` | AccountCommand_RedeemCode::AppendTypeName | confirmed | Appends the 27-byte literal "Redeem Code Account Command" via 01241650 to the std::string at the stack argument (ret 4 shape, as the other AppendTypeName slots… |

**AccountCommand_RequestAvatarImages** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014eaf90` | AccountCommand_RequestAvatarImages::Dtor | confirmed | Stores vtable 01909024 at [0]; frees the pointer at +0xc when non-null (00d83ac6 path); zeroes +0xc, +0x10 and +0x14; calls base destructor 01407bb0 on this. P… |
| `014eb020` | AccountCommand_RequestAvatarImages::GetTypeName | confirmed | Returns the address of the literal "AccountCommand_RequestAvatarImages" (mov eax,imm; ret). |
| `014eb0a0` | AccountCommand_RequestAvatarImages::ScalarDeletingDtor | confirmed | Calls the destructor body 014eaf90 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014eb0c0` | AccountCommand_RequestAvatarImages::Dump | confirmed | Virtual slot [19]: appends "Request Avatar Images Account Command\n" (38 bytes) to the std::string passed as the stack argument via 01241650, then calls 01407e… |
| `014eb0f0` | AccountCommand_RequestAvatarImages::AppendTypeName | confirmed | Appends the 37-byte literal "Request Avatar Images Account Command" via 01241650 to the std::string at the stack argument (mov ecx,[esp+4]; push 0x25; push lit… |
| `014eb110` | AccountCommand_RequestAvatarImages::ctor | confirmed | Calls base constructor 01407af0 on this, stores vtable 01909024 at [0], zeroes the dwords at +0xc, +0x10 and +0x14, and returns this in EAX. |
| `014eb180` | AccountCommand_RequestAvatarImages::Clone | confirmed | Allocates 0x18 bytes with operator new (00d833a6); if non-null constructs the object with 014eb110; then calls this->vtable[+0x48] (slot [18], 014eb1f0) with t… |
| `014eb1f0` | AccountCommand_RequestAvatarImages::CloneInto | confirmed | Dynamic-casts param_2 to AccountCommand_RequestAvatarImages (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandRequestAvatarImages.c… |
| `014eb2e0` | AccountCommand_RequestAvatarImages::Read | likely | Calls vtable slot [4] (+0x10) on param_2 with &this; if true calls 01407cf0(param_2) (base step); if true calls 01300540 with this+8 as the container (per the … |

**AccountCommand_SendAvatarImage** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014eaa40` | AccountCommand_SendAvatarImage::ctor | confirmed | Calls base constructor 01407af0 on this, stores vtable 01908f14 at [0], zeroes the dwords at +8 and +0xc, and returns this in EAX. |
| `014eaac0` | AccountCommand_SendAvatarImage::GetTypeName | confirmed | Returns the address of the string literal "AccountCommand_SendAvatarImage" (mov eax,0x01908f6c; ret). |
| `014eab00` | AccountCommand_SendAvatarImage::Dtor | confirmed | Stores vtable 01908f14 at [0]; if the pointer at +0xc is non-null frees it (00d83ac6) and nulls +0xc; zeroes +8; calls base destructor 01407bb0 on this. Plain … |
| `014eab80` | AccountCommand_SendAvatarImage::Clone | confirmed | Allocates 0x10 bytes with operator new (00d833a6); if non-null constructs the object with 014eaa40; then calls this->vtable[+0x48] (slot [18], 014eabf0) with t… |
| `014eabf0` | AccountCommand_SendAvatarImage::CloneInto | confirmed | Dynamic-casts param_2 to AccountCommand_SendAvatarImage (00d8d382); on null prints the "clone" trace via 012f5a60 (file AccountCommandSendAvatarImage.cpp line … |
| `014eadb0` | AccountCommand_SendAvatarImage::ScalarDeletingDtor | confirmed | Calls the destructor body 014eab00 on this; if bit 0 of the second argument is set, frees this (00d82463); returns this. |
| `014eaf20` | AccountCommand_SendAvatarImage::Dump | confirmed | Virtual slot [19]: appends "Send Avatar Image Account Command\n" (34 bytes) to the std::string passed as the stack argument via 01241650, then calls 01407e70 (… |
| `014eaf50` | AccountCommand_SendAvatarImage::AppendTypeName | confirmed | Takes the std::string& from the stack argument ([esp+4]) as receiver (mov ecx,[esp+4]) and appends the 33-byte literal "Send Avatar Image Account Command" via … |
| `014eac60` | AccountCommand_SendAvatarImage::Read | likely | Calls vtable slot [4] (+0x10) on param_2; if true calls 01407cf0(param_2); reads an integer into this+8 with 013c24d0; allocates that many bytes (00d81d91 -> 0… |
| `014eace0` | AccountCommand_SendAvatarImage::Write | likely | Calls vtable slot [2] (+8) on param_2, then 01407d30(param_2), writes the byte count this+8 with 013c23f0 (SerialAppendInteger), writes the buffer this+0xc wit… |

**AccountCommand_SetAchievementList** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ea6a0` | AccountCommand_SetAchievementList::AppendTypeName | confirmed | Ignores its ECX; takes the std::string& passed as the stack argument (loaded from [esp+4]) as the receiver and appends the 36-byte literal "Set Achievement Lis… |
| `014ea740` | AccountCommand_SetAchievementList::ctor | confirmed | Calls the base constructor 01407af0 on this, stores vtable 01908df4 at [0], zeroes the dword fields at +0xc, +0x10 and +0x14, and returns this in EAX. |
| `014ea7b0` | AccountCommand_SetAchievementList::Clone | confirmed | Allocates 0x18 bytes with operator new (00d833a6); if non-null constructs the object with 014ea740 and keeps the new pointer; then calls this->vtable[+0x48] (s… |
| `014ea960` | AccountCommand_SetAchievementList::Read | likely | Calls vtable slot [4] (+0x10) on param_2 with &this; if true calls 01407cf0(param_2) (base step); if true calls 014ea6c0 with ECX=&local (holding param_2) and … |

**AccountCommand_SetCampaignStatus** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01321920` | AccountCommand_SetCampaignStatus::ctor | confirmed | Fastcall, returns this. Calls 01407af0 (base constructor), stores AccountCommand_SetCampaignStatus::vftable at *this, and zeroes the dwords at this+8, +0xc, +0… |
| `013219a0` | AccountCommand_SetCampaignStatus::TypeName | confirmed | Virtual slot [11] of AccountCommand_SetCampaignStatus::vftable. Decoded from bytes: mov eax, 0x18d7a8c; ret. Returns the C string 'AccountCommand_SetCampaignSt… |
| `013219e0` | AccountCommand_SetCampaignStatus::~AccountCommand_SetCampai… | confirmed | Fastcall on this. Stores AccountCommand_SetCampaignStatus::vftable at *this, builds a local pointer to this, and calls 01407bb0, the AccountCommand base destru… |
| `01321ac0` | AccountCommand_SetCampaignStatus::Clone | confirmed | Virtual slot [17] (Clone) of AccountCommand_SetCampaignStatus::vftable. Allocates 0x18 bytes through 00d833a6, and if that succeeds constructs the object with … |
| `01321b30` | AccountCommand_SetCampaignStatus::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo) of AccountCommand_SetCampaignStatus::vftable. dynamic_casts its argument to AccountCommand_SetCampaignStatus through 00d8d382 … |
| `01321cf0` | AccountCommand_SetCampaignStatus::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_SetCampaignStatus::vftable. Stores the vtable at *this, calls 01407bb0 (AccountCommand base destructor body), frees this thr… |
| `01321d50` | AccountCommand_SetCampaignStatus::DumpText | confirmed | Virtual slot [19] (Dump, trace text only). Appends the title 'Set Campaign Status Account Command' through 01241650, calls 01407e70 on its argument for the bas… |
| `01321e80` | AccountCommand_SetCampaignStatus::DisplayName | confirmed | Virtual slot [20], stdcall with one argument (ret 4). Decoded from bytes: loads the argument into ECX and calls 01241650 (std::string append of 0x23 bytes) wit… |
| `01321c70` | AccountCommand_SetCampaignStatus::Factory | likely | Allocates 0x18 bytes through 00d833a6 (operator new). If the allocation is non-null it constructs the command with 01321920 and returns the object; otherwise r… |
| `01321cd0` | AccountCommand_SetCampaignStatus::RegisterFactory | likely | Registers the factory 01321c70 under classID 0x19f (415) with 013ce130 and again with 013ce230 (both registry calls, factory passed as the code pointer). Calle… |

**AccountCommand_SetChallengeStatus** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014074c0` | AccountCommand_SetChallengeStatus::AppendCommandName | confirmed | Virtual slot 20 (+0x50). Loads the first stack argument (the output string) into ECX, pushes the 28-character literal 'Set Challenge Status Command' and its le… |
| `014074e0` | AccountCommand_SetChallengeStatus::DtorBody | confirmed | Stores AccountCommand_SetChallengeStatus::vftable into this, calls 01321610 (unlabeled), and if the pointer at this+0xc is non-null calls 012327d0 on the range… |
| `014075a0` | AccountCommand_SetChallengeStatus::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the address of the literal 'AccountCommand_SetChallengeStatus' (bytes b8 imm32 / c3: mov eax, imm32; ret). |
| `014075e0` | AccountCommand_SetChallengeStatus::AccountCommand_SetChalle… | confirmed | Thiscall (ECX = this). Calls base constructor 01407af0 on this, stores AccountCommand_SetChallengeStatus::vftable, zeroes the fields at +0xc, +0x10 and +0x14, … |
| `01407650` | AccountCommand_SetChallengeStatus::~AccountCommand_SetChall… | confirmed | Scalar deleting destructor (vtable slot 0): calls the body 014074e0 on this, then calls _free(this) when bit 0 of the second argument is set; returns this. |
| `01407860` | AccountCommand_SetChallengeStatus::CloneInto | confirmed | Virtual slot 18 (+0x48). Casts its argument (NetworkCommand*) to AccountCommand_SetChallengeStatus through 00d8d382; asserts the cast ('clone', AccountCommandS… |
| `014075b0` | AccountCommand_SetChallengeStatus::GetSharedValue | likely | Virtual slot 12 (+0x30). Returns the dword at the global DAT_01cbe228 (bytes a1 imm32 / c3: mov eax,[DAT_01cbe228]; ret). |
| `014075c0` | AccountCommand_SetChallengeStatus::SetSharedValue | likely | Virtual slot 13 (+0x34). Stores its stack argument into the global dword DAT_01cbe228 and returns with ret 4 (bytes 8b 44 24 04 / a3 imm32 / c2 04 00). |
| `01407670` | AccountCommand_SetChallengeStatus::CreateAndCallSlot18 | likely | Virtual slot 17 (+0x44). Allocates 0x24 bytes with operator new (00d833a6); if the allocation is non-null it constructs the command in the block through 014075… |
| `014076e0` | AccountCommand_SetChallengeStatus::Create | likely | Allocates 0x24 bytes with operator new (00d833a6). Returns 0 when the allocation fails; otherwise constructs the command in the block through 014075e0 and retu… |

**AccountCommand_SetPreferences** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01408630` | AccountCommand_SetPreferences::AccountCommand_SetPreferences | confirmed | Thiscall (ECX = this). Calls base constructor 01407af0 on this, stores AccountCommand_SetPreferences::vftable, allocates 0x10 bytes with operator new (00d833a6… |
| `014086e0` | AccountCommand_SetPreferences::GetClassName | confirmed | Virtual slot 11 (+0x2c). Returns the address of the literal 'AccountCommand_SetPreferences' (mov eax, imm32; ret). |
| `01408720` | AccountCommand_SetPreferences::DtorBody | confirmed | Stores AccountCommand_SetPreferences::vftable into this; if the owned pointer at this+0x8 is non-null, calls its vtable slot 0 with argument 1 (scalar deleting… |
| `01408790` | AccountCommand_SetPreferences::ForwardToPropertySet | confirmed | Thiscall with one stack argument. Asserts the owned pointer at this+8 (mPropertySet) is non-null (AccountCommandSetPreferences.cpp line 0x1d), then calls 01312… |
| `01408a80` | AccountCommand_SetPreferences::~AccountCommand_SetPreferenc… | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01408720 on this, then calls _free(this) when bit 0 of the second argument is set; return… |
| `01408af0` | AccountCommand_SetPreferences::Dump | confirmed | Virtual slot 19 (+0x4c). Appends 'Set Preferences Account Command' plus newline (0x20 characters) through 01241650, calls 01407e70 (AccountCommand::Dump) on it… |
| `01408b90` | AccountCommand_SetPreferences::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Appends the 31-character literal 'Set Preferences Account Command' through 01241650 (std::string append) and returns. |
| `014086f0` | AccountCommand_SetPreferences::GetSharedValue | likely | Virtual slot 12 (+0x30). Returns the dword at the global DAT_01cbe238 (mov eax,[DAT_01cbe238]; ret). |
| `01408700` | AccountCommand_SetPreferences::SetSharedValue | likely | Virtual slot 13 (+0x34). Stores its stack argument into the global dword DAT_01cbe238 and returns. |
| `014087d0` | AccountCommand_SetPreferences::Execute | likely | Virtual slot 15 (+0x3c) of AccountCommand_SetPreferences::vftable. Asserts the owned pointer at this+0x8 (mPropertySet) is non-null (AccountCommandSetPreferenc… |
| `014088c0` | AccountCommand_SetPreferences::CreateAndCallSlot18 | likely | Virtual slot 17 (+0x44). Allocates 0xc bytes with operator new (00d833a6); when non-null it constructs the command in the block through 01408630 and keeps the … |
| `01408a00` | AccountCommand_SetPreferences::Create | likely | Allocates 0xc bytes with operator new (00d833a6). Returns 0 when the allocation fails; otherwise constructs an AccountCommand_SetPreferences in the block throu… |

**Achievement** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01435080` | Achievement::Achievement | confirmed | Constructor (ECX = object, returns this in EAX). Stores PersistentBase::vftable, then PersistentComponent::vftable, then Achievement::vftable at +0. Sets dword… |
| `01435120` | Achievement::TypeName | confirmed | Returns the address of the literal Achievement (mov eax, imm32; ret). |
| `01435130` | Achievement::GetStaticDword | confirmed | Returns the dword at global DAT_01cbe2b8 (mov eax,[imm32]; ret). |
| `01435140` | Achievement::SetStaticDword | confirmed | Stores its one stack argument into global DAT_01cbe2b8 and returns with ret 4 (mov eax,[esp+4]; mov [imm32],eax; ret 4). |
| `01435160` | Achievement::~Achievement | confirmed | Destructor body (ECX = object). Stores Achievement::vftable, frees the string buffer at +0x24 with _free when the capacity at +0x38 is greater than 15, sets ca… |
| `014351f0` | Achievement::Clone | confirmed | Virtual slot [14] (+0x38) of Achievement (ECX = source). Allocates 0x3c bytes with operator new; if the allocation is non-null it constructs an Achievement on … |
| `014352e0` | Achievement::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 01435160 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |
| `01435320` | Achievement::CopyFieldsTo | confirmed | Virtual slot [15] (+0x3c) of Achievement (ECX = source, one stack argument = destination). Copies the seven dwords at +4 through +0x1c from source to destinati… |
| `01435260` | Achievement::Create | likely | Allocates 0x3c bytes with operator new (00d833a6). If the allocation is non-null it constructs an Achievement with 01435080 and returns the object pointer (the… |
| `014352c0` | Achievement::RegisterFactory | likely | Static initializer: registers the factory 01435260 under classID 0x1e0 (480) with 013ce130, then calls 013ce230 with the same value in the second argument slot… |

**Action** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01327540` | ActionFactory | likely | Factory for classID 18: allocates 0x70 bytes through 00d833a6 and, when the allocation succeeds, constructs an Action through 013270d0; returns the object or 0. |
| `013275a0` | Action::RegisterFactory | likely | Registers the Action factory 01327540 under classID 0x12 in the serialization registry through 013ce130 and 013ce230, both called with 0x12 and the factory. |

**ActionNode** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0149e6d0` | ActionNode::TypeName | confirmed | Returns the constant string "ActionNode" (mov eax, imm32; ret, 6 bytes). |
| `0149e6e0` | ActionNode::ReadGlobal01cbe668 | confirmed | Returns the dword at global DAT_01cbe668 (mov eax, [0x1cbe668]; ret). |
| `0149e6f0` | ActionNode::WriteGlobal01cbe668 | confirmed | Stores its stack argument into global DAT_01cbe668 and returns with ret 4. |
| `0149ee20` | ActionNode::Factory | likely | Allocates 0x2c bytes with 00d833a6 (operator new); when the allocation succeeds it runs the ActionNode constructor 0149e640 and returns the object, otherwise r… |
| `0149ee80` | ActionNode::RegisterFactory | likely | Passes classID 0x2c and the factory 0149ee20 to 013ce130 and then 013ce230 (both registry calls) and returns. |

**CWCommandObject_AddCostMod** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145a8a0` | CWCommandObject_AddCostMod::Write | confirmed | Virtual slot [10] of the +8 vtable, thiscall with one stream argument. Calls slot [2] (+0x8) header write, the no-op trace 013c26e0 ('Starting CWCommandObject_… |

**CWCommandObject_Destroy** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145a230` | CWCommandObject_Destroy::Write | confirmed | Virtual slot [10] of the +8 vtable, thiscall with one stream argument. Calls slot [2] (+0x8) header write, the no-op trace 013c26e0 ('Starting CWCommandObject_… |

**CWGameCommand_SetupGame** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01459d20` | CWGameCommand_SetupGame::Write | confirmed | Virtual slot [10], thiscall with one stream argument. Calls slot [2] (+0x8) header write, the no-op trace 013c26e0 ('Starting CWGameCommand_SetupGame'), base b… |

**CWProcessEventEmissionStateForActivePlayer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01516820` | CWProcessEventEmissionStateForActivePlayer::Read | confirmed | Virtual slot 9: calls slot 4 (header read) with the stream and an output pointer and returns 0 if that fails; calls base Read 01510da0 and returns 0 if it fail… |
| `01516860` | CWProcessEventEmissionStateForActivePlayer::Write | confirmed | Virtual slot 10: calls slot 1 when param_2 is 0 or slot 2 with param_2 otherwise (header write); emits the Starting trace line through 013c26e0 (a no-op return… |

**CommandObject** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142a880` | CommandObject::Read | confirmed | Called on the secondary subobject (object+8). Header read (vtable slot 4) with a local out-dword, base read FUN_01344600, then SerialReadString into the Comman… |
| `0142acd0` | CommandObject::Write | confirmed | Called on the secondary subobject (object+8). Header write (vtable slot 2), base write FUN_013449d0, CommandTypeName via SerialAppendString (sub+0x40), then in… |

**CommandObject_EmitTextMessage** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141f020` | CommandObject_EmitTextMessage::Write | confirmed | Virtual slot 10 (Write) of CommandObject_EmitTextMessage (reached through the +8 table): calls slot 2 header write on the stream, base write 0142acd0, 013c2860… |
| `0141f0c0` | CommandObject_EmitTextMessage::CommandObject_EmitTextMessage | confirmed | Constructor: calls 0142b070, stores CommandObject_EmitTextMessage::vftable at this and this+8, zeroes the size and capacity of the string at +0x128 (cap 0xf, s… |
| `0141f190` | CommandObject_EmitTextMessage::TypeName | confirmed | Virtual slot 11 of the +8 table: returns the literal 'CommandObject_EmitTextMessage'. |
| `0141f1d0` | CommandObject_EmitTextMessage::AdjustDeletingDtor | confirmed | Secondary-table slot 0 thunk: subtracts 8 from ECX and jumps to the deleting destructor 0141f320. |
| `0141f1e0` | CommandObject_EmitTextMessage::~CommandObject_EmitTextMessa… | confirmed | Dtor body: stores both vtables, frees the string buffer at +0x128 when its capacity (+0x13c) exceeds 15, resets size and capacity, calls 012ce9d0 before the cl… |
| `0141f320` | CommandObject_EmitTextMessage::DeletingDtor | confirmed | Scalar deleting destructor (primary table slot 0): calls the dtor body 0141f1e0, then _free on this when flag bit 0 of the stack arg is set; returns this. |
| `0141f1a0` | CommandObject_EmitTextMessage::GetStaticDword | likely | Virtual slot 12 of the +8 table: returns the global dword DAT_01cbe270. |
| `0141f1b0` | CommandObject_EmitTextMessage::SetStaticDword | likely | Virtual slot 13 of the +8 table: stores its stack arg into the global DAT_01cbe270 (ret 4). |

**CommandObject_WaitFor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fa940` | CommandObject_WaitFor::Create | likely | No arguments. Allocates 0x130 bytes with operator new (00d833a6); when the allocation succeeds it calls the default constructor 014fa810 and returns the object… |
| `014fa9c0` | CommandObject_WaitFor::RegisterFactory | likely | Pushes the factory 014fa940 and the constant 0x18c, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x18c (ret 8)… |

**CommandObject_While** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fa060` | CommandObject_While::Create | likely | No arguments. Allocates 0x124 bytes with operator new (00d833a6); when the allocation succeeds it calls the default constructor 014f9e90 and returns the object… |
| `014fa0e0` | CommandObject_While::RegisterFactory | likely | Pushes the factory 014fa060 and the constant 0x197, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x197 (ret 8)… |

**CommandObject_WinGame** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014fa470` | CommandObject_WinGame::Create | likely | No arguments. Allocates 0x128 bytes with operator new (00d833a6); when the allocation succeeds it calls the default constructor 014fa350 and returns the object… |
| `014fa4f0` | CommandObject_WinGame::RegisterFactory | likely | Pushes the factory 014fa470 and the constant 0x182, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x182 (ret 8)… |

**CommandPacketInteractionStartWheel** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aad80` | CommandPacketInteractionStartWheel::CommandPacketInteractio… 🌐 | confirmed | Constructor: calls the command base constructor FUN_008a2820(0xb,0) (base opcode 26, sub-opcode 11) and installs CommandPacketInteractionStartWheel::vftable. |
| `009aceb0` | DeletingDtor_009aceb0 | likely | Scalar deleting destructor for the BaseCommandPacket-derived interaction-start-wheel packet: rewrites *this to BaseCommandPacket::vftable, then to BasePacket::… |

**DeckCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442450` | DeckCommand::RegisterFactory | likely | Pushes the constant 0x168 and the address 014423f0, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**ECommCommand_RedeemItemGrant** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01439c70` | ECommCommand_RedeemItemGrant::ECommCommand_RedeemItemGrant | confirmed | Constructor (ECX = object, returns this). Calls 014e3430 on the object first (base constructor), stores ECommCommand_RedeemItemGrant::vftable at +0, zeroes the… |
| `01439d30` | ECommCommand_RedeemItemGrant::TypeName | confirmed | Returns the address of the literal ECommCommand_RedeemItemGrant (mov eax, imm32; ret). |
| `01439d40` | ECommCommand_RedeemItemGrant::GetStaticDword | confirmed | Returns the dword at global DAT_01cbe340 (mov eax,[imm32]; ret). |
| `01439d50` | ECommCommand_RedeemItemGrant::SetStaticDword | confirmed | Stores its one stack argument into global DAT_01cbe340 and returns with ret 4 (mov eax,[esp+4]; mov [imm32],eax; ret 4). |
| `01439d70` | ECommCommand_RedeemItemGrant::~ECommCommand_RedeemItemGrant | confirmed | Destructor body (ECX = object). Stores ECommCommand_RedeemItemGrant::vftable at +0; if the pointer at +0x10 is non-null it calls its vtable slot 0 with flag 1 … |
| `01439f50` | ECommCommand_RedeemItemGrant::Clone | confirmed | Virtual slot [17] (+0x44) of ECommCommand_RedeemItemGrant (ECX = source). Allocates 0x24 bytes with operator new; if non-null constructs the object with 01439c… |
| `01439fc0` | ECommCommand_RedeemItemGrant::CopyFieldsTo | confirmed | Virtual slot [18] (+0x48) of ECommCommand_RedeemItemGrant (ECX = source, one stack argument = destination). Casts the destination from NetworkCommand to ECommC… |
| `0143a160` | ECommCommand_RedeemItemGrant::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Calls the destructor body 01439d70 on the object, calls _free on the object when bit 0 of the flag is set, and returns th… |
| `0143a0e0` | ECommCommand_RedeemItemGrant::Create | likely | Allocates 0x24 bytes with operator new (00d833a6); if non-null it constructs the command with 01439c70 and returns the object, else returns 0. |
| `0143a140` | ECommCommand_RedeemItemGrant::RegisterFactory | likely | Static initializer: pushes the factory 0143a0e0 and classID 0x1b9 (441), calls 013ce130 and then 013ce230 with the returned registry value in ECX. |
| `0143a4e0` | ECommCommand_RedeemItemGrant::AppendTypeName | likely | Virtual slot 20 of ECommCommand_RedeemItemGrant, ends 'ret 4'. Loads its first stack argument into ECX and calls FUN_01241650 with (literal 'Redeem Item Grant … |

**EventCommand** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01447280` | EventCommand::EventCommand | confirmed | Base constructor: calls 0140c090 for the base part, stores EventCommand::vftable, zeroes the dword at +0x4, and when 012d46a0 and 012d08e0 both report nonzero … |
| `01447330` | EventCommand::GetName | confirmed | Returns the address of the literal EventCommand (no callees). |
| `01447370` | EventCommand::~EventCommand | confirmed | Destructor body: stores EventCommand::vftable, keeps this in a stack local for the exception frame, and calls the base cleanup 0140c160. |
| `014473e0` | EventCommand::GetRoutingIds | confirmed | Virtual slot 21 (+0x54): writes 0x3b9aca05 (1000000005) to the first out pointer, this+0x4 to the second and 0 to the third. |
| `01447410` | EventCommand::CreateDefaultAndSink | confirmed | Virtual slot 17: allocates 8 bytes through 00d833a6, constructs an EventCommand base object there with 01447280 (result 0 on failure), then calls vtable slot 1… |
| `01447480` | EventCommand::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target (param_2) from NetworkCommand to EventCommand through 00d8d382; on failure reports the clone assert at line 0x3d thro… |
| `014474d0` | EventCommand::Deserialize | confirmed | Virtual slot 9 reader: calls vtable slot 4 (+0x10) for the header and version, then reads the 32-bit int at +0x4 through 013c24d0, then calls vtable slot 5 (+0… |
| `01447510` | EventCommand::Serialize | confirmed | Virtual slot 10 writer (stack argument = writer): calls vtable slot 2 (+0x8), writes the 32-bit int at +0x4 through 013c23f0, then calls vtable slot 3 (+0xc). |
| `014475c0` | EventCommand::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0) with the body inlined: stores EventCommand::vftable, calls the base cleanup 0140c160, then calls _free on this when … |
| `01447620` | EventCommand::TraceFields | confirmed | Virtual slot 19: appends the label Event Command with newline through 01241650, calls 0140c290 on the stream argument, formats AccountID as a decimal from the … |
| `014476b0` | EventCommand::TraceName | confirmed | Virtual slot 20: appends the 13-byte text Event Command through 01241650. |
| `01447340` | EventCommand::GetSharedDword | likely | Returns the 32-bit global DAT_01cbe48c. |
| `01447350` | EventCommand::SetSharedDword | likely | Stores its 32-bit argument into the global DAT_01cbe48c. |
| `01447540` | EventCommand::CreateDefault | likely | Allocates 8 bytes through 00d833a6 and constructs an EventCommand base object there with 01447280; returns it, or 0 when the allocation fails. |

**EventCommand_ForceIntoEvent** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01535a70` | EventCommand_ForceIntoEvent::EventCommand_ForceIntoEvent | confirmed | Fastcall constructor: calls the EventCommand base constructor 01447280, stores EventCommand_ForceIntoEvent::vftable at this, sets the dword at this+8 to 0 and … |
| `01535af0` | EventCommand_ForceIntoEvent::TypeName | confirmed | Virtual slot [11]: takes no arguments and returns a pointer to the literal string 'EventCommand_ForceIntoEvent'. |
| `01535bd0` | EventCommand_ForceIntoEvent::Clone | confirmed | Virtual slot [17] (fastcall): allocates 0x10 bytes with operator new (00d833a6); if the allocation is non-null it constructs the object with 01535a70; then cal… |
| `01535c40` | EventCommand_ForceIntoEvent::CopyFieldsTo | confirmed | Virtual slot [18]: dynamic_casts the target (param_2) from NetworkCommand to EventCommand_ForceIntoEvent via 00d8d382; if the cast fails it calls 012f5a60 with… |
| `01535ca0` | EventCommand_ForceIntoEvent::Read 🌐 | confirmed | Virtual slot [9] reader (thiscall, reader context as argument): calls slot [4] (+0x10) to read the header, which writes the class version back into the argumen… |
| `01535d20` | EventCommand_ForceIntoEvent::Write 🌐 | confirmed | Virtual slot [10] writer (thiscall, writer as argument): calls slot [2] (+0x8, header write), EventCommand::Serialize (01447510) for the base part, appends the… |
| `01535de0` | EventCommand_ForceIntoEvent::~EventCommand_ForceIntoEvent | confirmed | Scalar deleting destructor (vtable slot [0]): stores EventCommand_ForceIntoEvent::vftable, calls EventCommand::~EventCommand (01447370), then calls _free(this)… |
| `01535e40` | EventCommand_ForceIntoEvent::Dump | confirmed | Virtual slot [19] trace dump (thiscall, output string as stack argument): appends 'Request Timed Deck Selection Event Command' plus newline (0x2b bytes) to the… |
| `01535e70` | EventCommand_ForceIntoEvent::DisplayName | confirmed | Virtual slot [20]: appends 'Request Timed Deck Selection Event Command' (0x2a bytes) to the std::string whose pointer is the first stack argument, then returns… |
| `01535b30` | EventCommand_ForceIntoEvent::~EventCommand_ForceIntoEvent | likely | Non-deleting destructor body (fastcall): stores EventCommand_ForceIntoEvent::vftable at this, keeps this in an EH frame local, calls EventCommand::~EventComman… |
| `01535d60` | EventCommand_ForceIntoEvent::Create | likely | Allocates 0x10 bytes with operator new (00d833a6); if the allocation is non-null, constructs the object with 01535a70 and returns it; otherwise returns 0. |

**EventCommand_RequestCurrentSessionInfo** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144a540` | EventCommand_RequestCurrentSessionInfo::EventCommand_Reques… | confirmed | Constructor (ECX=this): calls the base constructor 01447280 under an EH frame, stores EventCommand_RequestCurrentSessionInfo::vftable at +0, returns this. The … |
| `0144a5c0` | EventCommand_RequestCurrentSessionInfo::TypeName | confirmed | Virtual slot [11]: returns the address of the literal EventCommand_RequestCurrentSessionInfo (mov eax,018eb464; ret). No arguments. |
| `0144a5d0` | EventCommand_RequestCurrentSessionInfo::ReadGlobal01cbe4c8 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4c8 (mov eax,[01cbe4c8]; ret). No arguments. |
| `0144a5e0` | EventCommand_RequestCurrentSessionInfo::WriteGlobal01cbe4c8 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4c8 (mov eax,[esp+4]; mov [01cbe4c8],eax; ret 4). |
| `0144a600` | EventCommand_RequestCurrentSessionInfo::~EventCommand_Reque… | confirmed | Destructor body (ECX=this): stores EventCommand_RequestCurrentSessionInfo::vftable at +0 and calls the base destructor 01447370. It does not free the object. |
| `0144a660` | EventCommand_RequestCurrentSessionInfo::Clone | confirmed | Virtual slot [17] (Clone): allocates 8 bytes with operator new (00d833a6); when the allocation is non-null constructs the copy with 0144a540; then calls this v… |
| `0144a6d0` | EventCommand_RequestCurrentSessionInfo::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast (00d8d382) of its argument from NetworkCommand to EventCommand_RequestCurrentSessionInfo; on failure calls 012f5… |
| `0144a720` | EventCommand_RequestCurrentSessionInfo::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] of this with the stream and an out version word; when that succeeds calls base read 014474d0; when that suc… |
| `0144a760` | EventCommand_RequestCurrentSessionInfo::Write | confirmed | Virtual slot [10] (Write): calls header write slot [2] of this with the stream, then base write 01447510, then body-end write slot [3]. Writes no payload field… |
| `0144a810` | EventCommand_RequestCurrentSessionInfo::ScalarDeletingDtor | confirmed | Vtable slot [0] of EventCommand_RequestCurrentSessionInfo: stores the vtable at +0, calls base destructor 01447370, and when bit 0 of the flag argument is set … |
| `0144a870` | EventCommand_RequestCurrentSessionInfo::Dump | confirmed | Virtual slot [19] (Dump): appends the literal Request Current Session Info Event Command with a newline (43 characters) to its std::string argument through 012… |
| `0144a8a0` | EventCommand_RequestCurrentSessionInfo::DisplayName | confirmed | Virtual slot [20]: appends the literal Request Current Session Info Event Command (42 characters, no newline) to its std::string argument through 01241650. |
| `0144a790` | EventCommand_RequestCurrentSessionInfo::Create | likely | Factory: allocates 8 bytes with operator new (00d833a6); when the allocation is non-null constructs the command with 0144a540 and returns the object, otherwise… |
| `0144a7f0` | EventCommand_RequestCurrentSessionInfo::RegisterFactory | likely | Registers the factory 0144a790 under classID 0x214: pushes the factory address and 0x214 and calls 013ce130, then moves its eax result into ECX and calls 013ce… |

**EventCommand_RequestDuelistStats** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152f2e0` | EventCommand_RequestDuelistStats::Ctor | confirmed | Runs base constructor FUN_01447280 and stores EventCommand_RequestDuelistStats::vftable at +0. The class adds no fields. Returns this. |
| `0152f360` | EventCommand_RequestDuelistStats::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RequestDuelistStats (mov eax, 0x019154ac; ret). |
| `0152f3a0` | EventCommand_RequestDuelistStats::DtorBody | confirmed | Stores EventCommand_RequestDuelistStats::vftable into *this and calls the base destructor body FUN_01447370. It owns no heap fields. |
| `0152f400` | EventCommand_RequestDuelistStats::Clone | confirmed | Virtual slot [17]. Allocates 8 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152f2e0 and uses it, otherwise 0. Calls this object's slot [18]… |
| `0152f470` | EventCommand_RequestDuelistStats::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandRequestDuelistS… |
| `0152f4c0` | EventCommand_RequestDuelistStats::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0 (EventCommand::vftable slot [9]), then calls slot [5] and returns its resul… |
| `0152f500` | EventCommand_RequestDuelistStats::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510 (EventCommand::vftable slot [10]) with the stream, then slot [3]. The class wri… |
| `0152f5b0` | EventCommand_RequestDuelistStats::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores the class vtable into *this, calls the base destructor body FUN_01447370, then calls _free(this) when bit 0 of the flag argument is se… |
| `0152f610` | EventCommand_RequestDuelistStats::Dump | confirmed | Virtual slot [19]. Appends Request Player Season Stats Command plus newline (count 0x24) through FUN_01241650, then calls the base dump FUN_01447620. The title… |
| `0152f640` | EventCommand_RequestDuelistStats::DumpTitle | confirmed | Virtual slot [20]. Appends the 35-byte text Request Player Season Stats Command (count 0x23) through FUN_01241650. |
| `0152f530` | EventCommand_RequestDuelistStats::Create | likely | Allocates 8 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152f2e0 and returns it. |
| `0152f590` | EventCommand_RequestDuelistStats::RegisterFactory 🌐 | likely | Registers factory 0152f530 under key 0x20f (527) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_RequestEventLeaderBoard** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144add0` | EventCommand_RequestEventLeaderBoard::EventCommand_RequestE… | confirmed | Constructor (ECX=this): calls the base constructor 01447280 under an EH frame, stores EventCommand_RequestEventLeaderBoard::vftable at +0, zeroes the int at +0… |
| `0144ae50` | EventCommand_RequestEventLeaderBoard::TypeName | confirmed | Virtual slot [11]: returns the address of the literal EventCommand_RequestEventLeaderBoard (mov eax,imm; ret). No arguments. |
| `0144ae60` | EventCommand_RequestEventLeaderBoard::ReadGlobal01cbe4d0 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4d0. No arguments. |
| `0144ae70` | EventCommand_RequestEventLeaderBoard::WriteGlobal01cbe4d0 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4d0 and returns (ret 4). |
| `0144ae90` | EventCommand_RequestEventLeaderBoard::~EventCommand_Request… | confirmed | Destructor body (ECX=this): stores EventCommand_RequestEventLeaderBoard::vftable at +0 and calls the base destructor 01447370. It does not free the object. |
| `0144af10` | EventCommand_RequestEventLeaderBoard::Clone | confirmed | Virtual slot [17] (Clone): allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the copy with 0144add0; then calls this… |
| `0144af80` | EventCommand_RequestEventLeaderBoard::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast (00d8d382) of its argument from NetworkCommand to EventCommand_RequestEventLeaderBoard; on failure calls 012f5a6… |
| `0144afd0` | EventCommand_RequestEventLeaderBoard::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] with the stream and an out version word; on success calls base read 014474d0; when the version word is grea… |
| `0144b030` | EventCommand_RequestEventLeaderBoard::Write | confirmed | Virtual slot [10] (Write): calls header write slot [2] with the stream, then base write 01447510, then writes the int at +0x8 with 013c23f0 (SerialAppendIntege… |
| `0144b0f0` | EventCommand_RequestEventLeaderBoard::ScalarDeletingDtor | confirmed | Vtable slot [0] of EventCommand_RequestEventLeaderBoard: stores the vtable at +0, calls base destructor 01447370, and when bit 0 of the flag argument is set fr… |
| `0144b150` | EventCommand_RequestEventLeaderBoard::Dump | confirmed | Virtual slot [19] (Dump): appends the literal Request Event Leader Board Event Command with a newline (41 characters) to its std::string argument through 01241… |
| `0144b180` | EventCommand_RequestEventLeaderBoard::DisplayName | confirmed | Virtual slot [20]: appends the literal Request Event Leader Board Event Command (40 characters, no newline) to its std::string argument through 01241650. |
| `0144b070` | EventCommand_RequestEventLeaderBoard::Create | likely | Factory: allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the command with 0144add0 and returns it, otherwise retur… |
| `0144b0d0` | EventCommand_RequestEventLeaderBoard::RegisterFactory | likely | Registers the factory 0144b070 under classID 0x203: pushes the factory address and 0x203 and calls 013ce130, then moves its eax result into ECX and calls 013ce… |

**EventCommand_RequestEventSessionInfo** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144b1a0` | EventCommand_RequestEventSessionInfo::EventCommand_RequestE… | confirmed | Constructor (ECX=this): calls the base constructor 01447280 under an EH frame, stores EventCommand_RequestEventSessionInfo::vftable at +0, returns this. It doe… |
| `0144b220` | EventCommand_RequestEventSessionInfo::TypeName | confirmed | Virtual slot [11]: returns the address of the literal EventCommand_RequestEventSessionInfo (mov eax,imm; ret). No arguments. |
| `0144b230` | EventCommand_RequestEventSessionInfo::ReadGlobal01cbe4d4 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4d4. No arguments. |
| `0144b240` | EventCommand_RequestEventSessionInfo::WriteGlobal01cbe4d4 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4d4 and returns (ret 4). |
| `0144b260` | EventCommand_RequestEventSessionInfo::~EventCommand_Request… | confirmed | Destructor body (ECX=this): stores EventCommand_RequestEventSessionInfo::vftable at +0 and calls the base destructor 01447370. It does not free the object. |
| `0144b2e0` | EventCommand_RequestEventSessionInfo::Clone | confirmed | Virtual slot [17] (Clone): allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the copy with 0144b1a0; then calls this… |
| `0144b350` | EventCommand_RequestEventSessionInfo::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast (00d8d382) of its argument from NetworkCommand to EventCommand_RequestEventSessionInfo; on failure calls 012f5a6… |
| `0144b3a0` | EventCommand_RequestEventSessionInfo::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] with the stream and an out version word; on success calls base read 014474d0; when the version word is grea… |
| `0144b400` | EventCommand_RequestEventSessionInfo::Write | confirmed | Virtual slot [10] (Write): calls header write slot [2] with the stream, then base write 01447510, then writes the int at +0x8 with 013c23f0 (SerialAppendIntege… |
| `0144b4c0` | EventCommand_RequestEventSessionInfo::ScalarDeletingDtor | confirmed | Vtable slot [0] of EventCommand_RequestEventSessionInfo: stores the vtable at +0, calls base destructor 01447370, and when bit 0 of the flag argument is set fr… |
| `0144b520` | EventCommand_RequestEventSessionInfo::Dump | confirmed | Virtual slot [19] (Dump): appends the literal Request Event Session Info Event Command with a newline (41 characters) to its std::string argument through 01241… |
| `0144b550` | EventCommand_RequestEventSessionInfo::DisplayName | confirmed | Virtual slot [20]: appends the literal Request Event Session Info Event Command (40 characters, no newline) to its std::string argument through 01241650. |
| `0144b440` | EventCommand_RequestEventSessionInfo::Create | likely | Factory: allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the command with 0144b1a0 and returns it, otherwise retur… |
| `0144b4a0` | EventCommand_RequestEventSessionInfo::RegisterFactory | likely | Registers the factory 0144b440 under classID 0x204: pushes the factory address and 0x204 and calls 013ce130, then moves its eax result into ECX and calls 013ce… |

**EventCommand_RequestEventSessionsOnDay** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144a8c0` | EventCommand_RequestEventSessionsOnDay::EventCommand_Reques… | confirmed | Constructor (ECX=this): calls the base constructor 01447280 under an EH frame, stores EventCommand_RequestEventSessionsOnDay::vftable at +0, zeroes the int fie… |
| `0144a940` | EventCommand_RequestEventSessionsOnDay::TypeName | confirmed | Virtual slot [11]: returns the address of the literal EventCommand_RequestEventSessionsOnDay (mov eax,imm; ret). No arguments. |
| `0144a950` | EventCommand_RequestEventSessionsOnDay::ReadGlobal01cbe4cc | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4cc. No arguments. |
| `0144a960` | EventCommand_RequestEventSessionsOnDay::WriteGlobal01cbe4cc | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4cc and returns (ret 4). |
| `0144a980` | EventCommand_RequestEventSessionsOnDay::~EventCommand_Reque… | confirmed | Destructor body (ECX=this): stores EventCommand_RequestEventSessionsOnDay::vftable at +0 and calls the base destructor 01447370. It does not free the object. |
| `0144aa40` | EventCommand_RequestEventSessionsOnDay::Clone | confirmed | Virtual slot [17] (Clone): allocates 0x14 bytes with operator new (00d833a6); when the allocation is non-null constructs the copy with 0144a8c0; then calls thi… |
| `0144aab0` | EventCommand_RequestEventSessionsOnDay::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast (00d8d382) of its argument from NetworkCommand to EventCommand_RequestEventSessionsOnDay; on failure calls 012f5… |
| `0144ab10` | EventCommand_RequestEventSessionsOnDay::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] with the stream and an out version word; on success calls base read 014474d0; then reads three ints with 01… |
| `0144ab80` | EventCommand_RequestEventSessionsOnDay::Write | confirmed | Virtual slot [10] (Write): calls header write slot [2] with the stream, then base write 01447510, then writes the three ints +0x8, +0xc and +0x10 with 013c23f0… |
| `0144ac50` | EventCommand_RequestEventSessionsOnDay::ScalarDeletingDtor | confirmed | Vtable slot [0] of EventCommand_RequestEventSessionsOnDay: stores the vtable at +0, calls base destructor 01447370, and when bit 0 of the flag argument is set … |
| `0144acb0` | EventCommand_RequestEventSessionsOnDay::Dump | confirmed | Virtual slot [19] (Dump): appends the literal Request Event Sessions On Day Event Command with a newline (44 characters) to its std::string argument, calls bas… |
| `0144adb0` | EventCommand_RequestEventSessionsOnDay::DisplayName | confirmed | Virtual slot [20]: appends the literal Request Event Sessions On Day Event Command (43 characters, no newline) to its std::string argument through 01241650. |
| `0144abd0` | EventCommand_RequestEventSessionsOnDay::Create | likely | Factory: allocates 0x14 bytes with operator new (00d833a6); when the allocation is non-null constructs the command with 0144a8c0 and returns it, otherwise retu… |
| `0144ac30` | EventCommand_RequestEventSessionsOnDay::RegisterFactory | likely | Registers the factory 0144abd0 under classID 0x216: pushes the factory address and 0x216 and calls 013ce130, then moves its eax result into ECX and calls 013ce… |

**EventCommand_RequestJoinMatchingQueue** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01446a80` | EventCommand_RequestJoinMatchingQueue::TraceFields | confirmed | Virtual slot 19: appends the 42-byte label Request Join Matching Queue Event Command with newline through 01241650, then calls the base dump 01447620 on the sa… |
| `01446ab0` | EventCommand_RequestJoinMatchingQueue::TraceName | confirmed | Virtual slot 20: appends the 41-byte text Request Join Matching Queue Event Command through 01241650. |
| `01446ad0` | EventCommand_RequestJoinMatchingQueue::Serialize | confirmed | Virtual slot 10 writer (stack argument = writer): calls vtable slot 2 (+0x8), the base body writer 01447510, the string at +0x8 (013c2860), the 32-bit int at +… |
| `01446b50` | EventCommand_RequestJoinMatchingQueue::~EventCommand_Reques… | confirmed | Destructor body: stores EventCommand_RequestJoinMatchingQueue::vftable, releases the object at +0x24 through its vtable slot 0 with flag 1 when non-null, relea… |
| `01446c10` | EventCommand_RequestJoinMatchingQueue::GetName | confirmed | Returns the address of the literal EventCommand_RequestJoinMatchingQueue (no callees). |
| `01446c50` | EventCommand_RequestJoinMatchingQueue::EventCommand_Request… | confirmed | Default constructor (ECX = this): calls the base constructor 01447280, stores EventCommand_RequestJoinMatchingQueue::vftable, makes the string at +0x8 empty, s… |
| `01446d00` | EventCommand_RequestJoinMatchingQueue::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01446b50, then calls _free on this when bit 0 of the argument byte is set, and returns th… |
| `01446d20` | EventCommand_RequestJoinMatchingQueue::CreateDefaultAndSink | confirmed | Virtual slot 17: allocates 0x38 bytes through 00d833a6, constructs a default object there with 01446c50 (result 0 on allocation failure), then calls vtable slo… |
| `01446f70` | EventCommand_RequestJoinMatchingQueue::Deserialize | confirmed | Virtual slot 9 reader (stack argument = reader): calls vtable slot 4 (+0x10) for the header and version, then the base body 014474d0, the string at +0x8 (013c2… |
| `01447090` | EventCommand_RequestJoinMatchingQueue::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target (param_2) to EventCommand_RequestJoinMatchingQueue through 00d8d382; on failure reports the clone label with the file… |
| `01446c20` | EventCommand_RequestJoinMatchingQueue::GetSharedDword | likely | Returns the 32-bit global DAT_01cbe488. |
| `01446c30` | EventCommand_RequestJoinMatchingQueue::SetSharedDword | likely | Stores its 32-bit argument into the global DAT_01cbe488. |
| `01446d90` | EventCommand_RequestJoinMatchingQueue::CreateDefault | likely | Allocates 0x38 bytes through 00d833a6 and constructs a default object there with 01446c50; returns the new object, or 0 when the allocation fails. |

**EventCommand_RequestPlayerStats** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152e590` | EventCommand_RequestPlayerStats::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_RequestPlayerStats::vftable at +0 and zeroes the integers at +0x8 and +0xc. Returns this. |
| `0152e610` | EventCommand_RequestPlayerStats::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RequestPlayerStats (mov eax, 0x0191528c; ret). |
| `0152e650` | EventCommand_RequestPlayerStats::DtorBody | confirmed | Stores EventCommand_RequestPlayerStats::vftable into *this and calls the base destructor body FUN_01447370. It owns no heap fields. |
| `0152e6f0` | EventCommand_RequestPlayerStats::Clone | confirmed | Virtual slot [17]. Allocates 0x10 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152e590 and uses it, otherwise 0. Calls this object's slot [… |
| `0152e760` | EventCommand_RequestPlayerStats::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandRequestPlayerSt… |
| `0152e7c0` | EventCommand_RequestPlayerStats::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; reads integers into +0x8 and +0xc (FUN_013c24d0); then calls slot [5] and … |
| `0152e820` | EventCommand_RequestPlayerStats::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, integers from +0x8 and +0xc (FUN_013c23f0), then slot [3]. |
| `0152e8e0` | EventCommand_RequestPlayerStats::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores the class vtable into *this, calls the base destructor body FUN_01447370, then calls _free(this) when bit 0 of the flag argument is se… |
| `0152e940` | EventCommand_RequestPlayerStats::Dump | confirmed | Virtual slot [19]. Appends Request Player Season Stats Command plus newline (count 0x24) through FUN_01241650, then calls the base dump FUN_01447620. It writes… |
| `0152e970` | EventCommand_RequestPlayerStats::DumpTitle | confirmed | Virtual slot [20]. Appends the 35-byte text Request Player Season Stats Command (count 0x23) through FUN_01241650. |
| `0152e860` | EventCommand_RequestPlayerStats::Create | likely | Allocates 0x10 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152e590 and returns it. |
| `0152e8c0` | EventCommand_RequestPlayerStats::RegisterFactory 🌐 | likely | Registers factory 0152e860 under key 0x20d (525) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_RequestTimedDeckSelection** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152fd50` | EventCommand_RequestTimedDeckSelection::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; reads integers into +0x8 and +0xc (FUN_013c24d0) and the string at +0x10 (… |
| `0152fdc0` | EventCommand_RequestTimedDeckSelection::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, integers from +0x8 and +0xc (FUN_013c23f0), the string at +0x10 (FUN_013c2860)… |
| `0152fe10` | EventCommand_RequestTimedDeckSelection::DtorBody | confirmed | Stores EventCommand_RequestTimedDeckSelection::vftable into *this, releases the heap buffer of the string at +0x10 when its capacity (+0x28) exceeds 15, resets… |
| `0152feb0` | EventCommand_RequestTimedDeckSelection::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RequestTimedDeckSelection (mov eax, 0x01915694; ret). |
| `0152fef0` | EventCommand_RequestTimedDeckSelection::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152fe10, then _free(this) when bit 0 of the flag argument is set. Returns this. |

**EventCommand_RespondCurrentSessionInfo** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152cb40` | EventCommand_RespondCurrentSessionInfo::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_RespondCurrentSessionInfo::vftable at +0, zeroes the ints at +0x8, +0xc, +0x10 and +0x14, and clears th… |
| `0152cbd0` | EventCommand_RespondCurrentSessionInfo::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RespondCurrentSessionInfo (mov eax, 0x01914e04; ret). |
| `0152cc10` | EventCommand_RespondCurrentSessionInfo::DtorBody | confirmed | Stores EventCommand_RespondCurrentSessionInfo::vftable into *this and calls the base destructor body FUN_01447370. It owns no heap fields. |
| `0152cdc0` | EventCommand_RespondCurrentSessionInfo::Clone | confirmed | Virtual slot [17]. Allocates 0x1c bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152cb40 and uses it, otherwise 0. Calls this object's slot [… |
| `0152ce30` | EventCommand_RespondCurrentSessionInfo::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandRespondCurrentS… |
| `0152cea0` | EventCommand_RespondCurrentSessionInfo::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0 (EventCommand::vftable slot [9]); then reads integers into +0x8, +0xc, +0x1… |
| `0152cf40` | EventCommand_RespondCurrentSessionInfo::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, integers from +0x8, +0xc, +0x10, +0x14 (FUN_013c23f0), the byte at +0x18 as an… |
| `0152d020` | EventCommand_RespondCurrentSessionInfo::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores the class vtable into *this, calls the base destructor body FUN_01447370 inline (it does not call 0152cc10), then calls _free(this) wh… |
| `0152d080` | EventCommand_RespondCurrentSessionInfo::Dump | confirmed | Virtual slot [19]. Appends Respond Current Session Info Event Command plus newline (count 0x2b), calls the base dump FUN_01447620, then appends SessionID, Num … |
| `0152d1f0` | EventCommand_RespondCurrentSessionInfo::DumpTitle | confirmed | Virtual slot [20]. Appends the 42-byte text Respond Current Session Info Event Command (count 0x2a) through FUN_01241650. |
| `0152cfa0` | EventCommand_RespondCurrentSessionInfo::Create | likely | Allocates 0x1c bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152cb40 and returns it. |
| `0152d000` | EventCommand_RespondCurrentSessionInfo::RegisterFactory 🌐 | likely | Registers factory 0152cfa0 under key 0x215 (533) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_RespondDuelistStats** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152ea30` | EventCommand_RespondDuelistStats::Dump | confirmed | Virtual slot [19]. Appends the 30-byte text Respond Duelist Stats Command plus newline (count 0x1e) through FUN_01241650, then calls the base dump FUN_01447620… |
| `0152ea60` | EventCommand_RespondDuelistStats::DumpTitle | confirmed | Virtual slot [20]. Appends the 29-byte text Respond Duelist Stats Command (count 0x1d) through FUN_01241650. |
| `0152ecf0` | EventCommand_RespondDuelistStats::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, integers from +0x8, +0xc, +0x10 and +0x14 (FUN_013c23f0), the container at +0x… |
| `0152ed60` | EventCommand_RespondDuelistStats::DtorBody | confirmed | Stores EventCommand_RespondDuelistStats::vftable into *this, calls FUN_01247310 (container destructor for the entry container), then calls the base destructor … |
| `0152ede0` | EventCommand_RespondDuelistStats::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RespondDuelistStats (mov eax, 0x019153e4; ret). |
| `0152ee20` | EventCommand_RespondDuelistStats::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_RespondDuelistStats::vftable at +0, zeroes the integers at +0x8, +0xc, +0x10 and +0x14, and calls the c… |
| `0152ee90` | EventCommand_RespondDuelistStats::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152ed60, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152eeb0` | EventCommand_RespondDuelistStats::Clone | confirmed | Virtual slot [17]. Allocates 0x24 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152ee20 and uses it, otherwise 0. Calls this object's slot [… |
| `0152f180` | EventCommand_RespondDuelistStats::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; reads integers into +0x8, +0xc, +0x10 and +0x14 (FUN_013c24d0); then calls… |
| `0152f220` | EventCommand_RespondDuelistStats::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandRespondDuelistS… |
| `0152ef20` | EventCommand_RespondDuelistStats::Create | likely | Allocates 0x24 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152ee20 and returns it. |
| `0152ef80` | EventCommand_RespondDuelistStats::RegisterFactory 🌐 | likely | Registers factory 0152ef20 under key 0x210 (528) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_RespondEventSessionsOnDay** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152c540` | EventCommand_RespondEventSessionsOnDay::DtorBody | confirmed | Stores EventCommand_RespondEventSessionsOnDay::vftable into *this, frees the heap block at +0x18 when it is non-null, zeroes +0x18, +0x1c and +0x20, then calls… |
| `0152c5d0` | EventCommand_RespondEventSessionsOnDay::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RespondEventSessionsOnDay (mov eax, 0x01914cbc; ret). |
| `0152c610` | EventCommand_RespondEventSessionsOnDay::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510 (EventCommand::vftable slot [10]), integers from +0x8, +0xc and +0x10 (FUN_013c… |
| `0152c670` | EventCommand_RespondEventSessionsOnDay::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152c540, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152c690` | EventCommand_RespondEventSessionsOnDay::Dump | confirmed | Virtual slot [19]. Appends Respond Event Sessions On Day Event Command plus newline (count 0x2c), calls the base dump FUN_01447620, then formats Year, Month an… |
| `0152c7d0` | EventCommand_RespondEventSessionsOnDay::DumpTitle | confirmed | Virtual slot [20]. Appends the 43-byte text Respond Event Sessions On Day Event Command (count 0x2b) through FUN_01241650. |
| `0152c7f0` | EventCommand_RespondEventSessionsOnDay::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_RespondEventSessionsOnDay::vftable at +0, zeroes the ints at +0x8, +0xc and +0x10 and the container wor… |
| `0152c900` | EventCommand_RespondEventSessionsOnDay::Clone | confirmed | Virtual slot [17]. Allocates 0x24 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152c7f0 and uses it, otherwise 0. Calls this object's slot [… |
| `0152c970` | EventCommand_RespondEventSessionsOnDay::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (file EventCommandRespondEventSessionsOnDay.cpp… |
| `0152cab0` | EventCommand_RespondEventSessionsOnDay::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0 (EventCommand::vftable slot [9]); then integers into +0x8, +0xc and +0x10 (… |
| `0152ca20` | EventCommand_RespondEventSessionsOnDay::Create | likely | Allocates 0x24 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152c7f0 and returns it. |
| `0152ca80` | EventCommand_RespondEventSessionsOnDay::RegisterFactory 🌐 | likely | Registers factory 0152ca20 under key 0x217 (535) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_RespondPlayerStats** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152dd20` | EventCommand_RespondPlayerStats::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_RespondPlayerStats::vftable at +0, and zeroes the 14 words from +0x8 to +0x3c. Returns this. |
| `0152ddc0` | EventCommand_RespondPlayerStats::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_RespondPlayerStats (mov eax, 0x01915184; ret). |
| `0152de00` | EventCommand_RespondPlayerStats::DtorBody | confirmed | Stores EventCommand_RespondPlayerStats::vftable into *this and calls the base destructor body FUN_01447370. It owns no heap fields. |
| `0152e130` | EventCommand_RespondPlayerStats::Clone | confirmed | Virtual slot [17]. Allocates 0x40 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152dd20 and uses it, otherwise 0. Calls this object's slot [… |
| `0152e1a0` | EventCommand_RespondPlayerStats::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandRespondPlayerSt… |
| `0152e390` | EventCommand_RespondPlayerStats::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, then writes 13 integers from +0x8 to +0x3c (FUN_013c23f0) and finally the inte… |
| `0152e4e0` | EventCommand_RespondPlayerStats::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores the class vtable into *this, calls the base destructor body FUN_01447370, then calls _free(this) when bit 0 of the flag argument is se… |
| `0152e540` | EventCommand_RespondPlayerStats::Dump | confirmed | Virtual slot [19]. Appends Respond Player Stats Command plus newline (count 0x1d) through FUN_01241650, then calls the base dump FUN_01447620. It writes none o… |
| `0152e570` | EventCommand_RespondPlayerStats::DumpTitle | confirmed | Virtual slot [20]. Appends the 28-byte text Respond Player Stats Command (count 0x1c) through FUN_01241650. |
| `0152e250` | EventCommand_RespondPlayerStats::Deserialize | likely | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; then reads 13 integers into +0x8, +0xc, +0x10, +0x14, +0x18, +0x1c, +0x24,… |
| `0152e460` | EventCommand_RespondPlayerStats::Create | likely | Allocates 0x40 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152dd20 and returns it. |
| `0152e4c0` | EventCommand_RespondPlayerStats::RegisterFactory 🌐 | likely | Registers factory 0152e460 under key 0x20e (526) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_SendByeNotification** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152f740` | EventCommand_SendByeNotification::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; reads integers into +0x8 and +0xc (FUN_013c24d0) and the string at +0x10 (… |
| `0152f7b0` | EventCommand_SendByeNotification::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, integers from +0x8 and +0xc (FUN_013c23f0), the string at +0x10 (FUN_013c2860)… |
| `0152f800` | EventCommand_SendByeNotification::DtorBody | confirmed | Stores EventCommand_SendByeNotification::vftable into *this, releases the heap buffer of the string at +0x10 when its capacity (+0x28) exceeds 15, resets it to… |
| `0152f8a0` | EventCommand_SendByeNotification::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_SendByeNotification (mov eax, 0x01915574; ret). |
| `0152f8e0` | EventCommand_SendByeNotification::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152f800, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152f970` | EventCommand_SendByeNotification::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_SendByeNotification::vftable at +0, zeroes the integers at +0x8 and +0xc, and assigns the empty literal… |
| `0152fa70` | EventCommand_SendByeNotification::Clone | confirmed | Virtual slot [17]. Allocates 0x2c bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152f970 and uses it, otherwise 0. Calls this object's slot [… |
| `0152fae0` | EventCommand_SendByeNotification::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandSendByeNotifica… |
| `0152fba0` | EventCommand_SendByeNotification::Dump | confirmed | Virtual slot [19]. Appends Request Timed Deck Selection Event Command plus newline (count 0x2b) through FUN_01241650, then calls the base dump FUN_01447620. It… |
| `0152fbd0` | EventCommand_SendByeNotification::DumpTitle | confirmed | Virtual slot [20]. Appends the 42-byte text Request Timed Deck Selection Event Command (count 0x2a) through FUN_01241650. |
| `0152fbf0` | EventCommand_SendByeNotification::Create | likely | Allocates 0x2c bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152f970 and returns it. |
| `0152fc50` | EventCommand_SendByeNotification::RegisterFactory 🌐 | likely | Registers factory 0152fbf0 under key 0x20b (523) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_SendDontPlay** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014476d0` | EventCommand_SendDontPlay::EventCommand_SendDontPlay | confirmed | Constructor (ECX = this): calls the base constructor 01447280, then stores EventCommand_SendDontPlay::vftable and returns this. |
| `01447750` | EventCommand_SendDontPlay::GetName | confirmed | Returns the address of the literal EventCommand_SendDontPlay (no callees). |
| `01447790` | EventCommand_SendDontPlay::~EventCommand_SendDontPlay | confirmed | Destructor body: stores EventCommand_SendDontPlay::vftable, keeps this in a stack local for the exception frame, and calls the base destructor 01447370. |
| `014477f0` | EventCommand_SendDontPlay::CreateDefaultAndSink | confirmed | Virtual slot 17: allocates through 00d833a6, constructs a default SendDontPlay with 014476d0 (result 0 when the allocation fails), then calls vtable slot 18 (+… |
| `01447860` | EventCommand_SendDontPlay::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts the target (param_1) to EventCommand_SendDontPlay through 00d8d382; on failure reports the clone label with file EventCommandSen… |
| `014478b0` | EventCommand_SendDontPlay::Deserialize | confirmed | Virtual slot 9 reader (stack argument = reader): calls vtable slot 4 (+0x10) for the header and version, then the base body reader 014474d0, then vtable slot 5… |
| `014478f0` | EventCommand_SendDontPlay::Serialize | confirmed | Virtual slot 10 writer (stack argument = writer): calls vtable slot 2 (+0x8), the base body writer 01447510, then vtable slot 3 (+0xc). The class has no fields… |
| `014479a0` | EventCommand_SendDontPlay::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores EventCommand_SendDontPlay::vftable, calls the base destructor 01447370, then calls _free on this when bit 0 … |
| `01447a00` | EventCommand_SendDontPlay::TraceFields | confirmed | Virtual slot 19: appends the 29-byte label Send Dont Play Event Command with newline through 01241650, then calls the base dump 01447620 on the same argument. |
| `01447a30` | EventCommand_SendDontPlay::TraceName | confirmed | Virtual slot 20: appends the 28-byte text Send Dont Play Event Command through 01241650. |
| `01447760` | EventCommand_SendDontPlay::GetSharedDword | likely | Returns the 32-bit global DAT_01cbe490. |
| `01447770` | EventCommand_SendDontPlay::SetSharedDword | likely | Stores its 32-bit argument into the global DAT_01cbe490. |
| `01447920` | EventCommand_SendDontPlay::CreateDefault | likely | Allocates through 00d833a6 and constructs a default SendDontPlay with 014476d0; returns the new object, or 0 when the allocation fails. |

**EventCommand_SendEventPlayerFlags** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152d2a0` | EventCommand_SendEventPlayerFlags::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510 (EventCommand::vftable slot [10]), the container at +0x8 with FUN_01351620, the… |
| `0152d2e0` | EventCommand_SendEventPlayerFlags::Dump | confirmed | Virtual slot [19]. Appends the 32-byte text Send Ready Status Event Command plus newline (count 0x20) through FUN_01241650, then calls the base dump FUN_014476… |
| `0152d360` | EventCommand_SendEventPlayerFlags::DumpTitle | confirmed | Virtual slot [20]. Appends the 31-byte text Send Ready Status Event Command (count 0x1f) through FUN_01241650. |
| `0152d430` | EventCommand_SendEventPlayerFlags::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_SendEventPlayerFlags (mov eax, 0x01914fe4; ret). |
| `0152d540` | EventCommand_SendEventPlayerFlags::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandSendEventPlayer… |
| `0152d5e0` | EventCommand_SendEventPlayerFlags::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0; resets the node list whose head is at +0xc (FUN_004ef7fa, head and size re… |
| `0152d660` | EventCommand_SendEventPlayerFlags::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_SendEventPlayerFlags::vftable at +0, runs the container initialiser FUN_005258fb, and initialises the n… |
| `0152d6f0` | EventCommand_SendEventPlayerFlags::ScalarDeletingDtor | confirmed | Virtual slot [0]. Calls the destructor body 0152d380, then _free(this) when bit 0 of the flag argument is set. Returns this. |
| `0152d770` | EventCommand_SendEventPlayerFlags::Clone | confirmed | Virtual slot [17]. Allocates 0x14 bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152d660 and uses it, otherwise 0. Calls this object's slot [… |
| `0152d210` | EventCommand_SendEventPlayerFlags::EncodeMessage 🌐 | likely | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero it. It writes type id 0xdd with FUN_012f8c70, a… |
| `0152d7e0` | EventCommand_SendEventPlayerFlags::Create | likely | Allocates 0x14 bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152d660 and returns it. |
| `0152d840` | EventCommand_SendEventPlayerFlags::RegisterFactory 🌐 | likely | Registers factory 0152d7e0 under key 0x212 (530) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_SendReadyStatus** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152d860` | EventCommand_SendReadyStatus::Ctor | confirmed | Runs base constructor FUN_01447280, stores EventCommand_SendReadyStatus::vftable at +0 and clears the Is Ready byte at +0x8. Returns this. |
| `0152d8e0` | EventCommand_SendReadyStatus::GetName | confirmed | Virtual slot [11]. Returns the address of the literal EventCommand_SendReadyStatus (mov eax, 0x019150ac; ret). |
| `0152d920` | EventCommand_SendReadyStatus::DtorBody | confirmed | Stores EventCommand_SendReadyStatus::vftable into *this and calls the base destructor body FUN_01447370. It owns no heap fields. |
| `0152da30` | EventCommand_SendReadyStatus::Clone | confirmed | Virtual slot [17]. Allocates 0xc bytes (FUN_00d833a6); if non-null constructs the block with FUN_0152d860 and uses it, otherwise 0. Calls this object's slot [1… |
| `0152daa0` | EventCommand_SendReadyStatus::CopyFieldsTo | confirmed | Virtual slot [18]. Dynamic-casts param_2 from NetworkCommand through FUN_00d8d382; asserts through FUN_012f5a60 (module clone, file EventCommandSendReadyStatus… |
| `0152daf0` | EventCommand_SendReadyStatus::Deserialize | confirmed | Virtual slot [9]. Calls slot [4] with the stream, then the base reader FUN_014474d0 (EventCommand::vftable slot [9]); reads an integer into a local with FUN_01… |
| `0152db50` | EventCommand_SendReadyStatus::Serialize | confirmed | Virtual slot [10]. Calls slot [2] with the stream, the base writer FUN_01447510, then the Is Ready byte at +0x8 as an integer (FUN_013c23f0), then slot [3]. |
| `0152dc10` | EventCommand_SendReadyStatus::ScalarDeletingDtor | confirmed | Virtual slot [0]. Stores the class vtable into *this, calls the base destructor body FUN_01447370, then calls _free(this) when bit 0 of the flag argument is se… |
| `0152dc70` | EventCommand_SendReadyStatus::Dump | confirmed | Virtual slot [19]. Appends Send Ready Status Event Command plus newline (count 0x20), calls the base dump FUN_01447620, then appends Is Ready: %d with the byte… |
| `0152dd00` | EventCommand_SendReadyStatus::DumpTitle | confirmed | Virtual slot [20]. Appends the 31-byte text Send Ready Status Event Command (count 0x1f) through FUN_01241650. |
| `0152d990` | EventCommand_SendReadyStatus::EncodeMessage 🌐 | likely | Virtual slot [15]. Allocates a 0x14-byte block (FUN_00d833a6); when that succeeds it calls FUN_012f9eb0 to zero it. It writes type id 0xdc with FUN_012f8c70, a… |
| `0152db90` | EventCommand_SendReadyStatus::Create | likely | Allocates 0xc bytes with FUN_00d833a6. If the allocation returns 0 it returns 0; otherwise it constructs the block with FUN_0152d860 and returns it. |
| `0152dbf0` | EventCommand_SendReadyStatus::RegisterFactory 🌐 | likely | Registers factory 0152db90 under key 0x211 (529) with FUN_013ce130 and then FUN_013ce230 using the same pair. Returns nothing. |

**EventCommand_SetReady** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01535960` | EventCommand_SetReady::~EventCommand_SetReady | confirmed | Scalar deleting destructor: stores EventCommand_SetReady::vftable at this, calls EventCommand::~EventCommand (01447370), then calls _free(this) only when bit 0… |
| `015359c0` | EventCommand_SetReady::Dump | confirmed | Virtual slot [19] trace dump: appends 'Set Ready Event Command' plus newline (0x18 bytes) to the output string, calls EventCommand::TraceFields (01447620) on t… |
| `01535a50` | EventCommand_SetReady::DisplayName | confirmed | Virtual slot [20]: appends the literal 'Set Ready Event Command' (0x17 bytes) to the std::string whose pointer is the first stack argument (loaded into ECX for… |

**Exception** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ef350` | Exception::Exception | confirmed | Constructor (fastcall, ECX = this): stores PersistentBase, then PersistentComponent, then Exception vtables into *this (Exception is the final value). Initiali… |
| `013ef410` | Exception::TypeName | confirmed | Virtual slot 11 (+0x2c): returns the C string constant at 0x018e34f8. |
| `013ef420` | Exception::GetStaticDword | confirmed | Virtual slot 12 (+0x30): returns the class-wide dword DAT_01cbe094. |
| `013ef430` | Exception::SetStaticDword | confirmed | Virtual slot 13 (+0x34): stores its one stack argument in the class-wide dword DAT_01cbe094; ret 4. |
| `013ef450` | Exception::~Exception | confirmed | Destructor body: stores the Exception vtable, frees each heap-backed string when its capacity exceeds 15 (the strings at +0x40, +0x20 and +4), resets those str… |
| `013ef5a0` | Exception::ScalarDeletingDtor | confirmed | Virtual slot 0 (+0x0) of Exception: calls the destructor body 013ef450, frees the object with _free when bit 0 of the flag argument is set, and returns this. |
| `013ef520` | Exception::Factory | likely | Allocates 100 bytes (0x64) with operator new (00d833a6); when the allocation is non-null it constructs Exception with 013ef350 and returns the object, otherwis… |

**FriendStatus** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143fa60` | FriendStatus::RegisterFactory | likely | Pushes the constant 0x1f9 and the address 0143fa00 as two stack arguments and calls 013ce130; moves its EAX into ECX and calls 013ce230. Returns with no value. |

**GameCommand** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141afd0` | GameCommand::RoutingTriple 🌐 | confirmed | Virtual slot 21 (routing triple) with three stack pointers (ret 0xc): writes 0x3b9aca01 to *arg1, 0 to *arg2, and this+0xc to *arg3. |
| `0141b240` | GameCommand::Dump | confirmed | Virtual slot 19 (Dump): calls 0140c290 (empty stub), then formats 'GameID: %d' from this+0xc and 'PlayerID: %d' from this+4 with _sprintf and appends each to t… |
| `0141b2f0` | GameCommand::AppendDisplayName | confirmed | Virtual slot 20 (display name) with one stack argument (ret 4): calls std::string::append("Game Command", 12) on the string object passed as its argument; igno… |
| `0141b310` | GameCommand::~GameCommand | confirmed | Dtor body: sets the GameCommand vtable at this, calls 00521a9c for member cleanup, then 0140c160 (base dtor). |
| `0141b390` | GameCommand::TypeName | confirmed | Virtual slot 11 (type name): returns the address of the literal "GameCommand" (mov eax,0x018e6784; ret). |
| `0141b480` | GameCommand::CopyFieldsTo | confirmed | Virtual slot 18: dynamic_cast of the arg to GameCommand (00d8d382, NetworkCommand -> GameCommand RTTI); asserts gameCommand (GameCommand.cpp line 0x5b) if null… |
| `0141b520` | GameCommand::Read | confirmed | Virtual slot 9 (Read), thiscall with one stack arg (stream). Calls slot 4 header read (the version is written to the arg slot), 0140c2a0 on the stream, 013c24d… |
| `0141b5d0` | GameCommand::GameCommand | confirmed | Constructor: calls 0140c090 and 005258fb, stores the GameCommand::vftable pointer at this, then zeroes dwords this+4 through this+0x14. |
| `0141b650` | GameCommand::DeletingDtor | confirmed | Scalar deleting destructor: calls the dtor body 0141b310, then _free(this) when flag bit 0 of the stack arg is set; returns this. |
| `0141b670` | GameCommand::Clone | confirmed | Virtual slot 17 (Clone), fastcall ECX=the object. Allocates 0x24 bytes via operator new 00d833a6; when non-null constructs the copy with 0141b5d0; then calls v… |
| `0141b020` | GameCommand::Handle | likely | Virtual slot 15 (client handler), fastcall ECX=this. Asserts this+8 (mGame) non-null via 012f5a60 (GameCommand.cpp line 0x1c). If the byte at this+0x14d is 0, … |
| `0141b3a0` | GameCommand::GetStaticDword | likely | Virtual slot 12: returns the global dword DAT_01cbe268 (mov eax,[01cbe268]; ret). |
| `0141b3b0` | GameCommand::SetStaticDword | likely | Virtual slot 13: stores its stack argument into the global DAT_01cbe268 (ret 4). |

**GameCommand_ActionPlayed** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e1da0` | GameCommand_ActionPlayed::~GameCommand_ActionPlayed | confirmed | Fastcall destructor body (non-deleting). Stores the ActionPlayed vftable into *this, then calls base destructor 0141b310 with this. |
| `014e1e10` | GameCommand_ActionPlayed::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_ActionPlayed" (mov eax, imm32; ret). |
| `014e1fb0` | GameCommand_ActionPlayed::Read | confirmed | Thiscall virtual (slot 9 override of base Read 0141b520). Saves param_2; calls virtual slot 4 (vtable +0x10) with (param_2, &param_2) (the stack copy receives … |
| `014e2100` | GameCommand_ActionPlayed::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting GameCommand_ActionPlayed " (no-op, lead-verified); … |
| `014e22b0` | GameCommand_ActionPlayed::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_ActionPlayed (00d8d382 with NetworkCommand and Ac… |
| `014e2490` | GameCommand_ActionPlayed::GameCommand_ActionPlayed | confirmed | Fastcall default constructor. Calls base ctor 0141b5d0 on this, stores the ActionPlayed vftable at +0, zeroes the dwords at +0x24 through +0x50 (twelve dwords,… |
| `014e2510` | GameCommand_ActionPlayed::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the ActionPlayed vftable into *this, calls base destructor 0141b310 with this, then if bit 0 of the flag arg… |
| `014e2570` | GameCommand_ActionPlayed::GameCommand_ActionPlayed_Arg | confirmed | Thiscall constructor with one 4-byte stack argument: calls base ctor 0141b5d0, stores the ActionPlayed vftable, zeroes +0x24, +0x28, +0x2c, +0x30, +0x34, +0x38… |
| `014e2700` | GameCommand_ActionPlayed::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Action Played Game Command" (len 0x1b), then base text routine 0141b240(param_2), then formats and emits lines… |
| `014e1e20` | GameCommand_ActionPlayed::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8bc (mov eax,[01cbe8bc]; ret). No this pointer is read. |
| `014e1e30` | GameCommand_ActionPlayed::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8bc (mov [01cbe8bc],eax; ret 4). No this pointer is read. |
| `014e2350` | GameCommand_ActionPlayed::ExecuteOnGame | likely | Fastcall virtual (slot 15). If this+8 is null, asserts "mGame" (GameCommandActionPlayed.cpp, line 0x127). Calls base 0141b020. Resolves the origin via 01371870… |
| `014e2610` | GameCommand_ActionPlayed::Clone | likely | Fastcall virtual (slot 17). Allocates 0x5c bytes with operator new (00d833a6); if non-null constructs the new object with 014e2570 using the dword at this+8 as… |
| `014e2680` | GameCommand_ActionPlayed::Create | likely | Non-member function. Allocates 0x5c bytes with operator new (00d833a6); if non-null calls 014e2490 (default ctor) with the block in ECX and returns its EAX; if… |

**GameCommand_BatchControl** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e18d0` | GameCommand_BatchControl::GameCommand_BatchControl | confirmed | Fastcall default constructor. Calls base ctor 0141b5d0 on this, stores the GameCommand_BatchControl vftable at +0, sets the dword at +0x24 to 0xffffffff, retur… |
| `014e1950` | GameCommand_BatchControl::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_BatchControl" (mov eax, imm32; ret). |
| `014e1990` | GameCommand_BatchControl::GameCommand_BatchControl_Arg | confirmed | Thiscall constructor with one 4-byte stack argument (ret 4): calls base ctor 0141b5d0, stores the BatchControl vftable, zeroes nothing but sets +0x24 to 0xffff… |
| `014e1a00` | GameCommand_BatchControl::~GameCommand_BatchControl | confirmed | Fastcall destructor body (non-deleting). Stores the BatchControl vftable into *this, then calls base destructor 0141b310 with this. |
| `014e1b30` | GameCommand_BatchControl::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_BatchControl (00d8d382 with NetworkCommand and Ba… |
| `014e1b80` | GameCommand_BatchControl::Read | confirmed | Thiscall virtual (slot 9 override of base Read 0141b520). Calls virtual slot 4 (vtable +0x10) with (param_2, &param_2); if nonzero calls base read 0141b520 wit… |
| `014e1bd0` | GameCommand_BatchControl::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting GameCommand_BatchControl " (no-op, lead-verified); … |
| `014e1cb0` | GameCommand_BatchControl::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the BatchControl vftable into *this, calls base destructor 0141b310 with this, then if bit 0 of the flag arg… |
| `014e1d10` | GameCommand_BatchControl::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Batch Control Game Command" (len 0x1b), then base text routine 0141b240(param_2), then _sprintf of "Batch Cont… |
| `014e1960` | GameCommand_BatchControl::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8b8 (mov eax,[01cbe8b8]; ret). No this pointer is read. |
| `014e1970` | GameCommand_BatchControl::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8b8 (mov [01cbe8b8],eax; ret 4). No this pointer is read. |
| `014e1a60` | GameCommand_BatchControl::Execute | likely | Fastcall virtual (slot 15, returns 1). If this+8 is null, calls 012f5a60 with "mGame", the GameCommandBatchControl.cpp path and line 0x20. Calls base 0141b020,… |
| `014e1aa0` | GameCommand_BatchControl::Clone | likely | Fastcall virtual (slot 17). Asserts (mGame, GameCommandBatchControl.cpp, line 0x28) if this+8 is null. Allocates 0x28 bytes with operator new (00d833a6); if no… |
| `014e1c30` | GameCommand_BatchControl::Create | likely | Non-member function. Allocates 0x28 bytes with operator new (00d833a6); if non-null calls 014e18d0 (default ctor) with the block in ECX and returns its EAX; if… |

**GameCommand_BeginCardPlayed** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e13a0` | GameCommand_BeginCardPlayed::GameCommand_BeginCardPlayed | confirmed | Fastcall default constructor. Calls base ctor 0141b5d0 on this, stores the GameCommand_BeginCardPlayed vftable at +0, zeroes the dword at +0x24, returns this. |
| `014e1420` | GameCommand_BeginCardPlayed::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_BeginCardPlayed" (mov eax, imm32; ret). |
| `014e1460` | GameCommand_BeginCardPlayed::GameCommand_BeginCardPlayed_Arg | confirmed | Thiscall constructor with one 4-byte stack argument (ret 4): calls base ctor 0141b5d0 on this, stores the BeginCardPlayed vftable, passes the argument to base … |
| `014e14d0` | GameCommand_BeginCardPlayed::~GameCommand_BeginCardPlayed | confirmed | Fastcall destructor body (non-deleting). Stores the BeginCardPlayed vftable into *this, then calls base destructor 0141b310 with this. |
| `014e15a0` | GameCommand_BeginCardPlayed::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_BeginCardPlayed (00d8d382 with NetworkCommand and… |
| `014e15f0` | GameCommand_BeginCardPlayed::Read | confirmed | Thiscall virtual (slot 9 override of base Read 0141b520). Saves param_2; calls virtual slot 4 (vtable +0x10) with (param_2, &param_2); if nonzero calls base re… |
| `014e1640` | GameCommand_BeginCardPlayed::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting GameCommand_BeginCardPlayed " (no-op, lead-verified… |
| `014e1720` | GameCommand_BeginCardPlayed::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the BeginCardPlayed vftable into *this, calls base destructor 0141b310 with this, then if bit 0 of the flag … |
| `014e1780` | GameCommand_BeginCardPlayed::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Begin Card Played Game Command" (len 0x1f), then base text routine 0141b240(param_2), then _sprintf of "Card I… |
| `014e1430` | GameCommand_BeginCardPlayed::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8b4 (mov eax,[01cbe8b4]; ret). No this pointer is read. |
| `014e1440` | GameCommand_BeginCardPlayed::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8b4 (mov eax,[esp+4]; mov [01cbe8b4],eax; ret 4). No this pointer is read. |
| `014e1530` | GameCommand_BeginCardPlayed::Clone | likely | Fastcall virtual (slot 17). Allocates 0x28 bytes with operator new (00d833a6); if non-null constructs the new object with 014e1460 using the dword at this+8 as… |
| `014e16a0` | GameCommand_BeginCardPlayed::Create | likely | Non-member function. Allocates 0x28 bytes with operator new (00d833a6); if non-null calls 014e13a0 (default ctor) with the block in ECX and returns its EAX; if… |
| `014e1810` | GameCommand_BeginCardPlayed::ExecuteOnGame | likely | Fastcall virtual (slot 15, returns a byte). If this+8 is null, calls 012f5a60 with "mGame", the GameCommandBeginCardPlayed.cpp path and line 0x5b. Calls base 0… |

**GameCommand_ButtonPressed** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e0d20` | GameCommand_ButtonPressed::GameCommand_ButtonPressed | confirmed | Fastcall default constructor. Calls base ctor 0141b5d0 on this, stores the GameCommand_ButtonPressed vftable at +0, zeroes the dwords at +0x24, +0x28 and +0x2c… |
| `014e0da0` | GameCommand_ButtonPressed::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_ButtonPressed" (mov eax, imm32; ret). |
| `014e0de0` | GameCommand_ButtonPressed::GameCommand_ButtonPressed_FromObj | confirmed | Thiscall constructor with one stack argument (ret 4, object pointer param_2). Calls base ctor 0141b5d0, stores the ButtonPressed vftable, passes param_2 to bas… |
| `014e0e80` | GameCommand_ButtonPressed::~GameCommand_ButtonPressed | confirmed | Fastcall destructor body (non-deleting). Stores the ButtonPressed vftable into *this, then calls base destructor 0141b310 with this. |
| `014e0f50` | GameCommand_ButtonPressed::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_ButtonPressed (00d8d382 with NetworkCommand and G… |
| `014e0fb0` | GameCommand_ButtonPressed::Read | confirmed | Thiscall virtual (slot 9 override of base Read 0141b520). Saves param_2; calls virtual slot 4 (vtable +0x10) with (param_2, &param_2), which overwrites the sta… |
| `014e1030` | GameCommand_ButtonPressed::Write | confirmed | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting GameCommand_ButtonPressed\n" (no-op, lead-verified)… |
| `014e1240` | GameCommand_ButtonPressed::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the ButtonPressed vftable into *this, calls base destructor 0141b310 with this, then if bit 0 of the flag ar… |
| `014e12a0` | GameCommand_ButtonPressed::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Button Pressed Game Command" (len 0x1c), then base text routine 0141b240(param_2), then formats and emits thre… |
| `014e0db0` | GameCommand_ButtonPressed::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8b0 (mov eax,[01cbe8b0]; ret). No this pointer is read. |
| `014e0dc0` | GameCommand_ButtonPressed::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8b0 (mov eax,[esp+4]; mov [01cbe8b0],eax; ret 4). No this pointer is read. |
| `014e0ee0` | GameCommand_ButtonPressed::Clone | likely | Fastcall virtual (slot 17). Allocates 0x30 bytes with operator new (00d833a6); if non-null constructs the new object with 014e0de0 using the dword at this+8 as… |
| `014e10c0` | GameCommand_ButtonPressed::ExecuteOnGame | likely | Fastcall virtual (slot 15). If this+8 is null, calls 012f5a60 with "mGame" and the GameCommandButtonPressed.cpp path (line 0x76), then calls base 0141b020. Cal… |
| `014e11c0` | GameCommand_ButtonPressed::Create | likely | Non-member function. Allocates 0x30 bytes with operator new (00d833a6); if non-null calls 014e0d20 (default ctor) with the block in ECX and returns its EAX; if… |

**GameCommand_CanPrevent** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e0770` | GameCommand_CanPrevent::~GameCommand_CanPrevent | confirmed | Fastcall destructor body (non-deleting). Stores the CanPrevent vftable into *this; if this+0x28 is non-null calls _free on it; zeroes this+0x28, +0x2c and +0x3… |
| `014e0800` | GameCommand_CanPrevent::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_CanPrevent" (mov eax, imm32; ret). |
| `014e0840` | GameCommand_CanPrevent::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls the destructor body 014e0770 on this, then if bit 0 of the flag argument is set calls _free on this; returns … |
| `014e0860` | GameCommand_CanPrevent::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Can Prevent Game Command" (len 0x19), then base text routine 0141b240(param_2). Then walks the dword array who… |
| `014e0950` | GameCommand_CanPrevent::GameCommand_CanPrevent | confirmed | Calls base ctor 0141b5d0 on this (ECX), stores the GameCommand_CanPrevent vftable at +0, zeroes the dwords at +0x28, +0x2c and +0x30, returns this (fastcall). |
| `014e09b0` | GameCommand_CanPrevent::GameCommand_CanPrevent_Arg | confirmed | Thiscall constructor with one 4-byte stack argument (ret 4): calls base ctor 0141b5d0, stores the CanPrevent vftable, zeroes +0x28, +0x2c, +0x30, passes the ar… |
| `014e0bd0` | GameCommand_CanPrevent::CloneInto | confirmed | Thiscall virtual (slot 18, argument = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_CanPrevent (00d8d382 with NetworkCommand and Game… |
| `014e0cb0` | GameCommand_CanPrevent::Read | confirmed | Thiscall virtual (slot 9 override of the base Read 0141b520). Calls virtual slot 4 (vtable +0x10) with (param_2, &local); if nonzero calls base read 0141b520(p… |
| `014e0710` | GameCommand_CanPrevent::Write | likely | Thiscall virtual (slot 10 of GameCommand_CanPrevent). Calls virtual slot 2 (vtable +8) with the buffer; calls 013c26e0 with "Starting GameCommand_CanPrevent\n"… |
| `014e0810` | GameCommand_CanPrevent::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8ac (mov eax,[01cbe8ac]; ret). No this pointer is read. |
| `014e0820` | GameCommand_CanPrevent::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8ac (mov eax,[esp+4]; mov [01cbe8ac],eax; ret 4). No this pointer is read. |
| `014e0a90` | GameCommand_CanPrevent::ExecuteOnGame | likely | Fastcall virtual (slot 15). If this+8 is null, calls 012f5a60 with "mGame" and the GameCommandCanPrevent.cpp path (assert). Calls base 0141b020, then calls 013… |
| `014e0b60` | GameCommand_CanPrevent::Clone | likely | Fastcall virtual (slot 17). Allocates 0x34 bytes with operator new (00d833a6); if non-null, constructs the new object with 014e09b0 using the dword at this+8 a… |
| `014e0c30` | GameCommand_CanPrevent::Create | likely | Non-member function. Allocates 0x34 bytes with operator new (00d833a6); if non-null calls 014e0950 (default ctor) with the block in ECX and returns its EAX; if… |

**GameCommand_CardDeSelected** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e0260` | GameCommand_CardDeSelected::GameCommand_CardDeSelected | confirmed | Calls base-class ctor 0141b5d0 on this (ECX), stores the GameCommand_CardDeSelected vftable at +0, zeroes the dword at +0x24, and returns this (fastcall). |
| `014e02e0` | GameCommand_CardDeSelected::GetTypeName | confirmed | Returns the address of the constant string "GameCommand_CardDeSelected" (mov eax, imm32; ret). No arguments read. |
| `014e0320` | GameCommand_CardDeSelected::GameCommand_CardDeSelected_Arg | confirmed | Thiscall constructor with one 4-byte stack argument (ret 4): calls base ctor 0141b5d0, zeroes +0x24, stores the CardDeSelected vftable, passes the argument to … |
| `014e0390` | GameCommand_CardDeSelected::~GameCommand_CardDeSelected | confirmed | Stores the GameCommand_CardDeSelected vftable into *this, then calls base destructor 0141b310 with this (ECX). No flag test and no free: this is the non-deleti… |
| `014e0460` | GameCommand_CardDeSelected::CloneInto | confirmed | Thiscall virtual (slot 18, argument param_2 = destination NetworkCommand*). Dynamic-casts the argument to GameCommand_CardDeSelected (00d8d382 with NetworkComm… |
| `014e0620` | GameCommand_CardDeSelected::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores the CardDeSelected vftable into *this, calls base destructor 0141b310 with this, then if bit 0 of the flag a… |
| `014e0680` | GameCommand_CardDeSelected::DebugPrint | confirmed | Thiscall virtual (slot 19). Calls 01241650 with "Card DeSelected Game Command" (len 0x1d), then base text routine 0141b240(param_2), then _sprintf of "DeSelect… |
| `014e02f0` | GameCommand_CardDeSelected::GetStaticValue | likely | Returns the 32-bit global DAT_01cbe8a8 (mov eax,[01cbe8a8]; ret). Not per-instance: no this pointer is read. |
| `014e0300` | GameCommand_CardDeSelected::SetStaticValue | likely | Stores its one stack argument into the global DAT_01cbe8a8 (mov eax,[esp+4]; mov [01cbe8a8],eax; ret 4). Takes one argument, thiscall-style return (ret 4), no … |
| `014e03f0` | GameCommand_CardDeSelected::Clone | likely | Fastcall virtual (slot 17). Allocates 0x28 bytes with operator new (00d833a6); if non-null, constructs the new object with 014e0320 using the dword at this+8 a… |
| `014e04b0` | GameCommand_CardDeSelected::Read | likely | Thiscall virtual (slot 9 override of the base Read 0141b520). Calls virtual slot 4 of this with (param_2, &param_2); if that returns nonzero, calls base read 0… |
| `014e0500` | GameCommand_CardDeSelected::Write | likely | Thiscall virtual (slot 10). Calls virtual slot 2 (vtable +8) with the buffer argument; calls 013c26e0 with the text "Starting GameCommand_CardDeSelected\n" (a … |
| `014e0560` | GameCommand_CardDeSelected::ExecuteOnGame | likely | Fastcall virtual (slot 15). If the pointer at this+8 is null, calls 012f5a60 with the assert name "mGame" and the GameCommandCardDeSelected.cpp path, line 0x5b… |
| `014e05a0` | GameCommand_CardDeSelected::Create | likely | Non-member function (no this). Allocates 0x28 bytes with operator new (00d833a6); if the block is non-null, calls 014e0260 (default constructor) with the block… |

**GameCommand_CardPlayed** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dfbe0` | GameCommand_CardPlayed::~GameCommand_CardPlayed | confirmed | Non-deleting destructor body: stores GameCommand_CardPlayed::vftable into the object, stores the object pointer in a local, and calls base destructor body 0141… |
| `014dfc50` | GameCommand_CardPlayed::GetClassName | confirmed | Returns the address of the literal 'GameCommand_CardPlayed' (mov eax, imm32; ret). |
| `014dff20` | GameCommand_CardPlayed::GameCommand_CardPlayed | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_CardPlayed::vftable into the object, sets the dwords at +0… |
| `014dff90` | GameCommand_CardPlayed::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_CardPlayed::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _free… |
| `014dfff0` | GameCommand_CardPlayed::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_CardPlayed::vftable into the object, clears the… |
| `014e0160` | GameCommand_CardPlayed::DebugPrint | confirmed | Thiscall virtual (slot 19 of GameCommand_CardPlayed). Calls 01241650 with the header "Card Played Game Command" (len 0x19), then base-class text routine 0141b2… |
| `014dfd90` | GameCommand_CardPlayed::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_CardPlayed (RTTI descriptors 01bb94f8 and 01bbfdcc). If the… |
| `014dfdf0` | GameCommand_CardPlayed::ReadViaBase | likely | Vtable slot 9 override: calls its own vtable slot 4 with the argument and a pointer to it; if that returns true, calls base routine 0141b520 with the argument;… |
| `014dfe70` | GameCommand_CardPlayed::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_CardPlayed' line; calls base rout… |
| `014e0070` | GameCommand_CardPlayed::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x34; when non-null, constructs a new object with 014dfff0 using field +8. Then calls its own vtable slot 18 (offset 0… |

**GameCommand_CardSelected** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014df4b0` | GameCommand_CardSelected::~GameCommand_CardSelected | confirmed | Non-deleting destructor body: stores GameCommand_CardSelected::vftable into the object, stores the object pointer in a local, and calls base destructor body 01… |
| `014df520` | GameCommand_CardSelected::GetClassName | confirmed | Returns the address of the literal 'GameCommand_CardSelected' (mov eax, imm32; ret). |
| `014df850` | GameCommand_CardSelected::GameCommand_CardSelected | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_CardSelected::vftable into the object, sets the dwords at … |
| `014df8c0` | GameCommand_CardSelected::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_CardSelected::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _fr… |
| `014df920` | GameCommand_CardSelected::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_CardSelected::vftable into the object, clears t… |
| `014df5c0` | GameCommand_CardSelected::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_CardSelected (RTTI descriptors 01bb94f8 and 01bbfda0). If t… |
| `014df670` | GameCommand_CardSelected::ReadViaBase | likely | Vtable slot 9 override, decoded from bytes: calls its own vtable slot 4 with the argument and the address of a local dword (version); if true, calls base routi… |
| `014df710` | GameCommand_CardSelected::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_CardSelected' line; calls base ro… |
| `014df9b0` | GameCommand_CardSelected::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x38; when non-null, constructs a new object with 014df920 using field +8. Then calls its own vtable slot 18 (offset 0… |
| `014dfaa0` | GameCommand_CardSelected::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Card Selected Game Command' followed by a newline (27 bytes, pushed as length 0x1b) to the std::string passed as … |

**GameCommand_DiscardCard** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014deb50` | GameCommand_DiscardCard::GameCommand_DiscardCard | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_DiscardCard::vftable into the object, zeroes the dwords at… |
| `014debf0` | GameCommand_DiscardCard::GetClassName | confirmed | Returns the address of the literal 'GameCommand_DiscardCard' (mov eax, imm32; ret). |
| `014dec30` | GameCommand_DiscardCard::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_DiscardCard::vftable into the object, clears th… |
| `014decc0` | GameCommand_DiscardCard::~GameCommand_DiscardCard | confirmed | Non-deleting destructor body: stores GameCommand_DiscardCard::vftable into the object; when the capacity at +0x48 is above 15, frees the heap buffer at +0x34 v… |
| `014dee30` | GameCommand_DiscardCard::DeletingDtor | confirmed | Scalar deleting destructor, decoded from bytes: calls the destructor body 014decc0 with the object in ECX, and calls _free on the object when bit 0 of the flag… |
| `014de900` | GameCommand_DiscardCard::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_DiscardCard (RTTI descriptors 01bb94f8 and 01bbfd74). If th… |
| `014de970` | GameCommand_DiscardCard::ReadViaBase | likely | Vtable slot 9 override: sets a local pointer to the object; calls its own vtable slot 4 with the argument and the address of that local; if true, calls base ro… |
| `014dea00` | GameCommand_DiscardCard::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_DiscardCard' line; calls base rou… |
| `014deab0` | GameCommand_DiscardCard::SetupCard | likely | Non-virtual method on the object in ECX: if the argument is zero, asserts (file GameCommandDiscardCard.cpp, line 0x29 via 012f5a60). Stores the dword returned … |
| `014ded40` | GameCommand_DiscardCard::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x50; when non-null, constructs a new object with 014dec30 using field +8. Then calls its own vtable slot 18 (offset 0… |
| `014dee50` | GameCommand_DiscardCard::SetupParentCardName | likely | Non-virtual method on the object in ECX: std::string::assign (00420ff1) of the argument string into the object at this+0x30 with count 0xffffffff, then takes t… |
| `014dee90` | GameCommand_DiscardCard::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Discard Card Game Command' followed by a newline (26 bytes, pushed as length 0x1a) to the std::string passed as i… |
| `014def70` | GameCommand_DiscardCard::ApplyDiscard | likely | Vtable slot 15 override: asserts mGame (+8) and mPlayerID (+4) non-zero (GameCommandDiscardCard.cpp lines 0x40 and 0x41 via 012f5a60). Calls 0141b020, gets an … |

**GameCommand_DragStopped** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014de440` | GameCommand_DragStopped::GameCommand_DragStopped | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_DragStopped::vftable into the object, sets the dword at +0… |
| `014de4c0` | GameCommand_DragStopped::GetClassName | confirmed | Returns the address of the literal 'GameCommand_DragStopped' (mov eax, imm32; ret). |
| `014de500` | GameCommand_DragStopped::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_DragStopped::vftable into the object, calls 014… |
| `014de570` | GameCommand_DragStopped::~GameCommand_DragStopped | confirmed | Non-deleting destructor body: stores GameCommand_DragStopped::vftable into the object, stores the object pointer in a local, and calls base destructor body 014… |
| `014de800` | GameCommand_DragStopped::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_DragStopped::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _fre… |
| `014de5d0` | GameCommand_DragStopped::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x28; when non-null, constructs a new object with 014de500 using field +8. Then calls its own vtable slot 18 (offset 0… |
| `014de640` | GameCommand_DragStopped::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_DragStopped (RTTI descriptors 01bb94f8 and 01bbfd48). If th… |
| `014de690` | GameCommand_DragStopped::ReadViaBase | likely | Vtable slot 9 override: calls its own vtable slot 4 with the argument and a pointer to it; if that returns true, calls base routine 0141b520 with the argument;… |
| `014de6e0` | GameCommand_DragStopped::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_DragStopped' line; calls base rou… |
| `014de860` | GameCommand_DragStopped::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Drag Stopped Game Command' followed by a newline (26 bytes, pushed as length 0x1a) to the std::string passed as i… |

**GameCommand_EndGame** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ddb10` | GameCommand_EndGame::GameCommand_EndGame | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_EndGame::vftable into the object, sets the byte at offset … |
| `014ddb90` | GameCommand_EndGame::GetClassName | confirmed | Returns the address of the literal 'GameCommand_EndGame' (mov eax, imm32; ret). |
| `014ddbd0` | GameCommand_EndGame::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, sets the byte at +0x24 to 0, stores GameCommand_EndGame::vftable i… |
| `014ddc40` | GameCommand_EndGame::~GameCommand_EndGame | confirmed | Non-deleting destructor body: stores GameCommand_EndGame::vftable into the object, stores the object pointer in a local, and calls base destructor body 0141b31… |
| `014ddee0` | GameCommand_EndGame::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_EndGame::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _free on… |
| `014ddcc0` | GameCommand_EndGame::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandEndGame.cpp, line 0x2b via 012f5a60). Calls 00d833a6 with size 0x28; when non-null, … |
| `014ddd50` | GameCommand_EndGame::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_EndGame (RTTI descriptors 01bb94f8 and 01bbfcf0). If the re… |
| `014ddda0` | GameCommand_EndGame::ReadVersionedFlag | likely | Vtable slot 9 override, decoded from bytes (Ghidra's unaff_EDI and the 'param_1 > 2' test are artifacts; the compared value is the version dword at [esp+0x10])… |
| `014dde10` | GameCommand_EndGame::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_EndGame' line; calls base routine… |
| `014ddf40` | GameCommand_EndGame::AppendTitleThenBase | likely | Vtable slot 19 override: appends the literal 'End Game Game Command' followed by a newline (22 bytes, pushed as length 0x16) to the std::string passed as its a… |

**GameCommand_EndObserverMode** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ddf70` | GameCommand_EndObserverMode::GameCommand_EndObserverMode | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_EndObserverMode::vftable into the object, sets the dword a… |
| `014ddff0` | GameCommand_EndObserverMode::GetClassName | confirmed | Returns the address of the literal 'GameCommand_EndObserverMode' (mov eax, imm32; ret). |
| `014de030` | GameCommand_EndObserverMode::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_EndObserverMode::vftable into the object, calls… |
| `014de0a0` | GameCommand_EndObserverMode::~GameCommand_EndObserverMode | confirmed | Non-deleting destructor body: stores GameCommand_EndObserverMode::vftable into the object, stores the object pointer in a local, and calls base destructor body… |
| `014de350` | GameCommand_EndObserverMode::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_EndObserverMode::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls … |
| `014de110` | GameCommand_EndObserverMode::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x28; when non-null, constructs a new object with 014de030 using field +8. Then calls its own vtable slot 18 (offset 0… |
| `014de180` | GameCommand_EndObserverMode::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_EndObserverMode (RTTI descriptors 01bb94f8 and 01bbfd18). I… |
| `014de1d0` | GameCommand_EndObserverMode::ReadViaBase | likely | Vtable slot 9 override: sets a local pointer to the object; calls its own vtable slot 4 with the argument and the address of that local; if true, calls base ro… |
| `014de230` | GameCommand_EndObserverMode::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_EndObserverMode' line; calls base… |
| `014de3b0` | GameCommand_EndObserverMode::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Set Observer Mode Game Command' followed by a newline (31 bytes, pushed as length 0x1f) to the std::string passed… |

**GameCommand_FatalError** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01429e90` | GameCommand_FatalError::Write | confirmed | Calls header write (vtable slot 2) on the writer, passes trace text to FUN_013c26e0 (no-op returning 1), calls vtable slot 10 Write on the sub-object at this+0… |

**GameCommand_FinishCustomChoice** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dd590` | GameCommand_FinishCustomChoice::GameCommand_FinishCustomCho… | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_FinishCustomChoice::vftable into the object, sets the dwor… |
| `014dd610` | GameCommand_FinishCustomChoice::GetClassName | confirmed | Returns the address of the literal 'GameCommand_FinishCustomChoice' (mov eax, imm32; ret). |
| `014dd650` | GameCommand_FinishCustomChoice::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_FinishCustomChoice::vftable into the object, ca… |
| `014dd6d0` | GameCommand_FinishCustomChoice::~GameCommand_FinishCustomCh… | confirmed | Non-deleting destructor body: stores GameCommand_FinishCustomChoice::vftable into the object, stores the object pointer in a local, and calls base destructor b… |
| `014dda30` | GameCommand_FinishCustomChoice::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_FinishCustomChoice::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and cal… |
| `014dd740` | GameCommand_FinishCustomChoice::CheckCustomChoiceDialogThen… | likely | Vtable slot 15 override: if field +8 is zero, asserts 'mGame' in GameCommandFinishCustomChoice.cpp at line 0x2b via 012f5a60. Calls 0141b020, 01347e10 and 013f… |
| `014dd7d0` | GameCommand_FinishCustomChoice::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandFinishCustomChoice.cpp, line 0x3b via 012f5a60). Calls 00d833a6 with size 0x2c; when… |
| `014dd860` | GameCommand_FinishCustomChoice::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_FinishCustomChoice (RTTI descriptors 01bb94f8 and 01bbfcbc)… |
| `014dd8c0` | GameCommand_FinishCustomChoice::ReadViaBaseWithVersion | likely | Vtable slot 9 override, decoded from bytes (Ghidra's unaff_ESI is an artifact; the compared value is the dword at [esp+0xc]): calls own vtable slot 4 with the … |
| `014dd930` | GameCommand_FinishCustomChoice::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_FinishCustomChoice' line; calls b… |
| `014dda90` | GameCommand_FinishCustomChoice::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Finish Multiaction Game Command' followed by a newline (32 bytes, pushed as length 0x20) to the std::string passe… |

**GameCommand_FinishMultiaction** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dd050` | GameCommand_FinishMultiaction::GameCommand_FinishMultiaction | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_FinishMultiaction::vftable into the object, and returns th… |
| `014dd0d0` | GameCommand_FinishMultiaction::GetClassName | confirmed | Returns the address of the literal 'GameCommand_FinishMultiaction' (mov eax, imm32; ret). |
| `014dd110` | GameCommand_FinishMultiaction::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_FinishMultiaction::vftable into the object, cal… |
| `014dd180` | GameCommand_FinishMultiaction::~GameCommand_FinishMultiacti… | confirmed | Non-deleting destructor body: stores GameCommand_FinishMultiaction::vftable into the object, stores the object pointer in a local, and calls base destructor bo… |
| `014dd4b0` | GameCommand_FinishMultiaction::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_FinishMultiaction::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and call… |
| `014dd1d0` | GameCommand_FinishMultiaction::CheckGetTargetState | likely | Vtable slot 15 override: if field +8 is zero, asserts 'mGame' in GameCommandFinishMultiaction.cpp at line 0x1e via 012f5a60. Calls 0141b020, 01347e10 and 013f7… |
| `014dd280` | GameCommand_FinishMultiaction::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandFinishMultiaction.cpp, line 0x30 via 012f5a60). Calls 00d833a6 with size 0x28; when … |
| `014dd330` | GameCommand_FinishMultiaction::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_FinishMultiaction (RTTI descriptors 01bb94f8 and 01bbfc8c).… |
| `014dd380` | GameCommand_FinishMultiaction::ReadViaBase | likely | Vtable slot 9 override: calls its own vtable slot 4 with the argument and a pointer to it; if that returns true, calls base routine 0141b520 with the argument;… |
| `014dd3d0` | GameCommand_FinishMultiaction::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_FinishMultiaction' line; calls ba… |
| `014dd510` | GameCommand_FinishMultiaction::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Finish Multiaction Game Command' followed by a newline (32 bytes, pushed as length 0x20) to the std::string passe… |

**GameCommand_ForceEndGetTarget** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dcb50` | GameCommand_ForceEndGetTarget::GameCommand_ForceEndGetTarget | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_ForceEndGetTarget::vftable into the object, sets the dword… |
| `014dcbd0` | GameCommand_ForceEndGetTarget::GetClassName | confirmed | Returns the address of the literal 'GameCommand_ForceEndGetTarget' (mov eax, imm32; ret). |
| `014dcc10` | GameCommand_ForceEndGetTarget::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_ForceEndGetTarget::vftable into the object, cal… |
| `014dcc80` | GameCommand_ForceEndGetTarget::~GameCommand_ForceEndGetTarg… | confirmed | Non-deleting destructor body: stores GameCommand_ForceEndGetTarget::vftable into the object, stores the object pointer in a local, and calls base destructor bo… |
| `014dcf70` | GameCommand_ForceEndGetTarget::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_ForceEndGetTarget::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and call… |
| `014dcce0` | GameCommand_ForceEndGetTarget::CheckGetTargetStateThenCall | likely | Vtable slot 15 override: if field +8 is zero, asserts 'mGame' in GameCommandForceEndGetTarget.cpp at line 0x24 via 012f5a60. Calls 0141b020, 01347e10 and 013f7… |
| `014dcd60` | GameCommand_ForceEndGetTarget::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandForceEndGetTarget.cpp, line 0x34 via 012f5a60). Calls 00d833a6 with size 0x28; when … |
| `014dcdf0` | GameCommand_ForceEndGetTarget::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_ForceEndGetTarget (RTTI descriptors 01bb94f8 and 01bbfc5c).… |
| `014dce40` | GameCommand_ForceEndGetTarget::ReadViaBase | likely | Vtable slot 9 override: calls its own vtable slot 4 with the argument and a pointer to it; if that returns true, calls base routine 0141b520 with the argument;… |
| `014dce90` | GameCommand_ForceEndGetTarget::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_ForceEndGetTarget' line; calls ba… |
| `014dcfd0` | GameCommand_ForceEndGetTarget::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Force End GetTarget Game Command' followed by a newline (33 bytes, pushed as length 0x21) to the std::string pass… |

**GameCommand_GameOverReached** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dc740` | GameCommand_GameOverReached::GameCommand_GameOverReached | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_GameOverReached::vftable into the object, and returns the … |
| `014dc7c0` | GameCommand_GameOverReached::GetClassName | confirmed | Returns the address of the literal 'GameCommand_GameOverReached' (mov eax, imm32; ret). |
| `014dc800` | GameCommand_GameOverReached::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_GameOverReached::vftable into the object, then … |
| `014dc870` | GameCommand_GameOverReached::~GameCommand_GameOverReached | confirmed | Non-deleting destructor body: stores GameCommand_GameOverReached::vftable into the object, stores the object pointer in a local, and calls base destructor body… |
| `014dcac0` | GameCommand_GameOverReached::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_GameOverReached::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls … |
| `014dc8d0` | GameCommand_GameOverReached::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandGameOverReached.cpp, line 0x1f via 012f5a60). Calls 00d833a6 with size 0x24; when no… |
| `014dc960` | GameCommand_GameOverReached::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_GameOverReached (RTTI descriptors 01bb94f8 and 01bbfc2c). I… |
| `014dc9f0` | GameCommand_GameOverReached::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_GameOverReached' line; calls base… |
| `014dcb20` | GameCommand_GameOverReached::AppendTitleThenBase | likely | Vtable slot 19 override: appends the literal 'Game Over Reached Game Command' followed by a newline (31 bytes, pushed as length 0x1f) to the std::string passed… |

**GameCommand_IntroduceCard** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142eda0` | GameCommand_IntroduceCard::Read | confirmed | Header read (vtable slot 4) on the reader, base read FUN_0141b520, then three integers into +0x24, +0x28 and +0x2c, then the body-end read (slot 5). No version… |
| `0142ee10` | GameCommand_IntroduceCard::Write | confirmed | Header write (vtable slot 2), trace text through the no-op FUN_013c26e0, base write FUN_0141b1a0, then integers from +0x24, +0x28 and +0x2c with FUN_013c23f0, … |

**GameCommand_LostPlayer** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dc140` | GameCommand_LostPlayer::GameCommand_LostPlayer | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_LostPlayer::vftable into the object, sets the dword at off… |
| `014dc1c0` | GameCommand_LostPlayer::GetClassName | confirmed | Returns the address of the literal 'GameCommand_LostPlayer' (mov eax, imm32; ret). |
| `014dc280` | GameCommand_LostPlayer::~GameCommand_LostPlayer | confirmed | Non-deleting destructor body: stores GameCommand_LostPlayer::vftable into the object, stores the object pointer in a local, and calls base destructor body 0141… |
| `014dc4e0` | GameCommand_LostPlayer::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_LostPlayer::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _free… |
| `014dc200` | GameCommand_LostPlayer::ConstructWithArgAndLabel | likely | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, sets the dword at +0x24 to 0, stores GameCommand_LostPlayer::vftab… |
| `014dc2f0` | GameCommand_LostPlayer::Clone | likely | Vtable slot 17: calls 00d833a6 with size 0x28; when non-null, constructs a new object with 014dc200 using field +8. Then calls its own vtable slot 18 (offset 0… |
| `014dc360` | GameCommand_LostPlayer::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_LostPlayer (RTTI descriptors 01bb94f8 and 01bbfc00). If the… |
| `014dc3b0` | GameCommand_LostPlayer::ReadViaBase | likely | Vtable slot 9 override: calls its own vtable slot 4 with the argument and a pointer to it; if that returns true, calls base routine 0141b520 with the argument;… |
| `014dc400` | GameCommand_LostPlayer::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_LostPlayer' line; calls base rout… |
| `014dc6b0` | GameCommand_LostPlayer::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Lost Player Game Command' followed by a newline (25 bytes, pushed as length 0x19) to the std::string passed as it… |

**GameCommand_OfferAdjournment** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dbb40` | GameCommand_OfferAdjournment::GameCommand_OfferAdjournment | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_OfferAdjournment::vftable into the object, sets the dword … |
| `014dbbc0` | GameCommand_OfferAdjournment::GetClassName | confirmed | Returns the address of the literal 'GameCommand_OfferAdjournment' (mov eax, imm32; ret). |
| `014dbc00` | GameCommand_OfferAdjournment::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_OfferAdjournment::vftable into the object, call… |
| `014dbc70` | GameCommand_OfferAdjournment::~GameCommand_OfferAdjournment | confirmed | Non-deleting destructor body: stores GameCommand_OfferAdjournment::vftable into the object, stores the object pointer in a local, and calls base destructor bod… |
| `014dbf50` | GameCommand_OfferAdjournment::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_OfferAdjournment::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls… |
| `014dbd90` | GameCommand_OfferAdjournment::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandOfferAdjournment.cpp, line 0x35 via 012f5a60). Calls 00d833a6 with size 0x28; when n… |
| `014dbe20` | GameCommand_OfferAdjournment::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_OfferAdjournment (RTTI descriptors 01bb94f8 and 01bbfbd0). … |
| `014dbe70` | GameCommand_OfferAdjournment::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_OfferAdjournment' line; calls bas… |
| `014dbfb0` | GameCommand_OfferAdjournment::ReadVersionedMessage | likely | Vtable slot 9 override, decoded from bytes (Ghidra's unaff_ESI/unaff_EDI are artifacts): calls own vtable slot 4 with the argument and the address of a local d… |
| `014dc0b0` | GameCommand_OfferAdjournment::AppendTextForm | likely | Vtable slot 19 override: appends the literal 'Offer Adjourn Game Command' followed by a newline (27 bytes, pushed as length 0x1b) to the std::string passed as … |

**GameCommand_OfferDraw** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014db690` | GameCommand_OfferDraw::GameCommand_OfferDraw | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_OfferDraw::vftable into the object, and returns the object… |
| `014db710` | GameCommand_OfferDraw::GetClassName | confirmed | Returns the address of the literal 'GameCommand_OfferDraw' (mov eax, imm32; ret). |
| `014db750` | GameCommand_OfferDraw::ConstructWithArg | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_OfferDraw::vftable into the object, then calls … |
| `014db7c0` | GameCommand_OfferDraw::~GameCommand_OfferDraw | confirmed | Non-deleting destructor body: stores GameCommand_OfferDraw::vftable into the object, stores the object pointer in a local, and calls base destructor body 0141b… |
| `014dbab0` | GameCommand_OfferDraw::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_OfferDraw::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _free … |
| `014db8c0` | GameCommand_OfferDraw::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandOfferDraw.cpp, line 0x28 via 012f5a60). Calls 00d833a6 with size 0x24; when non-null… |
| `014db950` | GameCommand_OfferDraw::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_OfferDraw (RTTI descriptors 01bb94f8 and 01bbfba8). If the … |
| `014db9e0` | GameCommand_OfferDraw::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_OfferDraw' line; calls base routi… |
| `014dbb10` | GameCommand_OfferDraw::AppendTitleThenBase | likely | Vtable slot 19 override: appends the literal 'Offer Draw Game Command' followed by a newline (24 bytes, pushed as length 0x18) to the std::string passed as its… |

**GameCommand_PrePass** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014db260` | GameCommand_PrePass::GameCommand_PrePass | confirmed | Default constructor body: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_PrePass::vftable into the object, and returns the object p… |
| `014db2e0` | GameCommand_PrePass::GetClassName | confirmed | Returns the address of the literal 'GameCommand_PrePass' (mov eax, imm32; ret). |
| `014db320` | GameCommand_PrePass::ConstructWithGame | confirmed | Constructor with one stack argument: calls base constructor 0141b5d0 with the object in ECX, stores GameCommand_PrePass::vftable into the object, then calls 01… |
| `014db390` | GameCommand_PrePass::~GameCommand_PrePass | confirmed | Non-deleting destructor body: stores GameCommand_PrePass::vftable into the object, stores the object pointer in a local, and calls base destructor body 0141b31… |
| `014db600` | GameCommand_PrePass::DeletingDtor | confirmed | Scalar deleting destructor: stores GameCommand_PrePass::vftable into the object, calls base destructor body 0141b310 with the object in ECX, and calls _free on… |
| `014db410` | GameCommand_PrePass::Clone | likely | Vtable slot 17: asserts that field +8 (mGame) is non-null (file GameCommandPrePass.cpp, line 0x20 via 012f5a60). Calls 00d833a6 with size 0x24; when that is no… |
| `014db4a0` | GameCommand_PrePass::CloneHookCast | likely | Vtable slot 18: calls 00d8d382 as a dynamic cast of its argument from NetworkCommand to GameCommand_PrePass (RTTI descriptors 01bb94f8 and 01bbfb80). If the re… |
| `014db530` | GameCommand_PrePass::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_PrePass' line; calls base routine… |
| `014db660` | GameCommand_PrePass::AppendTitleThenBase | likely | Vtable slot 19 override: appends the literal 'Pre Pass Game Command' followed by a newline (22 bytes, pushed as length 0x16) to the std::string passed as its a… |

**GameCommand_PrePassCancel** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014dae10` | GameCommand_PrePassCancel::GameCommand_PrePassCancel | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_PrePassCancel::vftable and returns this. It sets no fields. |
| `014dae90` | GameCommand_PrePassCancel::GetName | confirmed | Returns the address of the string literal GameCommand_PrePassCancel (mov eax, imm32; ret). |
| `014daf40` | GameCommand_PrePassCancel::DtorBody | confirmed | Sets the vtable to GameCommand_PrePassCancel::vftable and calls 0141b310 (base destructor body). |
| `014dafe0` | GameCommand_PrePassCancel::Clone | confirmed | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x22). Allocates 0x24 bytes (00d833a6) and, when that succeeds, constructs the copy with 014daed… |
| `014db070` | GameCommand_PrePassCancel::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_PrePassCancel (00d8d382); on failure reports the clone assertion (012f5a60, line 0x2d). It calls … |
| `014db1d0` | GameCommand_PrePassCancel::~GameCommand_PrePassCancel | confirmed | Scalar deleting destructor: stores GameCommand_PrePassCancel::vftable into the object, calls the base destructor body 0141b310 with the object in ECX, and call… |
| `014daea0` | GameCommand_PrePassCancel::GetSharedDword | likely | Returns the dword in global DAT_01cbe86c (mov eax, [0x01cbe86c]; ret). |
| `014daeb0` | GameCommand_PrePassCancel::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe86c (ret 4). |
| `014daed0` | GameCommand_PrePassCancel::GameCommand_PrePassCancel_Game | likely | Constructor with one argument: calls 0141b5d0, installs GameCommand_PrePassCancel::vftable and calls 0141b3d0 with the argument. Returns this. |
| `014daf90` | GameCommand_PrePassCancel::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x1a). Calls 01361290 with the int at this+4, then calls 013541f0 with the game object at this+8… |
| `014db100` | GameCommand_PrePassCancel::WriteViaBase | likely | Vtable slot 10 override: calls its own vtable slot 2 with the argument; calls 013c26e0 with the literal 'Starting GameCommand_PrePassCancel' line; calls base r… |
| `014db230` | GameCommand_PrePassCancel::AppendTitleThenBase | likely | Vtable slot 19 override: appends the literal 'Pre Pass Cancel Game Command' followed by a newline (29 bytes, pushed as length 0x1d) to the std::string passed a… |

**GameCommand_ReplaceCard** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014da5e0` | GameCommand_ReplaceCard::DtorBody | confirmed | Sets the vtable to GameCommand_ReplaceCard::vftable and calls 0141b310 (base destructor body) and returns. |
| `014da650` | GameCommand_ReplaceCard::GetName | confirmed | Returns the address of the string literal GameCommand_ReplaceCard (mov eax, imm32; ret). |
| `014da6a0` | GameCommand_ReplaceCard::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_ReplaceCard (00d8d382); on failure reports the clone assertion (012f5a60, line 0x6e). It calls th… |
| `014da700` | GameCommand_ReplaceCard::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer, calls 0141b520 and returns 0 on failure. It then reads four ints into +0x24, +0x28, +0x2c and +0… |
| `014da780` | GameCommand_ReplaceCard::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts through 013c26e0 (Starting and Ending GameCommand_ReplaceCard and the labels Replace C… |
| `014da830` | GameCommand_ReplaceCard::GameCommand_ReplaceCard | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_ReplaceCard::vftable and zeroes the words at +0x24, +0x28 and +0x2c. Retu… |
| `014da890` | GameCommand_ReplaceCard::ScalarDeletingDtor | confirmed | Sets the vtable to GameCommand_ReplaceCard::vftable, calls 0141b310 (base destructor body), then calls _free(this) only when bit 0 of the flag argument is set.… |
| `014da960` | GameCommand_ReplaceCard::Clone | confirmed | Allocates 0x34 bytes (00d833a6). When that succeeds it constructs the copy with 014da8f0, passing this+8. It then calls virtual slot [18] (0x48) on this with t… |
| `014dab00` | GameCommand_ReplaceCard::TraceFields | confirmed | Appends the text Replace Card Game Command and a newline (01241650, length 0x1a), calls 0141b240 on the output, then appends Card ID: %d from +0x24, Instance I… |
| `014da660` | GameCommand_ReplaceCard::GetSharedDword | likely | Returns the dword in global DAT_01cbe868 (mov eax, [0x01cbe868]; ret). |
| `014da670` | GameCommand_ReplaceCard::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe868 (ret 4). |
| `014da8f0` | GameCommand_ReplaceCard::GameCommand_ReplaceCard_Game | likely | Constructor with one argument: calls 0141b5d0, zeroes the words at +0x24, +0x28 and +0x2c, installs GameCommand_ReplaceCard::vftable and calls 0141b3d0 with th… |
| `014da9d0` | GameCommand_ReplaceCard::FactoryCreate | likely | Allocates 0x34 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014da830. Returns the object, or 0 when the allocation failed. |
| `014daa30` | GameCommand_ReplaceCard::RegisterFactory | likely | Pushes the creator 014da9d0 and the classID 0x90 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_Reshuffle** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014da1b0` | GameCommand_Reshuffle::GameCommand_Reshuffle | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor) and installs GameCommand_Reshuffle::vftable. Returns this; the class adds no fields in the cons… |
| `014da230` | GameCommand_Reshuffle::GetName | confirmed | Returns the address of the string literal GameCommand_Reshuffle (mov eax, imm32; ret). |
| `014da2e0` | GameCommand_Reshuffle::DtorBody | confirmed | Sets the vtable to GameCommand_Reshuffle::vftable and calls 0141b310 (base destructor body). |
| `014da360` | GameCommand_Reshuffle::Clone | confirmed | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x22). Allocates 0x24 bytes (00d833a6) and, when that succeeds, constructs the copy with 014da27… |
| `014da3f0` | GameCommand_Reshuffle::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_Reshuffle (00d8d382); on failure reports the clone assertion (012f5a60, line 0x2d). It calls the … |
| `014da440` | GameCommand_Reshuffle::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer, calls 0141b520 and returns 0 on failure. Otherwise it calls vtable slot 5 (0x14) and returns its… |
| `014da480` | GameCommand_Reshuffle::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts Starting and Ending GameCommand_Reshuffle through 013c26e0, calls 0141b1a0 (base write… |
| `014da550` | GameCommand_Reshuffle::ScalarDeletingDtor | confirmed | Sets the vtable to GameCommand_Reshuffle::vftable, calls 0141b310 (base destructor body), then calls _free(this) only when bit 0 of the flag argument is set. R… |
| `014da5b0` | GameCommand_Reshuffle::TraceFields | confirmed | Appends the text Reshuffle Command and a newline (01241650, length 0x12) to the output string, then calls 0141b240 on the output (base trace helper). |
| `014da240` | GameCommand_Reshuffle::GetSharedDword | likely | Returns the dword in global DAT_01cbe864 (mov eax, [0x01cbe864]; ret). |
| `014da250` | GameCommand_Reshuffle::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe864 (ret 4). |
| `014da270` | GameCommand_Reshuffle::GameCommand_Reshuffle_Game | likely | Constructor with one argument: calls 0141b5d0, installs GameCommand_Reshuffle::vftable and calls 0141b3d0 with the argument. Returns this. |
| `014da330` | GameCommand_Reshuffle::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x18), calls 0141b020 and returns 1. It makes no other call. |
| `014da4d0` | GameCommand_Reshuffle::FactoryCreate | likely | Allocates 0x24 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014da1b0. Returns the object, or 0 when the allocation failed. |
| `014da530` | GameCommand_Reshuffle::RegisterFactory | likely | Pushes the creator 014da4d0 and the classID 0x134 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_RevealCards** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d9a80` | GameCommand_RevealCards::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts Starting and Ending GameCommand_RevealCards through 013c26e0, calls 0141b1a0 (base wri… |
| `014d9ae0` | GameCommand_RevealCards::DtorBody | confirmed | Sets the vtable to GameCommand_RevealCards::vftable, frees the buffer at +0x28 when non-null, zeroes +0x28, +0x2c and +0x30, and calls 0141b310 (base destructo… |
| `014d9b70` | GameCommand_RevealCards::GetName | confirmed | Returns the address of the string literal GameCommand_RevealCards (mov eax, imm32; ret). |
| `014d9bb0` | GameCommand_RevealCards::ScalarDeletingDtor | confirmed | Calls the destructor body 014d9ae0 on this, then calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `014d9bd0` | GameCommand_RevealCards::TraceFields | confirmed | Appends the text Set Reveal CardsGame Command and a newline (01241650, length 0x1d), calls 0141b240 on the output, then walks the vector from its begin word at… |
| `014d9cc0` | GameCommand_RevealCards::GameCommand_RevealCards | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_RevealCards::vftable and zeroes the words at +0x28, +0x2c and +0x30. Retu… |
| `014d9d90` | GameCommand_RevealCards::Clone | confirmed | Allocates 0x34 bytes (00d833a6). When that succeeds it constructs the copy with 014d9d20, passing this+8. It then calls virtual slot [18] (0x48) on this with t… |
| `014d9e00` | GameCommand_RevealCards::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_RevealCards (00d8d382); on failure reports the clone assertion (012f5a60, line 0x31). It calls th… |
| `014d9fc0` | GameCommand_RevealCards::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer, calls 0141b520 and returns 0 on failure. It then calls 01300540 on the list at this+0x24 with th… |
| `014d9b80` | GameCommand_RevealCards::GetSharedDword | likely | Returns the dword in global DAT_01cbe860 (mov eax, [0x01cbe860]; ret). |
| `014d9b90` | GameCommand_RevealCards::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe860 (ret 4). |
| `014d9d20` | GameCommand_RevealCards::GameCommand_RevealCards_Game | likely | Constructor with one argument: calls 0141b5d0, installs GameCommand_RevealCards::vftable, zeroes +0x28, +0x2c and +0x30, then calls 0141b3d0 with the argument.… |
| `014d9e60` | GameCommand_RevealCards::FactoryCreate | likely | Allocates 0x34 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d9cc0. Returns the object, or 0 when the allocation failed. |
| `014d9ec0` | GameCommand_RevealCards::RegisterFactory | likely | Pushes the creator 014d9e60 and the classID 0x80 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |
| `014da020` | GameCommand_RevealCards::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x6a) and calls 0141b020, 01347e10 and 013f7f70, then gets an object from 013efcc0. When that ob… |

**GameCommand_SendSerializedGame** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d93b0` | GameCommand_SendSerializedGame::GameCommand_SendSerializedG… | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SendSerializedGame::vftable and zeroes the words at +0xc, +0x28 and +0x2c… |
| `014d9430` | GameCommand_SendSerializedGame::GetName | confirmed | Returns the address of the string literal GameCommand_SendSerializedGame (mov eax, imm32; ret). |
| `014d94e0` | GameCommand_SendSerializedGame::DtorBody | confirmed | Sets the vtable to GameCommand_SendSerializedGame::vftable, frees the buffer at this+0x24 (_free), then calls 0141b310 (base destructor body). |
| `014d9550` | GameCommand_SendSerializedGame::Execute | confirmed | Calls 0141b020, allocates this+0x28 plus one byte, and inflates the data at this+0x24 (length this+0x2c) into it with 01539dc0 (zlib 1.2.3 uncompress by its ve… |
| `014d9680` | GameCommand_SendSerializedGame::Clone | confirmed | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x3d). Allocates 0x30 bytes (00d833a6) and, when that succeeds, constructs the copy with 014d947… |
| `014d97d0` | GameCommand_SendSerializedGame::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_SendSerializedGame (00d8d382); on failure reports the clone assertion (012f5a60, line 0x56). It c… |
| `014d9830` | GameCommand_SendSerializedGame::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer to a local, calls 0141b520 and returns 0 on failure. It reads an int into +0x28 (the uncompressed… |
| `014d98c0` | GameCommand_SendSerializedGame::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts through 013c26e0 (Starting, Length, CompressedLength, mSerializedGame and Ending label… |
| `014d99e0` | GameCommand_SendSerializedGame::ScalarDeletingDtor | confirmed | Calls the destructor body 014d94e0 on this, then calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `014d9a00` | GameCommand_SendSerializedGame::TraceFields | confirmed | Appends the text Send Serialized Game Command and a newline (01241650, length 0x1d), calls 0141b240 on the output, then appends mSerializedGame.size: %d(%ld) f… |
| `014d9440` | GameCommand_SendSerializedGame::GetSharedDword | likely | Returns the dword in global DAT_01cbe85c (mov eax, [0x01cbe85c]; ret). |
| `014d9450` | GameCommand_SendSerializedGame::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe85c (ret 4). |
| `014d9470` | GameCommand_SendSerializedGame::GameCommand_SendSerializedG… | likely | Constructor with one argument: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SendSerializedGame::vftable and calls 0141b3d0 with the argu… |
| `014d9960` | GameCommand_SendSerializedGame::FactoryCreate | likely | Allocates 0x30 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d93b0. Returns the object, or 0 when the allocation failed. |
| `014d99c0` | GameCommand_SendSerializedGame::RegisterFactory | likely | Pushes the creator 014d9960 and the classID 0x106 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_SetObserverMode** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d8e20` | GameCommand_SetObserverMode::GameCommand_SetObserverMode | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SetObserverMode::vftable, zeroes the byte at +0x24 and the word at +0x28.… |
| `014d8ea0` | GameCommand_SetObserverMode::GetName | confirmed | Returns the address of the string literal GameCommand_SetObserverMode (mov eax, imm32; ret). |
| `014d8f50` | GameCommand_SetObserverMode::DtorBody | confirmed | Sets the vtable to GameCommand_SetObserverMode::vftable, calls 0141b310 (base destructor body) and returns. |
| `014d8fe0` | GameCommand_SetObserverMode::Clone | confirmed | Allocates 0x2c bytes (00d833a6). When that succeeds it constructs the copy with 014d8ee0, passing this+8. It then calls virtual slot [18] (0x48) on this with t… |
| `014d9050` | GameCommand_SetObserverMode::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_SetObserverMode (00d8d382); on failure reports the clone assertion (012f5a60, line 0x3d). It call… |
| `014d90b0` | GameCommand_SetObserverMode::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer to a local, calls 0141b520 and returns 0 on failure. It then reads an int into the local (013c24d… |
| `014d9120` | GameCommand_SetObserverMode::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts through 013c26e0 (Starting and Ending GameCommand_SetObserverMode, Observer Mode and A… |
| `014d9290` | GameCommand_SetObserverMode::ScalarDeletingDtor | confirmed | Sets the vtable to GameCommand_SetObserverMode::vftable, calls 0141b310 (base destructor body), then calls _free(this) only when bit 0 of the flag argument is … |
| `014d92f0` | GameCommand_SetObserverMode::TraceFields | confirmed | Appends the text Set Observer Mode Game Command and a newline (01241650, length 0x1f), calls 0141b240 on the output, then appends Observer Mode: %d from the by… |
| `014d8eb0` | GameCommand_SetObserverMode::GetSharedDword | likely | Returns the dword in global DAT_01cbe858 (mov eax, [0x01cbe858]; ret). |
| `014d8ec0` | GameCommand_SetObserverMode::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe858 (ret 4). |
| `014d8ee0` | GameCommand_SetObserverMode::GameCommand_SetObserverMode_Ga… | likely | Constructor with one argument: calls 0141b5d0, installs GameCommand_SetObserverMode::vftable, calls 0141b3d0 with the argument, then zeroes the byte at +0x24 a… |
| `014d9210` | GameCommand_SetObserverMode::FactoryCreate | likely | Allocates 0x2c bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d8e20. Returns the object, or 0 when the allocation failed. |
| `014d9270` | GameCommand_SetObserverMode::RegisterFactory | likely | Pushes the creator 014d9210 and the classID 0x42 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_SetPlayer** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d89e0` | GameCommand_SetPlayer::GameCommand_SetPlayer | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor) and installs GameCommand_SetPlayer::vftable. Returns this. |
| `014d8a60` | GameCommand_SetPlayer::GetName | confirmed | Returns the address of the string literal GameCommand_SetPlayer (mov eax, imm32; ret). |
| `014d8b10` | GameCommand_SetPlayer::DtorBody | confirmed | Sets the vtable to GameCommand_SetPlayer::vftable, calls 0141b310 (base destructor body) and returns. |
| `014d8b60` | GameCommand_SetPlayer::Clone | confirmed | Allocates 0x24 bytes (00d833a6). When that succeeds it constructs the copy with 014d8aa0, passing this+8. It then calls virtual slot [18] (0x48) on this with t… |
| `014d8bd0` | GameCommand_SetPlayer::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_SetPlayer (00d8d382); on failure reports the clone assertion (012f5a60, line 0x23). It then calls… |
| `014d8c20` | GameCommand_SetPlayer::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer, calls 0141b520 on the buffer and returns 0 when either step fails. Otherwise it calls vtable slo… |
| `014d8c60` | GameCommand_SetPlayer::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer, writes the no-op trace texts Starting GameCommand_SetPlayer and Ending GameCommand_SetPlayer through 013c26e0, call… |
| `014d8d90` | GameCommand_SetPlayer::ScalarDeletingDtor | confirmed | Sets the vtable to GameCommand_SetPlayer::vftable, calls 0141b310 (base destructor body), then calls _free(this) only when bit 0 of the flag argument is set. R… |
| `014d8df0` | GameCommand_SetPlayer::TraceFields | confirmed | Appends the text Set Player Game Command and a newline (01241650, length 0x18) to the output string, then calls 0141b240 on the output (base GameCommand trace). |
| `014d8a70` | GameCommand_SetPlayer::GetSharedDword | likely | Returns the dword in global DAT_01cbe854 (mov eax, [0x01cbe854]; ret). |
| `014d8a80` | GameCommand_SetPlayer::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe854 (ret 4). |
| `014d8aa0` | GameCommand_SetPlayer::GameCommand_SetPlayer_Game | likely | Constructor with one argument: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SetPlayer::vftable and calls 0141b3d0 with the argument. Ret… |
| `014d8cb0` | GameCommand_SetPlayer::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x48), calls 0141b020, then calls 01347d60 on the game with the word at this+4. When 013487f0 re… |
| `014d8d10` | GameCommand_SetPlayer::FactoryCreate | likely | Allocates 0x24 bytes (00d833a6) and, when the allocation is non-null, constructs the object with 014d89e0. Returns the object, or 0 when the allocation failed. |
| `014d8d70` | GameCommand_SetPlayer::RegisterFactory | likely | Pushes the creator 014d8d10 and the classID 0x41 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_SetupGame** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d7cd0` | GameCommand_SetupGame::TraceFields | confirmed | Appends the line Setup Game Game Command and a newline (01241650), calls 0141b240 on the output (base GameCommand trace), then appends lines formatted with _sp… |
| `014d8140` | GameCommand_SetupGame::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer (header write), writes the trace-text no-op 013c26e0 with Starting GameCommand_SetupGame, calls 0141b1a0 (base write… |
| `014d8320` | GameCommand_SetupGame::DtorBody | confirmed | Sets the vtable to GameCommand_SetupGame::vftable, calls 01247310, frees the buffers at +0x44 and +0x34 (the two vector begin pointers) when non-null and zeroe… |
| `014d83e0` | GameCommand_SetupGame::GetName | confirmed | Returns the address of the string literal GameCommand_SetupGame (mov eax, imm32; ret). |
| `014d8420` | GameCommand_SetupGame::GameCommand_SetupGame | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SetupGame::vftable, zeroes the vector words at +0x34, +0x38, +0x3c, +0x44… |
| `014d84d0` | GameCommand_SetupGame::ScalarDeletingDtor | confirmed | Calls the destructor body 014d8320 on this, then calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `014d8620` | GameCommand_SetupGame::Clone | confirmed | Allocates 0x80 bytes (00d833a6). When that succeeds it constructs the copy with the game-argument constructor 014d84f0 passed this+8. It then calls virtual slo… |
| `014d8750` | GameCommand_SetupGame::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer to the buffer parameter (the header read stores the version there), calls the gate 0141b520, then… |
| `014d88f0` | GameCommand_SetupGame::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_SetupGame (00d8d382); on failure reports the clone assertion (012f5a60, line 0x59). It calls the … |
| `014d7a90` | GameCommand_SetupGame::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x133) and calls 0141b020. It compares the game version from 01347aa0 with this+0x50; when they … |
| `014d83f0` | GameCommand_SetupGame::GetSharedDword | likely | Returns the dword in global DAT_01cbe850 (mov eax, [0x01cbe850]; ret). |
| `014d8400` | GameCommand_SetupGame::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe850 (ret 4). |
| `014d84f0` | GameCommand_SetupGame::GameCommand_SetupGame_FromGame | likely | Constructor with one argument: calls 0141b5d0, installs GameCommand_SetupGame::vftable, zeroes the vector words at +0x34..+0x3c and +0x44..+0x4c, calls 0124738… |
| `014d86a0` | GameCommand_SetupGame::FactoryCreate | likely | Allocates 0x80 bytes (00d833a6) and, when the allocation is non-null, constructs the object with the default constructor 014d8420. Returns the object, or 0 whe… |
| `014d8710` | GameCommand_SetupGame::RegisterFactory | likely | Pushes the creator 014d86a0 and the classID 0x43 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_StateSpecificMessage** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141ac40` | GameCommand_StateSpecificMessage::Read | confirmed | Thiscall, one stack arg (stream). Calls slot 4 header read on the stream (version written to the arg slot), then base GameCommand::Read 0141b520, then for head… |
| `0141ad50` | GameCommand_StateSpecificMessage::DeletingDtor | confirmed | Scalar deleting destructor: calls the dtor body 0141abb0, then frees this through _free when flag bit 0 of the stack arg is set; returns this. |
| `0141ad70` | GameCommand_StateSpecificMessage::Dump | confirmed | Virtual slot 19 (Dump) writing trace text: appends the class header line, calls base dump 0141b240, appends 'OriginatingClassID: %d' from this+0x24, then 'Valu… |

**GameCommand_SynchPoint** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d7440` | GameCommand_SynchPoint::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer (header write), calls the no-op writer 013c26e0 with Starting GameCommand_SynchPoint, calls 0141b1a0 (base write), w… |
| `014d74c0` | GameCommand_SynchPoint::TraceFields | confirmed | Appends the line Synch Point Game Command and a newline (01241650, length 0x19), calls 0141b240 on the output (base GameCommand trace), then appends the lines … |
| `014d75c0` | GameCommand_SynchPoint::DtorBody | confirmed | Sets the vtable to GameCommand_SynchPoint::vftable, calls 00521a9c (cleanup of an unlabelled member) and then calls 0141b310 (base destructor body). |
| `014d7640` | GameCommand_SynchPoint::GetName | confirmed | Returns the address of the string literal GameCommand_SynchPoint (mov eax, imm32; ret). |
| `014d7680` | GameCommand_SynchPoint::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_SynchPoint (00d8d382); on failure reports the clone assertion (012f5a60, line 0x3b). It calls the… |
| `014d7720` | GameCommand_SynchPoint::Deserialize | confirmed | Calls vtable slot 4 (0x10) with the buffer and an out-pointer to the buffer parameter, which the header read overwrites with the header version. It then calls … |
| `014d77b0` | GameCommand_SynchPoint::GameCommand_SynchPoint | confirmed | Constructor body: calls 0141b5d0 (GameCommand base constructor), installs GameCommand_SynchPoint::vftable, zeroes the ints at +0x24, +0x28, +0x2c, then calls 0… |
| `014d7820` | GameCommand_SynchPoint::ScalarDeletingDtor | confirmed | Calls the destructor body 014d75c0 on this, then calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `014d7940` | GameCommand_SynchPoint::Clone | confirmed | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x30). Allocates 0x3c bytes (00d833a6) and, when that succeeds, constructs the copy with 014d784… |
| `014d73e0` | GameCommand_SynchPoint::ApplyToGame | likely | Asserts that this+8 (string mGame) is non-null (012f5a60, line 0x25). Calls 0141b020, then calls virtual slot 0x100 (index 64) of the object at this+8 with the… |
| `014d7650` | GameCommand_SynchPoint::GetSharedDword | likely | Returns the dword in global DAT_01cbe84c (mov eax, [0x01cbe84c]; ret). |
| `014d7660` | GameCommand_SynchPoint::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe84c (ret 4). |
| `014d7840` | GameCommand_SynchPoint::GameCommand_SynchPoint_FromGame | likely | Constructor with one argument (the game object): calls 0141b5d0 (base constructor), installs GameCommand_SynchPoint::vftable, calls 005258fb, calls 0141b3d0 wi… |
| `014d79d0` | GameCommand_SynchPoint::FactoryCreate | likely | Allocates 0x3c bytes (00d833a6) and, when the allocation is non-null, constructs the object with the default constructor 014d77b0. Returns the object, or 0 whe… |
| `014d7a30` | GameCommand_SynchPoint::RegisterFactory | likely | Pushes the creator 014d79d0 and the classID 0x49 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GameCommand_ValueList** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014d6e50` | GameCommand_ValueList::Serialize | confirmed | Calls vtable slot 2 (0x08) with the buffer (header write), calls 013c26e0 (no-op returning 1) with the Starting text, calls 0141b1a0 on the buffer, calls 012fc… |
| `014d6eb0` | GameCommand_ValueList::TraceFields | confirmed | Appends the text ValueList Command and a newline (01241650, length 0x12) to the output string, then calls 0141b240 on the output (the base GameCommand trace he… |
| `014d6ee0` | GameCommand_ValueList::GameCommand_ValueList | confirmed | Constructor body (default form): calls 0141b5d0 (the GameCommand base constructor), installs GameCommand_ValueList::vftable and zeroes the words at +0x28, +0x2… |
| `014d6f60` | GameCommand_ValueList::GetName | confirmed | Returns the address of the string literal GameCommand_ValueList (mov eax, imm32; ret). |
| `014d7010` | GameCommand_ValueList::DtorBody | confirmed | Sets the vtable to GameCommand_ValueList::vftable. When the buffer pointer at this+0x28 is non-null it destroys the entries through 012327d0 (buffer, count at … |
| `014d7160` | GameCommand_ValueList::Clone | confirmed | Asserts that this+8 (the game object, string mGame) is non-null (012f5a60, line 0x2e). Allocates 0x34 bytes (00d833a6) and, when that succeeds, constructs the … |
| `014d7270` | GameCommand_ValueList::ScalarDeletingDtor | confirmed | Calls the destructor body 014d7010 on this, then calls _free(this) only when bit 0 of the flag argument is set. Returns this. |
| `014d7320` | GameCommand_ValueList::CopyFieldsTo | confirmed | Dynamic-casts its argument from NetworkCommand to GameCommand_ValueList (00d8d382); on failure it reports the clone assertion (012f5a60, line 0x39). It then ca… |
| `014d7380` | GameCommand_ValueList::Deserialize | confirmed | Reads the class header through vtable slot 4 (0x10) with an out-pointer, calls 0141b520 on the buffer and returns 0 when either fails. It then calls 01301640 o… |
| `014d6f70` | GameCommand_ValueList::GetSharedDword | likely | Returns the dword in global DAT_01cbe848 (mov eax, [0x01cbe848]; ret). |
| `014d6f80` | GameCommand_ValueList::SetSharedDword | likely | Stores its single stack argument into global DAT_01cbe848 (ret 4). |
| `014d6fa0` | GameCommand_ValueList::GameCommand_ValueList_Game | likely | Constructor with one argument (the game object): calls 0141b5d0 (GameCommand base constructor), installs GameCommand_ValueList::vftable, zeroes +0x28, +0x2c an… |
| `014d70a0` | GameCommand_ValueList::Dispatch | likely | Asserts that this+8 is non-null (012f5a60). Calls 0141b020, 01347e10 and 013f7f70, gets a value from 013efcc0 and casts it with 00d8d382. When the cast succeed… |
| `014d71f0` | GameCommand_ValueList::FactoryCreate | likely | Allocates 0x34 bytes (00d833a6) and, when the allocation is non-null, constructs the object with the default constructor 014d6ee0. Returns the object, or 0 whe… |
| `014d7250` | GameCommand_ValueList::RegisterFactory | likely | Pushes the creator 014d71f0 and the classID 0x140 and calls 013ce130; then passes the value it returned in ECX to 013ce230. |

**GetTargetState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0141d5a0` | GetTargetState::Read | confirmed | Virtual slot 9 (Read), thiscall, one stack arg (stream). Header read via slot 4 (version out), base read 0147c850, an int into this+0xf8 (bool), a list of targ… |
| `0141d870` | GetTargetState::Write | confirmed | Virtual slot 10 (Write), thiscall, one stack arg (stream). Header write via slot 2 (or slot 1 when the stream is null), base write 0147c8b0, int write of (char… |

**IgnoreStatus** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01440020` | IgnoreStatus::RegisterFactory | likely | Pushes the constant 0x1fa and the address 0143ffc0 as two arguments and calls 013ce130; then calls 013ce230 with the result passed as ECX (same sequence as Fri… |

**LobbyCommand** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b2760` | LobbyCommand::LobbyCommand | confirmed | Calls 0140c090 with this (base command constructor, not in this shard), stores LobbyCommand::vftable at *this, zeroes the dword at +4 (the AccountID field) and… |
| `013b27e0` | LobbyCommand::GetTypeName | confirmed | Virtual slot 11: returns the static C string LobbyCommand. |
| `013b2820` | LobbyCommand::DtorBody | confirmed | Stores LobbyCommand::vftable at *this, calls 0140c160 (base destructor body, not in this shard) and frees nothing. |
| `013b2910` | LobbyCommand::ReadFields | confirmed | Virtual slot 9: calls this->vtable slot 4 with (stream, &stream); then 0140c2a0 (base header read, not in this shard), 013c24d0 reads an int into this+4 (Accou… |
| `013b2960` | LobbyCommand::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2 with the stream, then 0140c2d0 (base header write), writes the AccountID dword at this+4 through 013c23f0, passes th… |
| `013b2a40` | LobbyCommand::~LobbyCommand | confirmed | Scalar deleting destructor in vtable slot 0: stores LobbyCommand::vftable, calls 0140c160, then calls _free(this) when bit 0 of the flag argument is set; retur… |
| `013b2aa0` | LobbyCommand::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to LobbyCommand with 00d8d382 (NetworkCommand and LobbyCommand RTTI descriptors); when the cast fails it asserts lobbyCo… |
| `013b2af0` | LobbyCommand::AppendDescription | confirmed | Virtual slot 19: calls 0140c290 (empty base stub), formats AccountID (this+4) with _sprintf into a 1000-byte buffer as AccountID: %d and appends the text to it… |
| `013b2b70` | LobbyCommand::AppendDisplayName | confirmed | Virtual slot 20: appends the 13-byte text Lobby Command to its std::string argument through 01241650. |
| `013b2880` | LobbyCommand::Clone | likely | Virtual slot 17: allocates 8 bytes (00d833a6), constructs a LobbyCommand in them with 013b2760, then calls this->vtable slot 18 (CopyTo) with the new object an… |
| `013b29c0` | LobbyCommand::CreateInstance | likely | Allocates 8 bytes (00d833a6); when the allocation succeeds it constructs a LobbyCommand with 013b2760 and returns the object, else returns 0. |
| `013b2a20` | LobbyCommand::RegisterClass | likely | Pushes creator 013b29c0 and id 0x5b, calls 013ce130, then calls 013ce230 with ECX set to 013ce130 return value and no stack arguments; returns void. |

**LobbyCommand_AcceptAdjournedGame** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d8520` | LobbyCommand_AcceptAdjournedGame::LobbyCommand_AcceptAdjour… | confirmed | Constructor: calls the base constructor 013b2760 first, then stores LobbyCommand_AcceptAdjournedGame::vftable and zeroes the fields at +8, +0xc (dwords), +0x10… |
| `013d85a0` | LobbyCommand_AcceptAdjournedGame::TypeName | confirmed | Virtual slot [11]: returns the address of the string "LobbyCommand_AcceptAdjournedGame". |
| `013d85b0` | LobbyCommand_AcceptAdjournedGame::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde34. |
| `013d85c0` | LobbyCommand_AcceptAdjournedGame::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde34. |
| `013d85e0` | LobbyCommand_AcceptAdjournedGame::~LobbyCommand_AcceptAdjou… | confirmed | Destructor body: stores LobbyCommand_AcceptAdjournedGame::vftable, calls the base destructor 013b2820, and does not free the object. It is the cleanup on the E… |
| `013d86d0` | LobbyCommand_AcceptAdjournedGame::Clone | confirmed | Virtual slot [17] (Clone): allocates 0x18 bytes with 00d833a6, constructs the object with 013d8520 when the allocation succeeded, then calls vtable slot +0x48 … |
| `013d8740` | LobbyCommand_AcceptAdjournedGame::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination through 00d8d382 (NetworkCommand and LobbyCommand_AcceptAdjournedGame RTTI descriptors); a nu… |
| `013d8960` | LobbyCommand_AcceptAdjournedGame::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_AcceptAdjournedGame::vftable, calls the base destructor 013b2820, then frees the object with … |
| `013d89c0` | LobbyCommand_AcceptAdjournedGame::Dump | confirmed | Virtual slot [19] (Dump): appends "Accept Adjourned Game Account Command\n" through 01241650, calls the base dump 013b2af0 on the output argument, then formats… |
| `013d8af0` | LobbyCommand_AcceptAdjournedGame::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x23-byte literal "Accept Adjourned Game Lobby Command" through 01241650 and returns. |

**LobbyCommand_AddGroups** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143e610` | LobbyCommand_AddGroups::~LobbyCommand_AddGroups | confirmed | Destructor body: stores LobbyCommand_AddGroups::vftable into *this, zeroes the local EH state, frees the block at this+0xc when non-null (_free), zeroes the dw… |
| `0143e6a0` | LobbyCommand_AddGroups::TypeName | confirmed | No arguments. Returns a pointer to the literal LobbyCommand_AddGroups (mov eax,018ea044; ret). |
| `0143e750` | LobbyCommand_AddGroups::~LobbyCommand_AddGroups | confirmed | Scalar deleting destructor (virtual slot 0): calls FUN_0143e610 (destructor body) on this, then if param_2 bit 0 is set calls _free on this; returns this. 27 b… |
| `0143f180` | LobbyCommand_AddGroups::LobbyCommand_AddGroups | confirmed | Constructor (fastcall, returns this): calls FUN_013b2760 on this (base constructor), stores LobbyCommand_AddGroups::vftable into *this, zeroes the dwords at th… |
| `0143e6b0` | LobbyCommand_AddGroups::ReadGlobal01cbe38c | likely | No arguments. Returns the 32-bit global at 01cbe38c (mov eax,[01cbe38c]; ret). The global is uninitialised .data in the file. |
| `0143e6c0` | LobbyCommand_AddGroups::WriteGlobal01cbe38c | likely | One stack argument, ret 4. Stores the argument to the global at 01cbe38c. |
| `0143ee40` | LobbyCommand_AddGroups::DescribeGroupIds | likely | One stack argument, ECX=this. Writes Add Groups Lobby Command plus newline with 01241650, calls 013b2af0 with the argument, then loops from [this+0xc] to [this… |
| `0143f340` | CreateLobbyCommandAddGroups | likely | Argument-less factory. Allocates 0x18 bytes through FUN_00d833a6; if non-null constructs LobbyCommand_AddGroups in it through 0143f180 and returns the object, … |
| `0143f3d0` | LobbyCommand_AddGroups::ProcessStreamBody | likely | Virtual slot 9 of LobbyCommand_AddGroups (thiscall). Calls this->vtbl+0x10(param_2, &local); on success calls FUN_013b2910(param_2), then stores param_2 into t… |

**LobbyCommand_ChangeStatus** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d8030` | LobbyCommand_ChangeStatus::LobbyCommand_ChangeStatus | confirmed | Constructor: calls the base constructor 013b2760 first, then stores LobbyCommand_ChangeStatus::vftable; returns this (fastcall). |
| `013d80b0` | LobbyCommand_ChangeStatus::TypeName | confirmed | Virtual slot [11]: returns the address of the string "LobbyCommand_ChangeStatus". |
| `013d80c0` | LobbyCommand_ChangeStatus::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde30. |
| `013d80d0` | LobbyCommand_ChangeStatus::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde30. |
| `013d80f0` | LobbyCommand_ChangeStatus::~LobbyCommand_ChangeStatus | confirmed | Destructor body: stores LobbyCommand_ChangeStatus::vftable, calls the base destructor 013b2820, and does not free the object. Also the cleanup on the EH unwind… |
| `013d8210` | LobbyCommand_ChangeStatus::Clone | confirmed | Virtual slot [17] (Clone): allocates 12 bytes with 00d833a6, constructs the object with 013d8030 when the allocation succeeded, then calls vtable slot +0x48 (C… |
| `013d8280` | LobbyCommand_ChangeStatus::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination through 00d8d382 (NetworkCommand and LobbyCommand_ChangeStatus RTTI descriptors); a null resu… |
| `013d8410` | LobbyCommand_ChangeStatus::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_ChangeStatus::vftable, calls the base destructor 013b2820, then frees the object with _free o… |
| `013d8470` | LobbyCommand_ChangeStatus::Dump | confirmed | Virtual slot [19] (Dump): appends "Change Status Lobby Command " through 01241650, calls the base dump 013b2af0 on the output argument, then formats "Lobby Sta… |
| `013d8500` | LobbyCommand_ChangeStatus::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x1b-byte literal "Change Status Lobby Command" through 01241650 and returns. |

**LobbyCommand_Concede** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0144b570` | LobbyCommand_Concede::LobbyCommand_Concede | confirmed | Constructor (ECX=this): calls the LobbyCommand base constructor 013b2760 under an EH frame, stores LobbyCommand_Concede::vftable at +0, zeroes the int at +0x8,… |
| `0144b5f0` | LobbyCommand_Concede::TypeName | confirmed | Virtual slot [11]: returns the address of the literal LobbyCommand_Concede (mov eax,imm; ret). No arguments. |
| `0144b600` | LobbyCommand_Concede::ReadGlobal01cbe4d8 | confirmed | Virtual slot [12]: returns the dword at global DAT_01cbe4d8. No arguments. |
| `0144b610` | LobbyCommand_Concede::WriteGlobal01cbe4d8 | confirmed | Virtual slot [13]: stores its stack argument to global DAT_01cbe4d8 and returns (ret 4). |
| `0144b630` | LobbyCommand_Concede::~LobbyCommand_Concede | confirmed | Destructor body (ECX=this): stores LobbyCommand_Concede::vftable at +0 and calls the LobbyCommand base destructor 013b2820. It does not free the object. |
| `0144b6a0` | LobbyCommand_Concede::Clone | confirmed | Virtual slot [17] (Clone): allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the copy with 0144b570; then calls this… |
| `0144b710` | LobbyCommand_Concede::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast (00d8d382) of its argument from NetworkCommand to LobbyCommand_Concede; on failure calls 012f5a60 with the clone… |
| `0144b760` | LobbyCommand_Concede::Read | confirmed | Virtual slot [9] (Read): calls header read slot [4] with the stream and an out version word and does NOT test its result; calls the LobbyCommand base read 013b… |
| `0144b7b0` | LobbyCommand_Concede::Write | confirmed | Virtual slot [10] (Write): calls header write slot [2] with the stream; passes the starting text to the no-op trace call 013c26e0; calls the LobbyCommand base … |
| `0144b890` | LobbyCommand_Concede::ScalarDeletingDtor | confirmed | Vtable slot [0] of LobbyCommand_Concede: stores the vtable at +0, calls the LobbyCommand base destructor 013b2820, and when bit 0 of the flag argument is set f… |
| `0144b8f0` | LobbyCommand_Concede::Dump | confirmed | Virtual slot [19] (Dump): appends the literal Change Lobby display Lobby Command with a newline (35 characters) to its std::string argument through 01241650, t… |
| `0144b920` | LobbyCommand_Concede::DisplayName | confirmed | Virtual slot [20]: appends the literal Concede Lobby Command (21 characters, no newline) to its std::string argument through 01241650. |
| `0144b810` | LobbyCommand_Concede::Create | likely | Factory: allocates 0xc bytes with operator new (00d833a6); when the allocation is non-null constructs the command with 0144b570 and returns it, otherwise retur… |
| `0144b870` | LobbyCommand_Concede::RegisterFactory | likely | Registers the factory 0144b810 under classID 0x12a: pushes the factory address and 0x12a and calls 013ce130, then moves its eax result into ECX and calls 013ce… |

**LobbyCommand_DeleteAdjournedGame** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d9080` | LobbyCommand_DeleteAdjournedGame::LobbyCommand_DeleteAdjour… | confirmed | Constructor: calls the base constructor 013b2760 first, then stores LobbyCommand_DeleteAdjournedGame::vftable and zeroes the dwords at +8, +0xc and +0x10; retu… |
| `013d9100` | LobbyCommand_DeleteAdjournedGame::TypeName | confirmed | Virtual slot [11]: returns the address of the string "LobbyCommand_DeleteAdjournedGame". |
| `013d9110` | LobbyCommand_DeleteAdjournedGame::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde3c. |
| `013d9120` | LobbyCommand_DeleteAdjournedGame::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde3c. |
| `013d9140` | LobbyCommand_DeleteAdjournedGame::~LobbyCommand_DeleteAdjou… | confirmed | Destructor body: stores LobbyCommand_DeleteAdjournedGame::vftable, calls the base destructor 013b2820, and does not free the object. It is the cleanup on the E… |
| `013d9210` | LobbyCommand_DeleteAdjournedGame::Clone | confirmed | Virtual slot [17] (Clone): allocates 0x14 bytes with 00d833a6, constructs the object with 013d9080 when the allocation succeeded, then calls vtable slot +0x48 … |
| `013d9280` | LobbyCommand_DeleteAdjournedGame::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination through 00d8d382 (NetworkCommand and LobbyCommand_DeleteAdjournedGame RTTI descriptors); a nu… |
| `013d9470` | LobbyCommand_DeleteAdjournedGame::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_DeleteAdjournedGame::vftable, calls the base destructor 013b2820, then frees the object with … |
| `013d94d0` | LobbyCommand_DeleteAdjournedGame::Dump | confirmed | Virtual slot [19] (Dump): appends "Delete Adjourned Game Lobby Command\n" through 01241650, calls the base dump 013b2af0 on the output argument, then formats a… |
| `013d95d0` | LobbyCommand_DeleteAdjournedGame::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x23-byte literal "Delete Adjourned Game Lobby Command" through 01241650 and returns. |

**LobbyCommand_GameServiceError** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c4070` | LobbyCommand_GameServiceError::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads ints into this+0xc (GroupID) and this+0x8 (Type) with 013c24d0, reads the string at th… |
| `014c40e0` | LobbyCommand_GameServiceError::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to the no-op 013c26e0, calls base writer 013b2960, appends the GroupID (+0xc) and Type (+0… |
| `014c4200` | LobbyCommand_GameServiceError::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_GameServiceError (mov eax, imm32; ret; bytes b8 24 25 90 01 c3). |
| `014c4210` | LobbyCommand_GameServiceError::ReadGlobal01cbe7b8 | confirmed | No arguments. Returns the 32-bit global at 01cbe7b8 (mov eax,[01cbe7b8]; ret; bytes a1 b8 e7 cb 01 c3). |
| `014c4220` | LobbyCommand_GameServiceError::WriteGlobal01cbe7b8 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7b8 (bytes 8b 44 24 04 a3 b8 e7 cb 01 c2 04 00). |
| `014c43b0` | LobbyCommand_GameServiceError::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls DtorBody 014c4240, then calls _free on this when flag bit 0 of the argument is set; returns this. |
| `014c4240` | LobbyCommand_GameServiceError::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, frees the string buffer at +0x14 when the capacity at +0x28 exceeds 15, resets size and … |
| `014c42c0` | LobbyCommand_GameServiceError::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x2c bytes (operator new 00d833a6), constructs with 014c4170 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c4330` | LobbyCommand_GameServiceError::CreateInstance | likely | Static factory: allocates 0x2c bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c4170 and returns it, otherwise returns 0. |
| `014c4390` | LobbyCommand_GameServiceError::RegisterFactory | likely | Registers factory 014c4330 under key 0x8a in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function point… |
| `014c43f0` | LobbyCommand_GameServiceError::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_GameServiceError (00d8d382); on failure asserts through 012f5a60 with the te… |

**LobbyCommand_InfoEnumMessage** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142f1a0` | LobbyCommand_InfoEnumMessage::Ctor | confirmed | Calls FUN_013b2760 (base constructor; argument is a stack-cookie artifact), stores LobbyCommand_InfoEnumMessage::vftable, and zeroes the dwords at +0x8, +0xc a… |
| `0142f220` | LobbyCommand_InfoEnumMessage::GetName | confirmed | Returns the address of the literal 'LobbyCommand_InfoEnumMessage' (6 bytes). |
| `0142f230` | LobbyCommand_InfoEnumMessage::GetSharedDword | confirmed | Returns the dword global at 01cbe298 (6 bytes). |
| `0142f240` | LobbyCommand_InfoEnumMessage::SetSharedDword | confirmed | Stores its stack argument into the dword global at 01cbe298 (12 bytes, ret 4). |
| `0142f260` | LobbyCommand_InfoEnumMessage::Dtor | confirmed | Stores the class vtable; when the pointer at +0xc is set, calls its vtable slot 0 with 1 (deleting) and clears it; when the pointer at +0x10 is set, calls its … |

**LobbyCommand_Join** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b2fa0` | LobbyCommand_Join::GetTypeName | confirmed | Virtual slot 11: returns the static C string LobbyCommand_Join. |
| `013b3090` | LobbyCommand_Join::~LobbyCommand_Join | confirmed | Scalar deleting destructor in vtable slot 0: calls 013b2e80 (destructor body), then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `013b3130` | LobbyCommand_Join::AppendDescription | confirmed | Virtual slot 19: appends Join Lobby Command newline, then 013b2af0 (LobbyCommand description), then formats and appends GroupID, RoleType, Password, AccountNam… |
| `013b3330` | LobbyCommand_Join::AppendDisplayName | confirmed | Virtual slot 20: appends the 18-byte text Join Lobby Command to its std::string argument through 01241650. |
| `013b3350` | LobbyCommand_Join::LobbyCommand_Join | confirmed | Calls 013b2760 (LobbyCommand constructor), stores LobbyCommand_Join::vftable at *this, sets the capacity dwords at +0x28, +0x54 and +0x74 to 0xF, the length dw… |
| `013b3560` | LobbyCommand_Join::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to LobbyCommand_Join (00d8d382 with NetworkCommand and LobbyCommand_Join RTTI descriptors); on failure asserts clone thr… |
| `013b2fb0` | LobbyCommand_Join::GetClassId | likely | Virtual slot 12: returns the dword in the global DAT_01cbdd10. |
| `013b2fc0` | LobbyCommand_Join::SetClassId | likely | Virtual slot 13: stores its stack argument into the global DAT_01cbdd10. |
| `013b34f0` | LobbyCommand_Join::Clone | likely | Virtual slot 17: allocates 0x7C bytes (00d833a6), constructs a LobbyCommand_Join with 013b3350 when the allocation succeeds, then calls this->vtable slot 18 (C… |
| `013b3610` | LobbyCommand_Join::CreateInstance | likely | Allocates 0x7C bytes (00d833a6); when the allocation succeeds constructs a LobbyCommand_Join with 013b3350 and returns it, else returns 0. |
| `013b3670` | LobbyCommand_Join::RegisterClass | likely | Pushes creator 013b3610 and id 0x5c, calls 013ce130, then calls 013ce230 with ECX set to 013ce130 return value and no stack arguments; returns void. |

**LobbyCommand_JoinLeague** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c3be0` | LobbyCommand_JoinLeague::LobbyCommand_JoinLeague | confirmed | Constructor with ECX=this: calls 013b3350 (the LobbyCommand_Join base constructor, vtable 018df744), stores LobbyCommand_JoinLeague::vftable at +0x0, returns t… |
| `014c3c60` | LobbyCommand_JoinLeague::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_JoinLeague (mov eax, imm32; ret; bytes b8 94 23 90 01 c3). |
| `014c3c70` | LobbyCommand_JoinLeague::ReadGlobal01cbe7b4 | confirmed | No arguments. Returns the 32-bit global at 01cbe7b4 (mov eax,[01cbe7b4]; ret; bytes a1 b4 e7 cb 01 c3). |
| `014c3c80` | LobbyCommand_JoinLeague::WriteGlobal01cbe7b4 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7b4 (bytes 8b 44 24 04 a3 b4 e7 cb 01 c2 04 00). |
| `014c3e90` | LobbyCommand_JoinLeague::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the vtable at +0x0, calls base destructor 013b2e80 (LobbyCommand_Join), calls _free on this when flag bit … |
| `014c3f20` | LobbyCommand_JoinLeague::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Join League Lobby Command (length 0x19), t… |
| `014c3ca0` | LobbyCommand_JoinLeague::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, calls base destructor 013b2e80 (LobbyCommand_Join), returns. It does not free memory. |
| `014c3d00` | LobbyCommand_JoinLeague::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x7c bytes (operator new 00d833a6), constructs with 014c3be0 when the allocation succeeded, then calls this vtbl[18] with … |
| `014c3d80` | LobbyCommand_JoinLeague::Read | likely | Virtual slot 9. Calls vtbl[4] (header read), then base reader 013b36a0 (LobbyCommand_Join base payload, not labelled), and on success tail-calls vtbl[5] and re… |
| `014c3dc0` | LobbyCommand_JoinLeague::Write | likely | Virtual slot 10. Calls vtbl[2] (header write), passes the trace literals Starting and Ending LobbyCommand_JoinLeague to the no-op 013c26e0, calls base writer 0… |
| `014c3e10` | LobbyCommand_JoinLeague::CreateInstance | likely | Static factory: allocates 0x7c bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c3be0 and returns it, otherwise returns 0. |
| `014c3e70` | LobbyCommand_JoinLeague::RegisterFactory | likely | Registers factory 014c3e10 under key 0x154 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c3ef0` | LobbyCommand_JoinLeague::Dump | likely | Virtual slot 19, one stack argument (the output std::string). Appends Join League Lobby Command plus newline through 01241650 (length 0x1a), then calls 013b313… |

**LobbyCommand_Leave** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d9680` | LobbyCommand_Leave::~LobbyCommand_Leave | confirmed | Destructor body: stores LobbyCommand_Leave::vftable, frees the buffer at this+0x14 with _free when it is set, zeroes +0x14, +0x18 and +0x1c, then calls the bas… |
| `013d9710` | LobbyCommand_Leave::TypeName | confirmed | Virtual slot [11]: returns the address of the string "LobbyCommand_Leave". |
| `013d9720` | LobbyCommand_Leave::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde40. |
| `013d9730` | LobbyCommand_Leave::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde40. |
| `013d9750` | LobbyCommand_Leave::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 013d9680, then frees the object with _free only when bit 0 of the flag argument is set;… |
| `013d9770` | LobbyCommand_Leave::Dump | confirmed | Virtual slot [19] (Dump): appends "Leave Lobby Command\n" through 01241650, calls the base dump 013b2af0 on the output argument, then formats and appends "Grou… |
| `013d9810` | LobbyCommand_Leave::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x13-byte literal "Leave Lobby Command" through 01241650 and returns. |
| `013d9830` | LobbyCommand_Leave::LobbyCommand_Leave | confirmed | Constructor: calls the base constructor 013b2760 first, then stores LobbyCommand_Leave::vftable and zeroes the dwords at +8, +0xc, +0x14, +0x18 and +0x1c; retu… |
| `013d98a0` | LobbyCommand_Leave::Clone | confirmed | Virtual slot [17] (Clone): allocates 0x20 bytes with 00d833a6, constructs the object with 013d9830 when the allocation succeeded, then calls vtable slot +0x48 … |
| `013d9910` | LobbyCommand_Leave::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination through 00d8d382 (NetworkCommand and LobbyCommand_Leave RTTI descriptors); a null result send… |

**LobbyCommand_LeaveLeague** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c3880` | LobbyCommand_LeaveLeague::LobbyCommand_LeaveLeague | confirmed | Constructor with ECX=this: calls 013d9830 (the LobbyCommand_Leave base constructor, vtable 018e1e1c, not LobbyCommand), stores LobbyCommand_LeaveLeague::vftabl… |
| `014c3900` | LobbyCommand_LeaveLeague::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_LeaveLeague (mov eax, imm32; ret; bytes b8 9c 22 90 01 c3). |
| `014c3910` | LobbyCommand_LeaveLeague::ReadGlobal01cbe7b0 | confirmed | No arguments. Returns the 32-bit global at 01cbe7b0 (mov eax,[01cbe7b0]; ret; bytes a1 b0 e7 cb 01 c3). |
| `014c3920` | LobbyCommand_LeaveLeague::WriteGlobal01cbe7b0 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7b0 (bytes 8b 44 24 04 a3 b0 e7 cb 01 c2 04 00). |
| `014c3b30` | LobbyCommand_LeaveLeague::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the vtable at +0x0, calls base destructor 013d9680 (LobbyCommand_Leave), calls _free on this when flag bit… |
| `014c3bc0` | LobbyCommand_LeaveLeague::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Leave League Lobby Command (length 0x1a), … |
| `014c3940` | LobbyCommand_LeaveLeague::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, calls base destructor 013d9680 (LobbyCommand_Leave), returns. It does not free memory. |
| `014c39a0` | LobbyCommand_LeaveLeague::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x20 bytes (operator new 00d833a6), constructs with 014c3880 when the allocation succeeded, then calls this vtbl[18] with … |
| `014c3ab0` | LobbyCommand_LeaveLeague::CreateInstance | likely | Static factory: allocates 0x20 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c3880 and returns it, otherwise returns 0. |
| `014c3b10` | LobbyCommand_LeaveLeague::RegisterFactory | likely | Registers factory 014c3ab0 under key 0x155 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c3b90` | LobbyCommand_LeaveLeague::Dump | likely | Virtual slot 19, one stack argument (the output std::string). Appends Leave League Lobby Command plus newline through 01241650 (length 0x1b), then calls 013d97… |

**LobbyCommand_NotifyOfBuddy** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c3310` | LobbyCommand_NotifyOfBuddy::LobbyCommand_NotifyOfBuddy | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_NotifyOfBuddy::vftable at +0x0, zeroes the dword at +0x8 and the byte at +0xc, … |
| `014c3390` | LobbyCommand_NotifyOfBuddy::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_NotifyOfBuddy (mov eax, imm32; ret; bytes b8 2c 21 90 01 c3). |
| `014c33a0` | LobbyCommand_NotifyOfBuddy::ReadGlobal01cbe7ac | confirmed | No arguments. Returns the 32-bit global at 01cbe7ac (mov eax,[01cbe7ac]; ret; bytes a1 ac e7 cb 01 c3). |
| `014c33b0` | LobbyCommand_NotifyOfBuddy::WriteGlobal01cbe7ac | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7ac (bytes 8b 44 24 04 a3 ac e7 cb 01 c2 04 00). |
| `014c35d0` | LobbyCommand_NotifyOfBuddy::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads the int at this+0x8 with 013c24d0, reads one more int into a local and stores (local !… |
| `014c3640` | LobbyCommand_NotifyOfBuddy::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends int +0x8 with 013c23f0 under Target… |
| `014c3740` | LobbyCommand_NotifyOfBuddy::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the vtable at +0x0, calls base destructor 013b2820, calls _free on this when flag bit 0 of the argument is… |
| `014c3860` | LobbyCommand_NotifyOfBuddy::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Notify of Buddy Lobby Command (length 0x1d… |
| `014c33d0` | LobbyCommand_NotifyOfBuddy::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0 and calls base destructor 013b2820; returns. It does not free memory. |
| `014c3500` | LobbyCommand_NotifyOfBuddy::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x10 bytes (operator new 00d833a6), constructs with 014c3310 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c3570` | LobbyCommand_NotifyOfBuddy::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_NotifyOfBuddy (00d8d382); on failure asserts through 012f5a60 with the text … |
| `014c36c0` | LobbyCommand_NotifyOfBuddy::CreateInstance | likely | Static factory: allocates 0x10 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c3310 and returns it, otherwise returns 0. |
| `014c3720` | LobbyCommand_NotifyOfBuddy::RegisterFactory | likely | Registers factory 014c36c0 under key 0x121 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c37a0` | LobbyCommand_NotifyOfBuddy::Dump | likely | Virtual slot 19. Appends Notify of Buddy Lobby Command plus newline through 01241650, calls base dump 013b2af0 with the output, then formats Target User ID: %d… |

**LobbyCommand_NotifyOfIgnore** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c2da0` | LobbyCommand_NotifyOfIgnore::LobbyCommand_NotifyOfIgnore | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_NotifyOfIgnore::vftable at +0x0, zeroes the dword at +0x8 and the byte at +0xc,… |
| `014c2e20` | LobbyCommand_NotifyOfIgnore::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_NotifyOfIgnore (mov eax, imm32; ret; bytes b8 bc 1f 90 01 c3). |
| `014c2e30` | LobbyCommand_NotifyOfIgnore::ReadGlobal01cbe7a8 | confirmed | No arguments. Returns the 32-bit global at 01cbe7a8 (mov eax,[01cbe7a8]; ret; bytes a1 a8 e7 cb 01 c3). |
| `014c2e40` | LobbyCommand_NotifyOfIgnore::WriteGlobal01cbe7a8 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7a8 (bytes 8b 44 24 04 a3 a8 e7 cb 01 c2 04 00). |
| `014c3060` | LobbyCommand_NotifyOfIgnore::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads the int at this+0x8 with 013c24d0, reads one more int into a local with 013c24d0 and s… |
| `014c30d0` | LobbyCommand_NotifyOfIgnore::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends int +0x8 with 013c23f0 under Target… |
| `014c31d0` | LobbyCommand_NotifyOfIgnore::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the vtable at +0x0, calls base destructor 013b2820, calls _free on this when flag bit 0 of the argument is… |
| `014c32f0` | LobbyCommand_NotifyOfIgnore::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Notify of Ignore Lobby Command (length 0x1… |
| `014c2e60` | LobbyCommand_NotifyOfIgnore::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, calls base destructor 013b2820 inside an EH frame, returns. It does not free memory. |
| `014c2f90` | LobbyCommand_NotifyOfIgnore::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x10 bytes (operator new 00d833a6), constructs with 014c2da0 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c3000` | LobbyCommand_NotifyOfIgnore::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_NotifyOfIgnore (00d8d382); on failure asserts through 012f5a60 with the text… |
| `014c3150` | LobbyCommand_NotifyOfIgnore::CreateInstance | likely | Static factory: allocates 0x10 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c2da0 and returns it, otherwise returns 0. |
| `014c31b0` | LobbyCommand_NotifyOfIgnore::RegisterFactory | likely | Registers factory 014c3150 under key 0x119 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c3230` | LobbyCommand_NotifyOfIgnore::Dump | likely | Virtual slot 19. Appends Notify of Ignore Lobby Command plus newline through 01241650, calls base dump 013b2af0 with the output, then formats Target User ID: %… |

**LobbyCommand_OfferAdjournedGame** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d8b10` | LobbyCommand_OfferAdjournedGame::LobbyCommand_OfferAdjourne… | confirmed | Constructor: calls the base constructor 013b2760 first, then stores LobbyCommand_OfferAdjournedGame::vftable and zeroes the dwords at +8, +0xc and +0x10; retur… |
| `013d8b90` | LobbyCommand_OfferAdjournedGame::TypeName | confirmed | Virtual slot [11]: returns the address of the string "LobbyCommand_OfferAdjournedGame". |
| `013d8ba0` | LobbyCommand_OfferAdjournedGame::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde38. |
| `013d8bb0` | LobbyCommand_OfferAdjournedGame::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde38. |
| `013d8bd0` | LobbyCommand_OfferAdjournedGame::~LobbyCommand_OfferAdjourn… | confirmed | Destructor body: stores LobbyCommand_OfferAdjournedGame::vftable, calls the base destructor 013b2820, and does not free the object. It is the cleanup on the EH… |
| `013d8ca0` | LobbyCommand_OfferAdjournedGame::Clone | confirmed | Virtual slot [17] (Clone): allocates 0x14 bytes with 00d833a6, constructs the object with 013d8b10 when the allocation succeeded, then calls vtable slot +0x48 … |
| `013d8d10` | LobbyCommand_OfferAdjournedGame::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination through 00d8d382 (NetworkCommand and LobbyCommand_OfferAdjournedGame RTTI descriptors); a nul… |
| `013d8f00` | LobbyCommand_OfferAdjournedGame::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_OfferAdjournedGame::vftable, calls the base destructor 013b2820, then frees the object with _… |
| `013d8f60` | LobbyCommand_OfferAdjournedGame::Dump | confirmed | Virtual slot [19] (Dump): appends "Offer Adjourned Game Lobby Command\n" through 01241650, calls the base dump 013b2af0 on the output argument, then formats an… |
| `013d9060` | LobbyCommand_OfferAdjournedGame::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x22-byte literal "Offer Adjourned Game Lobby Command" through 01241650 and returns. |

**LobbyCommand_RemoveAdjournedGame** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c28c0` | LobbyCommand_RemoveAdjournedGame::LobbyCommand_RemoveAdjour… | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RemoveAdjournedGame::vftable at +0x0, zeroes the dwords at +0x8 and +0xc, retur… |
| `014c2940` | LobbyCommand_RemoveAdjournedGame::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RemoveAdjournedGame (mov eax, imm32; ret; bytes b8 4c 1e 90 01 c3). |
| `014c2950` | LobbyCommand_RemoveAdjournedGame::ReadGlobal01cbe7a4 | confirmed | No arguments. Returns the 32-bit global at 01cbe7a4 (mov eax,[01cbe7a4]; ret; bytes a1 a4 e7 cb 01 c3). |
| `014c2960` | LobbyCommand_RemoveAdjournedGame::WriteGlobal01cbe7a4 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7a4 (bytes 8b 44 24 04 a3 a4 e7 cb 01 c2 04 00). |
| `014c2b00` | LobbyCommand_RemoveAdjournedGame::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, then reads ints into this+0x8 and this+0xc with 013c24d0; tail-calls vtbl[5] and returns its… |
| `014c2b60` | LobbyCommand_RemoveAdjournedGame::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends int +0x8 (GameID) and int +0xc (Mat… |
| `014c2c60` | LobbyCommand_RemoveAdjournedGame::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores the vtable at +0x0, calls base destructor 013b2820, calls _free on this when flag bit 0 of the argument is… |
| `014c2d80` | LobbyCommand_RemoveAdjournedGame::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Remove Adjourned Game Lobby Command (lengt… |
| `014c2980` | LobbyCommand_RemoveAdjournedGame::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, calls base destructor 013b2820 inside an EH frame, returns. It does not free memory. |
| `014c2a30` | LobbyCommand_RemoveAdjournedGame::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x10 bytes (operator new 00d833a6), constructs with 014c28c0 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c2aa0` | LobbyCommand_RemoveAdjournedGame::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RemoveAdjournedGame (00d8d382); on failure asserts through 012f5a60 with the… |
| `014c2be0` | LobbyCommand_RemoveAdjournedGame::CreateInstance | likely | Static factory: allocates 0x10 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c28c0 and returns it, otherwise returns 0. |
| `014c2c40` | LobbyCommand_RemoveAdjournedGame::RegisterFactory | likely | Registers factory 014c2be0 under key 0xa4 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function point… |
| `014c2cc0` | LobbyCommand_RemoveAdjournedGame::Dump | likely | Virtual slot 19. Appends Remove Adjourned Game Account Command plus newline through 01241650, calls base dump 013b2af0 with the output, then formats MatchID: %… |

**LobbyCommand_RemoveBuddies** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c2460` | LobbyCommand_RemoveBuddies::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, writes the account-id list at this+0x8 thro… |
| `014c2560` | LobbyCommand_RemoveBuddies::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RemoveBuddies (mov eax, imm32; ret; bytes b8 4c 1d 90 01 c3). |
| `014c2570` | LobbyCommand_RemoveBuddies::ReadGlobal01cbe7a0 | confirmed | No arguments. Returns the 32-bit global at 01cbe7a0 (mov eax,[01cbe7a0]; ret; bytes a1 a0 e7 cb 01 c3). |
| `014c2580` | LobbyCommand_RemoveBuddies::WriteGlobal01cbe7a0 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe7a0 (bytes 8b 44 24 04 a3 a0 e7 cb 01 c2 04 00). |
| `014c25a0` | LobbyCommand_RemoveBuddies::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls DtorBody 014c24d0, then calls _free on this when flag bit 0 of the argument is set; returns this. |
| `014c2680` | LobbyCommand_RemoveBuddies::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Remove Buddies Lobby Command (length 0x1c)… |
| `014c26a0` | LobbyCommand_RemoveBuddies::LobbyCommand_RemoveBuddies | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RemoveBuddies::vftable at +0x0, zeroes the dwords at +0xc, +0x10 and +0x14 (an … |
| `014c24d0` | LobbyCommand_RemoveBuddies::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, frees the buffer at +0xc when non-null, zeroes +0xc, +0x10 and +0x14, then calls base de… |
| `014c25c0` | LobbyCommand_RemoveBuddies::Dump | likely | Virtual slot 19. Appends Remove Buddies Lobby Command plus newline to the output through 01241650, calls base dump 013b2af0 with the output, then formats Buddy… |
| `014c2710` | LobbyCommand_RemoveBuddies::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x18 bytes (operator new 00d833a6), constructs with 014c26a0 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c2780` | LobbyCommand_RemoveBuddies::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RemoveBuddies (00d8d382); on failure asserts through 012f5a60 with the text … |
| `014c27e0` | LobbyCommand_RemoveBuddies::CreateInstance | likely | Static factory: allocates 0x18 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c26a0 and returns it, otherwise returns 0. |
| `014c2840` | LobbyCommand_RemoveBuddies::RegisterFactory | likely | Registers factory 014c27e0 under key 0x6d in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function point… |

**LobbyCommand_RemoveGroups** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c1f90` | LobbyCommand_RemoveGroups::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, writes the GroupID list at this+0x8 through… |
| `014c2080` | LobbyCommand_RemoveGroups::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RemoveGroups (mov eax, imm32; ret; bytes b8 04 1c 90 01 c3). |
| `014c2090` | LobbyCommand_RemoveGroups::ReadGlobal01cbe79c | confirmed | No arguments. Returns the 32-bit global at 01cbe79c (mov eax,[01cbe79c]; ret; bytes a1 9c e7 cb 01 c3). |
| `014c20a0` | LobbyCommand_RemoveGroups::WriteGlobal01cbe79c | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe79c (bytes 8b 44 24 04 a3 9c e7 cb 01 c2 04 00). |
| `014c20c0` | LobbyCommand_RemoveGroups::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls DtorBody 014c1ff0, then calls _free on this when flag bit 0 of the argument is set; returns this. |
| `014c21d0` | LobbyCommand_RemoveGroups::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Remove Groups Lobby Command (length 0x1b),… |
| `014c2210` | LobbyCommand_RemoveGroups::LobbyCommand_RemoveGroups | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RemoveGroups::vftable at +0x0, zeroes the three dwords at +0xc, +0x10 and +0x14… |
| `014c23e0` | LobbyCommand_RemoveGroups::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads the group id list at this+0x8 through 01300540, then tail-calls vtbl[5] and returns it… |
| `014c1ff0` | LobbyCommand_RemoveGroups::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, frees the buffer at +0xc when non-null, zeroes +0xc, +0x10 and +0x14, then calls base de… |
| `014c20e0` | LobbyCommand_RemoveGroups::Dump | likely | Virtual slot 19. Appends Remove Groups Lobby Command plus newline to the output through 01241650, calls base dump 013b2af0 with the output, then for each entry… |
| `014c2280` | LobbyCommand_RemoveGroups::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x18 bytes (operator new 00d833a6), constructs with 014c2210 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c22f0` | LobbyCommand_RemoveGroups::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RemoveGroups (00d8d382); on failure asserts through 012f5a60 with the text c… |
| `014c2350` | LobbyCommand_RemoveGroups::CreateInstance | likely | Static factory: allocates 0x18 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c2210 and returns it, otherwise returns 0. |
| `014c23b0` | LobbyCommand_RemoveGroups::RegisterFactory | likely | Registers factory 014c2350 under key 0x5f in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function point… |

**LobbyCommand_RemoveIgnore** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c1980` | LobbyCommand_RemoveIgnore::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, writes the account-id list at this+0x8 thro… |
| `014c1a80` | LobbyCommand_RemoveIgnore::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RemoveIgnore (mov eax, imm32; ret; bytes b8 ac 1a 90 01 c3). |
| `014c1a90` | LobbyCommand_RemoveIgnore::ReadGlobal01cbe798 | confirmed | No arguments. Returns the 32-bit global at 01cbe798 (mov eax,[01cbe798]; ret; bytes a1 98 e7 cb 01 c3). |
| `014c1aa0` | LobbyCommand_RemoveIgnore::WriteGlobal01cbe798 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe798 (bytes 8b 44 24 04 a3 98 e7 cb 01 c2 04 00). |
| `014c1ac0` | LobbyCommand_RemoveIgnore::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0] shape): calls DtorBody 014c19f0, then calls _free on this when flag bit 0 of the argument is set; returns this. |
| `014c1ba0` | LobbyCommand_RemoveIgnore::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Remove Ignore Lobby Command (length 0x1b),… |
| `014c1bc0` | LobbyCommand_RemoveIgnore::LobbyCommand_RemoveIgnore | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RemoveIgnore::vftable at +0x0, zeroes the three dwords at +0xc, +0x10 and +0x14… |
| `014c19f0` | LobbyCommand_RemoveIgnore::DtorBody | likely | Non-deleting destructor body with ECX=this: stores the vtable at +0x0, frees the buffer at +0xc when non-null, zeroes +0xc, +0x10 and +0x14, then calls base de… |
| `014c1ae0` | LobbyCommand_RemoveIgnore::Dump | likely | Virtual slot 19. Appends Remove Ignore Lobby Command plus newline to the output through 01241650, calls base dump 013b2af0 with the output, then for each entry… |
| `014c1c30` | LobbyCommand_RemoveIgnore::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x18 bytes (operator new 00d833a6), constructs with 014c1bc0 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c1ca0` | LobbyCommand_RemoveIgnore::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RemoveIgnore (00d8d382); on failure asserts through 012f5a60 with the text c… |
| `014c1d00` | LobbyCommand_RemoveIgnore::CreateInstance | likely | Static factory: allocates 0x18 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c1bc0 and returns it, otherwise returns 0. |
| `014c1d60` | LobbyCommand_RemoveIgnore::RegisterFactory | likely | Registers factory 014c1d00 under key 0x6f in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function point… |

**LobbyCommand_RequestAddBuddy** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c14a0` | LobbyCommand_RequestAddBuddy::LobbyCommand_RequestAddBuddy | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RequestAddBuddy::vftable at +0x0, zeroes the int at +0x8, returns this. |
| `014c1520` | LobbyCommand_RequestAddBuddy::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RequestAddBuddy (mov eax, imm32; ret; bytes b8 f4 18 90 01 c3). |
| `014c1530` | LobbyCommand_RequestAddBuddy::ReadGlobal01cbe794 | confirmed | No arguments. Returns the 32-bit global at 01cbe794 (mov eax,[01cbe794]; ret; bytes a1 94 e7 cb 01 c3). |
| `014c1540` | LobbyCommand_RequestAddBuddy::WriteGlobal01cbe794 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe794 (bytes 8b 44 24 04 a3 94 e7 cb 01 c2 04 00). |
| `014c1720` | LobbyCommand_RequestAddBuddy::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads one int into this+0x8 with 013c24d0, then tail-calls vtbl[5] and returns its result. R… |
| `014c1770` | LobbyCommand_RequestAddBuddy::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends int +0x8 with 013c23f0 under the la… |
| `014c1850` | LobbyCommand_RequestAddBuddy::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores RequestAddBuddy vtable at +0x0, calls base destructor 013b2820, calls _free on this when flag bit 0 of the… |
| `014c1940` | LobbyCommand_RequestAddBuddy::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Request Add Buddy Lobby Command (length 0x… |
| `014c1560` | LobbyCommand_RequestAddBuddy::DtorBody | likely | Non-deleting destructor body with ECX=this: stores RequestAddBuddy vtable at +0x0, calls base destructor 013b2820, returns. It does not free memory. |
| `014c1660` | LobbyCommand_RequestAddBuddy::Clone | likely | Virtual slot 17, ECX=this. Allocates 0xc bytes (operator new 00d833a6), constructs with 014c14a0 when the allocation succeeded, then calls this vtbl[18] (014c1… |
| `014c16d0` | LobbyCommand_RequestAddBuddy::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RequestAddBuddy (00d8d382); on failure asserts through 012f5a60 with the tex… |
| `014c17d0` | LobbyCommand_RequestAddBuddy::CreateInstance | likely | Static factory: allocates 0xc bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c14a0 and returns it, otherwise returns 0. |
| `014c1830` | LobbyCommand_RequestAddBuddy::RegisterFactory | likely | Registers factory 014c17d0 under key 0x11b in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c18b0` | LobbyCommand_RequestAddBuddy::Dump | likely | Virtual slot 19. Appends Request Add Buddy Status Lobby Command plus newline to the output through 01241650, calls base dump 013b2af0 with the output, then for… |

**LobbyCommand_RespondAddBuddy** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c0eb0` | LobbyCommand_RespondAddBuddy::LobbyCommand_RespondAddBuddy | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_RespondAddBuddy::vftable at +0x0, zeroes the ints at +0x8, +0xc and +0x10, retu… |
| `014c0f30` | LobbyCommand_RespondAddBuddy::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_RespondAddBuddy (mov eax, imm32; ret; bytes b8 54 17 90 01 c3). |
| `014c0f40` | LobbyCommand_RespondAddBuddy::ReadGlobal01cbe790 | confirmed | No arguments. Returns the 32-bit global at 01cbe790 (mov eax,[01cbe790]; ret; bytes a1 90 e7 cb 01 c3). |
| `014c0f50` | LobbyCommand_RespondAddBuddy::WriteGlobal01cbe790 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe790 (bytes 8b 44 24 04 a3 90 e7 cb 01 c2 04 00). |
| `014c11a0` | LobbyCommand_RespondAddBuddy::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, then reads ints into this+0x8, +0xc and +0x10 with 013c24d0; tail-calls vtbl[5] and returns … |
| `014c1210` | LobbyCommand_RespondAddBuddy::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends ints +0x8, +0xc and +0x10 with 013c… |
| `014c1320` | LobbyCommand_RespondAddBuddy::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores RespondAddBuddy vtable at +0x0, calls base destructor 013b2820, calls _free on this when flag bit 0 of the… |
| `014c1480` | LobbyCommand_RespondAddBuddy::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Respond Add Buddy Lobby Command (length 0x… |
| `014c0f70` | LobbyCommand_RespondAddBuddy::DtorBody | likely | Non-deleting destructor body with ECX=this: stores RespondAddBuddy vtable at +0x0, calls base destructor 013b2820, returns. It does not free memory. |
| `014c10d0` | LobbyCommand_RespondAddBuddy::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x14 bytes (operator new 00d833a6), constructs with 014c0eb0 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c1140` | LobbyCommand_RespondAddBuddy::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_RespondAddBuddy (00d8d382); on failure asserts through 012f5a60 with the tex… |
| `014c12a0` | LobbyCommand_RespondAddBuddy::CreateInstance | likely | Static factory: allocates 0x14 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c0eb0 and returns it, otherwise returns 0. |
| `014c1300` | LobbyCommand_RespondAddBuddy::RegisterFactory | likely | Registers factory 014c12a0 under key 0x11c in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c1380` | LobbyCommand_RespondAddBuddy::Dump | likely | Virtual slot 19. Appends Respond Add Buddy Lobby Command plus newline to the output through 01241650, calls base dump 013b2af0 with the output, then formats an… |

**LobbyCommand_SelectCard** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c0a60` | LobbyCommand_SelectCard::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_SelectCard (mov eax, imm32; ret; bytes b8 2c 16 90 01 c3). |
| `014c0a70` | LobbyCommand_SelectCard::ReadGlobal01cbe78c | confirmed | No arguments. Returns the 32-bit global at 01cbe78c (mov eax,[01cbe78c]; ret; bytes a1 8c e7 cb 01 c3). |
| `014c0a80` | LobbyCommand_SelectCard::WriteGlobal01cbe78c | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe78c (bytes 8b 44 24 04 a3 8c e7 cb 01 c2 04 00). |
| `014c0b70` | LobbyCommand_SelectCard::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, then reads integers into this+0x8, this+0x14, this+0xc and this+0x10 in that order with 013c… |
| `014c0bf0` | LobbyCommand_SelectCard::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends ints +0x8, +0x14, +0xc and +0x10 wi… |
| `014c0c90` | LobbyCommand_SelectCard::LobbyCommand_SelectCard | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_SelectCard::vftable at +0x0, zeroes the ints at +0x8, +0xc, +0x10 and +0x14, re… |
| `014c0cf0` | LobbyCommand_SelectCard::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_SelectCard::vftable at +0x0, calls base destructor 013b2820, calls _free on this when flag bi… |
| `014c0e90` | LobbyCommand_SelectCard::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument (mov ecx,[esp+4]) and calls 01241650 with the literal SelectCard Lobby Command… |
| `014c09f0` | LobbyCommand_SelectCard::DtorBody | likely | Non-deleting destructor body with ECX=this: stores LobbyCommand_SelectCard::vftable at +0x0, calls base destructor 013b2820 and returns. It does not free memor… |
| `014c0b10` | LobbyCommand_SelectCard::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_SelectCard (00d8d382); on failure asserts through 012f5a60 with the text clo… |
| `014c0d70` | LobbyCommand_SelectCard::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x18 bytes (operator new 00d833a6), constructs with 014c0c90 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c0de0` | LobbyCommand_SelectCard::CreateInstance | likely | Static factory: allocates 0x18 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c0c90 and returns it, otherwise returns 0. |
| `014c0e40` | LobbyCommand_SelectCard::RegisterFactory | likely | Registers factory 014c0de0 under key 0x130 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c0e60` | LobbyCommand_SelectCard::Dump | likely | Virtual slot 19, one stack argument (the output std::string). Appends the text Select Card Lobby Command plus newline to the output through 01241650, then call… |

**LobbyCommand_SelectTournamentDeck** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014c0110` | LobbyCommand_SelectTournamentDeck::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes trace literals to no-op 013c26e0, calls base writer 013b2960, appends the DeckID string at this+0x8 with … |
| `014c01f0` | LobbyCommand_SelectTournamentDeck::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads a string into this+0x8 with 013c28d0 (SerialReadString), reads an optional object into… |
| `014c0290` | LobbyCommand_SelectTournamentDeck::LobbyCommand_SelectTourn… | confirmed | Constructor with ECX=this: calls base constructor 013b2760, stores LobbyCommand_SelectTournamentDeck::vftable at +0x0, sets the string at +0xc to empty (first … |
| `014c0330` | LobbyCommand_SelectTournamentDeck::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_SelectTournamentDeck (mov eax, imm32; ret; bytes b8 5c 15 90 01 c3). |
| `014c0340` | LobbyCommand_SelectTournamentDeck::ReadGlobal01cbe788 | confirmed | No arguments. Returns the 32-bit global at 01cbe788 (mov eax,[01cbe788]; ret; bytes a1 88 e7 cb 01 c3). |
| `014c0350` | LobbyCommand_SelectTournamentDeck::WriteGlobal01cbe788 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe788 (bytes 8b 44 24 04 a3 88 e7 cb 01 c2 04 00). |
| `014c04f0` | LobbyCommand_SelectTournamentDeck::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls DtorBody 014c0370, then calls _free on this when flag bit 0 of the argument is set; returns this. |
| `014c0710` | LobbyCommand_SelectTournamentDeck::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Select Deck Tournament Lobby Command (leng… |
| `014c0370` | LobbyCommand_SelectTournamentDeck::DtorBody | likely | Non-deleting destructor body (also reached from an EH unwind funclet): stores the vtable at +0x0; when the pointer at +0x24 is non-null calls its scalar deleti… |
| `014c0400` | LobbyCommand_SelectTournamentDeck::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x34 bytes (operator new 00d833a6), constructs with 014c0290 when the allocation succeeded, then calls this vtbl[18] (014c… |
| `014c0470` | LobbyCommand_SelectTournamentDeck::CreateInstance | likely | Static factory: allocates 0x34 bytes (operator new 00d833a6); if the allocation succeeded, constructs with 014c0290 and returns it, otherwise returns 0. |
| `014c04d0` | LobbyCommand_SelectTournamentDeck::RegisterFactory | likely | Registers factory 014c0470 under key 0x133 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014c0510` | LobbyCommand_SelectTournamentDeck::SetDeckObject | likely | Thiscall with one stack argument (param_2, an object with a vtable). Asserts param_2 is non-null through 012f5a60 (file LobbyCommandSelectTournamentDeck.cpp, l… |
| `014c0580` | LobbyCommand_SelectTournamentDeck::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_SelectTournamentDeck (00d8d382); on failure asserts through 012f5a60 with th… |
| `014c0600` | LobbyCommand_SelectTournamentDeck::Dump | likely | Virtual slot 19. Appends Select Deck Tournament Lobby Command plus newline to the output string through 01241650, then calls base dump 013b2af0 on this with th… |

**LobbyCommand_SendPing** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bfb10` | LobbyCommand_SendPing::LobbyCommand_SendPing | confirmed | Constructor with ECX=this: calls base constructor 013b2760 on this, stores LobbyCommand_SendPing::vftable at +0x0, zeroes the ints at +0x8 and +0xc, returns th… |
| `014bfb90` | LobbyCommand_SendPing::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_SendPing (mov eax, imm32; ret; bytes b8 54 13 90 01 c3). |
| `014bfba0` | LobbyCommand_SendPing::ReadGlobal01cbe784 | confirmed | No arguments. Returns the 32-bit global at 01cbe784 (mov eax,[01cbe784]; ret; bytes a1 84 e7 cb 01 c3). |
| `014bfbb0` | LobbyCommand_SendPing::WriteGlobal01cbe784 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe784 (mov eax,[esp+4]; mov [01cbe784],eax; ret 4). |
| `014bfdc0` | LobbyCommand_SendPing::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read), base reader 013b2910, reads int +0x8 with 013c24d0; when an unresolved register value (ESI) is above 1 it also rea… |
| `014bfe20` | LobbyCommand_SendPing::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), passes the strings Starting LobbyCommand_SendPing and the field labels to 013c26e0 (a no-op that returns 1, so t… |
| `014bff20` | LobbyCommand_SendPing::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_SendPing::vftable at +0x0, calls base destructor 013b2820, calls _free on this when the flag … |
| `014c0040` | LobbyCommand_SendPing::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument (mov ecx,[esp+4]) and calls 01241650 with the literal Send Ping Lobby Command … |
| `014bfbd0` | LobbyCommand_SendPing::DtorBody | likely | Non-deleting destructor body with ECX=this inside an SEH frame: stores LobbyCommand_SendPing::vftable at +0x0, calls base destructor 013b2820, returns. It does… |
| `014bfcf0` | LobbyCommand_SendPing::Clone | likely | Virtual slot 17, ECX=this. Allocates 0x10 bytes (operator new), constructs a LobbyCommand_SendPing with 014bfb10, then calls this vtbl[18] (014bfd60, CopyState… |
| `014bfd60` | LobbyCommand_SendPing::CopyStateTo | likely | Virtual slot 18, one argument (target). dynamic_casts the argument to LobbyCommand_SendPing (00d8d382); on failure asserts through 012f5a60 with the text clone… |
| `014bfea0` | LobbyCommand_SendPing::CreateInstance | likely | Static factory: allocates 0x10 bytes (operator new 00d833a6); if the allocation succeeded, constructs the object with 014bfb10 and returns it, otherwise return… |
| `014bff00` | LobbyCommand_SendPing::RegisterFactory | likely | Registers factory 014bfea0 under key 0x118 in the factory registry: calls 013ce130 (registry lazy-create) and then 013ce230 with the same key and function poin… |
| `014bff80` | LobbyCommand_SendPing::Dump | likely | Virtual slot 19. Appends the text Send Ping Status Lobby Command plus newline to the output through 01241650, then calls base dump 013b2af0 with the output; fo… |

**LobbyCommand_SendUChatError** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bf630` | LobbyCommand_SendUChatError::LobbyCommand_SendUChatError | confirmed | Constructor with ECX=this: calls base constructor 013b2760 on this, stores LobbyCommand_SendUChatError::vftable at +0x0, zeroes the int at +0x8, returns this. |
| `014bf6b0` | LobbyCommand_SendUChatError::TypeName | confirmed | No arguments. Returns the address of the literal LobbyCommand_SendUChatError (mov eax, imm32; ret), verified from bytes b8 4c 12 90 01 c3. |
| `014bf6c0` | LobbyCommand_SendUChatError::ReadGlobal01cbe780 | confirmed | No arguments. Returns the 32-bit global at 01cbe780 (mov eax,[01cbe780]; ret; bytes a1 80 e7 cb 01 c3). |
| `014bf6d0` | LobbyCommand_SendUChatError::WriteGlobal01cbe780 | confirmed | One stack argument, stdcall ret 4. Stores the argument into the global at 01cbe780 (mov eax,[esp+4]; mov [01cbe780],eax; ret 4). |
| `014bf8e0` | LobbyCommand_SendUChatError::Read | confirmed | Virtual slot 9. Calls vtbl[4] (header read into a local), then base reader 013b2910, then reads one int with 013c24d0 into a local, stores it at this+0x8, and … |
| `014bf940` | LobbyCommand_SendUChatError::Write | confirmed | Virtual slot 10. Calls vtbl[2] (header write), base writer 013b2960, appends int this+0x8 with 013c23f0 (SerialAppendInteger), then tail-calls vtbl[3]. |
| `014bfa00` | LobbyCommand_SendUChatError::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores LobbyCommand_SendUChatError::vftable at +0x0, calls base destructor 013b2820, then calls _free on this whe… |
| `014bfaf0` | LobbyCommand_SendUChatError::DescribeName | confirmed | Virtual slot 20, one stack argument (a std::string). Loads ECX from the argument and calls 01241650 with the literal Send UChat Error Lobby Command (length 0x1… |
| `014bf6f0` | LobbyCommand_SendUChatError::DtorBody | likely | Non-deleting destructor body with ECX=this inside an SEH frame: stores LobbyCommand_SendUChatError::vftable at +0x0, calls base destructor 013b2820, restores t… |
| `014bf820` | LobbyCommand_SendUChatError::Clone | likely | Virtual slot 17, ECX=this. Allocates 0xc bytes (operator new), constructs a LobbyCommand_SendUChatError with 014bf630, then calls this vtbl[18] (014bf890, Copy… |
| `014bf890` | LobbyCommand_SendUChatError::CopyStateTo | likely | Virtual slot 18, one argument (the target NetworkCommand). dynamic_casts the argument to LobbyCommand_SendUChatError (00d8d382); on failure asserts through 012… |
| `014bf980` | LobbyCommand_SendUChatError::CreateInstance | likely | Static factory: allocates 0xc bytes (operator new 00d833a6); if the allocation succeeded, constructs the object with 014bf630 and returns it, otherwise returns… |
| `014bf9e0` | LobbyCommand_SendUChatError::RegisterFactory | likely | Registers factory 014bf980 under key 0x1bd in the factory registry: calls 013ce130 (registry lazy-create, which allocates 0x2c bytes once) and then 013ce230 wi… |
| `014bfa60` | LobbyCommand_SendUChatError::Dump | likely | Virtual slot 19. Appends the text 'Send UChat Error Lobby Command' with a newline to the output string argument through 01241650, calls base dump 013b2af0 with… |

**LobbyCommand_SendUChatText** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b47c0` | LobbyCommand_SendUChatText::GetTypeName | confirmed | Returns the address of the literal "LobbyCommand_SendUChatText" (mov eax,0x018dfb2c; ret). No arguments, no state read. Vtable slot [11] of LobbyCommand_SendUC… |
| `013b4800` | LobbyCommand_SendUChatText::~LobbyCommand_SendUChatText | confirmed | Stores LobbyCommand_SendUChatText::vftable at [this], then for five std::string members (buffer pointer at this+0x0c, +0x28, +0x44, +0x60, +0x7c; size at ptr+0… |
| `013b4a10` | LobbyCommand_SendUChatText::DeletingDtor | confirmed | Calls dtor body 013b4800 on this, then if bit 0 of the first stack argument is set calls _free(this); returns this (ret 4). Scalar deleting destructor shape. |
| `013b4ad0` | LobbyCommand_SendUChatText::CopyFieldsTo | confirmed | Takes the destination object as its one stack argument. dynamic_cast from NetworkCommand to LobbyCommand_SendUChatText (00d8d382, RTTI descriptors 01bbacc4 and… |
| `013b4b80` | LobbyCommand_SendUChatText::TraceDump | confirmed | Appends text to the sink std::string passed as its one stack argument (edi; each call sets ECX=sink before 01241650). Appends the header 'Send UChat Text Lobby… |
| `013b4cf0` | LobbyCommand_SendUChatText::AppendHeader | confirmed | Loads the first stack argument (the sink) into ECX, pushes 0x1d and the literal "Send UChat Text Lobby Command", calls std::string::append (01241650) on the si… |

**LobbyCommand_SetDraft** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014be840` | LobbyCommand_SetDraft::DestroyMembers | confirmed | Stores LobbyCommand_SetDraft::vftable, calls member cleanup 012c2de0 and base cleanup 013b2820. |
| `014be8c0` | LobbyCommand_SetDraft::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetDraft'. |
| `014be900` | LobbyCommand_SetDraft::~LobbyCommand_SetDraft | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014be840, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014bf2c0` | LobbyCommand_SetDraft::Dump | confirmed | Appends 'Change Lobby display Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bf2f0` | LobbyCommand_SetDraft::Describe | confirmed | Takes no arguments and appends the 22-character literal 'SetDraft Lobby Command' through 01241650. |
| `014bf310` | LobbyCommand_SetDraft::LobbyCommand_SetDraft | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetDraft::vftable, zeroes the dwords at this+8, +0xc, +0x10, +0x14, +0x1c, +0x20, +0x24, and the byte at +… |
| `014bf390` | LobbyCommand_SetDraft::Clone | confirmed | Allocates 0x2c bytes through 00d833a6; if non-null constructs with 014bf310; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bf400` | LobbyCommand_SetDraft::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetDraft (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetDraft.cpp line 0x112 on failure, call… |
| `014bf480` | LobbyCommand_SetDraft::Create | likely | Allocates 0x2c bytes through 00d833a6 and, if non-null, constructs with 014bf310 and returns it; returns 0 otherwise. |
| `014bf4e0` | LobbyCommand_SetDraft::RegisterFactory | likely | Registers creator 014bf480 under key 0x12f with 013ce130 and 013ce230. |

**LobbyCommand_SetDraftReady** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bdde0` | LobbyCommand_SetDraftReady::LobbyCommand_SetDraftReady | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetDraftReady::vftable, and zeroes the dwords at this+8, +0xc and +0x10. Returns this. |
| `014bde60` | LobbyCommand_SetDraftReady::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetDraftReady'. |
| `014bdea0` | LobbyCommand_SetDraftReady::DestroyBody | confirmed | Stores LobbyCommand_SetDraftReady::vftable and calls base cleanup 013b2820 through a local unwind record; releases no members. |
| `014bdf60` | LobbyCommand_SetDraftReady::Clone | confirmed | Allocates 0x14 bytes through 00d833a6; if non-null constructs with 014bdde0; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bdfd0` | LobbyCommand_SetDraftReady::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetDraftReady (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetDraftReady.cpp line 0x42 on fail… |
| `014be1b0` | LobbyCommand_SetDraftReady::~LobbyCommand_SetDraftReady | confirmed | Scalar deleting destructor in vftable slot 0: stores the SetDraftReady vftable, calls base cleanup 013b2820, then frees the block through _free when bit 0 of t… |
| `014be210` | LobbyCommand_SetDraftReady::Dump | confirmed | Appends 'Select Card Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014be130` | LobbyCommand_SetDraftReady::Create | likely | Allocates 0x14 bytes through 00d833a6 and, if non-null, constructs with 014bdde0 and returns it; returns 0 otherwise. |
| `014be190` | LobbyCommand_SetDraftReady::RegisterFactory | likely | Registers creator 014be130 under key 0x131 with 013ce130 and 013ce230. |

**LobbyCommand_SetDraftedCards** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014be380` | LobbyCommand_SetDraftedCards::Dump | confirmed | Appends 'LobbyCommand_SetDraftedCards' with a newline through 01241650, then calls base dump 013b2af0. |
| `014be3b0` | LobbyCommand_SetDraftedCards::Describe | confirmed | Takes no arguments and appends the 29-character literal 'SetDraftedCards Lobby Command' through 01241650. |
| `014be3d0` | LobbyCommand_SetDraftedCards::DestroyMembers | confirmed | Stores LobbyCommand_SetDraftedCards::vftable, calls member cleanup 00521a9c (the container at this+0xc) and base cleanup 013b2820. |
| `014be450` | LobbyCommand_SetDraftedCards::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetDraftedCards'. |
| `014be4b0` | LobbyCommand_SetDraftedCards::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetDraftedCards (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetDraftedCards.cpp line 0x4c on … |
| `014be590` | LobbyCommand_SetDraftedCards::LobbyCommand_SetDraftedCards | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetDraftedCards::vftable, calls member constructor 005258fb, and zeroes the dword at this+8. Returns this. |
| `014be600` | LobbyCommand_SetDraftedCards::~LobbyCommand_SetDraftedCards | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014be3d0, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014be620` | LobbyCommand_SetDraftedCards::Clone | confirmed | Allocates 0x18 bytes through 00d833a6; if non-null constructs with 014be590; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014be270` | LobbyCommand_SetDraftedCards::PostEvent | likely | Passes this+0xc (the card list) to the singleton from 012d46a0 through 012d3ab0, allocates 0x14 bytes, creates the block with 012f9eb0, sets type tag 0xb4 thro… |
| `014be690` | LobbyCommand_SetDraftedCards::Create | likely | Allocates 0x18 bytes through 00d833a6 and, if non-null, constructs with 014be590 and returns it; returns 0 otherwise. |
| `014be6f0` | LobbyCommand_SetDraftedCards::RegisterFactory | likely | Registers creator 014be690 under key 0x1cf with 013ce130 and 013ce230. |

**LobbyCommand_SetFlag** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bd9d0` | LobbyCommand_SetFlag::LobbyCommand_SetFlag | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetFlag::vftable, and zeroes the dword at this+8. Returns this. |
| `014bda50` | LobbyCommand_SetFlag::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetFlag'. |
| `014bda90` | LobbyCommand_SetFlag::DestroyBody | confirmed | Stores LobbyCommand_SetFlag::vftable and calls base cleanup 013b2820 through a local unwind record; releases no members. |
| `014bdb50` | LobbyCommand_SetFlag::Clone | confirmed | Allocates 0xc bytes through 00d833a6; if non-null constructs with 014bd9d0; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bdbc0` | LobbyCommand_SetFlag::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetFlag (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetFlag.cpp line 0x35 on failure, calls b… |
| `014bdd30` | LobbyCommand_SetFlag::~LobbyCommand_SetFlag | confirmed | Scalar deleting destructor in vftable slot 0: stores the SetFlag vftable, calls base cleanup 013b2820, then frees the block through _free when bit 0 of the fla… |
| `014bdd90` | LobbyCommand_SetFlag::Dump | confirmed | Appends 'Set Flag Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bdb00` | LobbyCommand_SetFlag::ApplyFlag | likely | Reads the flag id at this+8: id 1 calls the singleton from 012d46a0 and 012d3690(1, 0); id 2 calls 012d46a0 and 012d3690(0, 0); ids 3 and 4 set the byte at sin… |
| `014bdcb0` | LobbyCommand_SetFlag::Create | likely | Allocates 0xc bytes through 00d833a6 and, if non-null, constructs with 014bd9d0 and returns it; returns 0 otherwise. |
| `014bdd10` | LobbyCommand_SetFlag::RegisterFactory | likely | Registers creator 014bdcb0 under key 0x16c with 013ce130 and 013ce230. |

**LobbyCommand_SetLeagueCards** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bd4c0` | LobbyCommand_SetLeagueCards::Dump | confirmed | Appends 'SetLeagueCards Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bd4f0` | LobbyCommand_SetLeagueCards::Describe | confirmed | Takes no arguments and appends the 28-character literal 'SetLeagueCards Lobby Command' through 01241650. |
| `014bd680` | LobbyCommand_SetLeagueCards::DestroyBody | confirmed | Stores LobbyCommand_SetLeagueCards::vftable, calls member cleanup 01247310 and base cleanup 013b2820. |
| `014bd700` | LobbyCommand_SetLeagueCards::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetLeagueCards'. |
| `014bd740` | LobbyCommand_SetLeagueCards::LobbyCommand_SetLeagueCards | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetLeagueCards::vftable, calls member constructor 01247380, and zeroes the dword at this+8. Returns this. |
| `014bd7b0` | LobbyCommand_SetLeagueCards::~LobbyCommand_SetLeagueCards | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014bd680, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014bd7d0` | LobbyCommand_SetLeagueCards::Clone | confirmed | Allocates 0x18 bytes through 00d833a6; if non-null constructs with 014bd740; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bd940` | LobbyCommand_SetLeagueCards::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetLeagueCards (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetLeagueCards.cpp line 0x39 on fa… |
| `014bd840` | LobbyCommand_SetLeagueCards::Create | likely | Allocates 0x18 bytes through 00d833a6 and, if non-null, constructs with 014bd740 and returns it; returns 0 otherwise. |
| `014bd8a0` | LobbyCommand_SetLeagueCards::RegisterFactory | likely | Registers creator 014bd840 under key 0x14f with 013ce130 and 013ce230. |

**LobbyCommand_SetLeagueCollection** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bcdd0` | LobbyCommand_SetLeagueCollection::DestroyBody | confirmed | Stores LobbyCommand_SetLeagueCollection::vftable, calls member cleanup 012c2de0 and the base cleanup 013b2820. |
| `014bce50` | LobbyCommand_SetLeagueCollection::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetLeagueCollection'. |
| `014bce90` | LobbyCommand_SetLeagueCollection::~LobbyCommand_SetLeagueCo… | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014bcdd0, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014bd1b0` | LobbyCommand_SetLeagueCollection::Dump | confirmed | Appends 'Set League Collection Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bd200` | LobbyCommand_SetLeagueCollection::LobbyCommand_SetLeagueCol… | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetLeagueCollection::vftable, zeroes the dwords at this+0x14, +0x18, +0x1c, +8 and +0xc. Returns this. |
| `014bd270` | LobbyCommand_SetLeagueCollection::Clone | confirmed | Allocates 0x20 bytes through 00d833a6; if non-null constructs with 014bd200; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bd2e0` | LobbyCommand_SetLeagueCollection::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetLeagueCollection (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetLeagueCollection.cpp line … |
| `014bceb0` | LobbyCommand_SetLeagueCollection::ApplyCollection | likely | Looks up the 'leagueCollection' entry with 012ae1a0; when absent, allocates 0x90 bytes, constructs with 013a2630 and registers it (013a1780, 0139f7e0 with 4). … |
| `014bd340` | LobbyCommand_SetLeagueCollection::Create | likely | Allocates 0x20 bytes through 00d833a6 and, if non-null, constructs with 014bd200 and returns it; returns 0 otherwise. |
| `014bd3a0` | LobbyCommand_SetLeagueCollection::RegisterFactory | likely | Registers creator 014bd340 under key 0x15a with 013ce130 and 013ce230. |

**LobbyCommand_SetLeagueInfo** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bc620` | LobbyCommand_SetLeagueInfo::Dump | confirmed | Appends 'Set League Info Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bc650` | LobbyCommand_SetLeagueInfo::Describe | confirmed | Takes no arguments and appends the 29-character literal 'Set League Info Lobby Command' through 01241650. |
| `014bc670` | LobbyCommand_SetLeagueInfo::DestroyMembers | confirmed | Stores LobbyCommand_SetLeagueInfo::vftable, frees the buffer at this+0x40 when non-null and zeroes this+0x40..+0x48, then calls two member cleanups 00521a9c an… |
| `014bc720` | LobbyCommand_SetLeagueInfo::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetLeagueInfo'. |
| `014bc7a0` | LobbyCommand_SetLeagueInfo::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetLeagueInfo (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetLeagueInfo.cpp line 0x7a on fail… |
| `014bc880` | LobbyCommand_SetLeagueInfo::LobbyCommand_SetLeagueInfo | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetLeagueInfo::vftable, calls member constructor 005258fb twice, zeroes the buffer triple at this+0x40..+0… |
| `014bc920` | LobbyCommand_SetLeagueInfo::~LobbyCommand_SetLeagueInfo | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014bc670, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014bcb30` | LobbyCommand_SetLeagueInfo::Clone | confirmed | Allocates 0x4c bytes through 00d833a6; if non-null constructs with 014bc880; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bcc90` | LobbyCommand_SetLeagueInfo::Create | likely | Allocates 0x4c bytes through 00d833a6 and, if non-null, constructs with 014bc880 and returns it; returns 0 otherwise. |
| `014bccf0` | LobbyCommand_SetLeagueInfo::RegisterFactory | likely | Registers creator 014bcc90 under key 0x150 with 013ce130 and 013ce230. |

**LobbyCommand_SetTournament** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bbfe0` | LobbyCommand_SetTournament::LobbyCommand_SetTournament | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetTournament::vftable, and zeroes the dwords at this+8, +0xc, +0x10 and +0x14. Returns this. |
| `014bc060` | LobbyCommand_SetTournament::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_SetTournament'. |
| `014bc0a0` | LobbyCommand_SetTournament::DestroyBody | confirmed | Stores LobbyCommand_SetTournament::vftable and calls base cleanup 013b2820 through a local unwind record; releases no members of its own. |
| `014bc150` | LobbyCommand_SetTournament::Clone | confirmed | Allocates 0x18 bytes through 00d833a6; if non-null constructs with 014bbfe0; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bc1c0` | LobbyCommand_SetTournament::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetTournament (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetTournament.cpp line 0x3b on fail… |
| `014bc3d0` | LobbyCommand_SetTournament::~LobbyCommand_SetTournament | confirmed | Scalar deleting destructor in vftable slot 0: stores the SetTournament vftable, calls base cleanup 013b2820, then frees the block through _free when bit 0 of t… |
| `014bc430` | LobbyCommand_SetTournament::Dump | confirmed | Appends 'Change Lobby display Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bc130` | LobbyCommand_SetTournament::ApplyTournament | likely | Does not build an event: loads the dwords at this+8, +0x10, +0xc and +0x14, pushes them as four arguments, takes the singleton from 012d46a0 (which ignores the… |
| `014bc350` | LobbyCommand_SetTournament::Create | likely | Allocates 0x18 bytes through 00d833a6 and, if non-null, constructs with 014bbfe0 and returns it; returns 0 otherwise. |
| `014bc3b0` | LobbyCommand_SetTournament::RegisterFactory | likely | Registers creator 014bc350 under key 299 with 013ce130 and 013ce230. |

**LobbyCommand_SetupDraft** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bbb40` | LobbyCommand_SetupDraft::~LobbyCommand_SetupDraft | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014bba50, then frees the block through _free when bit 0 of the flag is set; returns this and p… |
| `014bbb60` | LobbyCommand_SetupDraft::Dump | confirmed | Appends 'SetupDraft Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bbb90` | LobbyCommand_SetupDraft::Describe | confirmed | Takes no arguments and appends the 24-character literal 'SetupDraft Lobby Command' through 01241650. |
| `014bbbb0` | LobbyCommand_SetupDraft::LobbyCommand_SetupDraft | confirmed | Calls base constructor 013b2760, stores LobbyCommand_SetupDraft::vftable, zeroes the buffers at this+0x10..+0x18 and this+0x20..+0x28 (three dwords each), and … |
| `014bbde0` | LobbyCommand_SetupDraft::Clone | confirmed | Allocates 0x30 bytes through 00d833a6; if non-null constructs with 014bbbb0; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bbe50` | LobbyCommand_SetupDraft::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_SetupDraft (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandSetupDraft.cpp line 0x55 on failure, c… |
| `014bbc30` | LobbyCommand_SetupDraft::PostEvent | likely | Builds two client events and queues both on the singleton from 012d46a0 with 012d3550: first a block with type tag 0x53 (set through 012f8c70) carrying the sub… |
| `014bbec0` | LobbyCommand_SetupDraft::Create | likely | Allocates 0x30 bytes through 00d833a6 and, if non-null, constructs with 014bbbb0 and returns it; returns 0 otherwise. |
| `014bbf20` | LobbyCommand_SetupDraft::RegisterFactory | likely | Registers creator 014bbec0 under key 0x132 with 013ce130 and 013ce230. |

**LobbyCommand_StartTournamentMsg** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bb070` | LobbyCommand_StartTournamentMsg::LobbyCommand_StartTourname… | confirmed | Calls base constructor 013b2760, stores LobbyCommand_StartTournamentMsg::vftable, and zeroes the dword at this+0xc. Returns this. |
| `014bb0f0` | LobbyCommand_StartTournamentMsg::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_StartTournamentMsg'. |
| `014bb130` | LobbyCommand_StartTournamentMsg::DestroyBody | confirmed | Stores LobbyCommand_StartTournamentMsg::vftable and calls base cleanup 013b2820 through a local unwind record; no members of its own are released. |
| `014bb240` | LobbyCommand_StartTournamentMsg::Clone | confirmed | Allocates 0x10 bytes through 00d833a6; if non-null constructs with 014bb070; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bb2b0` | LobbyCommand_StartTournamentMsg::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_StartTournamentMsg (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandStartTournamentMsg.cpp line 0x… |
| `014bb440` | LobbyCommand_StartTournamentMsg::~LobbyCommand_StartTournam… | confirmed | Scalar deleting destructor in vftable slot 0: stores the StartTournamentMsg vftable, calls base cleanup 013b2820, then frees the block through _free when bit 0… |
| `014bb4a0` | LobbyCommand_StartTournamentMsg::Dump | confirmed | Appends the literal 'Start Tournament Message' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bb4d0` | LobbyCommand_StartTournamentMsg::Describe | confirmed | Takes no arguments and appends the 38-character literal 'Start Tournament Message Lobby Command' through 01241650. |
| `014bb1a0` | LobbyCommand_StartTournamentMsg::PostEvent | likely | Allocates 0x14 bytes, creates the block with 012f9eb0, sets type tag 0xb3 through 012f8c70, appends the int fields at this+8 and this+0xc through 012fa2e0, tak… |
| `014bb3c0` | LobbyCommand_StartTournamentMsg::Create | likely | Allocates 0x10 bytes through 00d833a6 and, if non-null, constructs with 014bb070 and returns it; returns 0 otherwise. |
| `014bb420` | LobbyCommand_StartTournamentMsg::RegisterFactory | likely | Registers creator 014bb3c0 under key 0x1cc with 013ce130 and 013ce230. |

**LobbyCommand_StartTournamentRound** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014bb4f0` | LobbyCommand_StartTournamentRound::LobbyCommand_StartTourna… | confirmed | Calls base constructor 013b2760, stores LobbyCommand_StartTournamentRound::vftable, and zeroes the dwords at this+8 and this+0xc. Returns this. |
| `014bb570` | LobbyCommand_StartTournamentRound::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_StartTournamentRound'. |
| `014bb5b0` | LobbyCommand_StartTournamentRound::DestroyBody | confirmed | Stores LobbyCommand_StartTournamentRound::vftable and calls base cleanup 013b2820 through a local unwind record; releases no members of its own. |
| `014bb6b0` | LobbyCommand_StartTournamentRound::Clone | confirmed | Allocates 0x10 bytes through 00d833a6; if non-null constructs with 014bb4f0; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bb720` | LobbyCommand_StartTournamentRound::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_StartTournamentRound (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandStartTournamentRound.cpp lin… |
| `014bb8e0` | LobbyCommand_StartTournamentRound::~LobbyCommand_StartTourn… | confirmed | Scalar deleting destructor in vftable slot 0: stores the StartTournamentRound vftable, calls base cleanup 013b2820, then frees the block through _free when bit… |
| `014bb940` | LobbyCommand_StartTournamentRound::Dump | confirmed | Appends the literal 'Change Lobby display Lobby Command' with a newline through 01241650, then calls base dump 013b2af0. |
| `014bb970` | LobbyCommand_StartTournamentRound::Describe | confirmed | Takes no arguments and appends the 34-character literal 'StartTournamentRound Lobby Command' through 01241650. |
| `014bb620` | LobbyCommand_StartTournamentRound::PostEvent | likely | Allocates 0x14 bytes, creates the block with 012f9eb0, sets type tag 0x4e through 012f8c70, appends the int fields at this+8 and this+0xc through 012fa2e0, tak… |
| `014bb860` | LobbyCommand_StartTournamentRound::Create | likely | Allocates 0x10 bytes through 00d833a6 and, if non-null, constructs with 014bb4f0 and returns it; returns 0 otherwise. |
| `014bb8c0` | LobbyCommand_StartTournamentRound::RegisterFactory | likely | Registers creator 014bb860 under key 300 with 013ce130 and 013ce230. |

**LobbyCommand_UpdateLobbyStats** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ba1b0` | LobbyCommand_UpdateLobbyStats::Dump | confirmed | Appends the literal 'UpdateLobbyStats Lobby Command' with a newline through 01241650, then calls base-level helper 013b2af0 with the argument. |
| `014ba1e0` | LobbyCommand_UpdateLobbyStats::Describe | confirmed | Takes no arguments and appends the 30-character literal 'UpdateLobbyStats Lobby Command' through 01241650. |
| `014ba200` | LobbyCommand_UpdateLobbyStats::DestroyMembers | confirmed | Stores LobbyCommand_UpdateLobbyStats::vftable, then calls member cleanup 00521a9c four times (unwind states 3, 2, 1, 0) and base cleanup 013b2820. |
| `014ba2b0` | LobbyCommand_UpdateLobbyStats::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_UpdateLobbyStats'. |
| `014ba370` | LobbyCommand_UpdateLobbyStats::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_UpdateLobbyStats (RTTI from NetworkCommand), logs 'clone' assert at LobbyCommandUpdateLobbyStats.cpp line 0x47 on fa… |
| `014ba520` | LobbyCommand_UpdateLobbyStats::LobbyCommand_UpdateLobbyStats | confirmed | Calls base constructor 013b2760, stores LobbyCommand_UpdateLobbyStats::vftable, constructs four containers (005258fb at +8, +0x14, +0x20 and +0x30), and sets t… |
| `014ba5b0` | LobbyCommand_UpdateLobbyStats::~LobbyCommand_UpdateLobbySta… | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014ba200 then frees the block through _free when flag bit 0 is set; returns this and pops 4 by… |
| `014ba5d0` | LobbyCommand_UpdateLobbyStats::Clone | confirmed | Allocates 0x3c bytes through 00d833a6; if non-null constructs with 014ba520; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014ba050` | LobbyCommand_UpdateLobbyStats::PostEvent | likely | Moves the four containers at this+8, +0x14, +0x20 and +0x30 into list heads of the singleton (helpers 012d3450, 012d3490, 012d34d0, 012d3510, only 012d3450 and… |
| `014ba640` | LobbyCommand_UpdateLobbyStats::Create | likely | Allocates 0x3c bytes through 00d833a6 and, if non-null, constructs the object with 014ba520 and returns it; returns 0 otherwise. |
| `014ba6a0` | LobbyCommand_UpdateLobbyStats::RegisterFactory | likely | Registers creator 014ba640 under key 0x19c with 013ce130 and 013ce230. |

**LobbyCommand_UpdateTournament** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ba800` | LobbyCommand_UpdateTournament::DestroyMembers | confirmed | Stores LobbyCommand_UpdateTournament::vftable, frees the heap buffer at this+0x10 when non-null, zeroes this+0x10, +0x14 and +0x18, then calls base cleanup 013… |
| `014ba890` | LobbyCommand_UpdateTournament::TypeName | confirmed | Returns the address of the literal 'LobbyCommand_UpdateTournament'. |
| `014ba8d0` | LobbyCommand_UpdateTournament::~LobbyCommand_UpdateTourname… | confirmed | Scalar deleting destructor in vftable slot 0: calls body cleanup 014ba800, then frees the block through _free when flag bit 0 is set; returns this and pops 4 b… |
| `014ba8f0` | LobbyCommand_UpdateTournament::Dump | confirmed | Appends 'Change Lobby display Lobby Command' and then, with sprintf and 01241650, four lines: 'Group ID: %d' from this+0x1c, 'Tournament ID: %d' from this+8, '… |
| `014baa20` | LobbyCommand_UpdateTournament::Describe | confirmed | Takes no arguments and appends the 30-character literal 'UpdateTournament Lobby Command' through 01241650. |
| `014baa40` | LobbyCommand_UpdateTournament::LobbyCommand_UpdateTournament | confirmed | Calls base constructor 013b2760, stores LobbyCommand_UpdateTournament::vftable, and zeroes the dwords at this+0x10, +0x14, +0x18 (buffer), +8, +0x1c and +0x24.… |
| `014baab0` | LobbyCommand_UpdateTournament::Clone | confirmed | Allocates 0x28 bytes through 00d833a6; if non-null constructs with 014baa40; then calls vftable slot 18 (CopyTo) on this with the new object and returns it. |
| `014bab20` | LobbyCommand_UpdateTournament::CopyTo | confirmed | Dynamic-casts the argument to LobbyCommand_UpdateTournament (RTTI from NetworkCommand), logs the 'clone' assert at LobbyCommandUpdateTournament.cpp line 0x78 o… |
| `014bab90` | LobbyCommand_UpdateTournament::Create | likely | Allocates 0x28 bytes through 00d833a6 and, if non-null, constructs with 014baa40 and returns it; returns 0 otherwise. |
| `014babf0` | LobbyCommand_UpdateTournament::RegisterFactory | likely | Registers creator 014bab90 under key 0x125 with 013ce130 and 013ce230. |

**LoginCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441f10` | LoginCommand::RegisterFactory | likely | Pushes the constant 0x4d and the address 01441eb0, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**LoginCommand_DisplayMOTD** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b9cb0` | LoginCommand_DisplayMOTD::LoginCommand_DisplayMOTD | confirmed | Calls the LoginCommand base constructor 01441c70, stores LoginCommand_DisplayMOTD::vftable, and initialises two std::string members: inline buffer at +8 with s… |
| `014b9d40` | LoginCommand_DisplayMOTD::TypeName | confirmed | Returns the address of the literal 'LoginCommand_DisplayMOTD'. |
| `014b9e20` | LoginCommand_DisplayMOTD::Clone | confirmed | Allocates 0x44 bytes through 00d833a6; if non-null constructs the object with 014b9cb0 and keeps the result, otherwise keeps 0. Then calls vftable slot 18 (Cop… |
| `014b9f10` | LoginCommand_DisplayMOTD::~LoginCommand_DisplayMOTD | confirmed | Scalar deleting destructor in vftable slot 0: calls the body destructor 014b9d80, then frees the block through _free when bit 0 of the flag byte is set; return… |
| `014b9ff0` | LoginCommand_DisplayMOTD::Dump | confirmed | Appends the fixed text 'Display MOTD Command' through 01241650 and calls the 0140c290 no-op stub; prints no fields. |
| `014ba020` | LoginCommand_DisplayMOTD::Describe | confirmed | Takes no arguments and appends the 25-character literal 'Diplay MOTD Login Command' (sic, typo in the binary) through 01241650. |
| `014b9ad0` | LoginCommand_DisplayMOTD::PostEvent | likely | Gets the singleton from 012d46a0, allocates 0x14 bytes, zeroes them through 012f9eb0 (which returns the block), sets the type tag 0x54 through 012f8c70, append… |
| `014b9e90` | LoginCommand_DisplayMOTD::Create | likely | Allocates 0x44 bytes through 00d833a6 and, if the allocation is non-null, constructs the object with 014b9cb0 and returns it; returns 0 otherwise. |
| `014b9ef0` | LoginCommand_DisplayMOTD::RegisterFactory | likely | Registers creator 014b9e90 with the factory registry twice: 013ce130(0x135, creator) and 013ce230(0x135, creator). |

**LoginCommand_ForgotPassword** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b9970` | LoginCommand_ForgotPassword::~LoginCommand_ForgotPassword | confirmed | Scalar deleting destructor in vftable slot 0: calls the body destructor 014b9800, then if bit 0 of the flag byte is set it frees the block through _free. Retur… |
| `014b9ab0` | LoginCommand_ForgotPassword::Describe | confirmed | Takes no arguments and appends the 29-character literal 'Forgot Password Login Command' through 01241650 (std::string append). |

**MatchCommand** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b2320` | MatchCommand::MatchCommand | confirmed | Calls 013b2760 (LobbyCommand base constructor), stores MatchCommand::vftable at *this and zeroes the dword at +8; returns this. |
| `013b23a0` | MatchCommand::GetTypeName | confirmed | Virtual slot 11: returns the static C string MatchCommand. |
| `013b23e0` | MatchCommand::DtorBody | confirmed | Stores MatchCommand::vftable at *this, calls 013b2820 (LobbyCommand destructor body) and frees nothing. |
| `013b24e0` | MatchCommand::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to MatchCommand (00d8d382 with NetworkCommand and MatchCommand RTTI descriptors); on failure asserts clone through 012f5… |
| `013b2530` | MatchCommand::ReadFields | confirmed | Virtual slot 9: calls this->vtable slot 4 with (stream, &stream); then 013b2910 (LobbyCommand::ReadFields), 013c24d0 reads an int into this+8, and this->vtable… |
| `013b2580` | MatchCommand::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2 with the stream, passes trace texts to 013c26e0 (a no-op), calls 013b2960 (LobbyCommand base write), writes the int … |
| `013b2660` | MatchCommand::~MatchCommand | confirmed | Scalar deleting destructor in vtable slot 0: stores MatchCommand::vftable, calls 013b2820 (LobbyCommand destructor body), then calls _free(this) when bit 0 of … |
| `013b26c0` | MatchCommand::AppendDescription | confirmed | Virtual slot 19: calls 013b2af0 (LobbyCommand AppendDescription) on its argument, formats Match ID: %d newline from this+8 with _sprintf into a 1000-byte buffe… |
| `013b2740` | MatchCommand::AppendDisplayName | confirmed | Virtual slot 20: appends the 13-byte text Match Command to its std::string argument through 01241650. |
| `013b2470` | MatchCommand::Clone | likely | Virtual slot 17: allocates 0xC bytes (00d833a6), constructs a MatchCommand with 013b2320 when the allocation succeeds, then calls this->vtable slot 18 (CopyTo)… |
| `013b25e0` | MatchCommand::CreateInstance | likely | Allocates 0xC bytes (00d833a6); when the allocation succeeds constructs a MatchCommand with 013b2320 and returns it, else returns 0. |
| `013b2640` | MatchCommand::RegisterClass | likely | Pushes creator 013b25e0 and id 0x62, calls 013ce130, then calls 013ce230 with ECX set to 013ce130 return value and no stack arguments; returns void. |

**MatchCommand_BidPlaced** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b55e0` | MatchCommand_BidPlaced::MatchCommand_BidPlaced | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 013b2320 (MatchCommand base constructor) on this, then stores MatchCommand_BidPlaced::vftable into *th… |
| `014b5660` | MatchCommand_BidPlaced::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal MatchCommand_BidPlaced (mov eax,018ff28c; ret). |
| `014b56a0` | MatchCommand_BidPlaced::~MatchCommand_BidPlaced | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores MatchCommand_BidPlaced::vftable into *this, then calls 013b23e0 (MatchCommand base destructor bod… |
| `014b5780` | MatchCommand_BidPlaced::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0xc-byte block (00d833a6). When non-null it constructs it with 014b55e0 and keeps t… |
| `014b57f0` | MatchCommand_BidPlaced::Clone | confirmed | Virtual slot 18 (one stack argument, ret 4). dynamic_casts the NetworkCommand argument to MatchCommand_BidPlaced (00d8d382 with both RTTI descriptors); on null… |
| `014b5840` | MatchCommand_BidPlaced::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word, without testing the re… |
| `014b5880` | MatchCommand_BidPlaced::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 013… |
| `014b5950` | MatchCommand_BidPlaced::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores MatchCommand_BidPlaced::vftable into *this, calls 013b23e0 (MatchCommand base destructor body), then … |
| `014b59b0` | MatchCommand_BidPlaced::Dump | confirmed | Virtual slot 19 (one stack argument, ret 4). Appends 'Bid Placed Match Command' plus newline (length 0x19) to the string argument through 01241650, then calls … |
| `014b59e0` | MatchCommand_BidPlaced::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Bid Placed Match Command' (length 0x18) to it through 0124165… |
| `014b56f0` | MatchCommand_BidPlaced::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). Under an EH frame it news a 0x14-byte block (00d833a6) and, when non-null, constructs it with 012f9e… |

**MatchCommand_InitiateChallenge** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b5110` | MatchCommand_InitiateChallenge::MatchCommand_InitiateChalle… | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 013b2320 (MatchCommand base constructor) on this, stores MatchCommand_InitiateChallenge::vftable into … |
| `014b5190` | MatchCommand_InitiateChallenge::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal MatchCommand_InitiateChallenge (mov eax,018ff17c; ret). |
| `014b51d0` | MatchCommand_InitiateChallenge::~MatchCommand_InitiateChall… | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores MatchCommand_InitiateChallenge::vftable into *this, then calls 013b23e0 (MatchCommand base destru… |
| `014b5320` | MatchCommand_InitiateChallenge::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news an 0x18-byte block (00d833a6). When non-null it constructs it with 014b5110 and keeps… |
| `014b5390` | MatchCommand_InitiateChallenge::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to MatchCommand_InitiateChallenge (00d8d382 with both RTTI descriptor… |
| `014b53f0` | MatchCommand_InitiateChallenge::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word, without testing the re… |
| `014b5460` | MatchCommand_InitiateChallenge::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), calls 013b2580 with the argument (base write), wri… |
| `014b5530` | MatchCommand_InitiateChallenge::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores MatchCommand_InitiateChallenge::vftable into *this, calls 013b23e0 (MatchCommand base destructor body… |
| `014b5590` | MatchCommand_InitiateChallenge::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends 'Initiate Challenge Match Command' plus newline (length 0x21) to the string argument through 012… |
| `014b55c0` | MatchCommand_InitiateChallenge::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Initiate Challenge Match Command' (length 0x20) to it through… |
| `014b5250` | MatchCommand_InitiateChallenge::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). Under an EH frame it first runs the unlabelled helpers 015197f0, 01519860 (argument 1), 015198a0 (th… |

**MatchCommand_LaunchGame** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b4c90` | MatchCommand_LaunchGame::MatchCommand_LaunchGame | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 013b2320 (MatchCommand base constructor) on this, then stores MatchCommand_LaunchGame::vftable into *t… |
| `014b4d10` | MatchCommand_LaunchGame::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal MatchCommand_LaunchGame (mov eax,018ff04c; ret). |
| `014b4d50` | MatchCommand_LaunchGame::~MatchCommand_LaunchGame | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores MatchCommand_LaunchGame::vftable into *this, then calls 013b23e0 (MatchCommand base destructor bo… |
| `014b4e90` | MatchCommand_LaunchGame::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0xc-byte block (00d833a6). When non-null it constructs it with 014b4c90 and keeps t… |
| `014b4f00` | MatchCommand_LaunchGame::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument, ret 4). dynamic_casts the NetworkCommand argument to MatchCommand_LaunchGame (00d8d382 with both RTTI descriptor… |
| `014b4f50` | MatchCommand_LaunchGame::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word, without testing… |
| `014b4f90` | MatchCommand_LaunchGame::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 013… |
| `014b5060` | MatchCommand_LaunchGame::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores MatchCommand_LaunchGame::vftable into *this, calls 013b23e0 (MatchCommand base destructor body), then… |
| `014b50c0` | MatchCommand_LaunchGame::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends 'Launch Game Match Command' plus newline (length 0x1a) to the string argument through 01241650, … |
| `014b50f0` | MatchCommand_LaunchGame::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Launch Game Match Command' (length 0x19) to it through 012416… |
| `014b4da0` | MatchCommand_LaunchGame::EnsureClientGameAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). Under an EH frame it news a 0x14-byte record (00d833a6) and constructs it with 012f9eb0; it calls 01… |

**MatchCommand_SelectDeck** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b1d70` | MatchCommand_SelectDeck::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2 with the stream, passes trace texts to 013c26e0 (a no-op that emits nothing), writes the string member at this+0xC t… |
| `013b1ea0` | MatchCommand_SelectDeck::MatchCommand_SelectDeck | confirmed | Calls 013b2320 (MatchCommand base constructor), stores MatchCommand_SelectDeck::vftable at *this, sets the dword at +0x24 to 0xf and +0x20 to 0, clears the byt… |
| `013b1f30` | MatchCommand_SelectDeck::GetTypeName | confirmed | Virtual slot 11: returns the static C string MatchCommand_SelectDeck. |
| `013b1f70` | MatchCommand_SelectDeck::DtorBody | confirmed | Stores MatchCommand_SelectDeck::vftable at *this, releases the nested object at +0x2C (slot 0 with argument 1 when non-null, then clears it), frees the buffer … |
| `013b20f0` | MatchCommand_SelectDeck::~MatchCommand_SelectDeck | confirmed | Scalar deleting destructor in vtable slot 0: calls 013b1f70 (destructor body), then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `013b2180` | MatchCommand_SelectDeck::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to MatchCommand_SelectDeck (00d8d382 with NetworkCommand and MatchCommand_SelectDeck RTTI descriptors); on failure asser… |
| `013b21f0` | MatchCommand_SelectDeck::AppendDescription | confirmed | Virtual slot 19: appends the 26-byte text Select Deck Match Command newline to its std::string argument, calls 013b26c0 on the same argument, formats DeckID: %… |
| `013b2300` | MatchCommand_SelectDeck::AppendDisplayName | confirmed | Virtual slot 20: appends the 25-byte text Select Deck Match Command to its std::string argument through 01241650. |
| `013b2000` | MatchCommand_SelectDeck::Clone | likely | Virtual slot 17: allocates 0x30 bytes (00d833a6), constructs a MatchCommand_SelectDeck with 013b1ea0 when the allocation succeeds, then calls this->vtable slot… |
| `013b2070` | MatchCommand_SelectDeck::CreateInstance | likely | Allocates 0x30 bytes (00d833a6); when the allocation succeeds constructs a MatchCommand_SelectDeck with 013b1ea0 and returns it, else returns 0. |
| `013b20d0` | MatchCommand_SelectDeck::RegisterClass | likely | Pushes creator 013b2070 and id 0x65, calls 013ce130, then calls 013ce230 with ECX set to 013ce130 return value and no stack arguments; returns void. |
| `013b2110` | MatchCommand_SelectDeck::InitFromSource | likely | Asserts param_2 is non-null through 012f5a60 (file MatchCommandSelectDeck.cpp, line 0x1b); calls 013ce480, clears the bool at +0x28, assigns the string at +0xC… |

**MatchCommand_SelectStartPosition** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b4740` | MatchCommand_SelectStartPosition::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument, passes trace text to 013c26e0 (a no-op), calls 013b2580 with the … |
| `014b47b0` | MatchCommand_SelectStartPosition::~MatchCommand_SelectStart… | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores MatchCommand_SelectStartPosition::vftable into *this, frees the block at this+0x14 with _free whe… |
| `014b4840` | MatchCommand_SelectStartPosition::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal MatchCommand_SelectStartPosition (mov eax,018fef04; ret). |
| `014b4880` | MatchCommand_SelectStartPosition::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b47b0 (the SelectStartPosition destructor body) on this, frees this with _free when bit 0 of the fl… |
| `014b48a0` | MatchCommand_SelectStartPosition::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Select Start Position Match Command' plus newline (length 0x24) through 01241650, calls 013b26c0 with … |
| `014b49c0` | MatchCommand_SelectStartPosition::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Select Start Position Match Command' (length 0x23) to it thro… |
| `014b49e0` | MatchCommand_SelectStartPosition::MatchCommand_SelectStartP… | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 013b2320 (MatchCommand base constructor) on this, stores MatchCommand_SelectStartPosition::vftable int… |
| `014b4ac0` | MatchCommand_SelectStartPosition::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x20-byte block (00d833a6). When non-null it constructs it with 014b49e0 and keeps … |
| `014b4b30` | MatchCommand_SelectStartPosition::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to MatchCommand_SelectStartPosition (00d8d382 with both RTTI descript… |
| `014b4c20` | MatchCommand_SelectStartPosition::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word; returns 0 when that fa… |
| `014b4690` | MatchCommand_SelectStartPosition::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). Under an EH frame it news a 0x14-byte block (00d833a6) and, when non-null, constructs it with 012f9e… |

**MatchCommand_SetBid** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b41b0` | MatchCommand_SetBid::MatchCommand_SetBid | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 013b2320 (MatchCommand base constructor, which stores MatchCommand::vftable) on this, then stores Matc… |
| `014b4230` | MatchCommand_SetBid::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal string MatchCommand_SetBid (mov eax,018fed6c; ret). |
| `014b4270` | MatchCommand_SetBid::~MatchCommand_SetBid | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores MatchCommand_SetBid::vftable into *this, then calls 013b23e0 on this (the MatchCommand base destr… |
| `014b4370` | MatchCommand_SetBid::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x10-byte block (00d833a6). When the block is non-null it constructs it with 014b41… |
| `014b43e0` | MatchCommand_SetBid::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to MatchCommand_SetBid (00d8d382 with both RTTI descriptors); on null… |
| `014b4430` | MatchCommand_SetBid::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word; returns 0 when that fa… |
| `014b4480` | MatchCommand_SetBid::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 013… |
| `014b4560` | MatchCommand_SetBid::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores MatchCommand_SetBid::vftable into *this, calls 013b23e0 (MatchCommand base destructor body), then fre… |
| `014b45c0` | MatchCommand_SetBid::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends the literal 'Set Bid Match Command' plus newline (length 0x16) through 01241650, calls 013b26c0 with th… |
| `014b4650` | MatchCommand_SetBid::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Set Bid Match Command' (length 0x15) to it through 01241650 (… |
| `014b42e0` | MatchCommand_SetBid::CreateFieldEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). Under an EH frame it news a 0x14-byte block (00d833a6). If the block is non-null it constructs it wi… |

**MatchCommand_SetBulkMatchState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b4030` | MatchCommand_SetBulkMatchState::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument, ret 4). dynamic_casts the NetworkCommand argument to MatchCommand_SetBulkMatchState (00d8d382 with both RTTI des… |
| `014b4140` | MatchCommand_SetBulkMatchState::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). Calls vtable slot 4 (+0x10) with the argument and a local that receives a version word; that result is no… |

**MatchCommand_SetReady** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013b37b0` | MatchCommand_SetReady::MatchCommand_SetReady | confirmed | Calls 013b2320 (MatchCommand constructor), stores MatchCommand_SetReady::vftable at *this and clears the byte at +0xC; returns this. |
| `013b3830` | MatchCommand_SetReady::GetTypeName | confirmed | Virtual slot 11: returns the static C string MatchCommand_SetReady. |
| `013b3870` | MatchCommand_SetReady::DtorBody | confirmed | Stores MatchCommand_SetReady::vftable at *this, then calls 013b23e0 (MatchCommand destructor body); frees nothing. |
| `013b39f0` | MatchCommand_SetReady::CopyTo | confirmed | Virtual slot 18: dynamic-casts param_2 to MatchCommand_SetReady (00d8d382 with NetworkCommand and MatchCommand_SetReady RTTI descriptors); on failure asserts c… |
| `013b3a40` | MatchCommand_SetReady::ReadFields | confirmed | Virtual slot 9: calls this->vtable slot 4 with (stream, &local); then 013b2530 (MatchCommand read), reads an int into a local and stores its nonzero test as th… |
| `013b3aa0` | MatchCommand_SetReady::WriteFields | confirmed | Virtual slot 10: calls this->vtable slot 2 with the stream, passes trace texts to 013c26e0 (a no-op), calls 013b2580 (MatchCommand write), writes the byte at +… |
| `013b3b80` | MatchCommand_SetReady::~MatchCommand_SetReady | confirmed | Scalar deleting destructor in vtable slot 0: stores MatchCommand_SetReady::vftable, calls 013b23e0 (MatchCommand destructor body), then calls _free(this) when … |
| `013b3be0` | MatchCommand_SetReady::AppendDescription | confirmed | Virtual slot 19: appends the 24-byte text Set Ready Match Command newline, calls 013b26c0 (MatchCommand description) on its argument, formats Ready: %d newline… |
| `013b3c70` | MatchCommand_SetReady::AppendDisplayName | confirmed | Virtual slot 20: appends the 23-byte text Set Ready Match Command to its std::string argument through 01241650. |
| `013b3840` | MatchCommand_SetReady::GetClassId | likely | Virtual slot 12: returns the dword in the global DAT_01cbdd18. |
| `013b3850` | MatchCommand_SetReady::SetClassId | likely | Virtual slot 13: stores its stack argument into the global DAT_01cbdd18. |
| `013b3980` | MatchCommand_SetReady::Clone | likely | Virtual slot 17: allocates 0x10 bytes (00d833a6), constructs a MatchCommand_SetReady with 013b37b0 when the allocation succeeds, then calls this->vtable slot 1… |
| `013b3b00` | MatchCommand_SetReady::CreateInstance | likely | Allocates 0x10 bytes (00d833a6); when the allocation succeeds constructs a MatchCommand_SetReady with 013b37b0 and returns it, else returns 0. |

**NetworkCommand** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0140c090` | NetworkCommand::Ctor | confirmed | Stores PersistentBase::vftable, then PersistentComponent::vftable, then NetworkCommand::vftable into *param_1 (the last store is the final vptr); no callees. |
| `0140c120` | NetworkCommand::GetClassName | confirmed | Returns the address of the literal 'NetworkCommand' (0x018e4d44) in EAX; no arguments. |
| `0140c160` | NetworkCommand::DtorBody | confirmed | Stores the NetworkCommand, PersistentComponent and PersistentBase vtables into *param_1 in that order; releases no member storage. |
| `0140c390` | NetworkCommand::ScalarDeletingDtor | confirmed | Stores the NetworkCommand, PersistentComponent and PersistentBase vtables into *param_1 (the base teardown is inlined), frees the object when bit 0 of the stac… |
| `0140c310` | NetworkCommand::Create | likely | Allocates 4 bytes through 00d833a6; returns 0 when that fails, otherwise builds a NetworkCommand with 0140c090 and returns it. |

**NetworkCommand_ConnectionTimedout** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f59b0` | NetworkCommand_ConnectionTimedout::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090 and stores NetworkCommand_ConnectionTimedout::vftable into [this]. No members ar… |
| `014f5a30` | NetworkCommand_ConnectionTimedout::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_ConnectionTimedout" (mov eax, imm; ret). |
| `014f5ac0` | NetworkCommand_ConnectionTimedout::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 otherwise).… |
| `014f5b40` | NetworkCommand_ConnectionTimedout::Clone | confirmed | Virtual slot [17]. Allocates 4 bytes with operator new 00d833a6; if non-null, constructs a ConnectionTimedout in the block with 014f59b0. Then calls the base c… |
| `014f5bb0` | NetworkCommand_ConnectionTimedout::CopyFieldsTo | confirmed | Virtual slot [18]. RTTI-casts its stack argument with 00d8d382 to NetworkCommand_ConnectionTimedout; on failure calls 012f5a60 with "clone", the source file pa… |
| `014f5d10` | NetworkCommand_ConnectionTimedout::~NetworkCommand_Connecti… | confirmed | Sets the object's vtable to NetworkCommand_ConnectionTimedout::vftable, calls the base destructor body 0140c160 with this in ECX, and if bit 0 of the flag argu… |
| `014f5da0` | NetworkCommand_ConnectionTimedout::DisplayName | confirmed | Appends the 36-byte literal 'Connection Timed Out Network Command' (length 0x24) to the std::string passed as the first stack argument via 01241650 and returns… |
| `014f5a40` | NetworkCommand_ConnectionTimedout::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe96c (mov eax,[DAT_01cbe96c]; ret). No arguments. |
| `014f5a50` | NetworkCommand_ConnectionTimedout::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe96c; returns with ret 4. |
| `014f5a70` | NetworkCommand_ConnectionTimedout::~NetworkCommand_Connecti… | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_ConnectionTimedout::vftable into [this] and calls the base body 0140c160 on this. It owns no m… |
| `014f5c90` | NetworkCommand_ConnectionTimedout::Create | likely | Calls operator new (00d833a6) with size 4; if the result is non-null it calls constructor 014f59b0 with the new object in ECX and returns the constructor's EAX… |
| `014f5cf0` | NetworkCommand_ConnectionTimedout::RegisterFactory | likely | Pushes factory 014f5c90 and the constant 0x86, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x86 (ret 8). Retu… |
| `014f5d70` | NetworkCommand_ConnectionTimedout::Dump | likely | Appends the literal 'Connection Timed Out Network Command' plus newline (length 0x25) to the std::string passed as the first stack argument via 01241650, then … |

**NetworkCommand_Disconnect** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013eee30` | NetworkCommand_Disconnect::GetClassName | confirmed | Virtual slot 11. Returns the address of the C string NetworkCommand_Disconnect (bytes B8 C4 33 8E 01 C3: mov eax, imm32; ret). |
| `013eee90` | NetworkCommand_Disconnect::NetworkCommand_Disconnect | confirmed | Constructor: calls the base initialiser 0140c090 on this, stores NetworkCommand_Disconnect::vftable at +0, sets the std::string at +4 to empty (size 0 at +0x18… |
| `013eef40` | NetworkCommand_Disconnect::Clone | confirmed | Virtual slot 17 (+0x44) of NetworkCommand_Disconnect: allocates 0x20 bytes with operator new (00d833a6); if the allocation is non-null it constructs a NetworkC… |
| `013eefb0` | NetworkCommand_Disconnect::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48): dynamic_cast of the destination argument to NetworkCommand_Disconnect through 00d8d382; when the cast fails it traces 'clone' with Net… |
| `013ef010` | NetworkCommand_Disconnect::Dump | confirmed | Virtual slot 19 (+0x4c): appends the literal 'Disconnect Network Command' newline text through 01241650 to the std::string passed as its stack argument, then f… |
| `013ef0b0` | NetworkCommand_Disconnect::DisplayName | confirmed | Virtual slot 20 (+0x50): reads the destination std::string from its one stack argument into ECX and appends the 26-byte literal 'Disconnect Network Command' wi… |
| `013eecd0` | NetworkCommand_Disconnect::ReadFields | likely | Virtual slot 9, thiscall with one argument. Calls this->vtable +0x10 (slot 4) with the argument; when that returns non-zero it calls 0140c2a0, then reads a str… |
| `013eed20` | NetworkCommand_Disconnect::WriteFields | likely | Virtual slot 10, thiscall with one argument. Calls this->vtable +8 with the argument, passes the trace texts Starting NetworkCommand_Disconnect, Message: and E… |
| `013eed90` | NetworkCommand_Disconnect::~NetworkCommand_Disconnect | likely | Destructor body: stores the NetworkCommand_Disconnect vftable at *param_1, frees the string buffer at +8 when its capacity exceeds 15 and resets the string to … |
| `013eee70` | NetworkCommand_Disconnect::ScalarDeletingDtor | likely | Virtual slot 0: calls the destructor body 013eed90, frees this when flag bit 0 is set, and returns this. |
| `013ef0d0` | NetworkCommand_Disconnect::Factory | likely | Allocates 0x20 bytes with operator new (00d833a6); when the allocation is non-null it constructs NetworkCommand_Disconnect with 013eee90 and returns the object… |

**NetworkCommand_EULA** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f4a30` | NetworkCommand_EULA::ClientHandler | confirmed | Virtual slot [15] (fastcall, ECX=this). Calls 012d46a0 first, then 012d29c0 with this+4 and the dwords at this+0x20 and this+0x24. Allocates 0x14 bytes with op… |
| `014f4cf0` | NetworkCommand_EULA::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_EULA; on failure calls 012f5a60 wi… |
| `014f4d60` | NetworkCommand_EULA::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 21-byte literal "EULA Network Command\n" via 01241650 to the argument, then… |
| `014f4d90` | NetworkCommand_EULA::AppendTypeName | confirmed | Virtual slot [20]. Appends the 20-byte literal "EULA Network Command" via 01241650 to the std::string passed as the stack argument (19 bytes, the same display-… |
| `014f5200` | NetworkCommand_EULA::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_EULA" (mov eax, imm; ret). |
| `014f5240` | NetworkCommand_EULA::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_EULA::vftable into [this], sets the dword at this+0x1c to… |
| `014f52e0` | NetworkCommand_EULA::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f5130 on this, then if flag bit 0 is set calls _f… |
| `014f5300` | NetworkCommand_EULA::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x4c bytes with operator new 00d833a6; if non-null, constructs an EULA command in the block with 014f524… |
| `014f5130` | NetworkCommand_EULA::~NetworkCommand_EULA | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_EULA::vftable into [this], calls the member teardown 01321610 twice and the list teardown 00f9… |
| `014f5210` | NetworkCommand_EULA::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe964 (mov eax,[DAT_01cbe964]; ret). No arguments. |
| `014f5220` | NetworkCommand_EULA::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe964; returns with ret 4. |
| `014f5370` | NetworkCommand_EULA::Factory | likely | Allocates 0x4c bytes with operator new 00d833a6; if the allocation is non-null, constructs an EULA command in it with 014f5240 and returns the object, otherwis… |

**NetworkCommand_ErrorMsg** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f5410` | NetworkCommand_ErrorMsg::ClientHandler | confirmed | Virtual slot [15] (fastcall, ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 o… |
| `014f5660` | NetworkCommand_ErrorMsg::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_ErrorMsg" (mov eax, imm; ret). |
| `014f56a0` | NetworkCommand_ErrorMsg::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f55c0 on this, then if flag bit 0 is set calls _f… |
| `014f56c0` | NetworkCommand_ErrorMsg::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_ErrorMsg::vftable into [this], sets the dword at this+0x1… |
| `014f5770` | NetworkCommand_ErrorMsg::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x24 bytes with operator new 00d833a6; if non-null, constructs an ErrorMsg in the block with 014f56c0. T… |
| `014f57e0` | NetworkCommand_ErrorMsg::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_ErrorMsg; on failure calls 012f5a6… |
| `014f5840` | NetworkCommand_ErrorMsg::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 22-byte literal "Error Network Command\n" via 01241650, calls the base Dump… |
| `014f5910` | NetworkCommand_ErrorMsg::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument and appends the 21-byte literal "Error Network Command" via 01241650; returns with ret 4… |
| `014f55c0` | NetworkCommand_ErrorMsg::~NetworkCommand_ErrorMsg | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_ErrorMsg::vftable into [this]. If the capacity dword at this+0x1c is above 0xf, frees the buff… |
| `014f5670` | NetworkCommand_ErrorMsg::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe968 (mov eax,[DAT_01cbe968]; ret). No arguments. |
| `014f5680` | NetworkCommand_ErrorMsg::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe968; returns with ret 4. |
| `014f5930` | NetworkCommand_ErrorMsg::Factory | likely | Allocates 0x24 bytes with operator new 00d833a6; if the allocation is non-null, constructs an ErrorMsg in it with 014f56c0 and returns the object, otherwise re… |

**NetworkCommand_Gift** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f3390` | NetworkCommand_Gift::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_Gift::vftable into [this], zeroes the dwords at this+4 an… |
| `014f3410` | NetworkCommand_Gift::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_Gift" (mov eax, imm; ret). |
| `014f34f0` | NetworkCommand_Gift::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0xc bytes with operator new 00d833a6; if non-null, constructs a Gift in the block with 014f3390. Then ca… |
| `014f3560` | NetworkCommand_Gift::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_Gift; on failure calls 012f5a60 wi… |
| `014f36d0` | NetworkCommand_Gift::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags). Stores NetworkCommand_Gift::vftable into [this], calls the base body 0140c160 … |
| `014f3730` | NetworkCommand_Gift::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 21-byte literal "Gift Network Command\n" via 01241650, calls the base Dump … |
| `014f37f0` | NetworkCommand_Gift::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 20-byte literal "Gift Network Command" via 01241650; r… |
| `014f3420` | NetworkCommand_Gift::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe95c (mov eax,[DAT_01cbe95c]; ret). No arguments. |
| `014f3430` | NetworkCommand_Gift::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe95c; returns with ret 4. |
| `014f3450` | NetworkCommand_Gift::~NetworkCommand_Gift | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_Gift::vftable into [this] and calls the base body 0140c160 on this. It owns no members of its … |
| `014f3650` | NetworkCommand_Gift::Factory | likely | Allocates 0xc bytes with operator new 00d833a6; if the allocation is non-null, constructs a Gift in it with 014f3390 and returns the object, otherwise returns … |
| `014f36b0` | NetworkCommand_Gift::RegisterFactory | likely | Registers the Gift factory 014f3650 under classID 0x1a0: calls 013ce130(0x1a0, 014f3650) and then 013ce230(0x1a0, 014f3650). No arguments, no return value. |

**NetworkCommand_LeaderBoardData** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f3a80` | NetworkCommand_LeaderBoardData::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 32-byte literal "LeaderBoardData Network Command\n" to the argument via 012… |
| `014f3ab0` | NetworkCommand_LeaderBoardData::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 31-byte literal "LeaderBoardData Network Command" via … |
| `014f3d30` | NetworkCommand_LeaderBoardData::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_LeaderBoardData::vftable into [this], and zeroes the dwor… |
| `014f3e00` | NetworkCommand_LeaderBoardData::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_LeaderBoardData" (mov eax, imm; ret). |
| `014f3f80` | NetworkCommand_LeaderBoardData::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 100 bytes with operator new 00d833a6; if non-null, constructs a LeaderBoardData in the block with 014f3d… |
| `014f40a0` | NetworkCommand_LeaderBoardData::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f3e40 on this, then if flag bit 0 is set calls _f… |
| `014f3e10` | NetworkCommand_LeaderBoardData::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe960 (mov eax,[DAT_01cbe960]; ret). No arguments. |
| `014f3e20` | NetworkCommand_LeaderBoardData::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe960; returns with ret 4. |
| `014f3e40` | NetworkCommand_LeaderBoardData::~NetworkCommand_LeaderBoard… | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_LeaderBoardData::vftable into [this]. Frees the single buffers at this+0x58, this+0x38, this+0… |
| `014f4020` | NetworkCommand_LeaderBoardData::Factory | likely | Allocates 100 bytes with operator new 00d833a6; if the allocation is non-null, constructs a LeaderBoardData in it with 014f3d30 and returns the object, otherwi… |

**NetworkCommand_News** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f25e0` | NetworkCommand_News::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 21-byte literal "News Network Command\n" via 01241650 to the argument, then… |
| `014f2610` | NetworkCommand_News::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 20-byte literal "News Network Command" via 01241650; r… |
| `014f28b0` | NetworkCommand_News::ctor | confirmed | Fastcall constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_News::vftable into [this], zeroes the dwords at this+8, +… |
| `014f2950` | NetworkCommand_News::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_News" (mov eax, imm; ret). |
| `014f2a80` | NetworkCommand_News::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x34 bytes with operator new 00d833a6; if non-null, constructs a News command in the block with 014f28b0… |
| `014f2ba0` | NetworkCommand_News::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f2990 on this, then if flag bit 0 is set calls _f… |
| `014f2ee0` | NetworkCommand_News::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_News; on failure calls 012f5a60 with "clone… |
| `014f2960` | NetworkCommand_News::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe958 (mov eax,[DAT_01cbe958]; ret). No arguments. |
| `014f2970` | NetworkCommand_News::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe958; returns with ret 4. |
| `014f2990` | NetworkCommand_News::~NetworkCommand_News | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_News::vftable into [this]. Frees the raw buffer at this+0x28 when set and zeroes this+0x28, +0… |
| `014f2b20` | NetworkCommand_News::Factory | likely | Allocates 0x34 bytes with operator new 00d833a6; if the allocation is non-null, constructs the object with 014f28b0 and returns it, otherwise returns 0. Takes … |
| `014f2b80` | NetworkCommand_News::RegisterFactory | likely | Registers the News factory 014f2b20 under classID 0x1c9: calls 013ce130(0x1c9, 014f2b20) and then 013ce230(0x1c9, 014f2b20). No arguments, no return value. |

**NetworkCommand_PendingSessionKick** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f1f00` | NetworkCommand_PendingSessionKick::ctor | confirmed | Constructor (fastcall, ECX=this, with an EH frame): calls the base constructor 0140c090, stores NetworkCommand_PendingSessionKick::vftable into [this], zeroes … |
| `014f1f80` | NetworkCommand_PendingSessionKick::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_PendingSessionKick" (mov eax, imm; ret). |
| `014f2060` | NetworkCommand_PendingSessionKick::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 12 bytes with operator new 00d833a6; if non-null, constructs a PendingSessionKick in the block with 014f… |
| `014f20d0` | NetworkCommand_PendingSessionKick::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_PendingSessionKick; on failure cal… |
| `014f2250` | NetworkCommand_PendingSessionKick::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags). Stores NetworkCommand_PendingSessionKick::vftable into [this], calls the base … |
| `014f22b0` | NetworkCommand_PendingSessionKick::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 37-byte literal "Pending Session Kick Network Command\n" via 01241650, call… |
| `014f2370` | NetworkCommand_PendingSessionKick::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 36-byte literal "Pending Session Kick Network Command"… |
| `014f1f90` | NetworkCommand_PendingSessionKick::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe954 (mov eax,[DAT_01cbe954]; ret). No arguments. |
| `014f1fa0` | NetworkCommand_PendingSessionKick::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe954; returns with ret 4. |
| `014f1fc0` | NetworkCommand_PendingSessionKick::~NetworkCommand_PendingS… | likely | Thiscall body (ECX=this, EH frame). Stores NetworkCommand_PendingSessionKick::vftable into [this] and calls the base body 0140c160 on this. It owns no members … |
| `014f21d0` | NetworkCommand_PendingSessionKick::Factory | likely | Allocates 12 bytes with operator new 00d833a6; if the allocation is non-null, constructs a PendingSessionKick in it with 014f1f00 and returns the object, other… |
| `014f2230` | NetworkCommand_PendingSessionKick::RegisterFactory | likely | Registers the PendingSessionKick factory 014f21d0 under classID 0x1fc: calls 013ce130(0x1fc, 014f21d0) and then 013ce230(0x1fc, 014f21d0). No arguments, no ret… |

**NetworkCommand_Ping** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d9cf0` | NetworkCommand_Ping::Ctor | confirmed | Calls 0140c090 (shared base-class constructor) with the object, then stores the vtable 018e1ef4 at [ecx] and writes 0 to this+4; returns this. |
| `013d9d70` | NetworkCommand_Ping::TypeName | confirmed | Vtable slot [11] of NetworkCommand_Ping. Bytes b8 4c 1f 8e 01 c3: returns the address of the literal 018e1f4c, which is the string 'NetworkCommand_Ping'. |
| `013d9d80` | NetworkCommand_Ping::GetClassDword | confirmed | Vtable slot [12] of NetworkCommand_Ping. Bytes a1 44 de cb 01 c3: loads and returns the dword at 01cbde44. |
| `013d9d90` | NetworkCommand_Ping::SetClassDword | confirmed | Vtable slot [13] of NetworkCommand_Ping. Bytes 8b 44 24 04 a3 44 de cb 01 c2 04 00: stores the first stack argument into the dword at 01cbde44 and returns with… |
| `013d9db0` | NetworkCommand_Ping::~NetworkCommand_Ping | confirmed | Stores the NetworkCommand_Ping vtable 018e1ef4 into *this, then calls 0140c160 (shared base-class cleanup). It does not free the object; the scalar deleting wr… |
| `013d9e30` | NetworkCommand_Ping::Clone | confirmed | Vtable slot [17] of NetworkCommand_Ping. Allocates 8 bytes with 00d833a6; if the block is non-null it constructs NetworkCommand_Ping in it with 013d9cf0 and us… |
| `013d9ea0` | NetworkCommand_Ping::CopyFieldsTo | confirmed | Vtable slot [18] of NetworkCommand_Ping. Calls 00d8d382 as a dynamic_cast from NetworkCommand to NetworkCommand_Ping on the argument. If the cast gives 0 it ca… |
| `013da010` | NetworkCommand_Ping::ScalarDeletingDtor | confirmed | Vtable slot [0] of NetworkCommand_Ping. Stores the vtable 018e1ef4 into *this, calls 0140c160 (shared base cleanup), then if bit 0 of the argument is set calls… |
| `013da070` | NetworkCommand_Ping::Dump | confirmed | Vtable slot [19] (Dump) of NetworkCommand_Ping. Appends the literal 'Ping Network Command' newline text (length 0x15) to the output string, calls 0140c290 (a r… |
| `013da100` | NetworkCommand_Ping::DisplayName | confirmed | Vtable slot [20] (display name) of NetworkCommand_Ping. Bytes 8b 4c 24 04 6a 14 68 fc 1f 8e 01 e8 ..: loads the std::string pointer from the first stack argume… |

**NetworkCommand_QueryDisconnect** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f1af0` | NetworkCommand_QueryDisconnect::ctor | confirmed | Constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_QueryDisconnect::vftable into [this], and returns this. No members… |
| `014f1b70` | NetworkCommand_QueryDisconnect::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_QueryDisconnect" (mov eax, imm; ret). |
| `014f1c00` | NetworkCommand_QueryDisconnect::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 otherwise).… |
| `014f1c80` | NetworkCommand_QueryDisconnect::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 4 bytes with operator new 00d833a6; if non-null, constructs a QueryDisconnect in the block with 014f1af0… |
| `014f1cf0` | NetworkCommand_QueryDisconnect::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_QueryDisconnect; on failure calls 012f5a60 … |
| `014f1e50` | NetworkCommand_QueryDisconnect::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags). Stores NetworkCommand_QueryDisconnect::vftable into [this], calls the base bod… |
| `014f1eb0` | NetworkCommand_QueryDisconnect::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 32-byte literal "QueryDisconnect Network Command\n" to the argument through… |
| `014f1ee0` | NetworkCommand_QueryDisconnect::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 31-byte literal "QueryDisconnect Network Command" via … |
| `014f1b80` | NetworkCommand_QueryDisconnect::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe950 (mov eax,[DAT_01cbe950]; ret). No arguments. |
| `014f1b90` | NetworkCommand_QueryDisconnect::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe950; returns with ret 4. |
| `014f1bb0` | NetworkCommand_QueryDisconnect::~NetworkCommand_QueryDiscon… | likely | Thiscall body (ECX=this, EH frame). Stores NetworkCommand_QueryDisconnect::vftable into [this] and calls the base body 0140c160 on this. It owns no members, so… |
| `014f1dd0` | NetworkCommand_QueryDisconnect::Factory | likely | Allocates 4 bytes with operator new 00d833a6; if the allocation is non-null, constructs a QueryDisconnect in it with 014f1af0 and returns the object, otherwise… |
| `014f1e30` | NetworkCommand_QueryDisconnect::RegisterFactory | likely | Registers the QueryDisconnect factory 014f1dd0 under classID 0x85: calls 013ce130(0x85, 014f1dd0) and then 013ce230(0x85, 014f1dd0). No arguments, no return va… |

**NetworkCommand_RequestData** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013ee0d0` | NetworkCommand_RequestData::GetClassName | confirmed | Virtual slot 11. Returns the address of the C string NetworkCommand_RequestData (bytes B8 AC 30 8E 01 C3: mov eax, imm32; ret). |
| `013ee200` | NetworkCommand_RequestData::CloneInto | confirmed | Virtual slot 18. Dynamic-casts its target argument from NetworkCommand to NetworkCommand_RequestData (assert clone at line 0x2e on failure, source NetworkComma… |
| `013ee3e0` | NetworkCommand_RequestData::DumpDataVersion | confirmed | Virtual slot 19. Appends the text Update Data Version Network Command followed by a newline through 01241650, formats Data Version: %d with the value at this+4… |
| `013ee470` | NetworkCommand_RequestData::AppendTitle | confirmed | Virtual slot 20. Calls 01241650 with the 35-character text Update Data Version Network Command and returns. |
| `013ee050` | NetworkCommand_RequestData::NetworkCommand_RequestData | likely | Constructor body: calls 0140c090 for base initialisation, stores the NetworkCommand_RequestData vftable at *this, zeroes the DWORD at this+4 and returns this. |
| `013ee110` | NetworkCommand_RequestData::~NetworkCommand_RequestData | likely | Destructor body: stores the NetworkCommand_RequestData vftable at *param_1, then calls 0140c160 for base teardown; no free in this body. |
| `013ee2a0` | NetworkCommand_RequestData::WriteFields | likely | Virtual slot 10, thiscall with one argument. Calls this->vtable +8 (slot 2) with the argument, passes the trace texts Starting NetworkCommand_RequestData, Mess… |
| `013ee380` | NetworkCommand_RequestData::ScalarDeletingDtor | likely | Virtual slot 0. Stores the NetworkCommand_RequestData vftable at *this, calls 0140c160 (base teardown), frees this when flag bit 0 is set, and returns this. |

**NetworkCommand_SessionKick** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f1650` | NetworkCommand_SessionKick::ctor | confirmed | Constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_SessionKick::vftable into [this], zeroes the dword at this+4, and … |
| `014f16d0` | NetworkCommand_SessionKick::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_SessionKick" (mov eax, imm; ret). |
| `014f1780` | NetworkCommand_SessionKick::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 otherwise).… |
| `014f1810` | NetworkCommand_SessionKick::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 8 bytes with operator new 00d833a6; if non-null, constructs a SessionKick in the block with 014f1650. Th… |
| `014f1880` | NetworkCommand_SessionKick::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_SessionKick; on failure calls 012f… |
| `014f19e0` | NetworkCommand_SessionKick::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags). Stores NetworkCommand_SessionKick::vftable into [this], calls the base body 01… |
| `014f1a40` | NetworkCommand_SessionKick::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 29-byte literal "Session Kick Network Command\n" via 01241650, calls the ba… |
| `014f1ad0` | NetworkCommand_SessionKick::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 28-byte literal "Session Kick Network Command" via 012… |
| `014f16e0` | NetworkCommand_SessionKick::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe94c (mov eax,[DAT_01cbe94c]; ret). No arguments. |
| `014f16f0` | NetworkCommand_SessionKick::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe94c; returns with ret 4. |
| `014f1710` | NetworkCommand_SessionKick::~NetworkCommand_SessionKick | likely | Thiscall body (ECX=this, EH frame). Stores NetworkCommand_SessionKick::vftable into [this], then calls the base body 0140c160 on this. It owns no members of it… |
| `014f1960` | NetworkCommand_SessionKick::Factory | likely | Allocates 8 bytes with operator new 00d833a6; if the allocation is non-null, constructs a SessionKick in it with 014f1650 and returns the object, otherwise ret… |
| `014f19c0` | NetworkCommand_SessionKick::RegisterFactory | likely | Registers the SessionKick factory 014f1960 under classID 0x1fb: calls 013ce130(0x1fb, 014f1960) and then 013ce230(0x1fb, 014f1960). No arguments, no return val… |

**NetworkCommand_SystemMessage** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f1140` | NetworkCommand_SystemMessage::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 otherwise).… |
| `014f1220` | NetworkCommand_SystemMessage::Write | confirmed | Virtual slot [10] (ECX=this; stack argument = output stream). Calls vtable slot [2] (header write), the trace call 013c26e0 (no-op), the base Write 0140c2d0, w… |
| `014f1330` | NetworkCommand_SystemMessage::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_SystemMessage" (mov eax, imm; ret). |
| `014f1370` | NetworkCommand_SystemMessage::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f1290 on this, then if flag bit 0 is set calls _f… |
| `014f1390` | NetworkCommand_SystemMessage::ctor | confirmed | Constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_SystemMessage::vftable into [this], zeroes the dword at this+0x18,… |
| `014f1440` | NetworkCommand_SystemMessage::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x20 bytes with operator new 00d833a6; if non-null, constructs a SystemMessage in the block with 014f139… |
| `014f14b0` | NetworkCommand_SystemMessage::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_SystemMessage; on failure calls 01… |
| `014f1510` | NetworkCommand_SystemMessage::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 31-byte literal "System Message Network Command\n" via 01241650, calls the … |
| `014f15b0` | NetworkCommand_SystemMessage::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 30-byte literal "System Message Network Command" via 0… |
| `014f1290` | NetworkCommand_SystemMessage::~NetworkCommand_SystemMessage | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_SystemMessage::vftable into [this]. If the capacity dword at this+0x1c is above 0xf, frees the… |
| `014f1340` | NetworkCommand_SystemMessage::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe948 (mov eax,[DAT_01cbe948]; ret). No arguments. |
| `014f1350` | NetworkCommand_SystemMessage::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe948; returns with ret 4. |
| `014f15d0` | NetworkCommand_SystemMessage::Factory | likely | Allocates 0x20 bytes with operator new 00d833a6; if the allocation is non-null, constructs a SystemMessage in it with 014f1390 and returns the object, otherwis… |
| `014f1630` | NetworkCommand_SystemMessage::RegisterFactory | likely | Registers the SystemMessage factory 014f15d0 under classID 0x88: calls 013ce130(0x88, 014f15d0) and then 013ce230(0x88, 014f15d0). No arguments, no return valu… |

**NetworkCommand_Time** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f09e0` | NetworkCommand_Time::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 if the allocation succeeds (0 otherwise). C… |
| `014f0b90` | NetworkCommand_Time::ctor | confirmed | Constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_Time::vftable into [this], then zeroes the dwords at this+4 throug… |
| `014f0c30` | NetworkCommand_Time::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_Time" (mov eax, imm; ret). |
| `014f0cf0` | NetworkCommand_Time::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x40 bytes with operator new 00d833a6; if non-null, constructs a Time command in the block with 014f0b90… |
| `014f0fa0` | NetworkCommand_Time::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f0c70 on this, then if flag bit 0 is set calls _f… |
| `014f1000` | NetworkCommand_Time::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_Time; on failure calls 012f5a60 wi… |
| `014f1090` | NetworkCommand_Time::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 21-byte literal "Time Network Command\n" via 01241650, calls the base Dump … |
| `014f1120` | NetworkCommand_Time::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument and appends the 20-byte literal "Time Network Command" via 01241650 (no newline); return… |
| `014f0c40` | NetworkCommand_Time::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe944 (mov eax,[DAT_01cbe944]; ret). No arguments. |
| `014f0c50` | NetworkCommand_Time::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe944; returns with ret 4. |
| `014f0c70` | NetworkCommand_Time::~NetworkCommand_Time | likely | Thiscall body with an EH frame (ECX=this). Stores NetworkCommand_Time::vftable into [this]. If the capacity dword at this+0x34 is greater than 0xf, frees the h… |
| `014f0f20` | NetworkCommand_Time::Factory | likely | Allocates 0x40 bytes with operator new 00d833a6; if the allocation is non-null, constructs a Time command in it with 014f0b90 and returns the object, otherwise… |
| `014f0f80` | NetworkCommand_Time::RegisterFactory | likely | Registers the Time factory 014f0f20 under classID 0x138: calls 013ce130(0x138, 014f0f20) and then 013ce230(0x138, 014f0f20). No arguments, no return value. |

**NetworkCommand_UpdateDataVersion** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f0280` | NetworkCommand_UpdateDataVersion::ClientHandler | confirmed | Virtual slot [15] (ECX=this). Allocates 0x14 bytes with operator new 00d833a6 and constructs an event with 012f9eb0 when the allocation succeeds (0 otherwise);… |
| `014f04b0` | NetworkCommand_UpdateDataVersion::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 36-byte literal "Update Data Version Network Command\n" via 01241650, calls… |
| `014f0540` | NetworkCommand_UpdateDataVersion::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument and appends the 35-byte literal "Update Data Version Network Command" via 01241650 (no n… |
| `014f0640` | NetworkCommand_UpdateDataVersion::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_UpdateDataVersion" (mov eax, imm; ret). |
| `014f0680` | NetworkCommand_UpdateDataVersion::ctor | confirmed | Constructor (ECX=this, EH frame): calls the base constructor 0140c090, stores NetworkCommand_UpdateDataVersion::vftable into [this], zeroes the dword at this+4… |
| `014f0760` | NetworkCommand_UpdateDataVersion::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014f0560 on this, then if flag bit 0 is set calls _f… |
| `014f0780` | NetworkCommand_UpdateDataVersion::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 0x28 bytes with operator new 00d833a6; if non-null, constructs an UpdateDataVersion in the block with 01… |
| `014f0870` | NetworkCommand_UpdateDataVersion::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this = source; stack argument = destination). RTTI-casts the argument with 00d8d382 to NetworkCommand_UpdateDataVersion; on failure call… |
| `014f0560` | NetworkCommand_UpdateDataVersion::~NetworkCommand_UpdateDat… | likely | Thiscall body (ECX=this, EH frame). Stores NetworkCommand_UpdateDataVersion::vftable into [this]. For the buffer triple at this+0x1c (pointer, +0x20, +0x24) wi… |
| `014f0650` | NetworkCommand_UpdateDataVersion::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe940 (mov eax,[DAT_01cbe940]; ret). No arguments. |
| `014f0660` | NetworkCommand_UpdateDataVersion::SetClassDword | likely | Virtual slot [13]. Stores its stack argument into the global DAT_01cbe940; returns with ret 4. |
| `014f07f0` | NetworkCommand_UpdateDataVersion::Factory | likely | Allocates 0x28 bytes with operator new 00d833a6; if the allocation is non-null, constructs an UpdateDataVersion in it with 014f0680 and returns the object, oth… |
| `014f0850` | NetworkCommand_UpdateDataVersion::RegisterFactory | likely | Registers the UpdateDataVersion factory 014f07f0 under classID 0x222: calls 013ce130(0x222, 014f07f0) and then 013ce230(0x222, 014f07f0). No arguments, no retu… |

**NetworkCommand_UpdateLag** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014efdc0` | NetworkCommand_UpdateLag::Dump | confirmed | Virtual slot [19] (ECX=this; stack argument = destination std::string). Appends the 27-byte literal "Update Lag Network Command\n" via 01241650, calls the base… |
| `014efeb0` | NetworkCommand_UpdateLag::AppendTypeName | confirmed | Virtual slot [20]. Takes the std::string& passed as the stack argument (mov ecx,[esp+4]) and appends the 25-byte literal "UpdateLag Network Command" via 012416… |
| `014eff50` | NetworkCommand_UpdateLag::GetTypeName | confirmed | Virtual slot [11]. Returns the address of the literal "NetworkCommand_UpdateLag" (mov eax, imm; ret). |
| `014f0010` | NetworkCommand_UpdateLag::CopyFieldsTo | confirmed | Virtual slot [18] (ECX=this; stack argument = object cast to NetworkCommand_UpdateLag). RTTI-casts the argument with 00d8d382 (source type NetworkCommand, targ… |
| `014f00f0` | NetworkCommand_UpdateLag::ctor | confirmed | Thiscall (ECX=this) with an MSVC EH frame: calls the base constructor 0140c090 on this (it stores the PersistentBase, PersistentComponent and NetworkCommand vf… |
| `014f0150` | NetworkCommand_UpdateLag::DeletingDtor | confirmed | Virtual slot [0], the scalar deleting destructor (ECX=this, stack byte = flags): calls the destructor body 014efed0 on this, then if flag bit 0 is set calls _f… |
| `014f0170` | NetworkCommand_UpdateLag::Clone | confirmed | Virtual slot [17] (ECX=the original object). Allocates 16 bytes with operator new 00d833a6; if the allocation is non-null, constructs an UpdateLag in it with 0… |
| `014efd50` | NetworkCommand_UpdateLag::Write | likely | Virtual slot [10] (98 bytes): calls vtable slot [2] on param_2; calls 013c26e0 (no-op trace), 0140c2d0 (base step on param_2), a no-op trace, 01351620 with thi… |
| `014efed0` | NetworkCommand_UpdateLag::~NetworkCommand_UpdateLag | likely | Thiscall body with an MSVC EH frame: stores NetworkCommand_UpdateLag::vftable into [ECX], calls the member routine 00521a9c on this+4, then calls the base body… |
| `014eff60` | NetworkCommand_UpdateLag::GetClassDword | likely | Virtual slot [12]. Returns the dword in the global DAT_01cbe93c (mov eax,[DAT_01cbe93c]; ret). No arguments. |
| `014eff70` | NetworkCommand_UpdateLag::SetClassDword | likely | Virtual slot [13]. Stores its stack argument (mov eax,[esp+4]) into the global DAT_01cbe93c; returns with ret 4. |
| `014f01e0` | NetworkCommand_UpdateLag::Factory | likely | Allocates 16 bytes with operator new 00d833a6; if the allocation is non-null, constructs an UpdateLag in it with 014f00f0 and returns the object, otherwise ret… |
| `014f0240` | NetworkCommand_UpdateLag::RegisterFactory | likely | Registers the UpdateLag factory 014f01e0 under classID 0x69: calls 013ce130(0x69, 014f01e0) and then 013ce230(0x69, 014f01e0). No arguments, no return value. |

**Order** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f9700` | Order::TypeName | confirmed | Returns the C string 'Order' (no arguments, no side effects). |
| `014f9750` | Order::~Order | confirmed | Scalar deleting destructor: calls the Order dtor body 014f9580, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014f9840` | Order::Order | confirmed | Fastcall with the new object in ECX. Stores PersistentBase and PersistentComponent vtables, calls 01402960 (base constructor body), stores Order::vftable at +0… |
| `014f9580` | Order::DtorBody | likely | Stores Order::vftable at +0 and +4; if the object at +8 is non-null calls its virtual slot 0 with flag 1 and nulls the pointer; for each element of the pointer… |
| `014f9710` | Order::GetClassStatic | likely | Returns the dword at DAT_01cbe980 (no arguments). |
| `014f9720` | Order::SetClassStatic | likely | Stores its argument into DAT_01cbe980 (one stack argument, void). |
| `014f9740` | OrderSecondaryDtorThunk | likely | Thunk on the secondary Order base: adjusts ECX to the primary object and jumps to the deleting destructor 014f9750. Nothing else. |
| `014f9900` | Order::Create | likely | No arguments. Allocates 0x1c bytes with operator new (00d833a6); when the allocation succeeds it calls the Order constructor 014f9840 and returns the object; r… |
| `014f9960` | Order::RegisterFactory | likely | Pushes the factory 014f9900 and the constant 0x1ba, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x1ba (ret 8)… |

**OrderItem** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014f9980` | OrderItem::OrderItem | confirmed | Fastcall with the new object in ECX. Stores PersistentBase and PersistentComponent vtables, calls 01402960 (base constructor body), stores OrderItem::vftable a… |
| `014f9a50` | OrderItem::TypeName | confirmed | Returns the C string 'OrderItem' (no arguments, no side effects). |
| `014f9c20` | OrderItem::~OrderItem | confirmed | Scalar deleting destructor: calls the OrderItem dtor body 014f9a90, then frees this when bit 0 of the flag argument is set. Returns this. |
| `014f9a60` | OrderItem::GetClassStatic | likely | Returns the dword at DAT_01cbe984 (no arguments). |
| `014f9a70` | OrderItem::SetClassStatic | likely | Stores its argument into DAT_01cbe984 (one stack argument, void). |
| `014f9a90` | OrderItem::DtorBody | likely | Stores OrderItem::vftable at +0 and +4; if the object at +8 is non-null calls its virtual slot 0 with flag 1 and nulls the pointer; calls 01402970 (base destru… |
| `014f9b90` | OrderItemSecondaryDtorThunk | likely | Thunk on the secondary OrderItem base: adjusts ECX to the primary object and jumps to the deleting destructor 014f9c20. Nothing else. |
| `014f9ba0` | OrderItem::Create | likely | No arguments. Allocates 0xc bytes with operator new (00d833a6); when the allocation succeeds it calls the OrderItem constructor 014f9980 and returns the object… |
| `014f9c00` | OrderItem::RegisterFactory | likely | Pushes the factory 014f9ba0 and the constant 0x1bb, calls 013ce130, moves its EAX into ECX and calls 013ce230, which stores the factory under key 0x1bb (ret 8)… |

**PickListState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142b360` | PickListState::Read | confirmed | Header read (vtable slot 4) on the reader, base read FUN_0147c850 with the reader, then four integers into the object at +0xf8 (PlayerID), +0xfc (OriginCard), … |
| `0142b440` | PickListState::Write | confirmed | When the argument is null, obtains the writer through vtable slot 1 (0x4); otherwise uses the argument as the writer and calls vtable slot 2 (0x8) on it. Then … |

**PlayElement** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01344120` | PlayElement::Factory | likely | Allocates 0x48 bytes with operator new (00d833a6). If the allocation is non-null it calls 01343d20 (PlayElement constructor) on it, then returns the object poi… |
| `013441a0` | PlayElement::RegisterFactory | likely | Static initialiser: calls 013ce130 and 013ce230 with classID 0xa7 and the factory 01344120, registering that factory under the classID in the command-factory r… |

**Player** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01346340` | Player::Factory | likely | Allocates 0x4c bytes with operator new (00d833a6). When the allocation is non-null it calls 01345ef0 (Player constructor) on it and returns the result; returns… |

**Poll** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d7090` | Poll::~Poll | confirmed | Destructor body: stores Poll::vftable; deletes each object in the pointer list at +0x44..+0x48 through its vtable slot 0 with flag 1 (owned elements), frees th… |
| `013d7240` | Poll::TypeName | confirmed | Virtual slot [11] of Poll: returns the address of the 4-byte literal at 018e1680 (bytes "Poll" with a terminator). |
| `013d7250` | Poll::GetStaticDword | confirmed | Virtual slot [12] of Poll: returns the global dword at 01cbde28. |
| `013d7260` | Poll::SetStaticDword | confirmed | Virtual slot [13] of Poll: stores its stack argument into the global dword 01cbde28. |
| `013d72b0` | Poll::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): calls the destructor body 013d7090, then frees the object with _free only when bit 0 of the flag argument is set;… |
| `013d77d0` | Poll::Poll | confirmed | Constructor: stores PersistentBase, PersistentComponent and Poll vtables in that order, sets the dword at +4 to 0xffffffff, zeroes the string headers at +0xc a… |

**PollChoice** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01442d70` | PollChoice::RegisterFactory | likely | Pushes the constant 0x186 and the address 01442d10, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**PollCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014430d0` | PollCommand::RegisterFactory | likely | Pushes the constant 0x187 and the address 01443070, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**PollCommand_AddPollControl** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d7b60` | PollCommand_AddPollControl::PollCommand_AddPollControl | confirmed | Constructor: calls the base constructor 01442df0 first, then stores PollCommand_AddPollControl::vftable and zeroes the dword at +4 (the one integer field of th… |
| `013d7be0` | PollCommand_AddPollControl::TypeName | confirmed | Virtual slot [11]: returns the address of the string "PollCommand_AddPollControl". |
| `013d7bf0` | PollCommand_AddPollControl::GetStaticDword | confirmed | Virtual slot [12]: returns the global dword at 01cbde2c. |
| `013d7c00` | PollCommand_AddPollControl::SetStaticDword | confirmed | Virtual slot [13]: stores its stack argument into the global dword 01cbde2c. |
| `013d7c80` | PollCommand_AddPollControl::Clone | confirmed | Virtual slot [17] (Clone): allocates 8 bytes with 00d833a6, constructs the object with 013d7b60 when the allocation succeeded, then calls vtable slot +0x48 (sl… |
| `013d7cf0` | PollCommand_AddPollControl::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination to PollCommand_AddPollControl through 00d8d382 (NetworkCommand and PollCommand_AddPollControl… |
| `013d7e70` | PollCommand_AddPollControl::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores PollCommand_AddPollControl::vftable, calls the base destructor 01442eb0, then frees the object with _free … |
| `013d7fa0` | PollCommand_AddPollControl::Dump | confirmed | Virtual slot [19] (Dump): appends the text "Poll Command Add Poll Control\n" through 01241650, calls the base dump thunk 0140c290 (01442fd0) on the output argu… |
| `013d8010` | PollCommand_AddPollControl::DisplayName | confirmed | Virtual slot [20] (display name): appends the 0x1d-byte literal "Poll Command Add Poll Control" through 01241650 and returns. |

**RuleSetData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c5b70` | RuleSetData::Write 🌐 | confirmed | Serializer for RuleSetData: calls header-write slot [2] on the stream, appends the RuleSetID word at +4 with 013c23f0, appends the flag integer 1 when the expr… |
| `013c5ca0` | RuleSetData::Read 🌐 | confirmed | Deserializer for RuleSetData: calls header-read slot [4] on the stream and returns false when it fails, reads one integer into the word at +4 (RuleSetID) with … |

**StateMachineState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145cdc0` | StateMachineState::Write | confirmed | Virtual slot [10] of StateMachineState::vftable, thiscall with one stream argument (0 selects slot [1] to obtain a stream, otherwise slot [2] with the argument… |
| `0145d240` | StateMachineState::Read | confirmed | Virtual slot [9] of StateMachineState::vftable, thiscall with one stream argument. Calls slot [4] (+0x10) for the header; asserts 'false' (StateMachineState.cp… |

**SynchronizationCommand** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01435370` | SynchronizationCommand::SynchronizationCommand | confirmed | Constructor (ECX = object, returns this). Calls 0140c090 on the object first (base constructor), then stores SynchronizationCommand::vftable at +0. |
| `014353f0` | SynchronizationCommand::TypeName | confirmed | Returns the address of the literal SynchronizationCommand (mov eax, imm32; ret). |
| `01435400` | SynchronizationCommand::GetStaticDword | confirmed | Returns the dword at global DAT_01cbe2bc (mov eax,[imm32]; ret). |
| `01435410` | SynchronizationCommand::SetStaticDword | confirmed | Stores its one stack argument into global DAT_01cbe2bc and returns with ret 4 (mov eax,[esp+4]; mov [imm32],eax; ret 4). |
| `01435430` | SynchronizationCommand::~SynchronizationCommand | confirmed | Destructor body (ECX = object, no flag). Stores SynchronizationCommand::vftable at +0, then calls 0140c160 on the object (ECX = this), which references the Net… |
| `01435490` | SynchronizationCommand::Clone | confirmed | Virtual slot [17] (+0x44) of SynchronizationCommand (ECX = source). Allocates with operator new using the immediate 4 (push 4), constructs a SynchronizationCom… |
| `01435630` | SynchronizationCommand::ScalarDeletingDestructor | confirmed | Thiscall with one byte flag argument. Stores SynchronizationCommand::vftable at +0, calls 0140c160 on the object (the NetworkCommand base destructor, same as 0… |
| `01435690` | SynchronizationCommand::AppendDisplayName | confirmed | Virtual slot [20] (+0x50) of SynchronizationCommand (one stack argument, ret 4). Loads ECX from the argument, pushes the length 0x17 and the literal Synchroniz… |
| `014355b0` | SynchronizationCommand::Create | likely | Allocates with operator new using the immediate 4 (push 4). If the allocation is non-null it constructs a SynchronizationCommand with 01435370 and returns the … |
| `01435610` | SynchronizationCommand::RegisterFactory | likely | Static initializer: registers the factory 014355b0 under classID 0x52 (82) with 013ce130, then calls 013ce230 with the returned value in ECX (the same registry… |

**SynchronizationCommand_ChangeFlags** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014adaa0` | SynchronizationCommand_ChangeFlags::ApplyToInstances | confirmed | Virtual slot 15 (fastcall, ECX=this, returns 1). Calls 012adee0 and 012ae3f0 on the account at this+0x14; if the result is non-null, walks the instance array (… |
| `014adbc0` | SynchronizationCommand_ChangeFlags::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; base write 01435560; no-op trace label Instances; 013d3e10 writes the instanc… |
| `014adcb0` | SynchronizationCommand_ChangeFlags::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_ChangeFlags::vftable into *this, destroys the instance container at this+0x04 through 012c2de0 (ECX = this+4… |
| `014add30` | SynchronizationCommand_ChangeFlags::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_ChangeFlags. |
| `014ae070` | SynchronizationCommand_ChangeFlags::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Calls the DtorBody 014adcb0, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `014ae090` | SynchronizationCommand_ChangeFlags::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends the literal SynchronizationCommand_ChangeFlags newline first, calls the base dump thunk 01435510… |
| `014ae1e0` | SynchronizationCommand_ChangeFlags::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4, same 19-byte shape as the other AppendTypeName functions). Loads ECX from the argument and appends the 36-byte lite… |
| `014ae2a0` | SynchronizationCommand_ChangeFlags::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand base constructor 01435370, stores SynchronizationCommand_ChangeFlags::vftable, and zeroes the… |
| `014ae380` | SynchronizationCommand_ChangeFlags::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Runs a dynamic_cast of the destination from NetworkCommand to ChangeFlags (reports an asse… |
| `014ae310` | SynchronizationCommand_ChangeFlags::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x20 bytes with operator new; if non-null, constructs a ChangeFlags on it with 014ae2a0. Ca… |
| `014ae420` | SynchronizationCommand_ChangeFlags::Factory | likely | No arguments. Allocates 0x20 bytes with operator new; if non-null, constructs a ChangeFlags with 014ae2a0 and returns it, otherwise returns 0. |
| `014ae480` | SynchronizationCommand_ChangeFlags::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x1db and factory 014ae420, then 013ce230 with the same pair (ECX from 013ce130's return). |

**SynchronizationCommand_Complete** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013d5f60` | SynchronizationCommand_Complete::SynchronizationCommand_Com… | confirmed | Constructor: calls the base constructor 01435370 first, then stores SynchronizationCommand_Complete::vftable and zeroes the dword at +4 (the DeliveryID field),… |
| `013d5fe0` | SynchronizationCommand_Complete::TypeName | confirmed | Virtual slot [11] of SynchronizationCommand_Complete: returns the address of the string "SynchronizationCommand_Complete". |
| `013d5ff0` | SynchronizationCommand_Complete::GetStaticDword | confirmed | Virtual slot [12] of SynchronizationCommand_Complete: returns the global dword at 01cbde24. |
| `013d6000` | SynchronizationCommand_Complete::SetStaticDword | confirmed | Virtual slot [13] of SynchronizationCommand_Complete: stores its stack argument into the global dword 01cbde24. |
| `013d6020` | SynchronizationCommand_Complete::~SynchronizationCommand_Co… | confirmed | Destructor body: stores SynchronizationCommand_Complete::vftable, then calls the base destructor 01435430. Does not free the object (no flag test). |
| `013d60b0` | SynchronizationCommand_Complete::Clone | confirmed | Virtual slot [17] (Clone): allocates 8 bytes with 00d833a6, constructs the object with 013d5f60 when the allocation succeeded, then calls vtable slot +0x48 (sl… |
| `013d6120` | SynchronizationCommand_Complete::CopyFieldsTo | confirmed | Virtual slot [18] (CopyFieldsTo): dynamic_cast of the destination (param_2) to SynchronizationCommand_Complete through 00d8d382 with the NetworkCommand and Syn… |
| `013d62b0` | SynchronizationCommand_Complete::DeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]): stores SynchronizationCommand_Complete::vftable, calls the base destructor 01435430, then frees the object with _… |
| `013d6310` | SynchronizationCommand_Complete::Dump | confirmed | Virtual slot [19] (Dump): appends the text "Complete Synchronization Command\n" through 01241650 (std::string::append), formats "DeliveryID: %d\n" with _sprint… |
| `013d6390` | SynchronizationCommand_Complete::DisplayName | confirmed | Virtual slot [20] (display name): loads the output string from the stack argument into ECX and appends the 0x20-byte literal "Complete Synchronization Command"… |

**SynchronizationCommand_LockTournament** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ad3c0` | SynchronizationCommand_LockTournament::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand_ChangeFlags constructor 014ae2a0, stores SynchronizationCommand_LockTournament::vftable, then… |
| `014ad440` | SynchronizationCommand_LockTournament::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_LockTournament. |
| `014ad480` | SynchronizationCommand_LockTournament::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_LockTournament::vftable into *this, then calls the ChangeFlags destructor body 014adcb0 on this; no free. |
| `014ad560` | SynchronizationCommand_LockTournament::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; then 014ae550 (the next inheritance level read); then vtbl[5](buf). Return… |
| `014ad5a0` | SynchronizationCommand_LockTournament::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014adbc0 (ChangeFlags write level); no-op trace; vtbl[3](buf). |
| `014ad670` | SynchronizationCommand_LockTournament::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the LockTournament vtable, calls the ChangeFlags destructor body 014adcb0, then calls _free(this) when bit… |
| `014ad6d0` | SynchronizationCommand_LockTournament::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends the 38-byte literal SynchronizationCommand_LockTournament newline, calls the base dump thunk 01435510 (… |
| `014ad850` | SynchronizationCommand_LockTournament::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 49-byte literal Lock Tournament Instances Synchronization Command with… |
| `014ad4f0` | SynchronizationCommand_LockTournament::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x20 bytes with operator new; if non-null, constructs a LockTournament on it with 014ad3c0.… |
| `014ad5f0` | SynchronizationCommand_LockTournament::Factory | likely | No arguments. Allocates 0x20 bytes with operator new; if non-null, constructs a LockTournament with 014ad3c0 and returns it, otherwise returns 0. |
| `014ad650` | SynchronizationCommand_LockTournament::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x1dc and factory 014ad5f0, then 013ce230 with the same pair (ECX from 013ce130's return). |

**SynchronizationCommand_ProcessBulkFulfillment** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0143aeb0` | SynchronizationCommand_ProcessBulkFulfillment::~Synchroniza… | confirmed | Destructor body with EH frame: stores SynchronizationCommand_ProcessBulkFulfillment::vftable into *this, calls FUN_012c2de0 (member cleanup) and FUN_01435430 o… |
| `0143af30` | SynchronizationCommand_ProcessBulkFulfillment::GetTypeName | confirmed | Virtual slot 11. Bytes: mov eax, 0x018e937c; ret. Returns a pointer to the literal class name 'SynchronizationCommand_ProcessBulkFulfillment'. |
| `0143af70` | SynchronizationCommand_ProcessBulkFulfillment::~Synchroniza… | confirmed | Scalar deleting destructor (virtual slot 0): calls FUN_0143aeb0 (destructor body) on this, then if param_2 bit 0 is set calls _free on this; returns this. 27 b… |
| `0143afe0` | SynchronizationCommand_ProcessBulkFulfillment::Synchronizat… | confirmed | Constructor (fastcall, returns this): calls FUN_01435370 on this (base constructor; its map entry references SynchronizationCommand::vftable), stores Synchroni… |
| `0143b120` | CreateSynchronizationCommand_ProcessBulkFulfillment | likely | Static factory with no arguments. Calls FUN_00d833a6(0x14) (allocation of 0x14 bytes, see 0143b050); if the result is non-null calls 0143afe0 on it and returns… |
| `0143b200` | SynchronizationCommand_ProcessBulkFulfillment::ProcessStrea… | likely | Virtual slot 9. Calls this->vtbl+0x10(param_2, &local) and stops (returns 0) when that is false; otherwise calls FUN_01435520(param_2), and on success stores p… |

**SynchronizationCommand_RequestAttachHolocron** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013a9b00` | SynchronizationCommand_RequestAttachHolocron::Ctor | confirmed | Fastcall (ECX=command object). Calls 01435370 (base constructor), stores SynchronizationCommand_RequestAttachHolocron::vftable into the object, and sets the tw… |
| `013a9b80` | SynchronizationCommand_RequestAttachHolocron::TypeName | confirmed | No arguments. Returns the C string literal SynchronizationCommand_RequestAttachHolocron (6 bytes: mov eax, imm32; ret). |
| `013a9b90` | SynchronizationCommand_RequestAttachHolocron::GetStaticDword | likely | No arguments. Returns the dword at DAT_01cbdc70 (6 bytes: load and ret); the static per-class value of the command class. |

**SynchronizationCommand_RequestConsumeHolocronCharge** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0140bba0` | SynchronizationCommand_RequestConsumeHolocronCharge::DtorBo… | confirmed | Stores the SynchronizationCommand_RequestConsumeHolocronCharge vtable into *param_1, frees the pointer at +8 when non-null, zeroes +8, +0xc and +0x10, then cal… |
| `0140bc30` | SynchronizationCommand_RequestConsumeHolocronCharge::GetCla… | confirmed | Returns the address of the literal class-name string 'SynchronizationCommand_RequestConsumeHolocronCharge' (0x018e4b54) in EAX; no arguments (6 bytes: mov eax,… |
| `0140bcd0` | SynchronizationCommand_RequestConsumeHolocronCharge::Scalar… | confirmed | Scalar deleting destructor: calls the body teardown 0140bba0 on ECX, frees the object when bit 0 of the stack argument is set, returns this (ret 4). |
| `0140bd40` | SynchronizationCommand_RequestConsumeHolocronCharge::Ctor | likely | Calls base constructor 01435370 (SynchronizationCommand), stores the SynchronizationCommand_RequestConsumeHolocronCharge vtable into *param_1, zeroes +8, +0xc … |
| `0140be80` | SynchronizationCommand_RequestConsumeHolocronCharge::Create | likely | Allocates 0x14 bytes through 00d833a6; returns 0 when that fails, otherwise builds the command with 0140bd40 and returns it. |

**SynchronizationCommand_RespondAttachHolocron** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014acac0` | SynchronizationCommand_RespondAttachHolocron::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand base constructor 01435370, stores SynchronizationCommand_RespondAttachHolocron::vftable, and … |
| `014acb40` | SynchronizationCommand_RespondAttachHolocron::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_RespondAttachHolocron. |
| `014acb80` | SynchronizationCommand_RespondAttachHolocron::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_RespondAttachHolocron::vftable into *this, then calls the SynchronizationCommand destructor body 01435430 on… |
| `014accf0` | SynchronizationCommand_RespondAttachHolocron::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Runs a dynamic_cast of the destination from NetworkCommand to this class; if that is null,… |
| `014acd40` | SynchronizationCommand_RespondAttachHolocron::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; 01435520 base read; 013c24d0 reads one integer into this+0x04; then vtbl[5… |
| `014acd90` | SynchronizationCommand_RespondAttachHolocron::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; base write 01435560; 013c23f0 writes this+0x04 as an integer; no-op trace; vt… |
| `014ace60` | SynchronizationCommand_RespondAttachHolocron::ScalarDeletin… | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the class vtable, calls the SynchronizationCommand destructor body 01435430, then calls _free(this) when b… |
| `014acec0` | SynchronizationCommand_RespondAttachHolocron::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends the 46-byte literal RespondAttachHolocron Synchronization Command newline, then calls the base d… |
| `014acef0` | SynchronizationCommand_RespondAttachHolocron::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 45-byte literal RespondAttachHolocron Synchronization Command with 012… |
| `014acc80` | SynchronizationCommand_RespondAttachHolocron::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 8 bytes with operator new; if non-null, constructs the command on it with 014acac0. Calls t… |
| `014acde0` | SynchronizationCommand_RespondAttachHolocron::Factory | likely | No arguments. Allocates 8 bytes with operator new; if non-null, constructs the command with 014acac0 and returns it, otherwise returns 0. |
| `014ace40` | SynchronizationCommand_RespondAttachHolocron::RegisterFacto… | likely | No arguments. Calls 013ce130 with key 0x23c and factory 014acde0, then 013ce230 with the same pair (ECX from 013ce130's return). |

**SynchronizationCommand_RespondConsumeHolocronCharge** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ac670` | SynchronizationCommand_RespondConsumeHolocronCharge::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand base constructor 01435370, stores SynchronizationCommand_RespondConsumeHolocronCharge::vftabl… |
| `014ac6f0` | SynchronizationCommand_RespondConsumeHolocronCharge::GetCla… | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_RespondConsumeHolocronCharge. |
| `014ac730` | SynchronizationCommand_RespondConsumeHolocronCharge::DtorBo… | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_RespondConsumeHolocronCharge::vftable into *this, then calls the SynchronizationCommand destructor body 0143… |
| `014ac8a0` | SynchronizationCommand_RespondConsumeHolocronCharge::CopyFi… | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Runs a dynamic_cast of the destination from NetworkCommand to this class; if that is null,… |
| `014ac8f0` | SynchronizationCommand_RespondConsumeHolocronCharge::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). vtbl[4](buf,&buf) header read; 01435520 base read; 013c24d0 reads one integer into this+0x04; then vtbl[5… |
| `014ac940` | SynchronizationCommand_RespondConsumeHolocronCharge::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; base write 01435560 (SynchronizationCommand base, slot 10); 013c23f0 writes t… |
| `014aca10` | SynchronizationCommand_RespondConsumeHolocronCharge::Scalar… | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the class vtable, calls the SynchronizationCommand destructor body 01435430, then calls _free(this) when b… |
| `014aca70` | SynchronizationCommand_RespondConsumeHolocronCharge::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends the 53-byte literal RespondConsumeHolocronCharge Synchronization Command newline, then calls the… |
| `014acaa0` | SynchronizationCommand_RespondConsumeHolocronCharge::Append… | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument and appends the 52-byte literal RespondConsumeHolocronCharge Synchronization Command w… |
| `014ac830` | SynchronizationCommand_RespondConsumeHolocronCharge::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 8 bytes with operator new; if non-null, constructs the command on it with 014ac670. Calls t… |
| `014ac990` | SynchronizationCommand_RespondConsumeHolocronCharge::Factory | likely | No arguments. Allocates 8 bytes with operator new; if non-null, constructs the command with 014ac670 and returns it, otherwise returns 0. |
| `014ac9f0` | SynchronizationCommand_RespondConsumeHolocronCharge::Regist… | likely | No arguments. Calls 013ce130 with key 0x23e and factory 014ac990, then 013ce230 with the same pair (ECX from 013ce130's return). |

**SynchronizationCommand_StartDelivery** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aee50` | SynchronizationCommand_StartDelivery::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_StartDelivery::vftable into *this, destroys two containers through 012c2de0 (ECX set to this+0x4c and to thi… |
| `014aef10` | SynchronizationCommand_StartDelivery::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_StartDelivery. |

**SynchronizationCommand_StartInstances** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ae5e0` | SynchronizationCommand_StartInstances::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand base constructor 01435370, stores SynchronizationCommand_StartInstances::vftable, and zeroes … |
| `014ae660` | SynchronizationCommand_StartInstances::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_StartInstances. |
| `014ae6a0` | SynchronizationCommand_StartInstances::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_StartInstances::vftable into *this, then calls the SynchronizationCommand destructor body 01435430 on this; … |
| `014ae810` | SynchronizationCommand_StartInstances::CopyFieldsInto | confirmed | Thiscall (ECX=source this, one stack argument = destination, ret 4). Runs a dynamic_cast of the destination from NetworkCommand to this class (assertion throug… |
| `014ae900` | SynchronizationCommand_StartInstances::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; base write 01435560; then 013c23f0 writes this+0x04 (AccountID), this+0x08 (C… |
| `014aea30` | SynchronizationCommand_StartInstances::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the StartInstances vtable, calls the SynchronizationCommand destructor body 01435430, then calls _free(thi… |
| `014aea90` | SynchronizationCommand_StartInstances::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends the literal Start Instances Synchronization Command newline, calls the base dump thunk 01435510 … |
| `014aebc0` | SynchronizationCommand_StartInstances::AppendTypeName | confirmed | Virtual slot 20 (one stack argument, ret 4, same 19-byte shape). Loads ECX from the argument and appends the literal Start Instances Synchronization Command wi… |
| `014ae770` | SynchronizationCommand_StartInstances::ApplyToService | likely | Virtual slot 15 (fastcall, ECX=this, no stack arguments, returns 1). Calls 012c5080 with no arguments, then 012c3310 with the four fields this+0x04, +0x08, +0x… |
| `014ae7a0` | SynchronizationCommand_StartInstances::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x14 bytes with operator new; if non-null, constructs a StartInstances on it with 014ae5e0.… |
| `014ae9b0` | SynchronizationCommand_StartInstances::Factory | likely | No arguments. Allocates 0x14 bytes with operator new; if non-null, constructs a StartInstances with 014ae5e0 and returns it, otherwise returns 0. |
| `014aea10` | SynchronizationCommand_StartInstances::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x54 and factory 014ae9b0, then 013ce230 with the same pair (ECX from 013ce130's return). |

**SynchronizationCommand_UnlockTournament** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014acf10` | SynchronizationCommand_UnlockTournament::Ctor | confirmed | Fastcall constructor (ECX=this). Calls the SynchronizationCommand_ChangeFlags constructor 014ae2a0 on this, stores SynchronizationCommand_UnlockTournament::vft… |
| `014acf90` | SynchronizationCommand_UnlockTournament::GetClassName | confirmed | Virtual slot 11 (no arguments). Returns the address of the literal SynchronizationCommand_UnlockTournament. |
| `014acfd0` | SynchronizationCommand_UnlockTournament::DtorBody | confirmed | Fastcall (ECX=this). Stores SynchronizationCommand_UnlockTournament::vftable into *this, then calls the ChangeFlags destructor body 014adcb0 on this; no free. |
| `014ad0b0` | SynchronizationCommand_UnlockTournament::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). vtbl[4](buf,&buf) header read; then 014ae550 (the next inheritance level read); then vtbl[5](buf). Returns 0 whe… |
| `014ad0f0` | SynchronizationCommand_UnlockTournament::Write | confirmed | Virtual slot 10 (thiscall, one stack argument, ret 4). vtbl[2](buf); no-op trace; 014adbc0 (ChangeFlags write level); no-op trace; vtbl[3](buf). |
| `014ad1c0` | SynchronizationCommand_UnlockTournament::ScalarDeletingDtor | confirmed | Vtable slot 0 (ECX=this, flag byte on stack). Stores the UnlockTournament vtable, calls the ChangeFlags destructor body 014adcb0, then calls _free(this) when b… |
| `014ad220` | SynchronizationCommand_UnlockTournament::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends the 40-byte literal SynchronizationCommand_UnlockTournament newline, calls the base dump thunk 01435510… |
| `014ad3a0` | SynchronizationCommand_UnlockTournament::AppendTypeName | confirmed | Virtual slot 20 (one stack argument). Loads ECX from the argument and appends the 51-byte literal Unlock Tournament Instances Synchronization Command with 0124… |
| `014ad040` | SynchronizationCommand_UnlockTournament::Clone | likely | Virtual slot 17 (fastcall, ECX=this, no stack arguments). Allocates 0x20 bytes with operator new; if non-null, constructs an UnlockTournament on it with 014acf… |
| `014ad140` | SynchronizationCommand_UnlockTournament::Factory | likely | No arguments. Allocates 0x20 bytes with operator new; if non-null, constructs an UnlockTournament with 014acf10 and returns it, otherwise returns 0. |
| `014ad1a0` | SynchronizationCommand_UnlockTournament::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x1dd and factory 014ad140, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014428c0` | TradeCommand::RegisterFactory | likely | Pushes the constant 0x92 and the address 01442860, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**TradeCommand_AcceptTrade** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ac4e0` | TradeCommand_AcceptTrade::Factory | likely | No arguments. Allocates 0x10 bytes with operator new; if non-null, constructs an AcceptTrade with 014ac190 and returns it, otherwise returns 0. |
| `014ac540` | TradeCommand_AcceptTrade::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x96 and factory 014ac4e0, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_AcceptTradeRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014abd90` | TradeCommand_AcceptTradeRequest::Factory | likely | No arguments. Allocates 0x14 bytes with operator new; if non-null, constructs an AcceptTradeRequest with 014aba90 and returns it, otherwise returns 0. |
| `014abdf0` | TradeCommand_AcceptTradeRequest::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x94 and factory 014abd90, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_AddCardToTrade** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ab480` | TradeCommand_AddCardToTrade::Factory | likely | No arguments. Allocates 0x20 bytes with operator new; if non-null, constructs an AddCardToTrade with 014ab0d0 and returns it, otherwise returns 0. |
| `014ab4e0` | TradeCommand_AddCardToTrade::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x95 and factory 014ab480, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_AddPostedTrade** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aaf90` | TradeCommand_AddPostedTrade::Factory | likely | No arguments. Allocates 0x14 bytes with operator new; if non-null, constructs an AddPostedTrade with 014aabe0 and returns it, otherwise returns 0. |
| `014aaff0` | TradeCommand_AddPostedTrade::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x18e and factory 014aaf90, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_CancelTradeRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aa760` | TradeCommand_CancelTradeRequest::Factory | likely | No arguments. Allocates 0x14 bytes with operator new; if non-null, constructs a CancelTradeRequest with 014aa450 and returns it, otherwise returns 0. |
| `014aa7c0` | TradeCommand_CancelTradeRequest::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x98 and factory 014aa760, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_ChangeTradeSession** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aa330` | TradeCommand_ChangeTradeSession::Factory | likely | No arguments. Allocates 0x10 bytes with operator new; if non-null, constructs a ChangeTradeSession with 014aa000 and returns it, otherwise returns 0. |
| `014aa390` | TradeCommand_ChangeTradeSession::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x10e and factory 014aa330, then 013ce230 with the same pair (ECX from 013ce130's return). |

**TradeCommand_ClearAcceptState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a9ee0` | TradeCommand_ClearAcceptState::Factory | likely | No arguments. Allocates a 12-byte block with operator new (push 0x0c); if non-null, calls the ClearAcceptState constructor 014a9c00 on it and returns its resul… |
| `014a9f40` | TradeCommand_ClearAcceptState::RegisterFactory | likely | No arguments. Calls 013ce130 with key 0x9d and factory 014a9ee0, then 013ce230 with the same pair (ECX from 013ce130's return), the same registry pattern as 01… |

**TradeCommand_ClearTrade** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014a9ae0` | TradeCommand_ClearTrade::Factory | likely | No arguments. Allocates a 12-byte block with operator new (00d833a6, push 0x0c); if non-null, calls the ClearTrade constructor 014a97f0 on it and returns its r… |
| `014a9b40` | TradeCommand_ClearTrade::RegisterFactory | likely | No arguments. Pushes key 0x9a and factory 014a9ae0 and calls the factory-registry helper 013ce130; then calls 013ce230 with the same key and factory (ECX from … |

**UChatCommand_RequestAddFriendReciprocate** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441330` | UChatCommand_RequestAddFriendReciprocate::RegisterFactory | likely | Pushes the constant 0x1fd and the address 014412c0, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**UChatCommand_RequestAddIgnore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441a00` | UChatCommand_RequestAddIgnore::RegisterFactory | likely | Pushes the constant 0x1ea and the address 014419a0, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**UChatCommand_RequestRemoveFriendReciprocate** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014405b0` | UChatCommand_RequestRemoveFriendReciprocate::RegisterFactory | likely | Pushes the constant 0x1ff and the address 01440550, calls 013ce130; then calls 013ce230 with EAX moved to ECX (same sequence as the other factory registrations… |

**UChatCommand_RequestRemoveIgnore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01440bf0` | UChatCommand_RequestRemoveIgnore::RegisterFactory | likely | Pushes the constant 0x1ef and the address 01440b90, calls 013ce130; then calls 013ce230 with EAX moved to ECX. No return value. |

**UserDataCommand_RequestPlayerMatches** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0151a8a0` | UserDataCommand_RequestPlayerMatches::Ctor | confirmed | Calls base constructor 01528920 (UserDataCommand), then stores the RequestPlayerMatches vtable into *this and zeroes the dword at this+8. |
| `0151a920` | UserDataCommand_RequestPlayerMatches::TypeName | confirmed | Returns the address of the literal class name UserDataCommand_RequestPlayerMatches (mov eax, imm32; ret). |
| `0151a930` | UserDataCommand_RequestPlayerMatches::ReadGlobal01cbebb8 | confirmed | Returns the dword in global DAT_01cbebb8 (mov eax, [01cbebb8]; ret) and takes no arguments. |
| `0151a940` | UserDataCommand_RequestPlayerMatches::WriteGlobal01cbebb8 | confirmed | Stores its stack argument into DAT_01cbebb8 and returns with ret 4. |
| `0151a960` | UserDataCommand_RequestPlayerMatches::Dtor | confirmed | Stores the RequestPlayerMatches vtable into *this, calls base destructor 01528a10 (UserDataCommand), and returns without freeing the object. |
| `0151a9b0` | UserDataCommand_RequestPlayerMatches::ReadField08 | confirmed | Returns the dword at this+8 (mov eax, [ecx+8]; ret). |
| `0151a9c0` | UserDataCommand_RequestPlayerMatches::WriteField08 | confirmed | Stores its stack argument as a dword at this+8 and returns with ret 4. |

**ValidActionFilter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142a380` | ValidActionFilter::Read | confirmed | Called on the secondary subobject (object+8). Header read via vtable slot 4 (0x10); FUN_01344600 (base read); reads an integer (FUN_013c24d0) that, when nonzer… |
| `0142a4c0` | ValidActionFilter::Write | confirmed | Called on the secondary subobject (object+8). Header write via vtable slot 2 (0x8); FUN_013449d0 (base write); writes the expression tree's classID (vtable slo… |

**WhileState** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0145b060` | WhileState::Write | confirmed | Virtual slot [10] of WhileState::vftable, thiscall with one stream argument (0 selects slot [1] to obtain a stream, otherwise slot [2] is called with the argum… |
| `0145c3d0` | WhileState::Read | confirmed | Virtual slot [9] of WhileState::vftable, thiscall with one stream argument, returns a byte. Calls slot [4] (+0x10) for the header, then the base body reader 01… |

**(no class)** (65)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009aade0` | PacketDtor_009aade0 | likely | Destructor stub for a BaseCommandPacket-derived packet: rewrites *this to BaseCommandPacket::vftable and then to BasePacket::vftable, and returns; no member de… |
| `012f0a00` | FrameCommandToOutBuf | likely | Thiscall (ECX = connection object, saved in esi) with one stack arg (command object). Optionally dumps the command to text through its vtable +0x4c when 012d09… |
| `0130c050` | RegisterTradingCardLobbyFactory | likely | Registers factory 0130bff0 under key 0x67 by calling 013ce130 with (0x67, factory) and 013ce230 with (0x67, factory). |
| `013b4980` | LobbyCommand_SendUChatText::New | likely | Calls operator new(0x98) (00d833a6); if non-null calls ctor 013b46f0 on it and returns that object, else returns 0. Plain function (no this, no stack args). |
| `013b4a70` | StdStringAssignAt40 | likely | std::string::assign(src, 0, npos) on the string at this+0x40, with src = the one stack argument (add ecx,0x40; ret 4). |
| `013b4a90` | StdStringAssignAt5c | likely | std::string::assign(src, 0, npos) on the string at this+0x5c (add ecx,0x5c; ret 4). |
| `013b4ab0` | StdStringAssignAt78 | likely | std::string::assign(src, 0, npos) on the string at this+0x78 (add ecx,0x78; ret 4). |
| `013d6230` | SynchronizationCommand_CompleteFactory | likely | Allocates 8 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d5f60 and returns it, otherwise it returns 0. |
| `013d6290` | SynchronizationCommand_CompleteRegister | likely | Registers the factory 013d6230 under key 0x56: calls 013ce130 with (0x56, factory) and 013ce230 with the same key and factory, then returns. |
| `013d7960` | PollFactory | likely | Allocates 0x54 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d77d0 (Poll constructor) and returns it, … |
| `013d79c0` | PollRegisterFactory | likely | Registers the Poll factory 013d7960 under key 0x185: calls 013ce130 with (0x185, factory) and 013ce230 with the same key and factory, then returns. |
| `013d7df0` | PollCommand_AddPollControlFactory | likely | Allocates 8 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d7b60 and returns it, otherwise it returns 0. |
| `013d7e50` | PollCommand_AddPollControlRegister | likely | Registers the factory 013d7df0 under key 0x189: calls 013ce130 with (0x189, factory) and 013ce230 with the same key and factory, then returns. |
| `013d8390` | LobbyCommand_ChangeStatusFactory | likely | Allocates 12 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d8030 and returns it, otherwise it returns … |
| `013d83f0` | LobbyCommand_ChangeStatusRegister | likely | Registers the factory 013d8390 under key 0x89: calls 013ce130 with (0x89, factory) and 013ce230 with the same key and factory, then returns. |
| `013d88e0` | LobbyCommand_AcceptAdjournedGameFactory | likely | Allocates 0x18 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d8520 and returns it, otherwise it return… |
| `013d8940` | LobbyCommand_AcceptAdjournedGameRegister | likely | Registers the factory 013d88e0 under key 0x10a: calls 013ce130 with (0x10a, factory) and 013ce230 with the same key and factory, then returns. |
| `013d8e80` | LobbyCommand_OfferAdjournedGameFactory | likely | Allocates 0x14 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d8b10 and returns it, otherwise it return… |
| `013d8ee0` | LobbyCommand_OfferAdjournedGameRegister | likely | Registers the factory 013d8e80 under key 0x109: calls 013ce130 with (0x109, factory) and 013ce230 with the same key and factory, then returns. |
| `013d93f0` | LobbyCommand_DeleteAdjournedGameFactory | likely | Allocates 0x14 bytes with 00d833a6 (operator new); when the allocation is nonzero it constructs the object through 013d9080 and returns it, otherwise it return… |
| `013d9450` | LobbyCommand_DeleteAdjournedGameRegister | likely | Registers the factory 013d93f0 under key 0x160: calls 013ce130 with (0x160, factory) and 013ce230 with the same key and factory, then returns. |
| `013d9970` | AllocAndConstruct20 | likely | Allocates 0x20 bytes with the pooled allocator 00d833a6 (operator new); if the pointer is non-null it calls 013d9830 with ECX = that pointer and returns the ca… |
| `013d99d0` | RegisterCommandFactory_5d | likely | Pushes classID 0x5d and factory 013d9970 to 013ce130 and then to 013ce230 (two registry calls with the same pair); returns nothing. |
| `013d9f90` | AllocAndConstruct8 | likely | Allocates 8 bytes with 00d833a6; if the pointer is non-null it calls 013d9cf0 (NetworkCommand_Ping constructor) and returns its result, else 0. |
| `013d9ff0` | RegisterCommandFactory_70 | likely | Pushes classID 0x70 and factory 013d9f90 to 013ce130 and then to 013ce230; returns nothing. |
| `01407740` | RegisterFactory_Id0x1e8 | likely | Static initialiser: calls 013ce130(0x1e8, 014076e0) and 013ce230(0x1e8, 014076e0), registering the factory function 014076e0 under key 0x1e8. |
| `01407df0` | RegisterFactory_Id0x71 | likely | Static initialiser: calls 013ce130(0x71, 01407d90) and 013ce230(0x71, 01407d90), registering the factory 01407d90 under key 0x71. |
| `01408610` | RegisterFactory_Id0x1e7 | likely | Static initialiser: calls 013ce130(0x1e7, 014085b0) and 013ce230(0x1e7, 014085b0), registering the factory 014085b0 under key 0x1e7. |
| `01408a60` | RegisterFactory_Id0x76 | likely | Static initialiser: calls 013ce130(0x76, 01408a00) and 013ce230(0x76, 01408a00), registering the factory 01408a00 under key 0x76. |
| `0140bee0` | RegisterFactory0x23d | likely | Calls 013ce130(0x23d, 0140be80) and then 013ce230 with the same key and function; no return value. |
| `0140c370` | RegisterFactory0x36 | likely | Calls 013ce130(0x36, 0140c310) and then 013ce230 with the same key and function; no return value. |
| `0141b740` | RegisterFactory_Id37 | likely | Registers 0141b6e0 as the factory for classID 0x37 (55): calls 013ce130(0x37, factory), then 013ce230 on its returned registry entry. |
| `01446df0` | RegisterFactory0x23A | likely | Static initialiser: registers the factory 01446d90 under key 0x23a by calling 013ce130 with (0x23a, factory) and then 013ce230 with the same key and factory. |
| `014475a0` | RegisterFactory0x1DE | likely | Static initialiser: registers the factory 01447540 under key 0x1de by calling 013ce130 with (0x1de, factory) and then 013ce230 with the same key and factory. |
| `01447980` | RegisterFactory0x20A | likely | Static initialiser: registers the factory 01447920 under key 0x20a by calling 013ce130 with (0x20a, factory) and then 013ce230 with the same key and factory. |
| `0149ed80` | ReadOptionalCommandObject | likely | Reads a presence int; when it is non-zero it clears the destination pointer and returns 1. Otherwise it reads a classID int, creates the object through the reg… |
| `014a41c0` | ReadOptionalCommandObject | likely | Same body as 0149ed80 (identical code): reads a presence int; non-zero clears the destination and returns 1; otherwise reads a classID, creates the object thro… |
| `014b40f0` | RegisterFactoryKey0x126 | likely | Pushes factory 014b4090 and key 0x126, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `014b4540` | RegisterFactoryKey0x64 | likely | Pushes factory 014b44e0 and key 100, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns … |
| `014b4bf0` | RegisterFactoryKey0x66 | likely | Pushes factory 014b4b90 and key 0x66, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b5040` | RegisterFactoryKey0x74 | likely | Pushes factory 014b4fe0 and key 0x74, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b5510` | RegisterFactoryKey0x1e6 | likely | Pushes factory 014b54b0 and key 0x1e6, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `014b5930` | RegisterFactoryKey0x7d | likely | Pushes factory 014b58d0 and key 0x7d, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014e0140` | RegisterFactory0x39 | likely | Pushes the address of 014e00e0 and key 0x39, calls 013ce130, then calls 013ce230 with 013ce130's returned value in ECX (the pair that registers the factory und… |
| `014e0600` | RegisterFactory0x3d | likely | Pushes the address of 014e05a0 and key 0x3d, calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair as 014e0140, w… |
| `014e0c90` | RegisterFactory0x9e | likely | Pushes the address of 014e0c30 and key 0x9e (decimal 158), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair … |
| `014e1220` | RegisterFactory0x3e | likely | Pushes the address of 014e11c0 and key 0x3e (decimal 62), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair a… |
| `014e1700` | RegisterFactory0x46 | likely | Pushes the address of 014e16a0 and key 0x46 (decimal 70), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair a… |
| `014e1c90` | RegisterFactory0x48 | likely | Pushes the address of 014e1c30 and key 0x48 (decimal 72), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair a… |
| `014e26e0` | RegisterFactory0x3b | likely | Pushes the address of 014e2680 and key 0x3b (decimal 59), calls 013ce130, then calls 013ce230 with 013ce130's return value in ECX (the same registration pair a… |
| `014ea6c0` | SerialReadPtrContainer | likely | Thiscall: ECX points to a slot whose target is the reader (reader = [ECX]); stack argument is the container. Reads a count via 013c2690, calls 013a8080(count) … |
| `01525600` | UserDataCommand_RequestNewsMessage::RegisterFactory | likely | Registers creator 015255a0 under key 0x227 by calling 013ce130 with the key and function pointer, then 013ce230 with the same key and pointer. |
| `01526280` | UserDataCommand_RespondFeatureRotator::RegisterFactory | likely | Registers creator 01526220 under key 0x226 with 013ce130 and 013ce230. |
| `015267c0` | UserDataCommand_RequestFeatureRotator::RegisterFactory | likely | Registers creator 01526760 under key 0x225 with 013ce130 and 013ce230. |
| `015271d0` | UserDataCommand_RespondAdvertisement::RegisterFactory | likely | Registers creator 01527170 under key 0x224 with 013ce130 and 013ce230. |
| `01527670` | UserDataCommand_RequestAdvertisement::RegisterFactory | likely | Registers creator 01527610 under key 0x223 with 013ce130 and 013ce230. |
| `01528000` | UserDataCommand_DataUpdated::RegisterFactory | likely | Registers creator 01527fa0 under key 0x221 with 013ce130 and 013ce230. |
| `01528350` | UserDataCommand_Unsubscribe::RegisterFactory | likely | Registers creator 015282f0 under key 0x220 with 013ce130 and 013ce230. |
| `015287d0` | UserDataCommand_Subscribe::RegisterFactory | likely | Registers creator 01528770 under key 0x21f with 013ce130 and 013ce230. |
| `01528c40` | UserDataCommand::RegisterFactory | likely | Registers creator 01528be0 under key 0x21e with 013ce130 and 013ce230. |
| `01529110` | UChatCommand_OnReceiveFriendConfirmReciprocateResponse::Reg… | likely | Registers creator 015290b0 under key 0x201 with 013ce130 and 013ce230. |
| `015295e0` | UChatCommand_OnReceiveFriendConfirmResponse::RegisterFactory | likely | Registers creator 01529580 under key 0x1f2 with 013ce130 and 013ce230. |
| `01529a50` | UChatCommand_OnReceiveFriendConfirmReciprocateRequest::Regi… | likely | Registers creator 015299f0 under key 0x200 with 013ce130 and 013ce230. |
| `01529e60` | UChatCommand_OnReceiveFriendConfirmRequest::RegisterFactory | likely | Registers creator 01529e00 under key 0x1f1 with 013ce130 and 013ce230. |
| `0152a270` | UChatCommand_OnReceiveFriendLogout::RegisterFactory | likely | Registers creator 0152a210 under key 500 (0x1f4) with 013ce130 and 013ce230. |

### `net.event_command`

**EventCommand_NextMatchingTime** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015351b0` | EventCommand_NextMatchingTime::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_NextMatchingTime::vftable at +0x0, zeroes the dwords at +0x8 a… |
| `01535230` | EventCommand_NextMatchingTime::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_NextMatchingTime'. |
| `01535270` | EventCommand_NextMatchingTime::DestructBody | confirmed | Non-deleting destructor body: stores the NextMatchingTime vftable into this+0x0, then calls base-class destructor FUN_01447370 with this in ECX. |
| `015353a0` | EventCommand_NextMatchingTime::Clone | confirmed | Virtual at vftable slot 17: allocates 0x10 bytes with operator new (FUN_00d833a6); if non-null, constructs a NextMatchingTime in it (FUN_015351b0); then calls … |
| `01535410` | EventCommand_NextMatchingTime::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_NextMatchingTime (NetworkCommand to NMT descriptors; asserts non-null with… |
| `01535470` | EventCommand_NextMatchingTime::DeserializeBody | confirmed | Virtual at vftable slot 9: calls vtable slot 4 with the stream and &argument (returns bool); when true calls base slot 9 FUN_014474d0; when true reads integers… |
| `01535590` | EventCommand_NextMatchingTime::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (vftable slot 0): stores the NextMatchingTime vftable, calls base destructor FUN_01447370, and when bit 0 of the flag argument is se… |
| `015355f0` | EventCommand_NextMatchingTime::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Next Matching Time Event Command\n' (33 bytes) to the stream argument via FUN_01241650, then calls the base dump F… |
| `01535620` | EventCommand_NextMatchingTime::WriteClassLabel | confirmed | Virtual at vftable slot 20: writes the 32-character literal 'Next Matching Time Event Command' with FUN_01241650 to the stream in the first stack argument (no … |
| `01535300` | EventCommand_NextMatchingTime::MakeTagged0xd4Object | likely | Virtual at vftable slot 15: allocates a 0x14-byte object (operator new), constructs it with FUN_012f9eb0, stores the tag 0xd4 through FUN_012f8c70, writes this… |
| `015354d0` | EventCommand_NextMatchingTime::SerializeBody | likely | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510, then FUN_013c23f0 (appendInteger) on this+0x8 and this+0xc, then vt… |
| `01535510` | EventCommand_NextMatchingTime::Create | likely | Factory: allocates 0x10 bytes with operator new (FUN_00d833a6); returns 0 when the allocation is null, otherwise constructs a NextMatchingTime with FUN_015351b… |
| `01535570` | EventCommand_NextMatchingTime::RegisterFactory | likely | Registers the factory FUN_01535510 under id 0x202: calls FUN_013ce130(0x202, factory) and then FUN_013ce230(0x202, factory). |

**EventCommand_NotifyMatchEnded** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01534cd0` | EventCommand_NotifyMatchEnded::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_NotifyMatchEnded::vftable at +0x0, zeroes the dwords at +0x8, … |
| `01534d50` | EventCommand_NotifyMatchEnded::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_NotifyMatchEnded'. |
| `01534d90` | EventCommand_NotifyMatchEnded::DestructBody | confirmed | Non-deleting destructor body: stores the NotifyMatchEnded vftable into this+0x0, then calls base-class destructor FUN_01447370 with this in ECX. |
| `01534ee0` | EventCommand_NotifyMatchEnded::Clone | confirmed | Virtual at vftable slot 17: allocates 0x14 bytes with operator new (FUN_00d833a6); if non-null, constructs a NotifyMatchEnded in it (FUN_01534cd0); then calls … |
| `01534f50` | EventCommand_NotifyMatchEnded::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_NotifyMatchEnded (NetworkCommand to NME descriptors; asserts non-null with… |
| `01534fb0` | EventCommand_NotifyMatchEnded::DeserializeBody | confirmed | Virtual at vftable slot 9: vtable slot 4 with the stream and an out parameter that receives the header version; when true calls base slot 9 FUN_014474d0; when … |
| `01535100` | EventCommand_NotifyMatchEnded::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (vftable slot 0): stores the NotifyMatchEnded vftable, calls base destructor FUN_01447370, and when bit 0 of the flag argument is se… |
| `01534e40` | EventCommand_NotifyMatchEnded::MakeTagged0xe3Object | likely | Virtual at vftable slot 15: allocates a 0x14-byte object (operator new), constructs it with FUN_012f9eb0, stores the tag 0xe3 through FUN_012f8c70, writes this… |
| `01535030` | EventCommand_NotifyMatchEnded::SerializeBody | likely | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510, then FUN_013c23f0 (appendInteger) on this+0x8, +0xc and +0x10, then… |
| `01535080` | EventCommand_NotifyMatchEnded::Create | likely | Factory: allocates 0x14 bytes with operator new (FUN_00d833a6); returns 0 when the allocation is null, otherwise constructs a NotifyMatchEnded with FUN_01534cd… |
| `015350e0` | EventCommand_NotifyMatchEnded::RegisterFactory | likely | Registers the factory FUN_01535080 under id 0x213: calls FUN_013ce130(0x213, factory) and then FUN_013ce230(0x213, factory). |

**EventCommand_RequestEventTotalPlayers** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01534950` | EventCommand_RequestEventTotalPlayers::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_RequestEventTotalPlayers::vftable at +0x0 and returns this. Th… |
| `015349d0` | EventCommand_RequestEventTotalPlayers::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_RequestEventTotalPlayers'. |
| `01534a10` | EventCommand_RequestEventTotalPlayers::DestructBody | confirmed | Non-deleting destructor body: stores the RequestEventTotalPlayers vftable into this+0x0, then calls base-class destructor FUN_01447370 with this in ECX. |
| `01534a70` | EventCommand_RequestEventTotalPlayers::Clone | confirmed | Virtual at vftable slot 17: allocates 8 bytes with operator new (FUN_00d833a6); if non-null, constructs a RequestEventTotalPlayers in it (FUN_01534950); then c… |
| `01534ae0` | EventCommand_RequestEventTotalPlayers::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_RequestEventTotalPlayers (NetworkCommand to REP descriptors; asserts non-n… |
| `01534b30` | EventCommand_RequestEventTotalPlayers::DeserializeBody | confirmed | Virtual at vftable slot 9: calls vtable slot 4 with the stream and &argument (returns bool); when true calls base slot 9 FUN_014474d0 with the stream; when tru… |
| `01534b70` | EventCommand_RequestEventTotalPlayers::SerializeBody | confirmed | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510 with the stream, then vtable slot 3 with the stream. Writes no field… |
| `01534c20` | EventCommand_RequestEventTotalPlayers::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (vftable slot 0): stores the RequestEventTotalPlayers vftable, calls base destructor FUN_01447370, and when bit 0 of the flag argume… |
| `01534c80` | EventCommand_RequestEventTotalPlayers::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Request Event Total Players Event Command\n' (42 bytes) to the stream argument via FUN_01241650, then calls the ba… |
| `01534cb0` | EventCommand_RequestEventTotalPlayers::WriteClassLabel | confirmed | Virtual at vftable slot 20: writes the 41-character literal 'Request Event Total Players Event Command' with FUN_01241650 to the stream in the first stack argu… |
| `01534ba0` | EventCommand_RequestEventTotalPlayers::Create | likely | Factory: allocates 8 bytes with operator new (FUN_00d833a6); returns 0 when the allocation is null, otherwise constructs a RequestEventTotalPlayers with FUN_01… |
| `01534c00` | EventCommand_RequestEventTotalPlayers::RegisterFactory | likely | Registers the factory FUN_01534ba0 under id 0x205: calls FUN_013ce130(0x205, factory) and then FUN_013ce230(0x205, factory). |

**EventCommand_RequestTimedDeckSelection** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0152ff80` | EventCommand_RequestTimedDeckSelection::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_RequestTimedDeckSelection::vftable at +0x0, zeroes the dwords … |
| `015300f0` | EventCommand_RequestTimedDeckSelection::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_RequestTimedDeckSelection (NetworkCommand to RTD descriptors; asserts non-… |
| `015301b0` | EventCommand_RequestTimedDeckSelection::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Request Timed Deck Selection Event Command\n' (43 bytes) to the stream argument via FUN_01241650, then calls the b… |
| `015301e0` | EventCommand_RequestTimedDeckSelection::WriteClassLabel | confirmed | Virtual at vftable slot 20: takes the stream from the first stack argument (ECX is not used) and writes the 42-character literal 'Request Timed Deck Selection … |
| `01530200` | EventCommand_RequestTimedDeckSelection::Create | likely | Allocates 0x2c bytes with operator new (FUN_00d833a6); returns 0 when that returns null, otherwise constructs a RequestTimedDeckSelection in the block with FUN… |
| `01530260` | EventCommand_RequestTimedDeckSelection::RegisterFactory | likely | Registers the factory FUN_01530200 under id 0x209: calls FUN_013ce130(0x209, factory) and then FUN_013ce230(0x209, factory) with the same arguments. |

**EventCommand_RespondEventLeaderBoard** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01531ff0` | EventCommand_RespondEventLeaderBoard::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Respond Event Leader Board Event Command\n' (41 bytes) to the stream argument via FUN_01241650, then calls the bas… |
| `01532020` | EventCommand_RespondEventLeaderBoard::WriteClassLabel | confirmed | Virtual at vftable slot 20: writes the 40-character literal 'Respond Event Leader Board Event Command' with FUN_01241650 to the stream in the first stack argum… |
| `01533a80` | EventCommand_RespondEventLeaderBoard::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_RespondEventLeaderBoard::vftable at +0x0, zeroes about 30 head… |
| `01533cf0` | EventCommand_RespondEventLeaderBoard::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_RespondEventLeaderBoard'. |
| `01533d30` | EventCommand_RespondEventLeaderBoard::DestructBody | confirmed | Destructor body: stores the RespondEventLeaderBoard vftable into this+0x0, resets header words, runs container helpers over the members at +0x18 (FUN_0041f5e6)… |
| `01534080` | EventCommand_RespondEventLeaderBoard::Clone | confirmed | Virtual at vftable slot 17: allocates 0xa0 bytes with operator new (FUN_00d833a6); if non-null, constructs a RespondEventLeaderBoard in it (FUN_01533a80); then… |
| `015343f0` | EventCommand_RespondEventLeaderBoard::ScalarDeletingDestruc… | confirmed | Scalar deleting destructor (vftable slot 0): calls the destructor body FUN_01533d30 on this and, when bit 0 of the flag argument is set, calls _free on this an… |
| `01534560` | EventCommand_RespondEventLeaderBoard::DeserializeBody | confirmed | Virtual at vftable slot 9: calls vtable slot 4 with the stream and &local (header version out parameter, returns bool), base slot 9 FUN_014474d0, readInteger i… |
| `015348a0` | EventCommand_RespondEventLeaderBoard::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_RespondEventLeaderBoard (NetworkCommand to RELB descriptors; asserts non-n… |
| `015324d0` | EventCommand_RespondEventLeaderBoard::SerializeRows | likely | Writes the 12-byte row vector passed as param_2 (begin at +0x4, end at +0x8): appends the row count (FUN_013c2640 of (end - begin) / 12, or 0 when begin is nul… |
| `01534360` | EventCommand_RespondEventLeaderBoard::Create | likely | Factory: allocates 0xa0 bytes with operator new (FUN_00d833a6); returns 0 when the allocation is null, otherwise constructs a RespondEventLeaderBoard with FUN_… |
| `015343d0` | EventCommand_RespondEventLeaderBoard::RegisterFactory | likely | Registers the factory FUN_01534360 under id 0x206: calls FUN_013ce130(0x206, factory) and then FUN_013ce230(0x206, factory). |

**EventCommand_RespondEventSessionInfo** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `015309c0` | EventCommand_RespondEventSessionInfo::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Respond Event Session Info Event Command\n' (41 bytes) to the stream argument via FUN_01241650, then calls the bas… |
| `015309f0` | EventCommand_RespondEventSessionInfo::WriteClassLabel | confirmed | Virtual at vftable slot 20: writes the 40-character literal 'Respond Event Session Info Event Command' with FUN_01241650 to the stream in the first stack argum… |
| `01531010` | EventCommand_RespondEventSessionInfo::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_RespondEventSessionInfo'. |
| `01531050` | EventCommand_RespondEventSessionInfo::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_RespondEventSessionInfo::vftable at +0x0, zeroes the dwords at… |
| `01531120` | EventCommand_RespondEventSessionInfo::ScalarDeletingDestruc… | confirmed | Scalar deleting destructor (vftable slot 0): calls the destructor body FUN_01530f30 on this and, when bit 0 of the flag argument is set, calls _free on this an… |
| `01530e70` | EventCommand_RespondEventSessionInfo::SerializeBody | likely | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510 with the stream, then FUN_013c23f0 (appendInteger) on this+0x8, this… |
| `01531210` | EventCommand_RespondEventSessionInfo::RegisterFactory | likely | Registers the factory FUN_015311b0 under id 0x207: calls FUN_013ce130(0x207, factory) and then FUN_013ce230(0x207, factory). |

**EventCommand_RespondEventTotalPlayers** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01530280` | EventCommand_RespondEventTotalPlayers::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_RespondEventTotalPlayers::vftable at +0x0, zeroes the dword at… |
| `01530300` | EventCommand_RespondEventTotalPlayers::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_RespondEventTotalPlayers'. The base EventCommand slot 11 returns 'EventCommand'. |
| `01530340` | EventCommand_RespondEventTotalPlayers::DestructBody | confirmed | Non-deleting destructor body: stores the RespondEventTotalPlayers vftable into this+0x0, then calls base-class destructor FUN_01447370 with this in ECX. |
| `015304b0` | EventCommand_RespondEventTotalPlayers::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_RespondEventTotalPlayers (NetworkCommand to RETP descriptors; asserts non-… |
| `01530610` | EventCommand_RespondEventTotalPlayers::ScalarDeletingDestru… | confirmed | Scalar deleting destructor (vftable slot 0): stores the RespondEventTotalPlayers vftable, calls base destructor FUN_01447370, and when bit 0 of the flag argume… |
| `01530670` | EventCommand_RespondEventTotalPlayers::DebugDump | confirmed | Virtual at vftable slot 19: writes the line 'Respond Event Total Players Event Command\n' via FUN_01241650 to the stream argument, then calls the base dump FUN… |
| `015306a0` | EventCommand_RespondEventTotalPlayers::WriteClassLabel | confirmed | Virtual at vftable slot 20: writes the 41-character literal 'Respond Event Total Players Event Command' with FUN_01241650 to the stream in the first stack argu… |
| `015303b0` | EventCommand_RespondEventTotalPlayers::MakeTagged0xd7Object | likely | Virtual at vftable slot 15: allocates a 0x14-byte object (operator new), constructs it with FUN_012f9eb0, stores the dword 0xd7 at its +0 through FUN_012f8c70,… |
| `01530500` | EventCommand_RespondEventTotalPlayers::DeserializeBody | likely | Virtual at vftable slot 9: calls vtable slot 4 with the stream and &argument (returns bool); when true calls base slot 9 FUN_014474d0 with the stream; when tru… |
| `01530550` | EventCommand_RespondEventTotalPlayers::SerializeBody | likely | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510 with the stream, FUN_013c23f0(this+8) (appendInteger of the player-c… |
| `015305f0` | EventCommand_RespondEventTotalPlayers::RegisterFactory | likely | Registers the factory FUN_01530590 under id 0x208: calls FUN_013ce130(0x208, factory) and then FUN_013ce230(0x208, factory). |

**EventCommand_SetReady** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01535640` | EventCommand_SetReady::Construct | confirmed | Constructor: calls the EventCommand base constructor FUN_01447280 on this, installs EventCommand_SetReady::vftable at +0x0, clears the byte at +0x8, and return… |
| `015356c0` | EventCommand_SetReady::GetClassName | confirmed | Virtual at vftable slot 11: returns the literal string 'EventCommand_SetReady'. |
| `01535700` | EventCommand_SetReady::DestructBody | confirmed | Non-deleting destructor body: stores the SetReady vftable into this+0x0, then calls base-class destructor FUN_01447370 with this in ECX. |
| `015357f0` | EventCommand_SetReady::CloneInto | confirmed | Virtual at vftable slot 18: dynamic_cast of the destination argument to EventCommand_SetReady (NetworkCommand to SetReady descriptors; asserts non-null with 'c… |
| `01535840` | EventCommand_SetReady::DeserializeBody | confirmed | Virtual at vftable slot 9: calls vtable slot 4 with the stream and &local (returns bool); when true calls base slot 9 FUN_014474d0; when true reads an integer … |
| `015358a0` | EventCommand_SetReady::SerializeBody | likely | Virtual at vftable slot 10: calls vtable slot 2 with the stream, base slot 10 FUN_01447510, then FUN_013c23f0 on the byte at this+0x8 (cast to char), then vtab… |
| `015358e0` | EventCommand_SetReady::Create | likely | Factory: allocates 0x10 bytes with operator new (FUN_00d833a6); returns 0 when the allocation is null, otherwise constructs a SetReady with FUN_01535640 and re… |
| `01535940` | EventCommand_SetReady::RegisterFactory | likely | Registers the factory FUN_015358e0 under id 0x1df: calls FUN_013ce130(0x1df, factory) and then FUN_013ce230(0x1df, factory). |

### `net.factory`

**BaseFactoryPacket** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9070` | BaseFactoryPacket::BaseFactoryPacket 🌐 | confirmed | Default constructor: installs BasePacket::vftable, writes the base packet id 0xbc (188) at +4 and zero at +8, then installs BaseFactoryPacket::vftable. |
| `009d90d0` | BaseFactoryPacket::Init 🌐 | confirmed | Constructor variant taking a sub-opcode: installs BasePacket::vftable, writes the base packet id 0xbc (188) at +4, installs BaseFactoryPacket::vftable, stores … |
| `009d9140` | BaseFactoryPacket::~BaseFactoryPacket | confirmed | Destructor body with no member work: rewrites the object vtable to BaseFactoryPacket::vftable and then to BasePacket::vftable. Reached only from compiler-gener… |
| `009d9190` | BaseFactoryPacket::ScalarDeletingDtor | likely | Scalar deleting destructor: resets the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and calls _free(param_1) when (param_2 and 1) is set, ret… |
| `009d9420` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (one of several identical copies): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable. Only reached from EH un… |
| `009d94e0` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d9530` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d95e0` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d9690` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d9740` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d9850` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |
| `009d99c0` | BaseFactoryPacket::~BaseFactoryPacket | likely | Trivial destructor body (identical copy of the base destructor): rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable; its only caller i… |

**FactoryPacketAddBlueprintToFoundationFailure** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9580` | FactoryPacketAddBlueprintToFoundationFailure::FactoryPacket… 🌐 | confirmed | Constructor for FactoryPacketAddBlueprintToFoundationFailure: calls FUN_009d90d0(0x14), installs FactoryPacketAddBlueprintToFoundationFailure::vftable and clea… |
| `009db560` | FactoryPacketAddBlueprintToFoundationFailure::ScalarDeletin… | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketCreateBlueprintRequest** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e1440` | FactoryPacketCreateBlueprintRequest::Ctor 🌐 | confirmed | Initializes base packet, installs request vtable and constructs embedded ItemRequestEntry list. |
| `009e14b0` | FactoryPacketCreateBlueprintRequest::Dtor | confirmed | Calls 009df460 with ECX=this+0x0c to destroy the embedded ItemRequestEntry list, then restores BaseFactoryPacket and BasePacket vtables at this. |
| `009e4060` | FactoryPacketCreateBlueprintRequest::ScalarDeletingDtor | confirmed | Calls packet/list destructor and frees this when deleting flag is set. |

**FactoryPacketCreateBlueprintResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9480` | FactoryPacketCreateBlueprintResponse::FactoryPacketCreateBl… 🌐 | confirmed | Constructor for FactoryPacketCreateBlueprintResponse: calls FUN_009d90d0(0x19), installs FactoryPacketCreateBlueprintResponse::vftable and clears the byte at +… |
| `009db420` | FactoryPacketCreateBlueprintResponse::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketDoUpsell** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9960` | FactoryPacketDoUpsell::FactoryPacketDoUpsell 🌐 | confirmed | Constructor for FactoryPacketDoUpsell: calls FUN_009d90d0(0x1e), installs FactoryPacketDoUpsell::vftable and clears the words at +0x0c and +0x10. |
| `009db740` | FactoryPacketDoUpsell::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketEquipToolRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d96e0` | FactoryPacketEquipToolRequest::FactoryPacketEquipToolRequest 🌐 | confirmed | Constructor for FactoryPacketEquipToolRequest: calls FUN_009d90d0(7), installs FactoryPacketEquipToolRequest::vftable and clears +0x0c. |
| `009db640` | FactoryPacketEquipToolRequest::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketEquipToolResponse** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d97f0` | FactoryPacketEquipToolResponse::FactoryPacketEquipToolRespo… 🌐 | likely | Constructor for FactoryPacketEquipToolResponse: calls FUN_009d90d0(0x17), installs FactoryPacketEquipToolResponse::vftable, clears the byte at +0x0c and the wo… |
| `009db6b0` | FactoryPacketEquipToolResponse::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketListToolsRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9630` | FactoryPacketListToolsRequest::FactoryPacketListToolsRequest 🌐 | confirmed | Constructor for FactoryPacketListToolsRequest: calls FUN_009d90d0(6) and installs FactoryPacketListToolsRequest::vftable; no extra fields are initialised. |
| `009db5d0` | FactoryPacketListToolsRequest::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketListToolsResponse** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e41b0` | FactoryPacketListToolsResponse::Dtor | confirmed | Calls 009e4140 with ECX=this+0x0c to destroy the embedded tool map, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e5580` | FactoryPacketListToolsResponse_ctor 🌐 | confirmed | Constructor of FactoryPacketListToolsResponse: runs the BaseFactoryPacket base constructor with sub-opcode 0x16, stores the FactoryPacketListToolsResponse vtab… |
| `009e55e0` | FactoryPacketListToolsResponse_scalarDeletingDtor | likely | Scalar deleting destructor: calls the destructor body FUN_009e41b0 (which destroys the embedded tool map via 009e4140 and resets the packet vtables) and frees … |

**FactoryPacketNpcTooltipReply** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e40d0` | FactoryPacketNpcTooltipReply::Dtor | confirmed | Calls 009e2760 with ECX=this+0x18 to destroy the tooltip-effect list, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e5500` | FactoryPacketNpcTooltipReply_ctor 🌐 | confirmed | Constructor of FactoryPacketNpcTooltipReply: runs the BaseFactoryPacket base constructor with sub-opcode 0x15, stores the FactoryPacketNpcTooltipReply vtable a… |
| `009e5560` | FactoryPacketNpcTooltipReply_scalarDeletingDtor | likely | Scalar deleting destructor: calls the destructor body FUN_009e40d0 and frees this through _free when bit 0 of the flag byte is set, returning this otherwise. |

**FactoryPacketNpcTooltipRequest** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009db350` | FactoryPacketNpcTooltipRequest::FactoryPacketNpcTooltipRequ… 🌐 | confirmed | Constructor for FactoryPacketNpcTooltipRequest: calls FUN_009d90d0(5) and installs FactoryPacketNpcTooltipRequest::vftable; no extra fields initialised. |
| `009db3b0` | FactoryPacketNpcTooltipRequest::ScalarDeletingDtor | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketOpenBlueprintBrowserOnFoundation** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e1520` | FactoryPacketOpenBlueprintBrowserOnFoundation::Dtor | confirmed | Calls 0081ca60 with ECX=this+0x6c and 009e13d0 with ECX=this+0x18 to destroy the embedded maps, then restores BaseFactoryPacket and BasePacket vtables. |
| `009e4210` | FactoryPacketOpenBlueprintBrowserOnFoundation::Ctor 🌐 | confirmed | Initializes BaseFactoryPacket with subopcode 0x18, installs the packet vtable, and initializes the 16-bucket map at +0x18 and 128-bucket map at +0x6c. |

**FactoryPacketOpenBlueprintGenerator** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009de250` | FactoryPacketOpenBlueprintGenerator::FactoryPacketOpenBluep… 🌐 | confirmed | Constructor for FactoryPacketOpenBlueprintGenerator: calls FUN_009d90d0(0x1b), installs FactoryPacketOpenBlueprintGenerator::vftable and initialises the same e… |
| `009de2c0` | FactoryPacketOpenBlueprintGenerator::~FactoryPacketOpenBlue… | likely | Destructor body identical in shape to 009de1c0: restores the embedded IString member vtable, releases its reference-counted buffer when non-empty, then rewrite… |
| `009df980` | FactoryPacketOpenBlueprintGenerator::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009de2c0 and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketOpenBlueprintGene… |

**FactoryPacketOpenToolshed** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009de150` | FactoryPacketOpenToolshed::FactoryPacketOpenToolshed 🌐 | confirmed | Constructor for FactoryPacketOpenToolshed: calls FUN_009d90d0(0x1a), installs FactoryPacketOpenToolshed::vftable, then initialises an embedded SoeUtil::IString… |
| `009de1c0` | FactoryPacketOpenToolshed::~FactoryPacketOpenToolshed | likely | Destructor body: rewrites the embedded IString member vtable at +0x0c, releases the member's reference-counted buffer at param_1[4]-4 through vtable slot 2 whe… |
| `009df960` | FactoryPacketOpenToolshed::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009de1c0 (IString member release plus vtable rewrites) and frees the object when (param_2 and 1) is… |

**FactoryPacketRequestAddBlueprintToFoundation** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009db490` | FactoryPacketRequestAddBlueprintToFoundation::FactoryPacket… 🌐 | confirmed | Constructor for FactoryPacketRequestAddBlueprintToFoundation: calls FUN_009d90d0(2) and installs FactoryPacketRequestAddBlueprintToFoundation::vftable; no extr… |
| `009db4f0` | FactoryPacketRequestAddBlueprintToFoundation::ScalarDeletin… | likely | Scalar deleting destructor with a trivial destructor body: rewrites the vtables to BaseFactoryPacket::vftable then BasePacket::vftable and frees the object whe… |

**FactoryPacketUpdateBlueprint** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009db2d0` | FactoryPacketUpdateBlueprint::FactoryPacketUpdateBlueprint 🌐 | confirmed | Constructor for FactoryPacketUpdateBlueprint: calls FUN_009d90d0(0x11), installs FactoryPacketUpdateBlueprint::vftable and runs the member initialiser FUN_0105… |
| `009d93b0` | FactoryPacketUpdateBlueprint::~FactoryPacketUpdateBlueprint | likely | Destructor body: destroys the member at this+0x10 via FUN_01058e00, a folded member-destructor body that tears down two 10-bucket maps (member+0x60 and member+… |
| `009db330` | FactoryPacketUpdateBlueprint::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009d93b0, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateBlueprint… |

**FactoryPacketUpdateExperience** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009de350` | FactoryPacketUpdateExperience::FactoryPacketUpdateExperience 🌐 | confirmed | Constructor for FactoryPacketUpdateExperience: calls FUN_009d90d0(0x1d), installs FactoryPacketUpdateExperience::vftable, initialises the embedded IString memb… |
| `009de3d0` | FactoryPacketUpdateExperience::~FactoryPacketUpdateExperien… | likely | Destructor body identical in shape to the other IString-carrying factory packets: restores the embedded IString member vtable, releases its buffer when non-emp… |
| `009df9a0` | FactoryPacketUpdateExperience::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009de3d0 and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateExperience:… |

**FactoryPacketUpdateFactoryInfo** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d9200` | FactoryPacketUpdateFactoryInfo::FactoryPacketUpdateFactoryI… 🌐 | confirmed | Constructor for FactoryPacketUpdateFactoryInfo: calls the shared initialiser FUN_009d90d0(1), installs FactoryPacketUpdateFactoryInfo::vftable, initialises the… |
| `009d9280` | FactoryPacketUpdateFactoryInfo::~FactoryPacketUpdateFactory… | likely | Destructor body: destroys the member map via FUN_01058700 (HashListMap keyed by StrongType<u64,CharacterGuid> in the FactoryMisfortune family), then rewrites t… |
| `009db290` | FactoryPacketUpdateFactoryInfo::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009d9280, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateFactoryIn… |

**FactoryPacketUpdateFoundation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d92e0` | FactoryPacketUpdateFoundation::FactoryPacketUpdateFoundation 🌐 | confirmed | Constructor for FactoryPacketUpdateFoundation: calls FUN_009d90d0(0x10), installs FactoryPacketUpdateFoundation::vftable, runs the member initialiser FUN_01058… |
| `009d9350` | FactoryPacketUpdateFoundation::~FactoryPacketUpdateFoundati… | likely | Destructor body: destroys the member via FUN_01058960, then rewrites the vtables to BaseFactoryPacket::vftable and BasePacket::vftable. |
| `009db2b0` | FactoryPacketUpdateFoundation::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009d9350, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateFoundatio… |

**FactoryPacketUpdatePlot** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d98a0` | FactoryPacketUpdatePlot::FactoryPacketUpdatePlot 🌐 | confirmed | Constructor for FactoryPacketUpdatePlot: calls FUN_009d90d0(0x1c), installs FactoryPacketUpdatePlot::vftable and runs the member initialiser FUN_01059590 (Hash… |
| `009d9900` | FactoryPacketUpdatePlot::~FactoryPacketUpdatePlot | likely | Calls FUN_010596b0 on the embedded plot-data member, then rewrites this object?s vtable to BaseFactoryPacket::vftable and BasePacket::vftable. FUN_010596b0 des… |
| `009db720` | FactoryPacketUpdatePlot::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the class destructor body 009d9900 (which destroys the placement-node map member) and frees the object when (param_2 and 1) i… |

**(no class)** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009e9d40` | FactoryPacket_Dispatch 🌐 | confirmed | Reads the first two u16 fields of the payload with FUN_009e0340 (after constructing the BaseFactoryPacket-scoped command object with 009d9070) and switches on … |
| `009e4ed0` | Stream_ReadBool 🌐 | likely | Bounds-checked single-byte read from a serialized stream (cur at +8, end at +0xc, error flag at +0x10): first calls the shared stream helper FUN_009e0340, then… |
| `009e5100` | TooltipInnerRecord_read 🌐 | likely | Deserializer that fills this: reads an 8-byte pair into +8/+0xc, a dword into +0x10, a dword into +0x14 that is NaN-tested (setting the stream error byte at +0… |
| `009e6620` | BlueprintOpenRecord_read 🌐 | likely | Deserializer: runs the stream helper FUN_009e0340, reads an 8-byte pair into +0x10/+0x14, then reads two sub-objects through FUN_009e3bb0 into +0x18 and FUN_00… |
| `009e6690` | NestedFactoryRecord_read 🌐 | likely | Deserializer for a nested factory record: reads two dwords at +0/+4, a pair at +8/+0xc, pairs at +0x18/+0x1c and +0x10/+0x14, a bool at +0x22, pairs at +0x28, … |
| `009e6e70` | NpcTooltipRecord_read 🌐 | likely | Runs the stream helper FUN_009e0340 and reads an 8-byte pair into +0x10/+0x14 (zeroing both and flagging the stream on overflow), then always calls FUN_009e510… |
| `009e6f00` | FoundationRecord_read 🌐 | likely | Deserializes the factory-foundation record: reads an 8-byte pair into +0xF0/+0xF4, one dword into +0, calls FUN_009e6690 for the composite member, then reads 8… |
| `009e7020` | FoundationRecord_write 🌐 | likely | Serialises a factory record into a packet byte buffer: writes the two u16 fields with 009e6860, then the dword at this+0xc and the trailing payload at this+0x1… |
| `009e7090` | RecordWrite_U16PairU32 🌐 | likely | Serialises a record into a packet byte buffer: writes the u16 pair with 009e6860 and then the dword at this+0xc with a clamped four-byte copy through FUN_007cc… |
| `009e71f0` | PlacementMap_read 🌐 | likely | Reads a count-prefixed array of keyed placement records: drains the destination chain with FUN_009ddc40, reads a dword count, and per entry reads an 8-byte pai… |
| `009e74d0` | BlueprintUpdateRecord_read 🌐 | likely | Runs the stream helper FUN_009e0340, deserializes the composite member with FUN_009e6690 and then reads an 8-byte pair into +0xE8/+0xEC (or zeroes it and flags… |
| `009e75b0` | RecordWrite_U32List 🌐 | likely | Serialises a record into a packet byte buffer: first writes this+0xc with FUN_00911420 and then walks the list linked at this+4, calling FUN_009e6960 per node … |
| `009e75f0` | FoundationMap_read 🌐 | likely | Reads a count-prefixed map of foundation records: clears the destination chain with FUN_009ddac0, reads a dword count and per entry reads an 8-byte pair, creat… |
| `009e7880` | PlotRecord_read 🌐 | likely | Deserializes a composite record: reads a dword at +0, map at +4 via 009e75f0, subobject at +0x118 via 009e3d40, a pair at +0x230/+0x234, map at +0x250 via 009e… |
| `009e7d20` | UpdateFoundationPayload_read 🌐 | likely | Runs the stream helper FUN_009e0340, deserializes the foundation record with FUN_009e6f00 and then reads one dword into +0x110 (or zeroes it and flags the stre… |
| `009e8220` | DwordKeyedMap_read 🌐 | likely | Drains the destination chain, reads a dword count, then checks stream error at each loop head. A failed key read sets error and supplies key 0, but the current… |
| `009e8c20` | RequestAddBlueprintToFoundation 🌐 | likely | Constructs FactoryPacketRequestAddBlueprintToFoundation (009db490). The raw frame reserves a 0x18-byte request record, with constructor receiver at [esp+0x0c] … |
| `009e8cd0` | RequestEquipTool 🌐 | likely | Builds the FactoryPacketEquipToolRequest object (009d96e0), fills the local request record with the tool id and sends it through the gateway sender 009e7e80 wh… |
| `009e8d60` | RequestListTools 🌐 | likely | Builds the FactoryPacketListToolsRequest object (009d9630), sends it through the gateway sender 009e7f90 when the interface at client+0x2270+0x280 is set and r… |
| `009e9640` | CompositeMapRecord_read 🌐 | likely | Deserializes the large composite record: reads the map at +0x852 dwords with FUN_009e5290, then an 8-byte pair into +0/+4, the map at +8 with FUN_009e8220, the… |
| `009e9800` | RequestCreateBlueprint 🌐 | likely | Builds the FactoryPacketCreateBlueprintRequest object (009e1440, which owns a SoeUtil::List<ItemRequestEntry>), copies the caller's entry list into it (clear F… |
| `009e9a70` | FactoryInfoUpdate_read 🌐 | likely | Runs the stream helper FUN_009e0340, deserializes the composite record with FUN_009e9640 and then reads two dwords into the owner fields at +0x2270 and +0x2274… |

### `net.game_command`

**GameCommand_AcceptAdjournment** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01430c70` | GameCommand_AcceptAdjournment::GameCommand_AcceptAdjournment | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_AcceptAdjournment::vftable at +0, and returns this. |
| `01430cf0` | GameCommand_AcceptAdjournment::GetClassName | confirmed | Returns the address of the literal GameCommand_AcceptAdjournment (mov eax,imm32; ret). No arguments and no side effects. |
| `01430d30` | GameCommand_AcceptAdjournment::GameCommand_AcceptAdjournmen… | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_AcceptAdjournment::vftable at +0, then calls FUN_0141b3d0 with its stack argument; returns this… |
| `01430df0` | GameCommand_AcceptAdjournment::ApplyToGame | confirmed | If this+0x8 is null it asserts mGame at GameCommandAcceptAdjournment.cpp line 26. Otherwise it calls FUN_0141b020, then virtual slot 30 (+0x78) of the object a… |
| `01430e30` | GameCommand_AcceptAdjournment::Clone | confirmed | If this+0x8 (mGame) is null it asserts mGame at GameCommandAcceptAdjournment.cpp line 34. It allocates 0x28 bytes with FUN_00d833a6; if that succeeds it constr… |
| `01430ee0` | GameCommand_AcceptAdjournment::CloneInto | confirmed | Dynamic-casts the argument to GameCommand_AcceptAdjournment with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). On failure it calls FUN_012f5a60 w… |
| `01431070` | GameCommand_AcceptAdjournment::~GameCommand_AcceptAdjournme… | confirmed | Scalar deleting destructor: stores GameCommand_AcceptAdjournment::vftable at +0, calls FUN_0141b310 on this (base destructor body), then if bit 0 of the flag a… |
| `014310d0` | GameCommand_AcceptAdjournment::Display | confirmed | Appends the literal Accept Adjourn Game Command with a newline via FUN_01241650, calls FUN_0141b240 with the stream argument, then formats mAccept: %d from the… |
| `01430d00` | GameCommand_AcceptAdjournment::GetGlobal_01cbe2a8 | likely | Returns the dword at the absolute address DAT_01cbe2a8 (mov eax,[abs]; ret). It does not read this. |
| `01430d10` | GameCommand_AcceptAdjournment::SetGlobal_01cbe2a8 | likely | Stores its stack argument into the absolute address DAT_01cbe2a8 (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `01430da0` | GameCommand_AcceptAdjournment::DtorBody | likely | Stores GameCommand_AcceptAdjournment::vftable at +0 first, then calls FUN_0141b310 on this (base destructor body). It does not free the object. |
| `01430f30` | GameCommand_AcceptAdjournment::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and the address of a local pointer-sized value, and tests the result. It calls FUN_0141b520 with the strea… |
| `01430ff0` | GameCommand_AcceptAdjournment::Create | likely | Allocates 0x28 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_01430c70 (no-argument constructor) on the new object and returns that result… |

**GameCommand_AcceptDraw** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01430790` | GameCommand_AcceptDraw::GameCommand_AcceptDraw | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_AcceptDraw::vftable at +0, and returns this. |
| `01430810` | GameCommand_AcceptDraw::GetClassName | confirmed | Returns the address of the literal GameCommand_AcceptDraw (mov eax,imm32; ret). No arguments and no side effects. |
| `01430850` | GameCommand_AcceptDraw::GameCommand_AcceptDraw_WithArg | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_AcceptDraw::vftable at +0, then calls FUN_0141b3d0 with its stack argument; returns this (ret 4… |
| `01430910` | GameCommand_AcceptDraw::ApplyToGame | confirmed | If this+0x8 is null it asserts mGame at GameCommandAcceptDraw.cpp line 25. Otherwise it calls FUN_0141b020, then virtual slot 121 (+0x1e4) of the object at thi… |
| `01430950` | GameCommand_AcceptDraw::Clone | confirmed | If this+0x8 (mGame) is null it asserts mGame at GameCommandAcceptDraw.cpp line 33. It allocates 0x28 bytes with FUN_00d833a6; if that succeeds it constructs th… |
| `01430a00` | GameCommand_AcceptDraw::CloneInto | confirmed | Dynamic-casts the argument to GameCommand_AcceptDraw with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). On failure it calls FUN_012f5a60 with the… |
| `01430b90` | GameCommand_AcceptDraw::~GameCommand_AcceptDraw | confirmed | Scalar deleting destructor: stores GameCommand_AcceptDraw::vftable at +0, calls FUN_0141b310 on this (base destructor body), then if bit 0 of the flag argument… |
| `01430bf0` | GameCommand_AcceptDraw::Display | confirmed | Appends the literal Accept Draw Game Command with a newline via FUN_01241650, calls FUN_0141b240 with the stream argument, then formats mAccept: %d with the un… |
| `01430820` | GameCommand_AcceptDraw::GetGlobal_01cbe2a4 | likely | Returns the dword at the absolute address DAT_01cbe2a4 (mov eax,[abs]; ret). It does not read this. |
| `01430830` | GameCommand_AcceptDraw::SetGlobal_01cbe2a4 | likely | Stores its stack argument into the absolute address DAT_01cbe2a4 (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `014308c0` | GameCommand_AcceptDraw::DtorBody | likely | Stores GameCommand_AcceptDraw::vftable at +0 first, then calls FUN_0141b310 on this (base destructor body). It does not free the object. |
| `01430a50` | GameCommand_AcceptDraw::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and the address of a local pointer-sized value, and tests the result. It calls FUN_0141b520 with the strea… |
| `01430b10` | GameCommand_AcceptDraw::Create | likely | Allocates 0x28 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_01430790 (no-argument constructor) on the new object and returns that result… |

**GameCommand_ConcedeGame** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01431150` | GameCommand_ConcedeGame::GameCommand_ConcedeGame | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_ConcedeGame::vftable at +0, zeroes the byte at +0x24, and returns this. |
| `014311d0` | GameCommand_ConcedeGame::GetClassName | confirmed | Returns the address of the literal GameCommand_ConcedeGame (mov eax,imm32; ret). No arguments and no side effects. |
| `01431210` | GameCommand_ConcedeGame::GameCommand_ConcedeGame_WithArg | confirmed | Calls base constructor FUN_0141b5d0 on this, zeroes the byte at +0x24, stores GameCommand_ConcedeGame::vftable at +0, then calls FUN_0141b3d0 with its stack ar… |
| `014312d0` | GameCommand_ConcedeGame::ApplyToGame | confirmed | If this+0x8 is nonzero it calls FUN_0141b020, then if the byte at this+0x24 is nonzero it calls virtual slot 118 (+0x1d8) of the object at this+0x8 with this+0… |
| `01431310` | GameCommand_ConcedeGame::Clone | confirmed | If this+0x8 (mGame) is null it asserts mGame at GameCommandConcedeGame.cpp line 43. It allocates 0x28 bytes with FUN_00d833a6; if that succeeds it constructs t… |
| `014313a0` | GameCommand_ConcedeGame::CloneInto | confirmed | Dynamic-casts the argument to GameCommand_ConcedeGame with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). On failure it calls FUN_012f5a60 with th… |
| `01431530` | GameCommand_ConcedeGame::~GameCommand_ConcedeGame | confirmed | Scalar deleting destructor: stores GameCommand_ConcedeGame::vftable at +0, calls FUN_0141b310 on this (base destructor body), then if bit 0 of the flag argumen… |
| `01431590` | GameCommand_ConcedeGame::Display | confirmed | Appends the literal Concede Game Game Command with a newline to the stream argument via FUN_01241650, then calls FUN_0141b240 with this and the stream argument… |
| `014311e0` | GameCommand_ConcedeGame::GetGlobal_01cbe2ac | likely | Returns the dword at the absolute address DAT_01cbe2ac (mov eax,[abs]; ret). It does not read this. |
| `014311f0` | GameCommand_ConcedeGame::SetGlobal_01cbe2ac | likely | Stores its stack argument into the absolute address DAT_01cbe2ac (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `01431280` | GameCommand_ConcedeGame::DtorBody | likely | Stores GameCommand_ConcedeGame::vftable at +0 first, then calls FUN_0141b310 on this (base destructor body). It does not free the object. |
| `014313f0` | GameCommand_ConcedeGame::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and the address of the stream argument slot, and tests the result. It calls FUN_0141b520 with the stream a… |
| `014314b0` | GameCommand_ConcedeGame::Create | likely | Allocates 0x28 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_01431150 (no-argument constructor) on the new object and returns that result… |

**GameCommand_RemovePlayer** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142fbd0` | GameCommand_RemovePlayer::GameCommand_RemovePlayer | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_RemovePlayer::vftable at +0, zeroes the dword at +0x24, and returns this. |
| `0142fc50` | GameCommand_RemovePlayer::GetClassName | confirmed | Returns the address of the literal GameCommand_RemovePlayer (mov eax,imm32; ret). No arguments and no side effects. |
| `0142fc90` | GameCommand_RemovePlayer::GameCommand_RemovePlayer_WithArg | confirmed | Calls base constructor FUN_0141b5d0 on this, zeroes the dword at +0x24, stores GameCommand_RemovePlayer::vftable at +0, then calls FUN_0141b3d0 with its stack … |
| `0142fd60` | GameCommand_RemovePlayer::ApplyToGame | confirmed | If this+0x8 is null it asserts the text mGame at GameCommandRemovePlayer.cpp line 32. Otherwise it calls FUN_0141b020, then virtual slot 45 (+0xb4) of the obje… |
| `0142fda0` | GameCommand_RemovePlayer::Clone | confirmed | If this+0x8 (mGame) is null it asserts the text mGame at GameCommandRemovePlayer.cpp line 41. It allocates 0x2c bytes with FUN_00d833a6; if that succeeds it co… |
| `0142fe30` | GameCommand_RemovePlayer::CloneInto | confirmed | Dynamic-casts the argument to GameCommand_RemovePlayer with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). On failure it calls FUN_012f5a60 with t… |
| `0142ff60` | GameCommand_RemovePlayer::~GameCommand_RemovePlayer | confirmed | Scalar deleting destructor: stores GameCommand_RemovePlayer::vftable at +0, calls FUN_0141b310 on this (base destructor body), then if bit 0 of the flag argume… |
| `014300c0` | GameCommand_RemovePlayer::Display | confirmed | Appends the literal Remove Player Game Command with a newline, calls FUN_0141b240 on the stream argument, appends the literal Loss Type: and then the decimal v… |
| `0142fc60` | GameCommand_RemovePlayer::GetGlobal_01cbe29c | likely | Returns the dword at the absolute address DAT_01cbe29c (mov eax,[abs]; ret). It does not read this. |
| `0142fc70` | GameCommand_RemovePlayer::SetGlobal_01cbe29c | likely | Stores its stack argument into the absolute address DAT_01cbe29c (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `0142fd00` | GameCommand_RemovePlayer::DtorBody | likely | Stores GameCommand_RemovePlayer::vftable at +0 first, then calls FUN_0141b310 on this (base destructor body). It does not free the object. |
| `0142fee0` | GameCommand_RemovePlayer::Create | likely | Allocates 0x2c bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_0142fbd0 with the new object as this and returns that result, otherwise retu… |
| `0142ffc0` | GameCommand_RemovePlayer::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and the address of a 16-byte local, then tests the result. It calls FUN_0141b520 with the stream and tests… |

**GameCommand_SelectDeckForPlayer** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014301f0` | GameCommand_SelectDeckForPlayer::ApplyToGame | confirmed | If this+0x8 is null it asserts mGame at GameCommandSelectDeckForPlayer.cpp line 112. It calls FUN_0141b020, then resolves the player with FUN_01350660(this+0x4… |
| `014302c0` | GameCommand_SelectDeckForPlayer::GameCommand_SelectDeckForP… | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_SelectDeckForPlayer::vftable at +0, writes 0xf to the dword at +0x3c, 0 to +0x38, a zero byte a… |
| `01430350` | GameCommand_SelectDeckForPlayer::GetClassName | confirmed | Returns the address of the literal GameCommand_SelectDeckForPlayer (mov eax,imm32; ret). No arguments and no side effects. |
| `01430390` | GameCommand_SelectDeckForPlayer::GameCommand_SelectDeckForP… | confirmed | Calls base constructor FUN_0141b5d0 on this, stores GameCommand_SelectDeckForPlayer::vftable at +0, writes 0xf to +0x3c, 0 to +0x38, a zero byte at +0x28, then… |
| `014304a0` | GameCommand_SelectDeckForPlayer::Clone | confirmed | Allocates 0x44 bytes with FUN_00d833a6; if that succeeds it constructs the copy with FUN_01430390(this+0x8). Then it calls the virtual at vtable slot 18 (+0x48… |
| `01430590` | GameCommand_SelectDeckForPlayer::~GameCommand_SelectDeckFor… | confirmed | Scalar deleting destructor: calls the destructor body FUN_01430410 on this, then if bit 0 of the flag argument is set it calls _free on this; returns this. |
| `01430610` | GameCommand_SelectDeckForPlayer::CloneInto | confirmed | Dynamic-casts the argument to GameCommand_SelectDeckForPlayer with __RTDynamicCast (FUN_00d8d382, source RTTI NetworkCommand). On failure it calls FUN_012f5a60… |
| `01430680` | GameCommand_SelectDeckForPlayer::Display | confirmed | Appends the literal Select Deck Game Command with a newline via FUN_01241650, calls FUN_0141b240 with the stream argument, then selects the deck-id text: the i… |
| `01430260` | GameCommand_SelectDeckForPlayer::ReadFields | likely | Calls virtual slot 4 (+0x10) of this with the stream and the address of its argument, and tests the result. It calls FUN_0141b520 with the stream as a gate, th… |
| `01430360` | GameCommand_SelectDeckForPlayer::GetGlobal_01cbe2a0 | likely | Returns the dword at the absolute address DAT_01cbe2a0 (mov eax,[abs]; ret). It does not read this. |
| `01430370` | GameCommand_SelectDeckForPlayer::SetGlobal_01cbe2a0 | likely | Stores its stack argument into the absolute address DAT_01cbe2a0 (mov eax,[esp+4]; mov [abs],eax; ret 4). It does not touch this. |
| `01430410` | GameCommand_SelectDeckForPlayer::DtorBody | likely | Stores GameCommand_SelectDeckForPlayer::vftable at +0, releases the object at +0x40 by calling its vtable slot 0 with flag 1 and zeroes +0x40, frees the buffer… |
| `01430510` | GameCommand_SelectDeckForPlayer::Create | likely | Allocates 0x44 bytes with FUN_00d833a6; if the allocation is non-null it calls FUN_014302c0 (no-argument constructor) and returns its result, otherwise returns… |
| `014305b0` | GameCommand_SelectDeckForPlayer::SetDeck | likely | Asserts the text at GameCommandSelectDeckForPlayer.cpp line 27 when its argument is null. It releases the object at this+0x40 (vtable slot 0 with flag 1) and z… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0142fd50` | SetDwordAt24 | likely | Stores its stack argument into the dword at this+0x24 (mov eax,[esp+4]; mov [ecx+0x24],eax; ret 4). |
| `014309e0` | SetByte24_014309e0 | likely | Stores its stack argument (one byte) into this+0x24 (mov al,[esp+4]; mov [ecx+0x24],al; ret 4). |

### `net.synchronization`

**SynchronizationCommand_AddNonTradable** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b1ea0` | SynchronizationCommand_AddNonTradable::ctor | confirmed | Constructor (this in ECX). Calls 014ae2a0 (the SynchronizationCommand_ChangeFlags constructor), stores SynchronizationCommand_AddNonTradable::vftable, then pus… |
| `014b1f20` | SynchronizationCommand_AddNonTradable::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_AddNonTradable (mov eax, literal; ret). Reads no fields. |
| `014b1fd0` | SynchronizationCommand_AddNonTradable::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x20. If the block is non-null it constructs an AddNonTradable command in it with 014b1ea0. Th… |
| `014b2040` | SynchronizationCommand_AddNonTradable::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as bool), then 014ae550 with the argument (the ChangeF… |
| `014b2080` | SynchronizationCommand_AddNonTradable::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_AddNonTradable through 013c26e0 … |
| `014b2150` | SynchronizationCommand_AddNonTradable::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores SynchronizationCommand_AddNonTradable::vftable into *this, calls 014adcb0 (ChangeFlags-level teardown), then… |
| `014b21b0` | SynchronizationCommand_AddNonTradable::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (the output string). Appends SynchronizationCommand_AddNonTradable plus newline via 01241650, calls thunk 01435510 … |
| `014b2330` | SynchronizationCommand_AddNonTradable::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x28 and the literal Remove Instances Synchronization Command, … |
| `014b1f30` | SynchronizationCommand_AddNonTradable::ReadGlobal01cbe6f8 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6f8 (mov eax,[01cbe6f8]; ret). Reads no object fields. |
| `014b1f40` | SynchronizationCommand_AddNonTradable::WriteGlobal01cbe6f8 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6f8. |
| `014b20d0` | SynchronizationCommand_AddNonTradable::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x20. If the block is non-null it passes it to 014b1ea0 (the AddNonTradable constructor) and returns that… |

**SynchronizationCommand_ProcessFulfillment** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b1960` | SynchronizationCommand_ProcessFulfillment::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (SynchronizationCommand base constructor), stores SynchronizationCommand_ProcessFulfillment::vftable, calls 013d3300 … |
| `014b1a20` | SynchronizationCommand_ProcessFulfillment::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_ProcessFulfillment (mov eax, literal; ret). Reads no fields. |
| `014b1bb0` | SynchronizationCommand_ProcessFulfillment::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it constructs a ProcessFulfillment command in it with 014b1960.… |
| `014b1c20` | SynchronizationCommand_ProcessFulfillment::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to ProcessFulfillment (00d8d382). On failure reports via 012f5a60 (synchCommand, Sy… |
| `014b1c80` | SynchronizationCommand_ProcessFulfillment::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as bool), then 01435520 with the argument. Reads two i… |
| `014b1ce0` | SynchronizationCommand_ProcessFulfillment::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_ProcessFulfillment through 013c2… |
| `014b1dd0` | SynchronizationCommand_ProcessFulfillment::ScalarDeletingDt… | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014b1a60 (the ProcessFulfillment body destructor), then calls _free(this) only when bit 0 of the byte argumen… |
| `014b1df0` | SynchronizationCommand_ProcessFulfillment::Dump | confirmed | Virtual slot 19 (+0x4c). Calls thunk 01435510 (stub) with the output string, appends the header ProcessFulfillment Synchronization Command plus newline via 012… |
| `014b1e80` | SynchronizationCommand_ProcessFulfillment::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x2a and the literal ProcessFulfillment Synchronization Command… |
| `014b1a30` | SynchronizationCommand_ProcessFulfillment::ReadGlobal01cbe6… | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6f4 (mov eax,[01cbe6f4]; ret). Reads no object fields. |
| `014b1a40` | SynchronizationCommand_ProcessFulfillment::WriteGlobal01cbe… | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6f4. |
| `014b1d50` | SynchronizationCommand_ProcessFulfillment::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it passes it to 014b1960 (the ProcessFulfillment constructor) and returns … |

**SynchronizationCommand_RemoveInstances** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b1480` | SynchronizationCommand_RemoveInstances::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_RemoveInstances through 013c26e0… |
| `014b1570` | SynchronizationCommand_RemoveInstances::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_RemoveInstances (mov eax, literal; ret). Reads no fields. |
| `014b15b0` | SynchronizationCommand_RemoveInstances::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014b14f0 (body destructor), then calls _free(this) only when bit 0 of the byte argument is set. Returns this. |
| `014b15d0` | SynchronizationCommand_RemoveInstances::Dump | confirmed | Virtual slot 19 (+0x4c). Calls thunk 01435510 (stub) with the output string, then formats the header Remove Instances Synchronization Command (40 bytes) into t… |
| `014b16b0` | SynchronizationCommand_RemoveInstances::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x28 and the literal Remove Instances Synchronization Command, … |
| `014b16d0` | SynchronizationCommand_RemoveInstances::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (SynchronizationCommand base constructor), stores SynchronizationCommand_RemoveInstances::vftable, zeroes the dwords … |
| `014b1740` | SynchronizationCommand_RemoveInstances::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it constructs a RemoveInstances command in it with 014b16d0. Th… |
| `014b17b0` | SynchronizationCommand_RemoveInstances::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to RemoveInstances (00d8d382). On failure reports via 012f5a60 (synchCommand, Synch… |
| `014b1900` | SynchronizationCommand_RemoveInstances::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &local (header read; tested as bool), then 01435520 with the argument, then 013d53d0… |
| `014b14f0` | SynchronizationCommand_RemoveInstances::DtorBody | likely | Stores SynchronizationCommand_RemoveInstances::vftable into *this, calls 012c2de0 with ECX = this+0x4 (member teardown of the Instances container), then calls … |
| `014b1580` | SynchronizationCommand_RemoveInstances::ReadGlobal01cbe6f0 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6f0 (mov eax,[01cbe6f0]; ret). Reads no object fields. |
| `014b1590` | SynchronizationCommand_RemoveInstances::WriteGlobal01cbe6f0 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6f0. |
| `014b1810` | SynchronizationCommand_RemoveInstances::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it passes it to 014b16d0 (the RemoveInstances constructor) and returns tha… |

**SynchronizationCommand_RemoveNonTradable** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b0f30` | SynchronizationCommand_RemoveNonTradable::ctor | confirmed | Constructor (this in ECX). Calls 014ae2a0 (the SynchronizationCommand_ChangeFlags constructor, which stores ChangeFlags::vftable and zeroes +0x8 and +0xc), sto… |
| `014b0fb0` | SynchronizationCommand_RemoveNonTradable::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_RemoveNonTradable (mov eax, literal; ret). Reads no fields. |
| `014b1060` | SynchronizationCommand_RemoveNonTradable::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x20. If the block is non-null it constructs a RemoveNonTradable command in it with 014b0f30. … |
| `014b10d0` | SynchronizationCommand_RemoveNonTradable::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (header read; tested as a bool), calls 014ae550 with the argument (the Chan… |
| `014b1110` | SynchronizationCommand_RemoveNonTradable::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_RemoveNonTradable through 013c26… |
| `014b11e0` | SynchronizationCommand_RemoveNonTradable::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores SynchronizationCommand_RemoveNonTradable::vftable into *this, calls 014adcb0 (ChangeFlags-level teardown), t… |
| `014b1240` | SynchronizationCommand_RemoveNonTradable::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (the output string; Ghidra shows no receiver, so this is taken to be in ECX, as the no-argument calls 014ad880, 014… |
| `014b13c0` | SynchronizationCommand_RemoveNonTradable::AppendTypeName | confirmed | Virtual slot 20 (+0x50) of the RemoveNonTradable vftable. Loads the stack argument (std::string pointer) into ECX, pushes length 0x28 and the literal Remove In… |
| `014b0fc0` | SynchronizationCommand_RemoveNonTradable::ReadGlobal01cbe6ec | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6ec (mov eax,[01cbe6ec]; ret). Reads no object fields. |
| `014b0fd0` | SynchronizationCommand_RemoveNonTradable::WriteGlobal01cbe6… | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6ec. |
| `014b1160` | SynchronizationCommand_RemoveNonTradable::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x20. If the block is non-null it passes it to 014b0f30 (the RemoveNonTradable constructor) and returns t… |

**SynchronizationCommand_SendInstances** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b04f0` | SynchronizationCommand_SendInstances::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_SendInstances through 013c26e0 (… |
| `014b0650` | SynchronizationCommand_SendInstances::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_SendInstances (mov eax, literal; ret). Reads no fields. |
| `014b0690` | SynchronizationCommand_SendInstances::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014b05c0 (the SendInstances body destructor), then calls _free(this) only when bit 0 of the byte argument is … |
| `014b06b0` | SynchronizationCommand_SendInstances::Dump | confirmed | Virtual slot 19 (+0x4c). Appends SendInstances Command plus newline (22 bytes) via 01241650, calls thunk 01435510 (stub), then appends Delivery ID, Base Archet… |
| `014b08d0` | SynchronizationCommand_SendInstances::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x26 and the literal Send Instances Synchronization Command, ca… |
| `014b08f0` | SynchronizationCommand_SendInstances::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (SynchronizationCommand base constructor), stores SynchronizationCommand_SendInstances::vftable, zeroes the dwords at… |
| `014b0980` | SynchronizationCommand_SendInstances::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x34. If the block is non-null it constructs a SendInstances command in it with 014b08f0. Then… |
| `014b09f0` | SynchronizationCommand_SendInstances::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SendInstances (00d8d382). On failure reports via 012f5a60 (synchCommand, Synchro… |
| `014b0e90` | SynchronizationCommand_SendInstances::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &local (header read; tested as bool), then 01435520 with the argument. Reads with 01… |
| `014b05c0` | SynchronizationCommand_SendInstances::DtorBody | likely | Stores SynchronizationCommand_SendInstances::vftable into *this, then calls 012c2de0 with ECX = this+0x24 and again with ECX = this+0x10 (member teardown in th… |
| `014b0660` | SynchronizationCommand_SendInstances::ReadGlobal01cbe6e8 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6e8 (mov eax,[01cbe6e8]; ret). Reads no object fields. |
| `014b0670` | SynchronizationCommand_SendInstances::WriteGlobal01cbe6e8 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6e8. |
| `014b0a70` | SynchronizationCommand_SendInstances::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x34. If the block is non-null it passes it to 014b08f0 (the SendInstances constructor) and returns that … |

**SynchronizationCommand_SetDesired** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aff40` | SynchronizationCommand_SetDesired::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument, writes the text Starting SynchronizationCommand_SetDesired through 013c26e0 (no-op), calls 0143… |
| `014b0040` | SynchronizationCommand_SetDesired::TypeName | confirmed | Virtual slot 11 (+0x2c). Returns the constant C string SynchronizationCommand_SetDesired (mov eax, literal; ret). Reads no fields. |
| `014b0080` | SynchronizationCommand_SetDesired::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014affb0 (the SetDesired body destructor), then calls _free(this) only when bit 0 of the byte argument is set… |
| `014b01b0` | SynchronizationCommand_SetDesired::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x23 and the literal Set Desired Synchronization Command, calls… |
| `014b01d0` | SynchronizationCommand_SetDesired::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (SynchronizationCommand base constructor), stores SynchronizationCommand_SetDesired::vftable, and zeroes the dwords a… |
| `014b0240` | SynchronizationCommand_SetDesired::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it constructs a SetDesired command in it with 014b01d0. Then, u… |
| `014b02b0` | SynchronizationCommand_SetDesired::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer, dynamic_cast to SetDesired (00d8d382). On failure reports via 012f5a60 (synchCommand, Synchroniz… |
| `014b03c0` | SynchronizationCommand_SetDesired::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &local (the header read; result tested as a bool), then 01435520 with the argument, … |
| `014affb0` | SynchronizationCommand_SetDesired::DtorBody | likely | Stores SynchronizationCommand_SetDesired::vftable into *this, frees the buffer at +0x8 through _free when non-null, zeroes the dwords at +0x8, +0xc and +0x10, … |
| `014b0050` | SynchronizationCommand_SetDesired::ReadGlobal01cbe6e4 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6e4 (mov eax,[01cbe6e4]; ret). Reads no object fields. |
| `014b0060` | SynchronizationCommand_SetDesired::WriteGlobal01cbe6e4 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6e4. |
| `014b0310` | SynchronizationCommand_SetDesired::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it passes it to 014b01d0 (the SetDesired constructor) and returns that obj… |

**SynchronizationCommand_SetModID** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014afac0` | SynchronizationCommand_SetModID::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (SynchronizationCommand base constructor), stores SynchronizationCommand_SetModID::vftable, and zeroes the dword at +… |
| `014afb40` | SynchronizationCommand_SetModID::TypeName | confirmed | Virtual slot 11 (+0x2c). Bytes b8 imm32 c3: returns the constant C string SynchronizationCommand_SetModID (mov eax,018fdabc; ret). Reads no fields. |
| `014afc10` | SynchronizationCommand_SetModID::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 8. If the block is non-null it constructs a SetModID command in it with 014afac0. Then, uncond… |
| `014afc80` | SynchronizationCommand_SetModID::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SetModID (00d8d382). On failure reports via 012f5a60 (synchCommand, Synchronizat… |
| `014afcd0` | SynchronizationCommand_SetModID::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (result tested as a bool), then 01435520 with the argument, then reads the … |
| `014afd20` | SynchronizationCommand_SetModID::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) with the argument (header write), writes the text Starting SynchronizationCommand_SetModID through 013c26e0 (a no-… |
| `014afe00` | SynchronizationCommand_SetModID::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores SynchronizationCommand_SetModID::vftable into *this, calls 01435430 (base teardown), then calls _free(this) … |
| `014afe60` | SynchronizationCommand_SetModID::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (output string): appends SynchronizationCommand_SetModID plus newline via 01241650 (ECX = output string), calls thu… |
| `014afef0` | SynchronizationCommand_SetModID::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x22 and the literal Set Mod ID Synchronization Command, calls … |
| `014afb50` | SynchronizationCommand_SetModID::ReadGlobal01cbe6e0 | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6e0 (mov eax,[01cbe6e0]; ret). Reads no object fields. |
| `014afb60` | SynchronizationCommand_SetModID::WriteGlobal01cbe6e0 | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6e0. |
| `014afd80` | SynchronizationCommand_SetModID::New | likely | No arguments. Calls 00d833a6 (operator new) with size 8. If the block is non-null it passes it to 014afac0 (the SetModID constructor) and returns that object; … |

**SynchronizationCommand_Start** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014af620` | SynchronizationCommand_Start::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (the SynchronizationCommand base constructor), stores SynchronizationCommand_Start::vftable, and zeroes the dwords at… |
| `014af6a0` | SynchronizationCommand_Start::TypeName | confirmed | Virtual slot 11 (+0x2c). Bytes b8 imm32 c3: returns the constant C string SynchronizationCommand_Start (mov eax,018fd9b4; ret). Reads no fields. |
| `014af750` | SynchronizationCommand_Start::Clone | confirmed | Virtual slot 17 (+0x44). Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it constructs a Start command in it with 014af620. Then, uncond… |
| `014af840` | SynchronizationCommand_Start::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). Argument is a NetworkCommand pointer; dynamic_cast to SynchronizationCommand_Start (00d8d382). On failure reports via 012f5a60 (synchC… |
| `014af8a0` | SynchronizationCommand_Start::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (result tested as a bool), then 01435520 with the argument. Reads with 013c… |
| `014af930` | SynchronizationCommand_Start::WriteBody | confirmed | Virtual slot 10 (+0x28). Calls slot 2 (+0x8) on this with the argument (header write), then 01435560 with the argument (base write step), then 013c23f0 (Serial… |
| `014afa10` | SynchronizationCommand_Start::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Stores SynchronizationCommand_Start::vftable into *this, calls 01435430 (base teardown), then calls _free(this) onl… |
| `014afa70` | SynchronizationCommand_Start::Dump | confirmed | Virtual slot 19 (+0x4c). One stack argument (the output string, loaded into EDI and then ECX): appends the 30-byte literal Start Synchronization Command plus n… |
| `014afaa0` | SynchronizationCommand_Start::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the stack argument (std::string pointer) into ECX, pushes length 0x1d and the literal Start Synchronization Command, calls 01241… |
| `014af6b0` | SynchronizationCommand_Start::ReadGlobal01cbe6dc | likely | Virtual slot 12 (+0x30). Returns the 32-bit global at 01cbe6dc (mov eax,[01cbe6dc]; ret). Reads no object fields. |
| `014af6c0` | SynchronizationCommand_Start::WriteGlobal01cbe6dc | likely | Virtual slot 13 (+0x34). One stack argument (ret 4): stores it into the global at 01cbe6dc (mov eax,[esp+4]; mov [01cbe6dc],eax). Reads no object fields. |
| `014af990` | SynchronizationCommand_Start::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x14. If the block is non-null it passes it to 014af620 (the Start constructor) and returns that object; … |

**SynchronizationCommand_StartDelivery** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014aef50` | SynchronizationCommand_StartDelivery::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0). Calls 014aee50 (the StartDelivery body destructor), then calls _free(this) only when bit 0 of the byte argument is … |
| `014aef90` | SynchronizationCommand_StartDelivery::Dump | confirmed | Virtual slot 19. Calls thunk 01435510 (to 0140c290, a stub that returns) with the output string argument, then appends nine lines to that std::string via 01241… |
| `014af1b0` | SynchronizationCommand_StartDelivery::AppendTypeName | confirmed | Virtual slot 20 (+0x50). Loads the single stack argument (a std::string pointer) into ECX, pushes length 0x26 and the 38-character literal Start Delivery Synch… |
| `014af1d0` | SynchronizationCommand_StartDelivery::ctor | confirmed | Constructor (this in ECX). Calls 01435370 (the SynchronizationCommand base constructor), stores SynchronizationCommand_StartDelivery::vftable, makes the std::s… |
| `014af280` | SynchronizationCommand_StartDelivery::Clone | confirmed | Virtual slot 17. Calls 00d833a6 (operator new) with size 0x5c. If the result is non-null it constructs a StartDelivery object in it with 014af1d0. Then, uncond… |
| `014af2f0` | SynchronizationCommand_StartDelivery::CopyFieldsTo | confirmed | Virtual slot 18 (+0x48). The argument is a NetworkCommand pointer; it is dynamic_cast to SynchronizationCommand_StartDelivery (00d8d382). If the cast fails it … |
| `014af4f0` | SynchronizationCommand_StartDelivery::ReadBody | confirmed | Virtual slot 9 (+0x24). Calls slot 4 (+0x10) on this with the argument and &param_2 (its result is tested as a bool), then 01435520 with the argument. Reads ve… |
| `014af390` | SynchronizationCommand_StartDelivery::New | likely | No arguments. Calls 00d833a6 (operator new) with size 0x5c. If the block is non-null it passes it in ECX to 014af1d0 (the StartDelivery constructor) and return… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014afde0` | RegisterFactoryKey13c | likely | No arguments. Pushes the address 014afd80 (the SetModID New factory) and key 0x13c, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape as 01… |
| `014b0370` | RegisterFactoryKey10f | likely | No arguments. Pushes the address 014b0310 (the SetDesired New factory) and key 0x10f, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape as … |
| `014b0ad0` | RegisterFactoryKey55 | likely | No arguments. Pushes the address 014b0a70 (the SendInstances New factory) and key 0x55, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape a… |
| `014b11c0` | RegisterFactoryKey12e | likely | No arguments. Pushes the address 014b1160 (the RemoveNonTradable New factory) and key 0x12e, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same sh… |
| `014b1870` | RegisterFactoryKey8f | likely | No arguments. Pushes the address 014b1810 (the RemoveInstances New factory) and key 0x8f, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape… |
| `014b1db0` | RegisterFactoryKey8e | likely | No arguments. Pushes the address 014b1d50 (the ProcessFulfillment New factory) and key 0x8e, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same sh… |
| `014b2130` | RegisterFactoryKey12d | likely | No arguments. Pushes the address 014b20d0 (the AddNonTradable New factory) and key 0x12d, calls 013ce130, moves its EAX into ECX and calls 013ce230. Same shape… |

### `ui.broker`

**BrokerDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cee410` | BrokerDataSource::FieldName | confirmed | Maps indices 0 through 4 to broker GUID, maximum items, markup modifier, listing fee modifier, and listing duration; returns <error> otherwise. |
| `00cefe40` | BrokerDataSource::Ctor | confirmed | Installs BrokerDataSource vtables, initializes base data fields and strings, constructs SearchItems, MyItems, and PlayerInventoryItems child data sources, and … |
| `00cf0040` | BrokerDataSource::Dtor | confirmed | Releases the embedded IString buffer, three shared-object members when release mode allows, then destroys the base GUI user-data table subobject. |
| `00cf0150` | BrokerDataSource::ScalarDeletingDtor | confirmed | Calls BrokerDataSource cleanup and frees the object when the deleting flag bit is set; otherwise returns its pointer. |

**BrokerItemDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cee3a0` | BrokerItemDataSource::FieldName | confirmed | Returns field-name strings for indices 0x1d through 0x21, covering item GUID, listing price, remaining time, seller name, and listing status; delegates other i… |
| `00ceefe0` | BrokerItemDataSource::Ctor | confirmed | Initializes the base data source, installs BrokerItemDataSource vtables at four offsets, stores the broker pointer, and zeros its list and integer-array state. |
| `00cef0a0` | ThisAdjustToBrokerItemDataSourceDtor | confirmed | Subtracts 0x254 from ECX and jumps to 00cef210; this is an adjustor thunk and has no independent destructor body. |
| `00cef140` | BrokerItemDataSource::RebuildVisibleIndices | confirmed | Walks the linked item rows, keeps indices for rows accepted by CanDisplayRow, grows the integer array as needed, stores each row index, and clears the changed … |
| `00cef210` | BrokerItemDataSource::ScalarDeletingDtor | confirmed | Calls the index-array cleanup routine and frees the object when the deleting flag bit is set; otherwise returns the object pointer. |
| `00cef480` | BrokerItemDataSource::GetField | confirmed | Bounds-checks a row, resolves its item record, then switches across field IDs to emit names, values, display strings, item flags, prices, quantities, and relat… |
| `00ceeb70` | BrokerItemDataSource::GetCurrentRecord | likely | If the active-state byte at offset 0x2a0 is set, invokes a virtual method through the adjusted object pointer; returns the dword at offset 0x2d0. |
| `00cef0b0` | BrokerItemDataSource::ClearIndexArray | likely | Restores the embedded integer-array vtable, zeros its size, frees its allocation with alignment 4, clears the pointer, and invokes a member cleanup helper. |

**BrokerPlayerInventoryDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cee480` | BrokerPlayerInventoryDataSource::Ctor | confirmed | Initializes the named BaseClient.Broker.PlayerInventoryItems data source, installs its vtables at four object offsets, and stores the broker argument. |
| `00cee560` | BrokerPlayerInventoryDataSource::CanOfferItem | confirmed | Rejects invalid or unavailable item records and two state flags; when a limit is enabled, also requires the computed available count to be positive. |
| `00cee950` | ThisAdjustToBrokerPlayerInventoryDtor | confirmed | Subtracts 0x254 from ECX and jumps to the adjusted BrokerPlayerInventoryDataSource destructor entry; this is an adjustor thunk, not a separate destructor body. |
| `00ceece0` | BrokerPlayerInventoryDataSource::ScalarDeletingDtor | confirmed | Calls the member cleanup routine, then frees the object if the deleting flag bit is set; otherwise returns the object pointer. |

**Loggable** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cef230` | BrokerLoggableSubobjectDtor | confirmed | Cleans the embedded logging subobject under an exception frame, then restores its Loggable vtable at object offset 0x28. |

**(no class)** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c0c930` | RegisterBrokerModule | confirmed | Registration wrapper: pushes nup=0, the 6-entry table 0x0182afa8 and the 'Broker' module-name pointer, calls FUN_00e39e40 and returns 0. |
| `00c0ce10` | Broker_SelectInventoryCategory | likely | Lua 'SelectInventoryCategory' handler: with one numeric argument it resolves the broker window object via FUN_008a6780 and calls FUN_00ceec10(category); return… |
| `00ceeca0` | FindNodeByTwoKeys | likely | Walks the linked records at owner offset 0x2ec until both node fields +0x10 and +0x14 match the two arguments; returns that node or zero. |
| `00ceee80` | BrokerListRemoveByTwoKeys | likely | Finds the first list node matching two dword keys, unlinks it from the doubly linked list, decrements the count, sets a changed byte, and calls a virtual notif… |
| `00ceef10` | BrokerListRemoveAt290 | likely | Packages the two key arguments and calls the shared broker list removal routine using the list at owner offset 0x290. |
| `00ceef30` | BrokerListAppendItem | likely | Links the supplied node at the tail of the owner list, initializes its backward link, increments list count, marks the list changed, and requests the item defi… |
| `00ceefb0` | BrokerListRemoveAt294 | likely | Packages the two key arguments and calls the shared broker list removal routine using the list at owner offset 0x294. |
| `00cef2d0` | ScalarDeletingDtorWrapper_00cef230 | likely | Calls the object cleanup routine and frees the object when the deleting flag bit is set; otherwise returns its pointer. |
| `00cef450` | FormatInt64AsLld | likely | Formats the two dwords at the supplied value pointer with the wide format %lld and returns the destination string object's data pointer. |

## Source shards

[[S0248 009392c0-0093d740|S0248]] (1), [[S0269 009a6cf0-009aceb0|S0269]] (3), [[S0278 009d89c0-009e00c0|S0278]] (49), [[S0279 009e0340-009e4510|S0279]] (8), [[S0280 009e4570-009ea840|S0280]] (29), [[S0378 00becaa0-00bf1530|S0378]] (30), [[S0383 00c05780-00c0db00|S0383]] (2), [[S0384 00c0dcd0-00c171d0|S0384]] (6), [[S0386 00c1f190-00c26aa0|S0386]] (2), [[S0423 00ceb9b0-00cf13e0|S0423]] (29), [[S0675|S0675]] (1), [[S0688|S0688]] (2), [[S0693|S0693]] (1), [[S0697|S0697]] (13), [[S0698|S0698]] (2), [[S0704|S0704]] (3), [[S0722|S0722]] (3), [[S0724|S0724]] (81), [[S0725|S0725]] (14), [[S0728|S0728]] (2), [[S0730|S0730]] (1), [[S0731|S0731]] (91), [[S0732|S0732]] (14), [[S0735|S0735]] (21), [[S0736|S0736]] (16), [[S0740|S0740]] (49), [[S0741|S0741]] (28), [[S0744|S0744]] (37), [[S0747|S0747]] (18), [[S0748|S0748]] (79), [[S0749|S0749]] (30), [[S0750|S0750]] (23), [[S0751|S0751]] (35), [[S0752|S0752]] (48), [[S0753|S0753]] (75), [[S0756|S0756]] (10), [[S0769|S0769]] (7), [[S0771|S0771]] (116), [[S0772|S0772]] (132), [[S0773|S0773]] (80), [[S0774|S0774]] (181), [[S0775|S0775]] (218), [[S0779|S0779]] (147), [[S0780|S0780]] (220), [[S0781|S0781]] (138), [[S0782|S0782]] (66), [[S0783|S0783]] (185), [[S0784|S0784]] (189), [[S0785|S0785]] (31), [[S0791|S0791]] (9), [[S0794|S0794]] (14), [[S0795|S0795]] (147), [[S0796|S0796]] (110), [[S0797|S0797]] (17), [[S0809 0066ca4d-0147b010|S0809]] (10)
