---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Audio

Part of [[Build Guides]] › Client Engine. Related: [[Assets and Pack Files]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

421 accepted functions: 121 confirmed, 148 likely, 84 speculative, 68 unknown. Network-linked (`wire`): 0.

Subsystems: `audio.sound` (171), `audio.service` (100), `audio.definition` (60), `audio` (32), `audio.music` (21), `audio.ambient` (16), `audio.fishing` (7), `audio.listener` (4), `audio.minigame` (3), `audio.vehicle` (3), `audio.soccer` (3), `audio.derby` (1)

### `audio`

**ClientAreaAmbientSoundProperty** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ceb920` | ClientAreaAmbientSoundProperty::Log | speculative | Logs inherited fields plus TrackId, SoundDefinitionId, FinishPlaying, RetriggerDelay, and floorTolerance. |

**ClientAreaData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b0adc0` | ClientAreaData::RemoveAndResetBackgroundAudio | confirmed | Unlinks a node, looks up Sound(Background) and Music(Background), and resets each associated field when its lookup count is below one. |

**ClientTCGInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad1eb0` | CreateTcgAudioEmitter | likely | Calls the global-client helper with zero and two shared defaults, then calls 00b07c30 with the same values and supplied object. |

**SoeUtil::HashListMap** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa860` | HashListMap_SoundControlDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,SoundControlDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x400 by… |

**SoeUtil::HashMap** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008aa890` | HashMap_NameEntry_SoundControlDefinition_Construct | confirmed | Constructor of the name-lookup map for sound controls: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<SoundControlDefinition,256>::NameEntry,… |
| `008aadb0` | HashMap_PendingOneShotEntry_Construct | confirmed | Constructor of the one-shot-sound pending-load map: installs the SoeUtil::HashMap<int,SoundOneShotDefinitionLoader::LoadPendingOneShotEntry,32,-1> vtable, zero… |

**SoundGroup** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c1bb40` | SoundGroup::Ctor | confirmed | Constructs a SoundDefinitionAssetLoader base, initializes multiple SoundGroup vptrs plus the embedded array vptr, and zeroes four array fields. |
| `00c1bc80` | SoundGroup::ScalarDeletingDtor | confirmed | Adjusts the this pointer by -0x10 and jumps to the shared SoundGroup deleting-destructor body at 00c1db60. |
| `00c1bc90` | SoundGroup::ScalarDeletingDtor | confirmed | Adjusts the this pointer by -8 and jumps to the shared SoundGroup deleting-destructor body at 00c1db60. |
| `00c1da10` | SoundGroup::Dtor | confirmed | Restores SoundGroup vptrs, clears entries, restores the embedded array vptr, frees the array storage, restores RefCounted vptr and calls its cleanup routine. |
| `00c1db60` | SoundGroup::ScalarDeletingDtor | confirmed | Calls the SoundGroup destructor body, frees the object when the deleting flag is set, and returns the original pointer otherwise. |
| `00c1bd70` | SoundGroup::FindEntryPointerIndex | likely | Scans the pointer array at this+0x1c over count at this+0x20, compares each entry pointer with the argument and returns the first matching index or -1. |
| `00c1d210` | SoundGroup::ClearEntries | likely | Walks the counted pointer array, invokes the entry release virtual for non-null elements, and sets the count field at +0x20 to zero. |
| `00c1d2f0` | SoundGroup::Remove | likely | Finds matching IDs in the array, invokes each matching object's virtual release method, shifts later entries left, and decrements the count. |
| `00c1dac0` | SoundGroup::SetEntryAtIndex | likely | For a nonnegative index, increments the entry reference count, fills intervening array positions with null, stores the entry at that index and extends the high… |
| `00c1db30` | SoundGroup::SetEntryAtIndex | likely | If the supplied index is negative, replaces it with count at this+0x20; calls 007cd920 with the entry and chosen index, then calls 00c1dac0 with the same value… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0091e750` | ApplySoundVolume | confirmed | Reads a Sound-section setting via FUN_00919990 with the 1.0f default (0x3f800000) when this+0x6d8 is set, then passes the volume through FUN_00769710 and sets … |
| `00ca4b50` | InitializePirateBattleSoundEvents | confirmed | Looks up six MG_Pirate_Battle sound event names and creates/stores a shared sound-event object for each one. |
| `00a59810` | CreatePirateBattleVocalSoundEmitters | likely | Looks up the happy and sad vocal resources, increments each found resource reference count, allocates and constructs a 0x58-byte SoundEmitter for each found re… |
| `00b9d2d0` | CreateSoundEmitterForEvent | likely | Registers a sound emitter for an event object: increments the event's listener counter at event+8 and, on the 0-to-1 transition, calls the event's virtual slot… |
| `00c15cc0` | SoundLoadSucceeded | likely | Lua C function bound as SoundLoadSucceeded: requires exactly one argument, reads it as a string (FUN_00e39c30) and pushes the boolean result of FUN_00b06f60(st… |
| `00c15e60` | PauseSound | likely | Lua C function bound as PauseSound: with at least one numeric argument it reads the sound id (FUN_00e3a770 + FUN_00d83c90) and calls FUN_00b07040(id); returns … |
| `00c15eb0` | StopSound | likely | Lua C function bound as StopSound: with at least one numeric argument it reads the sound id and calls FUN_00b07000(id); returns 0. |
| `00c15f00` | ResumeSound | likely | Lua C function bound as ResumeSound: with at least one numeric argument it reads the sound id and calls FUN_00b07070(id); returns 0. |
| `00c160d0` | StopSoundEmitter | likely | Lua C function bound as StopSoundEmitter: with at least one numeric argument it reads the emitter id and calls FUN_00b08070(id, 0); returns 0. |
| `00c16120` | PauseSoundEmitter | likely | Lua C function bound as PauseSoundEmitter: with at least one numeric argument it reads the emitter id and calls FUN_00b07910(id); returns 0. |
| `00c16170` | ResumeSoundEmitter | likely | Lua C function bound as ResumeSoundEmitter: with at least one numeric argument it reads the emitter id and calls FUN_00b07950(id); returns 0. |
| `0090c8b0` | — | unknown | Checks sound state at owner+0x67010 and +0x67014, invokes a virtual method on the active sound object, advances a timer, then releases the sound allocation whe… |
| `00b62f80` | — | unknown | Increments the counter at this+8 and calls vtable slot 0x24 on this when it was zero; then allocates 0x58 bytes with FUN_00d833a6, and when the global game obj… |
| `00c15da0` | — | unknown | Lua C function bound as ChangeSoundVolume: with at least one argument it starts from the default volume constant _DAT_0175b420 and a 0.0 second value, override… |
| `00c15f50` | — | unknown | Lua C function bound as ChangeSoundEmitterPitch: with at least one argument it starts from the default pitch constant _DAT_0175b420 and 0.0, overrides them fro… |
| `00c16010` | — | unknown | Lua C function bound as ChangeSoundEmitterVolume: with at least one argument it starts from the default volume constant _DAT_0175b420 and 0.0, overrides them f… |

### `audio.ambient`

**AmbientRandomEffect** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f6c10` | AmbientRandomEffect::DtorBody | confirmed | Destructor body without free: fastcall, writes AmbientRandomEffect::vftable (0x0181110c) into [this] and returns. |
| `009f6cb0` | AmbientRandomEffect::ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot [0] of AmbientRandomEffect::vftable: stores AmbientRandomEffect::vftable into [this], then calls _free(this) through thunk 00d… |
| `009f74a0` | AmbientRandomEffect::CopyCtor | confirmed | Thiscall with one source pointer (ret 4). Installs AmbientRandomEffect::vftable into [this], then copies source fields field by field with no callee: dwords +0… |
| `009f75e0` | AmbientRandomEffect::ScalarDeletingDtor_Alt | confirmed | Scalar deleting destructor body of the same shape as slot [0] of AmbientRandomEffect::vftable: stores AmbientRandomEffect::vftable into [this], sets the EH sta… |
| `009f7640` | AmbientRandomEffect::DefaultCtor | confirmed | Fastcall default constructor: installs AmbientRandomEffect::vftable, zeroes most fields, loads global constants into +0x14, +0x18, +0x1c, +0x20, +0x3c, +0x50 a… |
| `009f6c20` | AmbientRandomEffect::ReadFloat50 | speculative | Virtual slot [1] of AmbientRandomEffect::vftable: returns the float at this+0x50 on the x87 stack (fld dword [ecx+0x50]; ret, 4 bytes). |
| `009f6c90` | AmbientRandomEffect::RaiseFloat50 | speculative | Virtual slot [2] of AmbientRandomEffect::vftable, thiscall with one float argument (ret 4): when the argument is greater than the float constant at DAT_01762a3… |
| `009f6f80` | AmbientRandomEffect::InitVtableOnly | speculative | Fastcall, SEH-framed: stores AmbientRandomEffect::vftable into [this] and restores the SEH chain. No other field is written and no callee is made. |

**(no class)** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f7c70` | ListRemoveAmbientRandomEffect | confirmed | SEH-framed thiscall with one stack argument (ret 4), this = the list in ECX. Unlinks the AmbientRandomEffect record passed in (its next pointer at +0x80 and pr… |
| `010be610` | AmbientTracksReadXml | confirmed | Stack-argument function (one argument: the ambient entry at param_1), no ECX use in the body. Reads XML through the TiXml helpers: for each AmbientSoundTrack e… |
| `009f8170` | AmbientRandomEffectsRebuild | likely | Thiscall with two stack arguments (collection A = param_2, context = param_3), SEH-framed. The initial gate reads a full dword from the pointer at this+0x30, o… |
| `009f8370` | Owner3Lists_DtorBody | likely | Fastcall teardown body of a composite owner that holds three inline SoeUtil lists: the BackgroundTrackManager* list at +0, the RoomTypeData list at +0x10 and t… |
| `010be880` | AmbientEntryDtorBody | likely | Fastcall destructor body with this in ECX. Stores the random-effect list vtable at +0x58 and clears that list through 010be530, stores the sound-track list vta… |
| `010bea10` | AmbientEntryCtorBody | likely | Fastcall constructor body with this in ECX. Zeroes +0x0. Installs SoeUtil::IString<char>::vftable at +0x4, +0x14, +0x24 and +0x34 with the empty buffer DAT_01c… |
| `010bec10` | AmbientEntryDeletingDtor | likely | Thiscall with one flag byte (ret 4). Calls 010be880 (the ambient entry dtor body) on this, frees this through the _free thunk when bit 0 of the flag is set, an… |
| `010beae0` | AmbientEntryDtorCall | speculative | Parameterless function that calls the ambient entry dtor body 010be880 once and returns. The register holding the receiver is not established in the shown code. |

### `audio.definition`

**ActorSoundEmitterDefinition** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e5eb0` | ActorSoundEmitterDefinition::DtorBody | confirmed | Fastcall (ECX = definition). Stores ActorSoundEmitterDefinition::vftable at +0, releases the reference-counted object at +0x14c (LOCK decrement of its count at… |
| `007e6010` | ActorSoundEmitterDefinition::ScalarDeletingDtor | confirmed | Thiscall (ECX = definition), one stack arg (flag byte). Calls the ActorSoundEmitterDefinition destructor body 007e5eb0, then calls _free on the object when fla… |
| `0078ed20` | ActorSoundEmitterDefinition::ReadProperties | likely | Virtual slot [7], the reader matching WriteProperties (slot [8]). Calls 007ca120 on its argument, then reads named values with 00d51810 (integer) and 00d517f0 … |
| `007e6080` | ActorSoundEmitterDefinition_ScalarDeletingDtorAlt | likely | Thiscall (ECX = definition, returns it), one stack arg (flag byte). Calls the ActorSoundEmitterDefinition destructor body 007e5eb0, then calls _free on the obj… |
| `007e5f40` | ActorSoundEmitterDefinition::VSlot6_Field14cPlus4Eq1 | speculative | Fastcall, virtual slot [6] of ActorSoundEmitterDefinition. Returns 1 when the dword at (the object at this+0x14c) + 4 equals 1, else 0. |
| `007e5f60` | ActorSoundEmitterDefinition::VSlot10_SetFlag1a0AndReset | speculative | Fastcall, virtual slot [10] of ActorSoundEmitterDefinition. Sets the byte at this+0x1a0 to 1, calls 007c75f0 on the holder at this+0x90, then tail-jumps to 007… |
| `007e5f90` | ActorSoundEmitterDefinition::VSlot1_HasPtr14c | speculative | Fastcall, virtual slot [1] of ActorSoundEmitterDefinition. Returns 1 when the pointer at this+0x14c is non-null, else 0. |
| `007e5fa0` | ActorSoundEmitterDefinition::VSlot2_HasPtr14cAndCountPositi… | speculative | Fastcall, virtual slot [2] of ActorSoundEmitterDefinition. Returns 1 when the pointer at this+0x14c is non-null and the dword at +0x14 of the object it points … |
| `007e5fc0` | ActorSoundEmitterDefinition::VSlot3_PredicateNotFlag1a0AndL… | speculative | Fastcall, virtual slot [3] of ActorSoundEmitterDefinition. Returns 1 when the pointer at this+0x14c is non-null, the byte at this+0x1a0 is zero, and 007c84e0 r… |
| `007e5ff0` | ActorSoundEmitterDefinition::VSlot4_Field164Field154 | speculative | Fastcall, virtual slot [4] of ActorSoundEmitterDefinition. Returns 0 when the dword at this+0x164 is non-zero and the dword at this+0x154 is not 3; otherwise r… |
| `0078e910` | — | unknown | Virtual slot [8]. Calls 007c9dd0 first (its result gates an EH-guarded block) and writes named properties through key/value helpers: integer keys (controlType … |

**ClientAreaAmbientSoundProperty** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cebd80` | ClientAreaAmbientSoundProperty::Enable | confirmed | Increments the referenced sound definition, sets the enabled byte, then runs the ambient playback start routine. |
| `00cebd20` | ClientAreaAmbientSoundProperty::Update | likely | When enabled, resolves the configured sound definition and starts or stops playback based on its state and client audio flag. |
| `00cebdc0` | — | unknown | Stops active track when the area audio system is available, clears the playing byte, and decrements the sound-definition reference. |

**ClientAreaCombatMusicProperty** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ced770` | ClientAreaCombatMusicProperty::WriteFields | confirmed | Writes base property fields and the SoundDefinitionId value. |
| `00ced7c0` | ClientAreaCombatMusicProperty::ReadSoundDefinitionId | confirmed | Reads SoundDefinitionId through the property reader and stores the resulting ID at object offset 0x38. |
| `00ced7f0` | ClientAreaCombatMusicProperty::Start | confirmed | Increments the referenced sound definition, resolves the global combat-music state, clears its active byte, stores this sound ID, and calls the music update ro… |
| `00ced860` | ClientAreaCombatMusicProperty::Stop | likely | Decrements the referenced sound-definition count, resolves the global combat-music state, clears its active byte, and zeroes the selected sound ID. |

**DeferredPlayListLoader** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e5e10` | DeferredPlayListLoader::DtorBody | confirmed | Fastcall (ECX = loader). Stores DeferredPlayListLoader::vftable at +0 and at +8, calls 007c75f0 with ECX = this, runs the ,4 pending-entry list destructor body… |
| `007e5e90` | DeferredPlayListLoader::ScalarDeletingDtor | confirmed | Thiscall (ECX = loader), one stack arg (flag byte). Calls the DeferredPlayListLoader destructor body 007e5e10, then calls _free on the object when flag bit 0 i… |
| `007c7760` | DeferredPlayListLoader::AdvanceUnflaggedEntry | likely | Virtual slot [5] (174 bytes). Calls 007cd920 with the two arguments and returns its result when the held reference at this+0xb8 is null. Otherwise walks the no… |

**SoeUtil::HashMap<int,SoundOneShotDefinitionLoader::LoadPendingOneShotEntry,32,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090c310` | HashMapSoundPendingOneShot::Cleanup | confirmed | Restores the pending-one-shot hash-map vtable and calls 008fa260 to release its stored map contents. |
| `0090f900` | SoundOneShotPendingMap::SoundOneShotPendingMap | confirmed | Installs the pending-one-shot hash-map vtable, clears the map size and bucket count, zeros its 32 bucket heads, and installs the matching hash vtable. |
| `00910530` | SoundOneShotPendingMap::ScalarDeletingDtor | confirmed | Restores the pending-entry map vtable, calls 008fa260 to clean up map contents, optionally frees the object, and returns it. |

**SoundControlDefinition** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008da880` | SoundControlDefinition::~SoundControlDefinition | confirmed | Destructor body: stores SoundControlDefinition::vftable (raw bytes c7 01 5c 2a 80 01) into the object's vtable slot, then calls 007e0b10, the base-class destru… |
| `008e8070` | SoundControlDefinition::ScalarDeletingDtor | confirmed | thiscall: ECX = object, one stack argument = flag byte. Stores SoundControlDefinition::vftable at [this], calls 007e0b10 (ActorEffectDefinition::DtorBody, conf… |
| `00928290` | SoundControlDefinition::Ctor | confirmed | Initializes the base definition, installs its RTTI vtable, and initializes an embedded member through 007bd400. |
| `008fbff0` | SoundControlDefinition::ListUnlinkRetype | likely | thiscall with one stack argument (the node, ret 4). Calls 008af070 with the node first (unlabelled). Unlinks the node from an intrusive list with next at node+… |
| `00928360` | SoundControlDefinition::InitWithKey | likely | Constructs a SoundControlDefinition and stores its key at object offset 0xd8. |

**SoundControlDefinitionManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00928060` | SoundControlDefinitionManager::Ctor | confirmed | Initializes base manager storage and installs the derived manager vtable into both subobjects. |
| `00928110` | SoundControlDefinitionManager::ScalarDeletingDtor | confirmed | Adjusts this by four bytes and jumps to the secondary-base deleting-destructor path. |
| `00928460` | SoundControlDefinitionManager::AddDefinition | confirmed | Copies the supplied key locally and delegates insertion to 009283e0. |
| `00928480` | SoundControlDefinitionManager::Dtor | confirmed | Restores manager RTTI vtables, then invokes inherited cleanup routine 00924fb0. |
| `009284d0` | SoundControlDefinitionManager::ScalarDeletingDtor | confirmed | Runs full manager cleanup and frees the object only when the deleting flag is set. |
| `009283e0` | CreateAndLinkSoundControlDefinition | likely | Allocates the definition through a virtual slot, initializes it on success, and links it into list and hash storage. |

**SoundDefinitionAssetLoader** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dfc60` | SoundDefinitionAssetLoader::DtorBody | confirmed | fastcall (this=ECX), SEH-framed, no args. Stores SoundDefinitionAssetLoader::vftable at [this] and at [this+8], then the AssetManagement AssetRequester vtable … |
| `007dfce0` | SoundDefinitionAssetLoader::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dfc60 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007e0410` | SoundDefinitionAssetLoader::SecondaryVSlot0_DeletingDtorThu… | likely | void, no args, 8 bytes: sub ecx,8 then jmp 007dfce0. Adjusts this by -8 to the primary object and tail-jumps to the scalar deleting destructor. |

**SoundDefinitionLoader** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dfc30` | SoundDefinitionLoader::DtorBody | confirmed | fastcall (this=ECX), no args, 7 bytes: stores SoundDefinitionLoader::vftable at [this] and returns. Frees nothing. |
| `007dfc40` | SoundDefinitionLoader::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4 (31 bytes). Stores SoundDefinitionLoader::vftable at [this], then if flag bit 0 is set calls _free on this (through … |

**SoundOneShotDefinitionLoader** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00915640` | ScalarDeletingDtor | confirmed | Virtual at vftable[0] of 01805074 (thiscall, one byte argument). Calls FUN_009155b0 (destructor body) with ECX=this; if (arg & 1) != 0 calls _free(this); retur… |
| `009155a0` | ThunkThisMinus8ToDeletingDtor | likely | Adjustor thunk (bytes 83 e9 08 e9 98 00 00 00): subtracts 8 from ECX and jumps to FUN_00915640. It is vftable[0] of the secondary vtable 01805060 of SoundOneSh… |
| `009155b0` | DtorBody | likely | __fastcall (this=ECX). Under an SEH frame: restores the SoundOneShotDefinitionLoader vtables (01805074, 01805060, 01805054) and calls three cleanup helpers FUN… |
| `00915510` | — | unknown | __fastcall-style thiscall (this=arg1, arg2 on stack; returns this). Constructs its bases and members: calls FUN_008a2530 (SoundDefinitionAssetLoader constructo… |

**(no class)** (16)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078f2b0` | LoadSoundEffectDefinitions | likely | Walks the linked list at this+0x34 (next pointer at node+0x1b0). For each node it calls the vtable slot +8 load routine; when that returns false it appends "Ca… |
| `008e7e30` | ScalarDeletingDtor_008e7e30 | likely | thiscall: ECX = object, one stack argument = flag byte. Calls 008cfa90 (confirmed label ActorLightEmitterDefinition destructor body) on the object, frees the o… |
| `008e7e90` | ScalarDeletingDtor_008e7e90 | likely | thiscall: ECX = object, one stack argument = flag byte. Calls 008cfba0 (confirmed label ActorCameraEffectDefinition destructor body) on the object, frees the o… |
| `008e7ef0` | ScalarDeletingDtor_008e7ef0 | likely | thiscall: ECX = object, one stack argument = flag byte. Calls 008cfcb0 (confirmed label ActorRumbleEffectDefinition destructor body) on the object, frees the o… |
| `008e7f50` | ScalarDeletingDtor_008e7f50 | likely | thiscall: ECX = object, one stack argument = flag byte. Calls 008cfdb0 (confirmed label ActorLightsaberTrailDefinition destructor body) on the object, frees th… |
| `00cebc60` | SetAmbientSoundDefinition | likely | Releases the prior sound-definition reference, increments the new definition reference, starts the new sound if eligible, and stores the new identifier. |
| `007dfd00` | BinXmlReadSoundFlagsTags1And2 | speculative | thiscall (ECX = pointer to the owner pointer), one stack arg (the BinXML reader). Calls 00f9f0b0 on the reader, then loops until 007df5e0 returns non-zero (end… |
| `007dfdc0` | BinXmlReadSoundFlagTag1 | speculative | thiscall (ECX = pointer to the owner pointer), one stack arg (the BinXML reader). Like 007dfd00 but only tag 1 is handled: a tag byte read by 00f9f4a0 equal to… |
| `007e5e80` | Thunk_Minus8_To007e5e90 | speculative | Adjustor thunk: subtracts 8 from ECX and jumps to 007e5e90 (DeferredPlayListLoader scalar deleting destructor). |
| `007e6030` | EhCleanup_CallsDtorBody007e5eb0 | speculative | No arguments. Exception-handling cleanup body: sets the frame state to -1 and calls the ActorSoundEmitterDefinition destructor body 007e5eb0 on the object whos… |
| `007e60e0` | DefinitionMapEraseAndDestroy | speculative | Thiscall (ECX = map object: list head at +8, tail at +0xc, count at +0x10), one stack arg = node. Removes the node from its hash bucket through 007e0380, unlin… |
| `00928310` | LinkDefinitionAfterTail | speculative | Links node after tail, updates empty-list head, inserts into the selected hash bucket, and increments count. |
| `00ceba60` | AmbientSoundStartIfEligible | speculative | Checks configured track, completion state, timing gate, area audio state, and sound definition; invokes the sound start routine and marks the property active w… |
| `00cebba0` | AmbientSoundStopIfEligible | speculative | Checks whether this active sound should remain, calls the sound-stop routine with track and definition identifiers, then clears the active flag. |
| `007e6580` | — | unknown | Thiscall (ECX = definition object, one stack arg = reader). Finds or creates an entry by name through 00709a70 and 007e0ca0, copies the record's dword and floa… |
| `00c1c5a0` | — | unknown | For more than one argument, resolves a GUI name, finds or creates a 0x2c-byte object using SoundGroup construction, stores a client field, registers it, increm… |

### `audio.derby`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009ec910` | SoundEvent_CreateInstance | likely | Creates a 0x58-byte event instance: allocates it with FUN_00d833a6, builds it through FUN_0078fea0 using four dwords of the definition object param_1 (+0x14c, … |

### `audio.fishing`

**FishingProcessor** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b65d90` | FishingProcessor::LoadMusicTracks | confirmed | Calls FUN_007699f0, then for each of the nine names MG_Fishing_Cast_Invalid, MG_Fishing_Cast_Perfect, MG_Fishing_HookFish, MG_Fishing_Lure_Tease, MG_Fishing_Ho… |
| `00b614a0` | FishingProcessor::AllSoundTracksReady | likely | Returns 1 only when all nine track objects at +0x320, +0x328, +0x330, +0x338, +0x340, +0x348, +0x350, +0x358 and +0x360 report non-zero from their vtable slot … |
| `00b61580` | FishingProcessor::SaveAudioState | likely | Reads three values from virtual getters (slots 0x1c, 0x8 and 0x30) of the mixer object at *(this+0x14)+0x8a0 and stores them into +0x36c, +0x36d and +0x370, th… |
| `00b61640` | FishingProcessor::RestoreAudioState | likely | Restores the mixer object at *(this+0x14)+0x8a0 from the three fields FUN_00b61580 saved: slot 0x18 = byte +0x36c, slot 0xc = byte +0x36d, slot 0x34 = dword +0… |
| `00b65040` | FishingProcessor::UpdateTrackPositions | likely | Resolves the local player's record with FUN_00b64300 (when its +0xd4 field is set it also calls FUN_00cd3cd0 and builds a position vector with FUN_00b640e0), t… |
| `00b615e0` | FishingProcessor::SetAudioStateUnderwater | speculative | Drives the mixer object at *(this+0x14)+0x8a0 with hard-coded values: slot 0x18 = 0, slot 0xc = 0, slot 0x34 = 0x10 and slot 0x50 = 0x10, then calls slot 0x58 … |
| `00b61290` | — | unknown | Walks the nine slot pairs at +0x320/+0x324, +0x328/+0x32c, ... +0x360/+0x364 of the processing object: for each non-zero slot it calls the deleting destructor … |

### `audio.listener`

**SoundListenerEntity** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089f020` | SoundListenerEntity::SoundListenerEntity | confirmed | Object constructor: first runs the Deep::Entity-derived base constructor FUN_007648d0 on this (it writes Deep::Entity::vftable, clears the base fields and appl… |
| `0089f0b0` | SoundListenerEntity::~SoundListenerEntity | confirmed | Destructor body: writes SoundListenerEntity::vftable into the object and then calls the Deep::Entity-derived base destructor FUN_00764e10 on it, which republis… |
| `0089f110` | SoundListenerEntity::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor: writes SoundListenerEntity::vftable, runs the base destructor FUN_00764e10 (reference-count release and handle cleanup) and th… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007adb10` | AudioSetListener3D | likely | Thiscall with three pointers: copies the four dwords at the second argument into this+0x70..0x7c, and when the driver handle at this+0x6c is non-zero calls _AI… |

### `audio.minigame`

**ClientTradingCardGameManager** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad4e80` | ClientTradingCardGameManager::ApplyMusicLevels | confirmed | When global audio setting +0x6d8 is active, sends two values to Sound/Minigames and Sound/MiniGameMusic controls; always writes the values to settings 0xb and … |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ad3f30` | SetMusicLevelB | speculative | Clamps the supplied float against DAT_0175b420 and zero, then writes setting index 0xb through 007bf6b0. |
| `00ad3f80` | SetMusicLevelC | speculative | Clamps the supplied float against DAT_0175b420 and zero, then writes setting index 0xc through 007bf6b0. |

### `audio.music`

**BackgroundMusicTrackManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009f6b40` | BackgroundMusicTrackManager::BackgroundMusicTrackManager | confirmed | Thiscall with one stack argument (ret 4), SEH-framed. Calls 00c67ad0 on this with the argument, installs BackgroundMusicTrackManager::vftable into [this], zero… |
| `009f7060` | BackgroundMusicTrackManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor, slot [0] of BackgroundMusicTrackManager::vftable: installs an SEH frame, calls 00c67b70 with this in ECX and no arguments (bytes ch… |
| `00c68020` | — | unknown | Vftable slot [1] of 018110fc BackgroundMusicTrackManager::vftable. Calls 00c67cb0 first, then takes the first entry of the list at this+8 (head at +0xc), and r… |

**BackgroundTrackManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c67ad0` | BackgroundTrackManager::Ctor | confirmed | Constructor with one stack arg (ret 4). Installs vtable 01833378 at this, installs the List<BackgroundTrack*,-1> vtable 01833364 at this+8, +0x18 and +0x28, ze… |
| `00c67b70` | BackgroundTrackManager::DtorBody | likely | Destructor body. Installs 01833378 at this and walks the list at this+0x1c: for each entry it releases the sub-object at +0x10 (vtable+0xc and vtable+0 with ar… |
| `00c67cb0` | BackgroundTrackManager::Slot1Update | speculative | Vftable slot [1] of 01833378 BackgroundTrackManager::vftable. Calls 0078fcc0, 00c67020, 00c67270 (the flag predicate) and 00c67a20 on the entries of the track … |

**SoeUtil::List<BackgroundTrack*,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c67860` | ListBackgroundTrackScalarDeletingDtor | confirmed | Scalar deleting destructor in vftable slot [0] of 01833364: installs vtable 01833364, calls 00c67350 to clear the nodes, tests bit 0 of the flag and frees this… |
| `00c67800` | ListBackgroundTrackCopyCtor | likely | Copy constructor with one stack arg (source list, ret 4). Installs vtable 01833364, zeroes the count and the head and tail, and when the source is not this it … |
| `00c67850` | ListBackgroundTrackDtorBody | likely | Destructor body (no free): installs vtable 01833364 and jumps to 00c67350 to clear the list nodes (tail call, ret from that routine). |

**(no class)** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c677a0` | ListAnyEntryFlagMatch | speculative | Thiscall (ret 0). Returns 1 when the object at [this+0xc]+0 has a sub-object at +0x10 with byte +0x4c nonzero and dword +0x20 nonzero. Otherwise it walks the l… |
| `00c67890` | LookupConfigAndInstantiate | speculative | Fastcall (this in ECX, no stack args). Calls 00769710 and 007699f0, looks up a key with 00709a70 (256-bucket hash), stores a global dword at this+0xc, and when… |
| `00c67a20` | TrackStateUpdateFromConfig | speculative | Thiscall with one float stack arg (ret 4). Its callees are 00769710, 0078f560, 0078fb70, 007bd510, 008b80a0, 00beb020 and 00c67890: the float is forwarded to 0… |
| `00c680c0` | BackgroundTrackScalarDeletingDtor | speculative | Scalar deleting destructor shape: calls 00c67b70 (BackgroundTrackManager destructor body) on this, tests bit 0 of the flag and frees this through the _free thu… |
| `00c680e0` | TrackListFindOrCreateInsert | speculative | Thiscall with stack args (ret 0xc). Finds an entry with 00c676b0, allocates a new track with 0xd833a6 when none exists, instantiates its config with 00c67890, … |
| `00c68210` | BackgroundTrackManagerRebuildLists | speculative | Thiscall with one stack arg (ret 4). Calls 00c67f30 to collect matching entries, allocates an object with 00d833a6, instantiates a config through 00c67890, app… |
| `00c68340` | ListForEachCallRemoveTail | speculative | Fastcall (this in ECX). When the head at this+0x34 is non-null it walks the list at this+0x28: for each node it calls 00c680e0 with the node's value and the co… |
| `00c683c0` | ListForEachCallWithZero | speculative | Fastcall (this in ECX, ret 4). Walks the list whose head is at this+0x10 (next at node+8) to the first node whose dword +4 is -1 and unlinks it from the list a… |
| `00c67730` | — | unknown | Thiscall with one float stack arg (ret 4). Walks the list at this+0x1c. For each node whose object has byte +0x14 equal to 1 and a non-null sub-object at +0x10… |
| `00c67e90` | — | unknown | Thiscall with one stack arg block (ret 0x10). Looks up a track by key pair with 00c67640 in the list at this+8; when none is found it pushes a new entry at the… |
| `00c67f30` | — | unknown | Thiscall with stack args (ret 4). Walks entries and appends matching tracks through 00c67410 when the entry's byte +0x14 is 1 and the sub-object at +0x10 has b… |
| `00c68450` | — | unknown | Thiscall with three stack args (ret 0xc). Finds an entry by key pair with 00c67640; when found it forwards the float argument to the entry's sub-object (007bd5… |

### `audio.service`

**Audio::AudioService** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9aec0` | AudioService::ScalarDeletingDtor | confirmed | Virtual slot 0 of Audio::AudioService: stores the vtable at +0, calls 0123a4c0 (CloneWarsAudioService destructor body), then frees this through _free when bit … |
| `0123a580` | Audio::AudioService::SetParamOrDefault | speculative | Virtual slot [3] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the argument is outside the range (0,15000) or negative it rep… |
| `0123a600` | Audio::AudioService::StopHandlesMatchingKey | speculative | Virtual slot [4] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: when the sub-object at +0x34 is present it walks the list at +0x38/… |
| `0123a770` | Audio::AudioService::StopAllHandles | speculative | Virtual slot [6] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls its own slot [5], then when the sub-object at +0x34 is presen… |
| `0123a8a0` | Audio::AudioService::GetNameString | speculative | Virtual slot [8] of Audio::AudioService::vftable and Audio::CloneWarsAudioService::vftable: calls FUN_0123aaa0 to locate a list position and compares it with t… |
| `00f9ae10` | — | unknown | Fastcall constructor: calls 0123a340 (Audio::CloneWarsAudioService::Ctor) with an EH cookie, then stores Audio::AudioService::vftable at +0 and returns this. |
| `00f9ae70` | — | unknown | Stores Audio::AudioService::vftable at +0, then calls 0123a4c0 (Audio::CloneWarsAudioService destructor body) with an EH cookie; does not free this. |

**Audio::AudioServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f8840` | Audio::AudioServiceBase::DtorBody | confirmed | Stores Audio::AudioServiceBase::vftable into *this, sets the local unwind state to -1, then calls 01230d30 for base teardown. Returns no value. |
| `012f88c0` | Audio::AudioServiceBase::ScalarDeletingDtor | confirmed | Stores Audio::AudioServiceBase::vftable, calls 01230d30, then calls _free(this) only when bit 0 of its byte argument is set; returns this. |
| `012f87e0` | — | unknown | Calls 01230d10 with the cookie-initialised frame, then stores Audio::AudioServiceBase::vftable into *this and returns this. |

**Audio::CloneWarsAudioService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123a340` | Audio::CloneWarsAudioService::Ctor | confirmed | Stores Audio::CloneWarsAudioService::vftable into the object, assigns the string at +8 to 'MX_SD_Loop_1', calls FUN_0041ba81(0,0), FUN_0123ac60 and FUN_0123ad5… |
| `0123a4c0` | Audio::CloneWarsAudioService::DtorBody | confirmed | Stores Audio::CloneWarsAudioService::vftable into the object, then calls FUN_00508490, FUN_00505ffd, FUN_0123ac90 and FUN_0041ba81(1,0) in sequence, releasing … |
| `0123aa00` | Audio::CloneWarsAudioService::ScalarDeletingDtor | confirmed | Calls the destructor body FUN_0123a4c0 and, when bit 0 of the flag argument is set, frees the object with _free; returns the object pointer. |

**SoeUtil::HashList<SoundCategory,256,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b4120` | HashListSoundCategory256::Ctor | confirmed | Fastcall constructor: stores the HashListMap<int,SoundCategory,256,-1> vftable, zeroes this+0x10, +8 and +0xc, memsets 0x400 bytes at this+0x14 (256 bucket hea… |
| `007b41a0` | HashListSoundCategory256::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the destructor body 007b3e40 (256-bucket SoundCategory map), then calls _free(this) when bit 0 of the flag is set; returns thi… |
| `007b4380` | HashListSoundCategory256::CtorWrapper | likely | Fastcall inside an SEH frame (handler LAB_0155e7c8): calls the constructor 007b4120 on ECX = this, restores the previous exception list and returns the same po… |

**SoeUtil::HashList<SoundCategoryData,16,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b35c0` | HashListSoundCategoryData16::Ctor | confirmed | Fastcall constructor inside an SEH frame (handler LAB_0155e428): stores the 16-bucket map vftable, zeroes this+0x10, +8 and +0xc, memsets 0x40 bytes at this+0x… |
| `007b3640` | HashListSoundCategoryData16::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the destructor body 007b3550 (16-bucket SoundCategoryData map), then calls _free(this) when bit 0 of the flag is set; returns … |
| `007b3780` | HashListSoundCategoryData16::CtorWrapper | likely | Fastcall inside an SEH frame (handler LAB_0155e4c8): calls the constructor 007b35c0 on ECX = this, restores the previous exception list and returns the same po… |

**SoeUtil::HashListMap<int,SoundCategory,256,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b3dd0` | HashListMapSoundCategory256::DtorBody | confirmed | Fastcall destructor body (two EH unwind callers): stores the HashListMap<int,SoundCategory,256,-1> vftable at this, then calls 007b39a0 on the list head at thi… |
| `007b3e40` | HashListMapSoundCategory256::DtorBodySeh | confirmed | Fastcall SEH-framed destructor body (handler LAB_0155e628): stores the 256-bucket SoundCategory map vftable at this, then calls 007b39a0 on the list head at th… |
| `007b3db0` | HashListMapSoundCategory256::ClearAll | likely | Fastcall: while the head pointer at this+8 is non-null, calls 007b39a0 (remove and destroy that node); no return value. |
| `007b41c0` | HashListMapSoundCategory256::DtorBodySehWrapper | likely | Void(void) with ECX = this, inside an SEH frame (handler LAB_0155e6e8): calls the 256-bucket SoundCategory map destructor body 007b3e40 on this, then restores … |

**SoeUtil::HashListMap<int,SoundCategoryData,16,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b3490` | HashListMapSoundCategoryData16::DtorBody | confirmed | Fastcall destructor body (two EH unwind callers): stores the HashListMap<int,SoundCategoryData,16,-1> vftable at this, then calls 007b2360 on the list head at … |
| `007b3550` | HashListMapSoundCategoryData16::DtorBodySeh | confirmed | Fastcall SEH-framed destructor body (handler LAB_0155e3f8): stores the 16-bucket SoundCategoryData map vftable at this, then calls 007b2360 on the list head at… |
| `007b2360` | HashListMapSoundCategoryData16::RemoveAndDestroyNode | likely | Thiscall with one node argument: indexes the 16-bucket array with (node+0x34)&0xf, unlinks the node from its bucket chain (chain link at node+0x38) and from th… |
| `007b31e0` | HashListMapSoundCategoryData16::EmplaceWithKey | likely | Thiscall with three stack arguments inside an SEH frame: allocates a node through the container vtable slot +4; when that returns non-null, constructs the Soun… |
| `007b3290` | HashListMapSoundCategoryData16::ClearAll | likely | Fastcall: while the head pointer at this+8 is non-null, calls 007b2360 (16-bucket removal and destruction of that node); no return value. |
| `007b36d0` | HashListMapSoundCategoryData16::DtorBodySehWrapper | likely | Void(void) with ECX = this, inside an SEH frame (handler LAB_0155e458): calls the 16-bucket SoundCategoryData map destructor body 007b3550 on this, then restor… |

**SoeUtil::List<SoundCategoryData,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b3660` | ListSoundCategoryData::DtorBody | confirmed | Fastcall SEH-less destructor body (two EH unwind callers): stores the List<SoundCategoryData,-1> vftable at this, then calls 007b2560 on the head at this+4 unt… |
| `007b2560` | ListSoundCategoryData::RemoveAndDestroyNode | likely | Thiscall with one node argument: unlinks the node from the list (head at this+4, tail at this+8, count at this+0xc; node next at +0x2c, prev at +0x30), calls t… |
| `007b3140` | ListSoundCategoryData::EmplaceBack | likely | Thiscall with two stack arguments (int, C string) inside an SEH frame: allocates a node through the container vtable slot +8; when that returns non-null, const… |
| `007b3500` | ListSoundCategoryData::ClearAll | likely | Fastcall: while the head pointer at this+4 is non-null, calls 007b2560 (list unlink and destruction of that node); no return value. |

**SoundCategory** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b3820` | SoundCategory::Ctor | confirmed | Thiscall with three stack arguments (int id, C string name, int extra) inside an SEH frame: calls 007b1c60 (SoundCategoryData base constructor) with the id and… |
| `007b38a0` | SoundCategory::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the SoundCategory destructor body 007b3720, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `007b3720` | SoundCategory::DtorBodySeh | likely | Thiscall with ECX = this and no stack arguments, inside an SEH frame (handler LAB_0155e49b): calls the 16-bucket SoundCategoryData map destructor body 007b3550… |
| `007b37d0` | SoundCategory::DtorSehWrapper | likely | Void(void) with ECX = this, inside an SEH frame (handler LAB_0155e4f8): calls the SoundCategory dtor body 007b3720 on this, then restores the previous exceptio… |
| `007b38c0` | SoundCategory::ScalarDeletingDtorSeh | likely | Thiscall with a flag byte inside an SEH frame (handler LAB_0155e568): calls the SoundCategory destructor body 007b3720 on this, calls _free(this) when bit 0 of… |
| `007b3920` | SoundCategory::CtorWithKey | likely | Thiscall with four stack pointer arguments (SEH frame, handler LAB_0155e598): calls 007b3820 with the values at *param_3, *param_4 and *param_5, stores the int… |

**SoundCategoryData** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b1c60` | SoundCategoryData::Ctor | confirmed | Thiscall with two stack arguments (int, C string): stores SoundCategoryData::vftable, the int at +4, the IString vftable at +8 with the static empty string at … |
| `007b13a0` | SoundCategoryData::DtorSehWrapper | likely | Thiscall with no stack arguments: installs a SEH frame (handler LAB_0155def8), passes its ECX object to 007b0860 (the SoundCategoryData dtor body), then restor… |
| `007b14c0` | SoundCategoryData::DtorSehWrapperB | likely | Thiscall with no stack arguments: installs a SEH frame (handler LAB_0155df28), passes ECX (this) to 007b0860 (the SoundCategoryData dtor body), then restores t… |
| `007b1ad0` | SoundCategoryData::ScalarDeletingDtor | likely | Thiscall with a flag byte: an SEH frame wraps the call to 007b0860 (SoundCategoryData dtor body) on this; then calls _free(this) when bit 0 of the flag is set;… |
| `007b1c00` | SoundCategoryData::ScalarDeletingDtorB | likely | Thiscall with a flag byte: an SEH frame (handler LAB_0155e018) wraps the call to 007b0860 (SoundCategoryData dtor body) on this; then calls _free(this) when bi… |
| `007b2480` | SoundCategoryData::CtorFromRefArgs | likely | Thiscall with two stack pointer arguments inside an SEH frame (handler LAB_0155e1c8): calls 007b1c60 with the int at *param_2 and the string pointer at *param_… |
| `007b24e0` | SoundCategoryData::CtorWithKey | likely | Thiscall with three stack pointer arguments inside an SEH frame (handler LAB_0155e1f8): calls 007b1c60 with the int at *param_3 and the string at *param_4, sto… |

**SoundControl** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007affc0` | SoundControlCtor | confirmed | Constructor body: calls 0071b950 (RefCounted base construction), stores SoundControl::vftable at this[0], calls 007bd400, and returns this. |

**SoundManager** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b4210` | SoundManager::DtorBody | confirmed | Fastcall SEH-framed destructor body of the sound manager: stores SoundManager::vftable at this, frees the block at this+8 through 007b0a20 and _free when non-n… |
| `007b43d0` | SoundManager::Ctor | confirmed | Thiscall with three stack arguments (the three driver option levels) inside an SEH frame: stores SoundManager::vftable, clears the byte at this+4, sets the Sou… |
| `007b4630` | SoundManager::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the SoundManager destructor body 007b4210, then calls _free(this) when bit 0 of the flag is set; returns this. |
| `007b15b0` | SoundManager::Update | likely | Thiscall with a float frame delta: returns at once when the enabled byte at this+4 is zero; otherwise advances the volume ramp fields at this+0x80..+0x94, push… |
| `007ae190` | SoundManagerSlot4_NodeFindNonNegative | speculative | Virtual slot [4] of SoundManager: calls 007bef70 (list find by key, one stack argument) and returns true when a node is found and the dword at **(node+4) is gr… |
| `007ae1b0` | — | unknown | Virtual slot [3] of SoundManager: when the pointer at this+4 is non-null it returns whether the dword at **(this+4) is greater than -1; otherwise false. |

**SoundPlayer** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b098a0` | SoundPlayer::Constructor | likely | Installs SoundPlayer vtable; initializes emitter list, DefinitionEntry lists, integer array and enabled flag. |
| `00b09930` | SoundPlayer::Cleanup | likely | Resets the SoundPlayer vtable and cleans its integer array, DefinitionEntry container and EmitterEntry list. |
| `00b099b0` | SoundPlayer::ScalarDeletingDtor | likely | Calls SoundPlayer cleanup and frees this only when delete flag bit 0 is set. |
| `00b099d0` | UpdateSoundCompletions | likely | Updates each emitter, removes finished entries, accumulates completion ids, dispatches HUD:OnSoundComplete and clears the completion count. |

**(no class)** (43)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b01d0` | SoundMasterCategoriesSave | confirmed | Writes the master sound categories: opens a masterCategories block through 00d52e70, then for each node of the chain at this+0x4c8 (next at +0x2c) opens a mast… |
| `007b0bc0` | SoundMixVolumesSave | confirmed | Writes the sound mix volumes through the settings writer: reads the global object (00769710) field at +0x28 and, when it is not the -0xb0 sentinel, writes mast… |
| `007b1ea0` | SoundDriverLoadOptions | confirmed | Fastcall on the driver object (ECX): when the soundDriver section lookup succeeds, reads channelOptions, sampleOptions and reverbOptions, each with the keys lo… |
| `007b3310` | SoundMasterCategoriesLoad | confirmed | Reads the masterCategories section through 00d50af0, then for each masterCategory entry (00d50af0 first, 00d50b60 to advance) reads the int value (DAT_017eda64… |
| `007b3b70` | SoundCategoriesLoad | confirmed | Thiscall with one stack argument: reads the categories section; stores masterMixVolume at this+0xac (kept only when in (0,1], else 1.0 from 0175b420); then for… |
| `007b3eb0` | SoundSettingsLoad | confirmed | Thiscall with a config object argument: opens the settings through 00fbf4a0 into a local IString; when the flag at the local result is zero and the soundSettin… |
| `009157e0` | AppServicesInitSoundSettings | confirmed | __fastcall (this=ECX). Calls FUN_006febe0("Libraries","SoundSettingsPath",".\\SoundSettings.xml",buf512,0x200,1,-1,-1) to read the path, then virtual [0xf8] of… |
| `007ada70` | AudioDriverClose | likely | If the driver handle at this+0x6c is non-zero it closes it with _AIL_close_digital_driver_4, clears the handle and tail-calls _AIL_shutdown_0; otherwise it ret… |
| `007ade40` | AudioDriverConfigQuery | likely | Thiscall with three output pointers: when the owner flag at this+0x98 and the driver handle at this+0x6c are both non-zero it calls _AIL_digital_configuration_… |
| `007ae040` | AudioGetRoomType | likely | Fastcall: reads the driver handle at (this+8)+0x6c; when it is non-zero it returns _AIL_room_type_4 of that handle, otherwise returns 0. |
| `007ae060` | AudioSetRoomType | likely | Thiscall with one argument: when the driver handle at (this+8)+0x6c is non-zero it calls _AIL_set_room_type_8 with the argument; otherwise it returns. |
| `007ae080` | AudioDriverConfigQueryOwner | likely | Loads ECX from this+8 and jumps to 007ade40 (AudioDriverConfigQuery) (bytes 8b 49 08; e9 b8 fd ff ff). |
| `007af110` | SoundDriverStartup | likely | Fastcall with the settings object in ECX: maps three settings indices through tables at +0x10, +0x30 and +0x50 into this[0], this[1] and this[2]; sets the Mile… |
| `007b02b0` | SoundManagerDrainLists | likely | Fastcall under a guard (enter 00fc45f0, leave 00fc4610 unless the ECX sentinel is -0x8510): empties the list at this+0x24 (head, tail at +0x28, count at +0x2c)… |
| `007b0ba0` | SoundManagerPartDeleteWrapper | likely | Calls 007b0a20 (SoundManager part teardown) on this, then frees this with _free when bit 0 of the argument byte is set; returns this. |
| `007b0dd0` | SoundManagerShutdown | likely | Fastcall: calls 007b02b0 (SoundManagerDrainLists) on this, then, when the driver handle at (this+8)+0x6c is non-zero, closes it with _AIL_close_digital_driver_… |
| `007b39a0` | SoundCategoryMapRemoveAndDestroy | likely | Thiscall with one node argument inside an SEH frame (handler LAB_0155e5c8): calls 007ae5a0 with the node as its argument, unlinks it from the list at this+8/+0… |
| `007b3a50` | SoundCategoryMapEmplaceWithKey | likely | Thiscall with four stack arguments inside an SEH frame (handler LAB_0155e601): allocates a node through the owner vtable slot +4; when that returns non-null, b… |
| `007b3b10` | SoundCategoryMapClearAll | likely | Fastcall: while the head pointer at this+8 is non-null, calls 007b39a0 (remove and destroy that node); no return value. |
| `007b4010` | SoundCategoriesReload | likely | Thiscall with a config object argument: opens the settings through 00fbf4a0 into a local IString; when the flag at the local result is zero, repeatedly removes… |
| `00f9d220` | GetOrCreateAudioService | likely | Fastcall registry accessor identical in structure to 00f9ce10 but keyed by DAT_01cbd704: creates Audio::AudioService through 00f9ae10 on first use, registers i… |
| `007bd590` | FieldGet_Dword8 | speculative | 4 bytes: `8b4108 c3` = returns the dword at this+8. |
| `007af290` | — | unknown | Writes the sound settings through a settings writer: the soundDriver value (+0x84) through 00d518f0, three option blocks taken from the pointers at +0x10, +0x3… |
| `007b0a20` | — | unknown | Destructor body of a SoundManager part: frees the block at this+0xa0 with _free, stores the IString<char> vtable at this+0x80 and releases the string buffer at… |
| `007b1d10` | — | unknown | Thiscall with three int arguments: stores each clamped to the range 0..2 at this+0, +4 and +8; for each of the three counters at this+0x14, +0x34 and +0x54 tha… |
| `00b06ec0` | — | unknown | Calls a base initializer with four supplied values and stores the final argument at object offset 0x58. |
| `00b06f40` | — | unknown | Checks the global audio manager; if an object exists at offset 0x24, calls its virtual method, otherwise returns success. |
| `00b06f60` | — | unknown | Gets the audio manager and returns whether its object at offset 0x24 accepts the supplied sound identifier through a virtual method. |
| `00b06f90` | — | unknown | Finds a sound by id, sets target volume, and either stores interpolation rate/duration or applies volume immediately. |
| `00b07000` | — | unknown | Finds a sound by id, calls the stop/removal operation, and reports success only when manager and sound exist. |
| `00b07040` | — | unknown | Finds a sound by id, sets its paused byte, and reports whether the sound exists. |
| `00b07070` | — | unknown | Finds a sound by id, resets playback flags and timing fields, and begins a volume ramp toward the stored default. |
| `00b071f0` | — | unknown | Calls FUN_0078ff90 with a stack-derived cookie, then restores the prior exception-list pointer. |
| `00b073a0` | — | unknown | Initializes an object using four dereferenced constructor arguments through the shared base initializer, then returns this. |
| `00b077c0` | — | unknown | Calls FUN_0078ff90 with a stack-derived cookie, then restores the prior exception-list pointer. |
| `00b07890` | — | unknown | Finds the emitter by identifier and delegates volume update to the shared sound routine. |
| `00b078d0` | — | unknown | Walks the emitter list by id and delegates pitch update when a matching node exists. |
| `00b07910` | — | unknown | Finds an emitter by id, checks its active field, pauses it through a helper, and returns success/failure. |
| `00b07950` | — | unknown | Finds an emitter by id, resumes it through a helper, and returns success/failure. |
| `00b07980` | — | unknown | Searches emitters by id and returns whether the matching node?s active field is nonzero. |
| `00b079b0` | — | unknown | Unlinks an emitter node from the doubly linked list, decrements count, releases its reference, then calls the list node free slot. |
| `00b08070` | — | unknown | Searches emitter nodes by id; stops an active node or removes the matching node from associated and primary lists. |
| `0123ac60` | — | unknown | Builds three byte-sized local flags with their addresses, calls FUN_0123afb0 with them and the this value, returns the this pointer and pops two stack argument… |

### `audio.soccer`

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b77d70` | LoadSoccerSounds | confirmed | Resolves five named sound definitions with FUN_0070aab0 (MG_Soccer_AMB_Crowd_Goal, MG_Soccer_AMB_Crowd_OppGoal, MG_Soccer_Whistle_Long, MG_Soccer_30secondsLeft… |
| `00b77e30` | NewAudioBoundObject68 | speculative | Allocates 0x68 bytes with FUN_00d833a6 and, when the allocation succeeds, runs FUN_0071bed0 with (*(this+4), the audio context word at FUN_00769710()+0x34, 1) … |
| `00b75460` | — | unknown | Bumps the owner's live-count at +8 (calling owner vtable slot +0x24 when it was zero), allocates 0x58 bytes with FUN_00d833a6, constructs the handle with FUN_0… |

### `audio.sound`

**ActorSoundEmitter** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776160` | ActorSoundEmitterCtor | confirmed | Thiscall constructor (ECX = object, one stack arg). Calls the base constructor 00775f70 with ECX = object and the three stack arguments passed on, stores Actor… |
| `007762a0` | ActorSoundEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007bccd0 on the object, frees it through _free when flag bit 0 is set,… |
| `007bcba0` | ActorSoundEmitter_vt1_EnsureEntity | likely | Virtual slot 1 of ActorSoundEmitter: when this+0xac is empty it resolves a scene object through 007cfe50, allocates 0x80 bytes and constructs a SoundEntity (00… |
| `00776210` | ActorSoundEmitter::StateReadyCheck | speculative | Thiscall virtual slot [6] (ECX = object, no stack args) returning a dword 1 or 0. Returns 1 when the byte at +0xb4 is non-zero. Otherwise returns 1 when the po… |
| `00776250` | ActorSoundEmitter::SubObjectFlag4CCheck | speculative | Thiscall virtual slot [3] (ECX = object, no stack args). Returns 1 when the pointer at +0xac is non-null and the byte at (+0xac object)+0x4c is non-zero, other… |
| `00776270` | ActorSoundEmitter::SubObjectSlot10Check | speculative | Thiscall virtual slot [5] (ECX = object, no stack args). Returns 1 when the pointer at +0xac is null. Otherwise it calls the sub-object's vtable slot +0x10 and… |
| `007762e0` | ActorSoundEmitter::SubObjectRefCheck | speculative | Thiscall virtual slot [4] (ECX = object). Bytes 56 8b f1 83 be ac 00 00 00 00 74 1b 8b 06 8b 50 0c ff d2 84 c0 74 10 8b 86 ac 00 00 00 83 78 20 00 74 04 33 c0 … |
| `007bccd0` | — | unknown | Destructor body (ECX = this, SEH framed): stores ActorSoundEmitter::vftable at +0; when the byte at +0x98 is set and the byte at +0xb4 is clear it decides (thr… |

**ActorSoundEmitterDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078e3d0` | ActorSoundEmitterDefinition::ClearFlagAndCallHelper | speculative | Clears the byte at this+0x1a0, then calls 007c7700 with ECX = this+0x90 and one stack argument taken from this+0x14c (ret 0). The callee is unlabeled. |

**ActorSoundReference** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00777170` | ActorSoundReferenceDtor | confirmed | Fastcall destructor body (ECX = object). Stores ActorSoundReference::vftable at +0, calls 007c3890 on the object (unlabelled cleanup, call at the end of its fr… |
| `007771f0` | ActorSoundReference::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 00777170 on the object, frees it through _free when flag bit 0 is set,… |
| `007c3890` | ActorSoundReference::ReleaseInstanceAndResetTarget | likely | Virtual slot [4]: calls this->vtable[+8] with (0,1), which is the instance release at 007c3930, then tail-calls this->vtable[+0x24], the target clear at 007c39… |
| `007c3930` | ActorSoundReference::ReleaseInstance | likely | Thiscall with an unused dword and a char flag on the stack. If this+0x34 is non-null: when the flag is nonzero, sets byte +0x4d of the instance to 1 and calls … |
| `007c3990` | ActorSoundReference::ClearTargetRef | likely | Virtual slot [9] (34 bytes): when the pointer at this+0x30 is non-null, decrements the count at target+8 if it is positive, then stores zero at this+0x30. Make… |
| `007c39d0` | ActorSoundReference::ResolveTarget | likely | Fastcall, ECX = this (129 bytes). Returns this+0x30 when set. Otherwise asks the provider at this+0x24 through its vtable[+0x5c] whether it can resolve; if so,… |
| `007771d0` | ActorSoundReference::SubObjectFlag4CCheck | speculative | Thiscall virtual slot [3] (ECX = object). Returns 1 when the pointer at +0x34 is non-null and the byte at (+0x34 object)+0x4c is non-zero, otherwise 0 (bytes 8… |
| `007c38b0` | ActorSoundReference::ResolvedTargetSlot4 | speculative | Virtual slot [5] (37 bytes): calls this->vtable[+0x28] (ResolveTarget, 007c39d0); when that returns non-null, calls the same slot again and returns the vtable[… |
| `007c38e0` | ActorSoundReference::ResolvedTargetSlot3IsTrue | speculative | Virtual slot [6] (47 bytes): resolves the target through this->vtable[+0x28]; when it is non-null, calls the same slot again and calls the result's vtable[+0xc… |
| `007c3910` | ActorSoundReference::InstanceSlot4IsTrue | speculative | Virtual slot [7] (29 bytes): when the pointer at this+0x34 is non-null, calls its vtable[+0x10] and returns 1 if the result is nonzero, otherwise 0. Returns 0 … |
| `007c3980` | ActorSoundReference::ForwardToResolveTarget | speculative | Virtual slot [8] (7 bytes, verified): mov eax,[ecx]; mov edx,[eax+0x28]; jmp edx. Tail-jumps to the object's own vtable slot [10] (ResolveTarget, 007c39d0) wit… |
| `007c3a60` | — | unknown | Thiscall with one float stack argument dt (1026 bytes). When this+0x34 is null: if this+0x30 is null it first resolves it via vtable[+0x28]; if this+0x30 is th… |

**DeferredPlayListLoader** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c7a30` | — | unknown | Thiscall, two stack arguments (171 bytes). Logs 'Failed to load asset %s!' under the 'Sound' category (006faad0), then calls 007cb2c0 with the two arguments. I… |

**PlayListIterator** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c7fa0` | PlayListIterator::ScalarDeletingDtor | confirmed | Virtual slot [0] (165 pseudocode chars): calls 007c7f20 (the iterator destructor body), then calls _free on the object when the deleting flag bit 0 is set, and… |
| `007c7f20` | PlayListIterator::DtorBody | likely | Fastcall, ECX = iterator (118 bytes, SEH-framed). Stores the PlayListIterator vtable at +0, releases the reference-counted object at +8 (LOCK decrement of its … |
| `007c8340` | PlayListIterator::Ctor | likely | Thiscall, one pointer argument (547 pseudocode chars). Calls 0071b950 (RefCounted base constructor, label accepted as likely), stores the PlayListIterator vtab… |

**RandomNoRepeatPlayListIterator** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c8d90` | RandomNoRepeatPlayListIterator::ScalarDeletingDtor | confirmed | Virtual slot [0] (thiscall with a flag byte): stores the RandomNoRepeatPlayListIterator vtable at +0, calls 007c8c00 (the RandomPlayListIterator destructor bod… |
| `007c8c90` | RandomNoRepeatPlayListIterator::Ctor | likely | Thiscall, one argument (401 pseudocode chars). Calls the RandomPlayListIterator constructor 007c8ac0 with the argument, then stores the RandomNoRepeatPlayListI… |
| `007c8d40` | RandomNoRepeatPlayListIterator::DtorBody | likely | Fastcall, ECX = iterator (470 pseudocode chars). Stores the RandomNoRepeatPlayListIterator vtable at +0, then calls the RandomPlayListIterator destructor body … |
| `007c8cf0` | RandomNoRepeatPlayListIterator::TakeCurrentWeight | speculative | Virtual slot [4] (439 pseudocode chars, thiscall with a flag byte). Fetches the current node through vtable[+0x14]; when there is none it returns 0. Otherwise … |

**RandomPlayListIterator** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c8c70` | RandomPlayListIterator::ScalarDeletingDtor | confirmed | Virtual slot [0] (165 pseudocode chars, thiscall with a flag byte): calls 007c8c00 (the RandomPlayListIterator destructor body), frees the object when flag bit… |
| `007c83b0` | RandomPlayListRebuildWeights | likely | Virtual slot [3] shared by RandomPlayListIterator (017f01ac) and RandomNoRepeatPlayListIterator (017f01c8), fastcall with ECX = iterator (488 pseudocode chars)… |
| `007c8ac0` | RandomPlayListIterator::Ctor | likely | Thiscall, one argument (614 pseudocode chars). Calls the PlayListIterator constructor 007c8340 with the argument, installs the RandomPlayListIterator vtable at… |
| `007c8bb0` | RandomPlayListIterator::PickWeightedEntry | likely | Virtual slot [5] shared by RandomPlayListIterator and RandomNoRepeatPlayListIterator (fastcall, ECX = iterator; 606 pseudocode chars). When this+0x20 is null, … |
| `007c8c00` | RandomPlayListIterator::DtorBody | likely | Fastcall, ECX = iterator (515 pseudocode chars, SEH-framed). Stores the RandomPlayListIterator vtable at +0 and the SoeUtil::List<PlayListEntry*,-1> vtable at … |
| `007c8b40` | RandomPlayListIterator::TakeCurrentValue | speculative | Virtual slot [4] (291 pseudocode chars, thiscall with a char flag). Calls the object's vtable[+0x14] to get a node pointer; when the flag is nonzero it clears … |
| `007c8b90` | RandomPlayListIterator::IsExhausted | speculative | Virtual slot [2] (fastcall, ECX = iterator): returns true when the dword at this+0x1c is zero, otherwise false. |

**SequentialPlayListIterator** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c8410` | SequentialPlayListRebuild | likely | Virtual slot [3] of SequentialPlayListIterator (017f01e4), fastcall with ECX = iterator (312 pseudocode chars). When the source list at +8 is set, walks its en… |
| `007c8df0` | SequentialPlayListIterator::Ctor | likely | Thiscall, one argument (721 pseudocode chars). Calls the PlayListIterator constructor 007c8340 with the argument, installs the SequentialPlayListIterator vtabl… |
| `007c8ea0` | SequentialPlayListIterator::IsExhausted | speculative | Virtual slot [2] (fastcall, ECX = iterator, 96 pseudocode chars): returns true when the dword at this+0x18 is zero. |
| `007c8eb0` | SequentialPlayListIterator::TakeCurrentValue | speculative | Virtual slot [4] (329 pseudocode chars, thiscall with a flag byte): when the node pointer at this+0x10 is non-null, returns the first dword of that node; when … |

**SoeUtil::HashList<SoundGroup*,64,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00819c10` | HashList64SoundGroup::Slot1_AllocNode20 | confirmed | Virtual slot 1, no arguments. Same shape as 00819890: TLS lookup, the thread or global allocator object vtable slot 1 when present, otherwise the result of 006… |
| `00819c50` | HashListSoundGroup64::Slot2_FreeNode4 | confirmed | Virtual slot 2, one stack argument (a block pointer). Calls 00401c00(ptr, 4) and returns. |

**SoeUtil::HashListMap<int,ISoundInstanceNotificationHandler*,64,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ae3a0` | NotifyHandlerMap64Ctor | confirmed | Constructor body: stores the HashListMap<int,ISoundInstanceNotificationHandler*,64,-1> vtable at this[0], zeroes this[1..4] and memsets the 0x100-byte bucket a… |

**SoeUtil::HashListMap<int,InstanceTrackingData,1024,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ae370` | InstanceTrackingMap1024Ctor | confirmed | Constructor body: stores the HashListMap<int,InstanceTrackingData,1024,-1> vtable at this[0], zeroes this[1..4] and memsets the 0x1000-byte bucket array at thi… |

**SoeUtil::HashListMap<int,SoundCategory,256,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ae2f0` | SoundCategoryMap256Ctor | confirmed | Constructor body: stores the HashListMap<int,SoundCategory,256,-1> vtable at this[0], zeroes this[1..4] and memsets the 0x400-byte bucket array at this+0x14, t… |

**SoeUtil::HashListMap<int,SoundCategoryData,16,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ae200` | SoundCategoryMap16Ctor | confirmed | Constructor body: stores the HashListMap<int,SoundCategoryData,16,-1> vtable at this[0], zeroes this[1..4] and memsets the 0x40-byte bucket array at this+0x14,… |

**SoeUtil::HashListMap<int,SoundGroup*,64,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00818900` | HashListMapIntSoundGroup64::Ctor | confirmed | Fastcall ctor. Stores SoeUtil::HashListMap<int,SoundGroup*,64,-1>::vftable at *this, zeroes this+4, this+8, this+0x10, memsets the 0x100 bytes at this+0x14 (64… |

**SoeUtil::List<PlayListPlayback,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078fe30` | ListPlayback::DtorBody | confirmed | Stores the SoeUtil::List<PlayListPlayback,-1> vtable into this, then removes and destroys nodes from the head at this+4 through 0078fc10 until the head is null… |
| `0078fe60` | ListPlayback::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): stores the SoeUtil::List<PlayListPlayback,-1> vtable, removes and destroys every node from the head at this+4 through 0078f… |
| `0078f830` | ListPlayback_ReleaseActiveOrGlobal | speculative | Virtual slot [2]. Calls 006fb9c0 and then 006fb860; if 006fb860 returns a nonnull object, calls its vtable +4 slot on it and returns. Otherwise, if the global … |

**SoundCategoryData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007b0860` | SoundCategoryDataDtorBody | confirmed | Destructor body: stores the SoundCategoryData vtable at this[0], then stores the IString<char> vtable at this+8; when the capacity at this+0x14 is above 0 it d… |
| `007b08d0` | SoundCategoryDataScalarDeletingDtor | confirmed | Scalar deleting destructor (SoundCategoryData vtable slot [0]): calls 007b0860 (SoundCategoryData dtor body) on this, frees this with _free when bit 0 of the a… |

**SoundControl** (20)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078e600` | SoundControl::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): stores SoundControl::vftable then SoeUtil::RefCounted::vftable into this, calls _free(this) only when the flag argument bit… |
| `007be020` | SoundControl::ReadBlocks_0x70_0x60 | confirmed | Virtual slot [5] (ret 8, two stack out-pointers): calls 00769710 with no arguments and copies four dwords from result+0x70..0x7c to the first out-buffer and fo… |
| `0078e720` | SoundControl::CopyCtor | likely | Calls 0071b950 (RefCountedImplicit constructor) on this, stores SoundControl::vftable, then copies 16 dwords (64 bytes) from the source argument into this+0x8 … |
| `007bd470` | SoundControl_GetFloat_1c | likely | 4 bytes: `d9411c c3` = loads the float at this+0x1c and returns it on the x87 stack. |
| `007bd490` | SoundControl_GetFloat_20 | likely | 4 bytes: `d94120 c3` = loads the float at this+0x20 and returns it on the x87 stack. |
| `007bd4b0` | SoundControl_GetFloat_24 | likely | 4 bytes: `d94124 c3` = loads the float at this+0x24. |
| `007bd4c0` | SoundControl_SetFloat_24 | likely | 14 bytes: stores the dword argument at this+0x24 and returns. |
| `007bd4d0` | SoundControl_GetFloat_28 | likely | 4 bytes: `d94128 c3` = loads the float at this+0x28. |
| `007bd4f0` | SoundControl_GetFloat_34 | likely | 4 bytes: `d94134 c3` = loads the float at this+0x34. |
| `007bd510` | SoundControl_GetFloat_38 | likely | 4 bytes: `d94138 c3` = loads the float at this+0x38. |
| `007bd530` | SoundControl_GetFloat_3c | likely | 4 bytes: `d9413c c3` = loads the float at this+0x3c. |
| `007bd550` | SoundControl_GetFloat_2c | likely | 4 bytes: `d9412c c3` = loads the float at this+0x2c. |
| `007bd560` | SoundControl_SetFloat_2c | likely | 14 bytes: stores the dword argument at this+0x2c and returns. |
| `007bd570` | SoundControl_GetFloat_30 | likely | 4 bytes: `d94130 c3` = loads the float at this+0x30. |
| `0078e4f0` | SoundControl::Ctor | speculative | Stores SoundControl::vftable into the object and then SoeUtil::RefCounted::vftable (base store), with an EH frame and no arguments beyond this. Ret with no val… |
| `0078e550` | SoundControl::ReturnZeroVector4 | speculative | Writes four dwords from the float at 0x01762a30 (0.0) into the 16-byte buffer whose address is the hidden first stack argument, returns that address in EAX, an… |
| `0078e5d0` | SoundControl::CopyVector4 | speculative | Copies 16 bytes (four dwords) from the buffer at the first stack argument to the buffer at the second stack argument. ECX is unused. Ret 8. Virtual slot [9] of… |
| `007bce20` | — | unknown | Initialises a SoundControl from a source record: stores the source pointer at +0xa0, copies the source's strings [param_2+0x10] and [param_2+0x2c] into tempora… |
| `007bd400` | — | unknown | Writes the default field block of a SoundControl record (no vtable store): dwords +0 = 1, +4 = 0, +8 = 0, +0xc = 1, +0x10 = 500, +0x14 = DAT_017649d4, +0x18 = … |
| `007bd480` | — | unknown | 14 bytes: stores the dword argument at this+0x1c (`mov [ecx+0x1c],edx`) and returns. |

**SoundControl3d** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078e860` | SoundControl3d::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): stores the SoundControl3d, SoundControl and RefCounted vtables into this in turn, then calls _free(this) only when the flag… |
| `007bd670` | SoundControl3d::Read4Dwords_0x50 | confirmed | Virtual slot [3]: copies four dwords from this+0x50..0x5c to the 16-byte buffer at the stack argument, then ret 4 (bytes 8b 51 50 ... c2 04 00). |
| `007bd690` | SoundControl3d::Write4Dwords_0x50 | confirmed | Virtual slot [4]: stores four dwords from the stack argument into this+0x50..0x5c; ret 4 (bytes 8b 44 24 04 ... c2 04 00). |
| `007bd6b0` | SoundControl3d::Read4DwordsPair_0x60_0x70 | confirmed | Virtual slot [5]: copies this+0x60..0x6c to the first stack buffer and this+0x70..0x7c to the second; ret 8 (bytes 8b 51 60 ... c2 08 00). |
| `007bd6f0` | SoundControl3d::Write4DwordsPair_0x60_0x70 | confirmed | Virtual slot [7]: stores the first stack buffer into this+0x60..0x6c and the second into this+0x70..0x7c; ret 8 (bytes 8b 44 24 04 ... c2 08 00). |
| `007be070` | SoundControl3d::SetCosSinFromAngle | confirmed | Virtual slot [6] (thiscall, one float stack argument, ret 4): builds the vector (cos(angle), 0, sin(angle)) on the stack, calls 00769710, then calls this->vtab… |
| `007be0c0` | SoundControl3d::Diff4Floats_0x50 | confirmed | Virtual slot [9] (thiscall, two stack args, ret 8): computes the four-float difference (this+0x50..0x5c) minus the first argument vector (movups/subps) and sto… |
| `007bea60` | SoundControl3d::SumSqDiff4_0x50 | confirmed | Virtual slot [8] (thiscall, one stack arg (pointer to four floats), ret 4): returns in st0 the sum of squares of the four component differences between the arg… |
| `010c0500` | SoundControl3d::Ctor | confirmed | Fastcall constructor body (ECX = this). Calls 007affc0 on this (base constructor, not labelled here), then stores SoundControl3d::vftable into [this]. Returns … |
| `0078e790` | SoundControl3d::CopyCtor | likely | Calls the SoundControl copy constructor 0078e720 on this with the source argument, then stores SoundControl3d::vftable into this. Ret 4, returns this in EAX. |
| `0078e800` | SoundControl3d::Ctor | speculative | Stores SoundControl3d::vftable, then SoundControl::vftable, then SoeUtil::RefCounted::vftable into this (no arguments, no call). Ret with no value. |

**SoundEmitter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0078fea0` | SoundEmitter::Ctor | confirmed | Constructor with five stack arguments. Stores SoundEmitter::vftable, sets the IString member (+4), a PlayListPlayback list member (+0x14, vtable SoeUtil::List<… |
| `0078ff90` | SoundEmitter::DtorBody | confirmed | Stores SoundEmitter::vftable, calls 0078fdd0 (SoundEntity::ClearEntries) with 0 to clear the entries, then releases the refcounted objects at +0x3c, +0x38 and … |
| `007900a0` | SoundEmitter::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ret 4): calls 0078ff90 (SoundEmitter::DtorBody) with this, then calls _free(this) only when flag bit 0 is set. Returns this implici… |
| `0078f7e0` | SoundEmitter::SetOwnerRef40 | likely | Virtual slot [1] of SoundEmitter::vftable. Saves the old pointer at this+0x40, increments the refcount at +4 of the new argument, stores it at +0x40, then rele… |
| `007bc230` | SoundEmitter_vt2_StopChild | speculative | Virtual slot 2 of SoundEmitter: when this+0xac holds an object pointer, optionally (when the char flag is non-zero) sets its byte +0x4d to 1 and calls its vtab… |

**SoundEntity** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bc330` | SoundEntity_Ctor | confirmed | Constructor with five stack arguments: runs SoundEmitter::Ctor (0078fea0), Deep::Entity::ctor (007648d0) and the RefCountedImplicit constructor (0071b950), sto… |
| `007bc4e0` | SoundEntity_vt8_SetByte1d | confirmed | 10 bytes: stores the byte argument at this+0x1d and returns (`mov [ecx+0x1d],dl`). |
| `007bc4f0` | SoundEntity_vt4_GetByte75 | confirmed | 4 bytes: `0fb64175 c3` = returns the byte at this+0x75. |
| `007bc6e0` | SoundEntity_DtorBody | confirmed | Destructor body: stores SoundEntity vftables at +0, +0x58 and +0x6c, releases the refcounted pointer at +0x7c (decrement at +4 under LOCK, vtable +4 at zero), … |
| `007bc8d0` | SoundEntity_ScalarDeletingDtor | confirmed | Scalar deleting destructor (primary vtable slot 0): calls the destructor body 007bc6e0 with this in ECX, frees this with _free when flag bit 0 is set, returns … |
| `0078fdd0` | SoundEntity::ClearEntries | likely | Virtual slot [3] shared by SoundEmitter::vftable and SoundEntity::vftable. Walks the node list at this+0x18 (next at +0xc0) calling 007cadc0 with its stack arg… |
| `007bc500` | SoundEntity_vt5_ReplaceRef7c | likely | Virtual slot 5: does nothing when either byte +0x75 or byte +0x74 is non-zero; otherwise releases the refcounted object at +0x7c (decrement at +4 under LOCK, c… |
| `007bc550` | SoundEntity_vt3_DrawDebugSpheres | likely | When the debug flag DAT_01be0ac0 is set: calls 007bd470 (x87), calls the vtable slot +0xc of the object at [this-0x18] to fill a 76-byte stack record, resolves… |
| `007bc640` | SoundEntity_Release | likely | Idempotent release (guarded by the byte at +0x74, set on entry): clears bit 0 of the word at +0x68 and calls vtable slot +0x38 of the +0x58 sub-object (calling… |
| `007bc7a0` | SoundEntity_vt7_ApplyToSubObject | likely | 45-byte virtual (secondary vtable slot 7): calls vtable slot +0x10 of the object at [this-0x18] with the argument pointer +0x30, then vtable slot +0x1c of the … |
| `007bc7d0` | SoundEntity_vt12_DetachOrRelease | likely | 44-byte virtual (secondary vtable slot 12): when the dword at this+0x18 equals 1 it calls 007bc640 (release); otherwise it calls 0075c060 (Deep::Entity::Slot12… |
| `007bc800` | SoundEntity_vt10_ReattachOrRelease | likely | 78-byte virtual (secondary slot 10): resolves the current owner node (from [this+4]+0x38, else 00729610 on [this+8]) and returns when the argument equals it; w… |
| `007bc8f0` | SoundEntity_FrameUpdate | likely | Per-frame update (secondary vtable slot 2, float delta argument): returns immediately when the flag byte at +0x1c is set; otherwise calls 007bc850 (distance fl… |
| `007bcb00` | SoundEntity_vt9_DistanceQuery | likely | 147-byte virtual (secondary slot 9, one float-4 pointer argument): when the object at [this-0x18] exists and its vtable +8 reports true, reads its position thr… |
| `007bc4a0` | SoundEntity_vt1_SetOwnerAndAccumulate | speculative | Virtual slot 1 of SoundEntity: calls 0078f7e0 (SoundEmitter::SetOwnerRef40) with the argument and, if it reports success, computes the float at [[this+0x34]+0x… |
| `0078fa20` | — | unknown | Virtual slot [2] shared by SoundEmitter::vftable and SoundEntity::vftable. When this+0x4d is zero: acquires a reference through 0078f4e0 (SelectRefByOwnerKind)… |
| `007bc6c0` | — | unknown | 8-byte virtual thunk: calls 007bc8d0 (no argument adjustment visible) and returns. |
| `007bc6d0` | — | unknown | 8-byte virtual thunk: calls 007bc8d0 and returns. |
| `007bc850` | — | unknown | When the object at +0x40 exists and its vtable +8 reports true, it takes that object's position (vtable +0xc into a 4-float local) and its own position through… |

**SoundManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bd9b0` | SoundManager::AllocConstruct0x98 | confirmed | Virtual slot [5] (thiscall, two stack args, ret 8): allocates 0x98 bytes through allocator 00d833a6 and, if the result is non-null, runs constructor 007cf450 o… |
| `007bff80` | SoundManager::CallKeyedGroupSlot0 | confirmed | Virtual slot [0] of SoundManager (thiscall, one stack arg, the event pointer P). Finds the first node in the 64-bucket table at this+0x85ac (bucket (P.+0x80) &… |
| `007c00e0` | SoundManager::CallKeyedGroupSlot1 | confirmed | Virtual slot [1] of SoundManager (thiscall, one stack arg, the event pointer P). Same collection as SoundManager::CallKeyedGroupSlot0 (007bff80): the stored po… |

**SoundOneShotDefinitionLoader** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c0560` | SoundOneShotDefinitionLoader::RemoveMatchingEntries | likely | Thiscall virtual (slot [0] of the secondary vtable at +0x10) with one stack argument. Walks the list headed at this+0x98 (next link at node+4). For each node w… |
| `010c0d20` | SoundOneShotDefinitionLoader::RefreshEntries | likely | Thiscall virtual (slot [5]) with an object argument and an output pointer. Calls 007cd920 on its arguments, reads the current name through 007cc6e0 and hashes … |

**(no class)** (59)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ad7c0` | AudioSampleMsPosition | likely | Fastcall with the object in ECX: calls _AIL_sample_ms_position_12 on the sample handle at object+8 and returns the millisecond position as a float (x87 ST0). |
| `007ad7e0` | AudioSamplePollAndNotify | likely | Polls one Miles sample (playback rate, millisecond position, loop count and status through the imported AIL functions); updates the loop count stored in the wr… |
| `007ad8a0` | AudioSampleSlotRelease | likely | Thiscall with a key pointer: if the key equals the dword at this+0x10 it clears the EOS callback of the sample at this+8 (_AIL_register_EOS_callback_8 with 0),… |
| `007adf10` | AudioSampleLoadNamedFile | likely | Thiscall with one pointer argument whose +0xc holds a stream object: fetches its data through the stream's vtable slot [7] (+0x1c), builds a name value with 00… |
| `007adf80` | AudioSampleLoadNamedFileRate | likely | Thiscall with one pointer argument whose +0xc holds a stream object: fetches its data through the stream's vtable slot [7] (+0x1c), builds a name value with 00… |
| `007ae090` | AudioFilterSetCallback | likely | Thiscall with one argument: when the handle at (this+8)+0x98 is non-zero it calls _AIL_filter_property_20 with the property name Callback, value 0 and the argu… |
| `007ae0c0` | AudioFilterClearCallback | likely | Fastcall: when the handle at (this+8)+0x98 is non-zero it calls _AIL_filter_property_20 with the property name Callback and zero for the other arguments; other… |
| `007ae0e0` | SoundVoiceRecordNew | likely | Allocates a 0x1c-byte record through 00d833a6 (operator new) and sets its index dword to -1, the two words at +4 and +8 to 0, the two float words at +0xc and +… |
| `007ae1e0` | AudioSampleLoadNamedFileOwner | likely | Loads ECX from this+8 and jumps to 007adf10 (AudioSampleLoadNamedFile) (bytes 8b 49 08; e9 28 fd ff ff). |
| `007ae1f0` | AudioSampleLoadNamedFileRateOwner | likely | Loads ECX from this+8 and jumps to 007adf80 (AudioSampleLoadNamedFileRate) (bytes 8b 49 08; e9 88 fd ff ff). |
| `007ae9c0` | SoundInstanceComputeVolume | likely | Fastcall: returns the float product this+0x2c * this+0x1c times, when the parent object at this+0x14 is non-null, the chain product 007becf0(parent+0x2c) and (… |
| `007aed70` | SoundInstanceStart | likely | Thiscall with a two-dword argument: copies the two dwords into this+0x10 and this+0x14, calls 007aea60 (SoundInstanceStartSample) with the sample handle at thi… |
| `007aee30` | SampleVoicePoolInit | likely | Thiscall with a count, a value and a driver handle: stores the count at this+0, allocates count entries of 0x20 bytes (00d833a6, constructed through the array-… |
| `007aef20` | SampleVoiceAllocateOrSteal | likely | Thiscall with a sound-instance pair argument: returns the index of the first pool entry whose instance slot at +0x10 is empty after copying the pair into it, s… |
| `007b0170` | SoundVoicePairUpdateAll | likely | Thiscall with a float delta: calls 007b0060 with the delta once for each of the count entries at this+0 and returns. |
| `007b0900` | SoundRampChainUpdate | likely | Thiscall with a float delta: advances the ramp of this fader and then each fader in the chain at +0x38 (next at +0x2c): when the time at +0x28 is positive it c… |
| `007c6c00` | CreateActorSoundReference | likely | Thiscall: ECX is the storage pointer (returned unchanged), two stack arguments are dword pointers (90 bytes). Calls the ActorSoundReference constructor 007bb11… |
| `007c8760` | ListRemoveByNameHash | likely | Thiscall, one C string argument (390 pseudocode chars). Hashes the argument with 00705b00 and walks the list headed at this+0xc (next link at node+0x18) for th… |
| `010c08a0` | KeyedSoundControlAcquireWithArgs | likely | Takes four stack arguments: key, a pointer, and two dwords. Does the same lookup and acquisition as 010c07a0, then calls the vtable slot [4] of the acquired ob… |
| `010c09f0` | NamedSoundControl3dCreate | likely | Thiscall with the map in ECX, a name pointer and three dwords (stack). Hashes the name with 00705b00 and inserts a keyed IString entry (010c06d0) that receives… |
| `0078e410` | SetField50OfSubobjectIfChanged | speculative | Reads the pointer at this+0x14c; if it is nonnull and its dword at +0x50 differs from the stack argument, stores the argument at +0x50. Ret 4. |
| `0078e680` | SetHashedNameValue | speculative | Hashes the string argument with 00705b00 (case-folded CString hash), calls 007c8450 on the hash, computes mask = (+0x14 of the object at this+0x14c) - 1, calls… |
| `0078fae0` | ForEachListNodeCall_007ca5c0 | speculative | Walks the node list headed at this+0x18 (next pointer at node+0xc0), saving each next pointer before calling 007ca5c0 with ECX = node and no stack arguments. R… |
| `0078fb10` | ForEachListNodeCall_007ca640 | speculative | Walks the node list headed at this+0x18 (next pointer at node+0xc0), calling 007ca640 with ECX = node and no stack arguments for each node. Ret 0. |
| `0078fb70` | ForEachListNodeCallFloat_007ca790 | speculative | Thiscall with one float stack argument (ret 4). Walks the node list headed at this+0x18 (next at node+0xc0); for each node calls 007ca790 with ECX = node and t… |
| `007b0030` | SoundInstanceIsInaudible | speculative | Fastcall with the instance in ECX: returns 1 when the byte at +0x95 is set; otherwise it multiplies the value from 007ae9c0 (volume) by the float at +0x8c and … |
| `007bd8f0` | Write4Dwords_0x50 | speculative | Thiscall, one stack argument: stores four dwords from the argument into this+0x50..0x5c; ret 4. Same body as SoundControl3d::Write4Dwords_0x50 (007bd690). |
| `007bd910` | Read4Dwords_0x50 | speculative | Thiscall, one stack argument: copies four dwords from this+0x50..0x5c into the argument buffer; ret 4. Same body as SoundControl3d::Read4Dwords_0x50 (007bd670). |
| `007bd930` | Write4DwordsSwapped_0x60_0x70 | speculative | Thiscall, two stack arguments: stores the first argument into this+0x70..0x7c and the second into this+0x60..0x6c; ret 8. Differs from SoundControl3d::Write4Dw… |
| `007c6f00` | BuildReferencesFromEntryList | speculative | Fastcall, ECX = owner (169 bytes). Walks the entry list headed at owner+0x290 (next link at entry+0x28). For each entry, by its kind dword at +0x24: kind 0 cre… |
| `007c6fb0` | BuildReferencesAndFlush | speculative | Thiscall, one object argument (121 bytes). Returns 0 when the argument is null. Otherwise copies owner+0x288 into the argument at +0x4a4, calls 007c6f00 on the… |
| `007c78c0` | NamedFlaggedValueCtor | speculative | Thiscall with two stack arguments (543 pseudocode chars). Installs the SoeUtil::IString<char> vtable at offset 0, sets the buffer pointer at +4 to the static e… |
| `007c7940` | ListUnlinkEntryThenSlot3 | speculative | Thiscall, one node argument (133 bytes). Unlinks the node from a doubly linked list: the next link at node+0x18 and the prev link at node+0x1c, with the list h… |
| `007c79d0` | CreateNamedFlaggedValue | speculative | Thiscall with two stack dword-pointer arguments (374 pseudocode chars). Calls 007c78c0 with the dwords at *param_2 and *param_3 and returns the first argument … |
| `007c7e60` | DwordKeyEntryCtor | speculative | Thiscall with one pointer argument (105 bytes). Stores the constant at 0x0175b420 at +0, zeroes +4, +8, +0x10 and +0x14, and copies the dword pointed to by the… |
| `007c7fc0` | SetSourceRefAndKeyHash | speculative | Thiscall, one object argument (669 pseudocode chars). Clears the dword at +0xc and the dword at +0x14 when the argument is null. Otherwise increments the argum… |
| `007c8460` | ListNodeSetSourceAt | speculative | Thiscall, two arguments: an index and an object (392 pseudocode chars). Walks the list headed at this+0xc (next link at node+0x18) for index steps, and when a … |
| `007c84a0` | PlayListNodesReset | speculative | Fastcall, ECX = list owner (437 pseudocode chars). Walks the list headed at owner+0xc (next link at node+0x18). For each node it zeroes the dword at node+0xc a… |
| `007c8560` | ListUnlinkNodeThenReleaseChild | speculative | Thiscall, one node argument (560 pseudocode chars). Unlinks the node from the list (next at node+0x18, prev at node+0x1c; head at this+4, tail at this+8, count… |
| `007c8670` | PlayListEntrySetFromRef | speculative | Thiscall, one pointer argument (405 pseudocode chars). Calls 007c7fc0 on this with the dword at *param_2 (SetSourceRefAndKeyHash), then zeroes this+4 and this+… |
| `007c8880` | ListDrainHeadReleaseChildCallSlot3 | speculative | Fastcall, ECX = list (1217 pseudocode chars). While the head at list+4 is non-null, unlinks the head (next at node+0x18, prev at node+0x1c; tail at list+8, cou… |
| `007c8a00` | PlayListEntryCopyFrom | speculative | Thiscall, one pointer argument (474 pseudocode chars, returns this). Calls 007c7fc0 on this with the dword at argument+0x10, then copies the dwords at argument… |
| `00d36260` | StoreFields_14_9c_a0_a4_a8 | speculative | Thiscall with five stack arguments (param_2..param_6): stores param_2 at +0x14, param_3 at +0xa0, param_4 at +0x9c, param_5 at +0xa4 and param_6 at +0xa8, and … |
| `010c07a0` | KeyedSoundControlAcquire | speculative | Takes one argument, the key. Looks up the key in the global table (007699f0 then 00709a70). When the entry's +0x14c is non-zero, obtains an object from the man… |
| `0078e8d0` | — | unknown | Reads the pointer at this+0x14c. Returns 0 if it is null or equals -8, or if its list head at +0xc is null. With a zero stack argument it returns 1. Otherwise … |
| `0078f150` | — | unknown | Reads the kind dword at this+0x150. Kind 1 allocates 0x48 bytes with 00d833a6 and copy-constructs a SoundControl through 0078e720 from the template at this+0x1… |
| `007adcd0` | — | unknown | Advances a timed playback by the float delta argument: computes the position from the float at this+0x68 plus the delta, compares it with the duration returned… |
| `007aea60` | — | unknown | Cdecl with the Miles sample handle and the sound-instance pointer: loads the named sound file through the stream at the instance's vtable slot [7] (+0x1c) and … |
| `007aec00` | — | unknown | Thiscall-style per-update refresh of a Miles sample from a sound instance: sets the sample volume and pan from the instance gains and the parent-chain factor (… |
| `007af4a0` | — | unknown | Thiscall: calls 007aef20 (SampleVoiceAllocateOrSteal) on this and returns whether the dword at **(this+4) is greater than -1. |
| `007af4d0` | — | unknown | Thiscall: when the dword at **(this+4) is greater than -1 it calls 007ad8a0 (AudioSampleSlotRelease) on this, then frees the block at this+4 with _free and cle… |
| `007b0060` | — | unknown | Thiscall with a float delta: when the record at this+0x18 is set it steps its ramp (value at +0xc moves by the rate at +0x14 times the delta, clamped to the ra… |
| `007c6d20` | — | unknown | Thiscall, two stack arguments (148 bytes). Calls the builder's vtable[+8] with a stack temporary; when that is nonzero it creates a sound reference through 007… |
| `007c7ae0` | — | unknown | Thiscall, two stack arguments (148 bytes). Calls the builder's vtable[+8] with a stack temporary; when that is nonzero it creates a value through 007c79d0 and … |
| `007c7b80` | — | unknown | Plain function with two stack arguments and a byte flag (53 bytes). Calls 007c7ae0 with the addresses of its two arguments; when the flag byte is nonzero it ca… |
| `007c81d0` | — | unknown | Thiscall, one stack argument (748 pseudocode chars). Calls the builder's vtable[+8] with a stack temporary; when that is nonzero it creates an entry through 00… |
| `007c8450` | — | unknown | Plain function with no arguments of its own (83 pseudocode chars). Calls 007c81d0 with a pointer to its own stack slot (a Ghidra stack artifact in the pseudoco… |
| `007c84e0` | — | unknown | Fastcall, ECX = list owner (295 pseudocode chars). Walks the list headed at owner+0xc (next link at node+0x18). Returns 1 when the list is empty or no node has… |
| `007c8940` | — | unknown | Thiscall, one stack argument (748 pseudocode chars). Calls the list's vtable[+8] with a stack temporary; when that is nonzero it creates an entry with 007c8670… |

### `audio.vehicle`

**VehicleRaceProcessor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b9ae90` | VehicleRaceProcessor::StopLoopingRaceSounds | likely | For the three non-null emitter pointers at this+0x1f4 (Vehicle_Shield_Active), +0x204 (Vehicle_CowCatcher_Active) and +0x244 (Vehicle_Wrong_Way): sets byte [em… |
| `00b9f360` | VehicleRaceProcessor::CreateRaceSoundEmitters | likely | Looks up 24 named race events and creates one sound emitter per event: for each name (Vehicle_Pickup_Hit, Vehicle_Pickup_Lost, Vehicle_Boost_Created, Vehicle_S… |
| `00b9ac50` | — | unknown | Takes an int volume in param_2 (converted to float) and, for each of 22 non-null object pointers at this+0x1d4, +0x1dc, +0x1e4 ... +0x28c (all listener slots e… |

## Source shards

[[S0165|S0165]] (9), [[S0170|S0170]] (28), [[S0175|S0175]] (46), [[S0176|S0176]] (46), [[S0177|S0177]] (35), [[S0178 007bd5b0-007c2fe0|S0178]] (14), [[S0179|S0179]] (51), [[S0184|S0184]] (7), [[S0185|S0185]] (15), [[S0195|S0195]] (3), [[S0219 0089d990-008a3c80|S0219]] (3), [[S0221 008a86b0-008b1d80|S0221]] (3), [[S0229 008d92f0-008dea46|S0229]] (1), [[S0231|S0231]] (5), [[S0236|S0236]] (1), [[S0239 0090a890-009105b0|S0239]] (4), [[S0240 009105e0-009161d0|S0240]] (5), [[S0242 0091af50-009200d0|S0242]] (1), [[S0244 00925fe0-0092b110|S0244]] (9), [[S0281 009ea8a0-009f0100|S0281]] (1), [[S0283 009f39b0-009f8370|S0283]] (13), [[S0300 00a59220-00a5da20|S0300]] (1), [[S0322 00acd6d0-00ad3250|S0322]] (1), [[S0323 00ad3270-00ad83b0|S0323]] (3), [[S0333 00b052f0-00b0a290|S0333]] (21), [[S0334 00b0a370-00b10840|S0334]] (1), [[S0349 00b5ffe0-00b66a80|S0349]] (8), [[S0353 00b71490-00b77eb0|S0353]] (3), [[S0361 00b9a660-00ba0260|S0361]] (4), [[S0384 00c0dcd0-00c171d0|S0384]] (10), [[S0385 00c17390-00c1f0a0|S0385]] (11), [[S0398|S0398]] (19), [[S0409 00ca0f50-00ca6310|S0409]] (1), [[S0422 00ce4e30-00ceb920|S0422]] (1), [[S0423 00ceb9b0-00cf13e0|S0423]] (10), [[S0439|S0439]] (1), [[S0542|S0542]] (4), [[S0592|S0592]] (11), [[S0654 012393a0-0123f010|S0654]] (8), [[S0690|S0690]] (3)
