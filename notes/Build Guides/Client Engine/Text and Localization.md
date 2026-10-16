---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Text and Localization

Part of [[Build Guides]] › Client Engine. Related: [[UI Framework (GFC and XML)]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

820 accepted functions: 178 confirmed, 151 likely, 301 speculative, 190 unknown. Network-linked (`wire`): 0.

Subsystems: `lib.t4macro` (244), `engine.string` (162), `lib.t4macro.es_es` (99), `lib.t4` (97), `ui.locale` (40), `lib.t4macro.it_it` (29), `lib.t4.string` (29), `lib.t4macro.sv_se` (26), `lib.t4macro.fr_fr` (25), `lib.t4.strutil` (19), `lib.t4.locale` (15), `engine.locale` (10), `lib.t4.tagapi` (7), `lib.t4.base` (5), `lib.t4macro.ja_jp` (4), `lib.t4macro.grammar` (2), `lib.t4macro.lexer` (2), `lib.t4.stream` (2), `lib.t4macro.parser` (1), `lib.t4.trace` (1), `lib.t4macro.de_de` (1)

### `engine.locale`

**Localization::LocalizationManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e1d80` | Localization::LocalizationManager::LocalizationManager | confirmed | Constructor: runs the base constructor 012e0c60 on the object, then stores Localization::LocalizationManager::vftable and returns this. |
| `012e1ea0` | Localization::LocalizationManager::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of Localization::LocalizationManager::vftable): runs the base destructor body 012e0cc0 on the object, then frees it w… |
| `012e1e30` | — | unknown | Lazy singleton accessor: when DAT_01cbd760 is null, allocates 0x2c bytes with operator new and constructs the object through 012e1d80, storing the result in DA… |

**Localization::LocalizationManagerBase** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e0c60` | Localization::LocalizationManagerBase::LocalizationManagerB… | confirmed | Constructor: stores Localization::LocalizationManagerBase::vftable, zeroes the valid flag at +4 and the owned pointer at +8, and makes the std::string at +0xc … |
| `012e0cc0` | Localization::LocalizationManagerBase::DtorBody | confirmed | Stores the base vtable; when the valid flag at +4 is set it clears it and calls the owned object at +8 (vtable slot 0, flag 1); then frees the string buffer at… |
| `012e0d50` | Localization::LocalizationManagerBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0 of Localization::LocalizationManagerBase::vftable): calls 012e0cc0 on the object, then frees it with _free when the f… |
| `012e0de0` | Localization::LocalizationManagerBase::Init | confirmed | Init(locale string): asserts !IsValid (012f5a60 on failure), sets the valid flag at +4, assigns the argument to the string at +0xc, asserts it is non-empty, ma… |
| `012e0c20` | Localization::LocalizationManagerBase::ClearOwned | likely | Asserts this->IsValid() (012f5a60 on failure), clears the valid flag at +4, calls the owned object at +8 through its vtable slot 0 with flag 1 when non-null, a… |
| `012e1de0` | Localization::LocalizationManagerBase::DtorFunclet | likely | EH-unwind cleanup for a LocalizationManagerBase held in a local: sets up an exception frame and calls the base destructor body 012e0cc0 on that stack object. |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012e1c80` | LocalizeOrCopy | likely | Assigns to the out string param_1 the localised text for key param_2 when 012e0f60 (receiver in ECX) returns true, otherwise a copy of param_2; the result is a… |

### `engine.string`

**BaseClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008ece50` | BaseClient::LookupCodeStringWithArgs | confirmed | Virtual slot [50] of BaseClient and PlayClient. Hashes the key (00705b00) and searches the 256-bucket table at this+0x290 (reached through this+0x20); when fou… |
| `008ed220` | — | unknown | Virtual at the +0x4 slot of the BaseClient/PlayClient vtable. For id > 0 and this+0x850 non-zero: when this+0xc4 is null it calls 0097be40(out, id); otherwise … |

**IdToStringLookup** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007263e0` | IdToStringLookup::ScalarDeletingDtor | confirmed | Scalar deleting destructor in vtable slot [0] of IdToStringLookup. Bytes: push esi; mov esi,ecx; call the destructor body 00725ff0 on this; if bit 0 of the arg… |
| `00726060` | IdToStringLookup::LoadTextFile | likely | Vtable slot [1] of IdToStringLookup. Builds a local IStringFixed<char,256> with FUN_00725ee0 (ECX = stack object, param_2 pushed), appends the C string at this… |

**SoeUtil::IString<char>** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007a50b0` | SoeUtil::IStringRefDtorBody | confirmed | Stores SoeUtil::IString<char>::vftable at this; if the count at this+0xc is positive, drops one reference on the buffer at this+0x4 (lock xadd on buffer-4) and… |
| `007a5c60` | SoeUtil::IStringCopyCtor20 | confirmed | Copy constructor for a 0x20-byte IString<char>-based element: stores the IString vtable, points the buffer at the static empty string (a second empty-string co… |
| `00d34bb0` | IStringCopyCtor | confirmed | Constructor of an IString<char>-based object with one stack argument (source). Installs the IString vtable, sets the static empty buffer, zeroes length and cap… |
| `00d34c80` | IStringCtorWithOwner | confirmed | Constructor with two stack arguments: source IString (param_2) and owner pointer (param_3). Installs the IString vtable, zeroes its fields, copy-assigns the so… |
| `007a53b0` | SoeUtil::IStringScalarDtorA | likely | Calls 007a4ea0 (IString<char> destructor body) on this, then calls _free(this) when bit 0 of the stack flag is set; returns this with ret 4. |
| `007a5410` | SoeUtil::IStringScalarDtorB | likely | Calls 007a4fb0 (IString<char> destructor body, byte-identical to 007a4ea0) on this, then calls _free(this) when bit 0 of the stack flag is set; returns this wi… |
| `007a5470` | SoeUtil::IStringScalarDtorC | likely | Scalar deleting destructor: runs the IString<char> destructor body 007a50b0 on this, then calls _free(this) when bit 0 of the stack flag is set; returns this w… |
| `007a5b70` | SoeUtil::IStringCtorFromCStrPtr | likely | Constructor: stores SoeUtil::IString<char>::vftable at this, points the buffer at the static empty string, zeroes the dwords at +0x8 and +0xc, assigns the C st… |
| `00d34db0` | IStringCopyCtorWrapper | likely | Thiscall wrapper with ECX = object and one stack argument: calls IStringCopyCtor 00d34bb0 on the same ECX with the argument, then returns this in EAX (ret 4). |
| `01088940` | IStringMemberDtor_Off10 | likely | Stores the IString<char> vftable at this+0x10. If the embedded string length at this+0x1c is positive, atomically decrements the reference count stored 4 bytes… |
| `01088ca0` | IStringMemberDtor_Off08 | likely | Stores the IString<char> vftable at this+8. If the embedded length at this+0x14 is positive, atomically decrements the count stored 4 bytes before the buffer a… |
| `01088e20` | IStringMemberDtor_Off08B | likely | Identical body to IStringMemberDtor_Off08 (01088ca0): stores the IString<char> vftable at this+8 and, when the embedded length at this+0x14 is positive, decrem… |
| `010891c0` | IStringMemberDtor_Off0C | likely | Stores the IString<char> vftable at this+0xc. If the embedded length at this+0x18 is positive, atomically decrements the count stored 4 bytes before the buffer… |
| `0108a890` | IStringCtorAssign | likely | Thiscall, SEH framed: zeroes this+8 and this+0xc, stores SoeUtil::IString<char> vftable at this+0, points this+4 at DAT_01cb83aa (static empty storage), then a… |
| `010c05b0` | — | unknown | Fastcall constructor body with this in ECX. Installs SoeUtil::IString<char>::vftable at +0, points the buffer at DAT_01cb9088 (+4), zeroes the length (+8) and … |

**SoeUtil::IString<unsigned_short>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00be8120` | — | unknown | Stores the SoeUtil::IString<unsigned_short> vtable at this+4, and when the length field at this+0x10 is positive decrements the reference count at buffer-4 wit… |

**SoeUtil::IStringFixed<char,4098>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7b2e0` | IStringFixed4098_AllocOrInlineBuffer | likely | Virtual slot [1] of the IStringFixed<char,4098> and StringFixed<4098> vftables (thiscall with three stack arguments; returns a pointer in EAX). For a requested… |
| `00d7b310` | IStringFixed4098_FreeHeapBuffer | likely | Virtual slot [2] (thiscall, one argument). If the argument pointer is not the inline buffer (this+0x10) it calls FreeMemory(argument, 0); otherwise it does not… |
| `00d7b5c0` | IStringFixed4098_DtorBody | likely | Fastcall (ECX = this). Stores the IStringFixed<char,4098> vftable at +0, releases the heap buffer at +4 through its count and the vftable slot +8 when the coun… |
| `00d7b610` | IStringFixed4098_ScalarDeletingDtor | likely | Virtual slot [0] of IStringFixed<char,4098>. Stores the IStringFixed<char,4098> vftable at this+0, releases the heap buffer at this+4 when its count (at buffer… |
| `00d7b750` | IStringFixed4098_DtorBody_B | likely | Fastcall (ECX = this). Stores the IStringFixed<char,4098> vftable at +0, releases the heap buffer at +4 through its count and vftable slot +8 when it reaches z… |

**SoeUtil::IStringFixed<char,8>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00792630` | IStringFixed8::DtorBody | confirmed | Sets the vtable to SoeUtil::IStringFixed<char,8>; if this[3] > 0 it atomically decrements the reference count stored 4 bytes before this[1] and calls the buffe… |
| `007926f0` | IStringFixed8::AllocBuffer | confirmed | Thiscall with (requested size, out capacity*, out owns-heap flag*). If size <= 12 it stores 0 to the flag, 12 to the capacity, and returns this+0x10 (the inlin… |
| `00792720` | IStringFixed8::FreeBuffer | confirmed | Thiscall with one pointer argument. If the pointer differs from this+0x10 (the inline buffer) it calls 00401c00(ptr, 0), the free wrapper; otherwise it returns… |
| `00792740` | IStringFixed8::ScalarDeletingDtor | confirmed | Thiscall with one flag argument. Calls 00792630 (the dtor body) on this, tests bit 0 of the flag, and when it is set passes this to _free (00d82463, a thunk). … |
| `00793780` | IStringFixed8::CtorFromCString | confirmed | Thiscall (ECX = this, param_2 = C string). Stores the SoeUtil::IString<char> vtable, sets the buffer to the empty string DAT_01bdfec8 with length and capacity … |
| `00792c40` | IStringFixed8::DtorGuarded | likely | Thiscall (ECX = this). Installs a SEH frame with the cookie guard (handler 0155aca8, state -1), calls the IStringFixed8 dtor body 00792630 on this, and unwinds… |

**SoeUtil::StringFixed<4098>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d7b7e0` | StringFixed4098_ScalarDeletingDtor | likely | Virtual slot [0] of StringFixed<char,4098> (vftable 01843918). Body is identical to IStringFixed4098_ScalarDeletingDtor (00d7b610): stores the IStringFixed<cha… |

**SoeUtil::StringFixed<8>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00793d30` | StringFixed8::Ctor | confirmed | Thiscall (ECX = this, param_2 = C string). Calls 00793780 (IStringFixed<char,8> constructor) with param_2, then stores the SoeUtil::StringFixed<8> vtable at th… |
| `00793d90` | StringFixed8::ScalarDeletingDtor | confirmed | Thiscall with one flag byte. Calls 00792630 (IStringFixed<char,8> destructor body) on this, then calls _free (00d82463 thunk) on this when flag bit 0 is set. R… |

**(no class)** (128)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739570` | Utf8EncodeCodePoint | confirmed | Encodes the Unicode code point in param_2 as UTF-8 into the buffer addressed by *param_1: 1 byte below 0x80, 2 bytes below 0x800, 3 below 0x10000, 4 below 0x11… |
| `008ed320` | ResolveStringIdOrPlaceholder | confirmed | Thiscall with an output string (param_2) and a numeric id (param_3). Default-constructs a 256-char fixed string (004029b0), calls the virtual at slot +8 of the… |
| `00726340` | Bucket100RemoveKey | likely | thiscall with one stack argument (key). Looks up the chain at this+0xc for bucket key % 100 for a node whose +0x18 equals key. If found, calls 00725970 with EC… |
| `009898a0` | IStringMemberDeletingDestructor | likely | Calls 00989240 on this; if deleting flag bit 0 is set it frees this, otherwise returns this. |
| `009898c0` | IStringMemberDeletingDestructor | likely | Calls 009892b0 on this; if deleting flag bit 0 is set it frees this, otherwise returns this. |
| `00bbe510` | ParseSignedDecimal | likely | Bytes checked (259 bytes; Ghidra's negative-branch arithmetic was misleading): reads a char string at arg1 and writes an int to *arg2. Skips leading chars with… |
| `00d374e0` | ReadFieldIntoInt2048 | likely | Cdecl with three arguments: fills a 2048-byte stack buffer with one delimited field through 006fbe20 (size 0x800, delimiters param_3, trim 1), then passes the … |
| `00d7b870` | IStringFixed16_Pair64_DtorBody | likely | Fastcall (ECX = this). Stores the IStringFixed<char,64> vftable at this+0x24, releases the buffer at +0x28 when its count reaches zero, resets it to DAT_01bf32… |
| `00d7bec0` | IStringFixed16_Pair64_ScalarDeletingDtor | likely | Thiscall with a flag byte. Calls 00d7b870 (the non-deleting body that destroys the IStringFixed<16> base and the IStringFixed<64> member) and calls _free(this)… |
| `00d7fb90` | CmdLine_QuotedArgLength | likely | Given an argument string, returns the number of command-line characters it occupies: its length plus 1 for the trailing space, or plus 3 when a space occurs be… |
| `00d7fbd0` | CmdLine_AppendQuotedArg | likely | Appends an argument string to a command-line buffer at the write position found by scanning to the end of the buffer. The argument is wrapped in double quotes … |
| `01088750` | ParseDecimalInt | likely | Skips leading chars that _iswctype classifies as 'space', takes an optional '+' or a '-' sign, accumulates decimal digits (iswctype 'digit') into a 32-bit int … |
| `01088ea0` | ParseDecimalIntB | likely | Byte-for-byte the same decimal parser as ParseDecimalInt (01088750): skips 'space' characters, reads an optional sign, accumulates 'digit' characters into an i… |
| `0108a2f0` | RecordCopyCtor | likely | Thiscall (ECX = new record, param_2 = source), SEH framed: installs SoeUtil::IString<char> vftable and the static buffer DAT_01cb8370 with zero length/capacity… |
| `0108b120` | ParseDecimalIntC | likely | Byte-for-byte the same decimal parser as ParseDecimalInt (01088750): skips 'space' characters, reads an optional sign, accumulates 'digit' characters into an i… |
| `012a5f60` | PluralSuffixTokens_InstallPlural | likely | Calls 012a5e10 three times on param_1 with the rule pairs ("#s#" -> "s"), ("#es#" -> "es") and ("#ies#" -> "ies"), taking the key and value strings from fixed … |
| `012a5fa0` | PluralSuffixTokens_InstallSingular | likely | Calls 012a5e10 three times on param_1 with the rule pairs ("#s#" -> empty), ("#es#" -> empty) and ("#ies#" -> "y"), taking the key and value strings from fixed… |
| `012f3cc0` | StringReplaceAllChar | likely | Given a std::string object pointer (first stack argument), a 'from' char and a 'to' char: calls the find helper 01231320 with the one-char needle, starting at … |
| `012f4720` | CopyPathBackslashToSlash | likely | Assigns the std::string argument into the global string object at DAT_01bb953c (std::basic_string assign of the whole argument via 00420ff1), then calls 012f3c… |
| `0130f3a0` | StringToUpperInPlace | likely | Upper-cases the std::string at this in place: validates the range [data, data+size) against the buffer (size at +0x14; data at +4, or the heap pointer when cap… |
| `0130f4a0` | StringToLowerCopy | likely | Makes *param_1 an empty std::string (capacity 15, size 0, SSO), assigns the whole of param_2 into it (assign(src,0,npos)), then calls 0130f2a0 on the copy; ret… |
| `0130f520` | StringToUpperCopy | likely | Makes *param_1 an empty std::string (capacity 15, size 0, SSO), assigns the whole of param_2 into it (assign(src,0,npos)), then calls 0130f3a0 to upper-case th… |
| `0130fbe0` | FormatThreadLocalBuffer | likely | Cdecl with a printf-style format (param_1) and variadic arguments. Gets the calling thread id (GetCurrentThreadId), looks it up through 0130f200 in the static … |
| `013cb800` | StaticStrMapCopy_01cbdd88 | likely | cdecl(std::string *out, int key): runs the static-table initialiser 013cae70 when DAT_01cbdd84 is zero, looks up key in the static int-keyed string map at 01cb… |
| `013cb890` | StaticStrMapCopy_01cbddbc | likely | cdecl(std::string *out, int key): same accessor shape as 013cb800 (first-use guard, 012a49c0 lookup, assign_substr copy into *out, EAX = out), but the static m… |
| `013cb920` | StaticStrMapCopy_01cbdde0 | likely | cdecl(std::string *out, int key): same accessor shape as 013cb800 (first-use guard, 012a49c0 lookup on the static map at 01cbdde0, assign_substr copy into *out… |
| `013cb9b0` | StaticStrMapRef_01cbddec | likely | cdecl(int key): first-use guard (013cae70 when DAT_01cbdd84 is zero), then returns in EAX the mapped std::string& for key from the int-keyed map at 01cbddec th… |
| `013cb9d0` | StaticStrMapRef_01cbddb0 | likely | cdecl(int key): first-use guard, then returns in EAX the mapped std::string& for key from the int-keyed map at 01cbddb0 through 013ca7f0. |
| `013cb9f0` | StaticStrMapRef_01cbddc8 | likely | cdecl(int key): first-use guard, then returns in EAX the mapped std::string& for key from the int-keyed map at 01cbddc8 through 013ca7f0. |
| `013cba10` | StaticPairStrMapRef_01cbddd4 | likely | cdecl(int a, int b): first-use guard, then builds a two-int key on the stack and returns in EAX the mapped std::string& for (a, b) from the pair-key map at 01c… |
| `013cba70` | StaticStrVecFindAssign | likely | cdecl(std::string *param_1), void, ret plain: first-use guard, then scans the 28-byte string vector at 01cbdd94 (begin 01cbdd98, end 01cbdd9c) for an element e… |
| `01447ab0` | OrdinalSuffix | likely | Maps the argument modulo 10 to a static suffix literal: 1 gives the bytes st, 2 gives nd, 3 gives rd, and every other remainder gives th (no 11-13 special case… |
| `01447cc0` | StringGetCopyAt38 | likely | Returns a copy of the std::string at this+0x38 through the hidden return buffer param_2: initialises the buffer (size 0, capacity 15, first byte 0) and assigns… |
| `01447d30` | StringGetCopyAt54 | likely | Returns a copy of the std::string at this+0x54 through the hidden return buffer param_2 (same body as 01447cc0 with offset 0x54). |
| `004c882c` | PascalPrefixStrnEqual | speculative | Cdecl, three stack args (buf, text, len). Returns 1 when buf[0] equals (char)len and _strncmp(buf+1, text, len) returns 0, otherwise 0. |
| `004ca737` | FindWideStringFrom | speculative | Thiscall(this = holder whose first dword points to {dword length, ushort chars}, start index, limit, needle ushort*, needle length or -1 for zero-terminated). … |
| `004ca7cb` | FindWideStringLastFrom | speculative | Thiscall(this, start index, end index, needle ushort*, needle length or -1 for zero-terminated). Scans candidate positions backward from (end - needle length +… |
| `004ca98d` | ReplaceWideCharAll | speculative | Thiscall(this = holder with {dword length, ushort chars}, old ushort, new ushort): replaces every character equal to old with new, scanning the length stored a… |
| `004ca9bc` | ParseFloatAtIndex | speculative | Thiscall(this = holder, index): when index is below the length, returns (float) FUN_00d8707a(chars+index, &index), i.e. a numeric parse of the wide text starti… |
| `004ca9e6` | ParseDoubleAtIndex | speculative | Thiscall(this = holder, index): when index is below the length returns (float10) FUN_00d8707a(chars+index, &index), a floating-point parse of the wide text at … |
| `004caa0a` | ParseLongAtIndex | speculative | Thiscall(this = holder, base, index): when index is below the length returns _wcstol(chars+index, &end, base); otherwise returns 0. |
| `004caa32` | ParseUlongAtIndex | speculative | Thiscall(this = holder, base, index): when index is below the length returns _wcstoul(chars+index, &end, base); otherwise returns 0. |
| `004caa5a` | WideIntAtIndexVia8D733D | speculative | Thiscall(this = holder, index): when index is below the length returns FUN_00d8733d(chars+index) as a dword; otherwise returns 0. |
| `004caa96` | WideIntAtIndexViaThunk | speculative | Thiscall(this = holder, index): when index is below the length returns thunk_FUN_00d8730d(chars+index) as a dword; otherwise returns 0. |
| `00793e30` | ReadTokenToStringBuf2K | speculative | Cdecl with three stack args. Clears a 0x800-byte stack buffer, calls 006fba40(param_1, buffer, 0x800, param_3, 1), then assigns the buffer to the output string… |
| `007a51b0` | ScalarDeletingDtorIStringTriple | speculative | Calls 007a4b80 (releases the three IString<char> members at this+0x1c, +0x34, +0x48), then calls _free(this) when bit 0 of the stack flag is set; returns this … |
| `007a51d0` | ScalarDeletingDtorIStringPair | speculative | Calls 007a4c60 (releases two IString<char> members at this+0x24 and related fields), then calls _free(this) when bit 0 of the stack flag is set; returns this w… |
| `007a51f0` | ScalarDeletingDtorIStringSingle | speculative | Calls 007a4d10 (releases one IString<char> member at this+0x14), then calls _free(this) when bit 0 of the stack flag is set; returns this with ret 4. |
| `007a5210` | ScalarDeletingDtorIStringPairB | speculative | Calls 007a4d80 (releases two IString<char> members at this+0x24 and related fields; byte-identical to 007a4c60), then calls _free(this) when bit 0 of the stack… |
| `007a58f0` | StringMembersDtorBody7 | speculative | Destructor body (no free of this): releases in order the IString<char> members at this+0xcc, the fixed-string member via 00703350 (ECX = this+0x98), then the I… |
| `007a5ab0` | IStringArrayTruncate0x20 | speculative | Shrinks an array of 0x20-byte elements (base at this+0x4, count at this+0x8) to count minus the stack argument: for each removed element (ECX = base + index*0x… |
| `007a5cf0` | IStringPairRecordCtorCopy | speculative | Constructs a record with a dword at +0x10, an IString<char> at +0x14 assigned from the C string *(param_3+4) through 00402ce0, and an IString<char> at +0x24 co… |
| `007a5ed0` | IStringDwordRecordCtor | speculative | Constructs a record with a dword at +0x10, an IString<char> at +0x14 assigned from the C string param_3 through 00402ce0, and a dword at +0x24 set from param_4… |
| `007a5f60` | StringMembersCtorBody7 | speculative | Default constructor (ECX = this, no stack arguments): zeroes the header dwords, installs the IString<char> vtable and static empty buffer in the members at +0x… |
| `007a6120` | IStringPairCStrRecordCtor | speculative | Constructs a record with a dword at +0x10 from param_2, an IString<char> at +0x14 assigned from *(param_3+4) through 00402ce0, and an IString<char> at +0x24 as… |
| `007a6220` | IStringPathRecordCtor512 | speculative | Constructs a record: zeroes the header dwords, stores a dword at +0x20 as zero, installs the IString<char> vtable at +0x10 with the static empty buffer, assign… |
| `007a62d0` | IStringDwordRecordCtorB | speculative | Constructor: stores the IString<char> vtable at this, points the buffer at the static empty string, zeroes the dwords at +0x8 and +0xc, assigns the C string pa… |
| `007a63d0` | AssignPathEnsureTrailingSlash | speculative | Assigns the C string param_2 into the IString<char> at this+0x49f4 through 00402ce0; then, when the dword at this+0x49fc is positive and the last buffer charac… |
| `007a8290` | ScalarDeletingDtorStringMembers7 | speculative | Scalar deleting destructor: runs the destructor body 007a58f0 on this, then calls _free(this) when bit 0 of the stack flag is set; returns this with ret 4. |
| `007a82c0` | IStringArrayReallocate0x20 | speculative | Reallocates an array of 0x20-byte IString-based elements: allocates the new block through vtable slot [1] of this, pushing three values (the two stack argument… |
| `007a96c0` | IStringArrayEmplaceBack0x20 | speculative | Grows the 0x20-element array at this+0x4 (count at this+0x8, capacity at this+0xc) through 007a82c0 when count+1 exceeds capacity, increments the count, and re… |
| `0081b610` | IStringMember28_Release | speculative | Fastcall with ECX = owner. Stores the SoeUtil::IString<char> vtable at owner+0x28. If the dword at owner+0x34 is positive it atomically decrements the referenc… |
| `0081b680` | IStringPair_Init | speculative | Fastcall ctor. Stores the SoeUtil::IString<char> vtable at *this and at this+0x10, sets the buffer pointers at this+4 and this+0x14 to the empty-string DAT_01b… |
| `00b3cff0` | FormatAndTransformTextBuffers | speculative | Formats the third argument into a 1024-byte local buffer, passes both local buffers and the remaining arguments to the transform helper, then forwards its resu… |
| `00bbe650` | ParseCsvIntField | speculative | Bytes checked (108 bytes): takes (src, int* out, delim). Zeroes a 0x800-byte stack buffer, calls ParseCsvField (006fba40) with (src, buffer, 0x800, delim, trim… |
| `00be81a0` | ScalarDeleteWrap_00be8120 | speculative | Calls 00be8120 (IString<unsigned_short> release body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c588a0` | AssignTwoStringsAndExpand | speculative | Assigns the stack-arg string through 00402ce0 (IStringAssign), assigns the static string DAT_0175b400 through 00402ce0, calls 00726400 and then 0097c280 on the… |
| `00d73ab0` | StringToLowerInPlaceChanged | speculative | Lowercases a NUL-terminated char string in place, one character at a time through _towlower (00d8a390); returns 1 in AL when any character changed and 0 otherw… |
| `00d7e200` | CStrCopyToEcx | speculative | Thiscall-style (ECX = destination address, one stack argument = NUL-terminated source). Copies the source byte by byte to the address ECX plus the source index… |
| `00d7e490` | CStrIsNonEmpty | speculative | Fastcall (ECX = string). Returns in AL 1 when the first byte is nonzero, else 0; 8 bytes. |
| `00d7e580` | DashSuffix_CompareGreater | speculative | Takes two string pointers. Returns 0 when the first string has no dash character (0x2d); returns 1 when only the second lacks a dash. Otherwise compares the te… |
| `00fa0d30` | ParseSignedDecimalInt_00fa0d30 | speculative | Reads a char string: skips characters that iswctype 'space' accepts, takes an optional '-' or '+', then accumulates digits (iswctype 'digit') into a signed int… |
| `00fa0ed0` | ParseSignedDecimalInt_00fa0ed0 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa0fe0` | ParseSignedDecimalInt_00fa0fe0 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa10f0` | ParseSignedDecimalInt_00fa10f0 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa1200` | ParseSignedDecimalInt_00fa1200 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa1310` | ParseSignedDecimalInt_00fa1310 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa1420` | ParseSignedDecimalInt_00fa1420 | speculative | Same code as 00fa0d30: skips iswctype-space characters, optional sign, then digits (iswctype digit) accumulated into an int stored through the int* argument. |
| `00fa15c0` | ParseSignedDecimalChar_00fa15c0 | speculative | Char-sized variant of the 00fa0d30 parser: skips iswctype-space characters, optional sign, digits (iswctype digit) accumulated into a char value stored through… |
| `00fa19c0` | CurrentTextParseInt_19c0 | speculative | Thiscall with a byte flag at this and one stack argument. Calls 010176b0 (no arguments) to get the current text pointer; when non-null it parses that text into… |
| `00fa1ab0` | CurrentTextParseInt_1ab0 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa0ed0 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1b40` | CurrentTextParseInt_1b40 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa0fe0 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1bd0` | CurrentTextParseInt_1bd0 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa10f0 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1c60` | CurrentTextParseInt_1c60 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa1200 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1cf0` | CurrentTextParseInt_1cf0 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa1310 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1d80` | CurrentTextParseInt_1d80 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa1420 into the int stack argument; sets the byte at this to 1 when no text. |
| `00fa1f00` | CurrentTextParseChar_1f00 | speculative | Same shape as 00fa19c0: 010176b0 text, parsed by 00fa15c0 into the char stack argument; sets the byte at this to 1 when no text. |
| `00fa20f0` | FieldParseInt_20f0 | speculative | Thiscall with a byte flag at this, a key argument and an int output argument. If the key is 0 it takes the current text from 010176b0. Otherwise, when this+2 i… |
| `00fa2200` | FieldParseInt_2200 | speculative | Same shape as 00fa20f0 with the parser 00fa0ed0. |
| `00fa2260` | FieldParseInt_2260 | speculative | Same shape as 00fa20f0 with the parser 00fa0fe0. |
| `00fa22c0` | FieldParseInt_22c0 | speculative | Same shape as 00fa20f0 with the parser 00fa10f0. |
| `00fa2320` | FieldParseInt_2320 | speculative | Same shape as 00fa20f0 with the parser 00fa1200. |
| `00fa2380` | FieldParseInt_2380 | speculative | Same shape as 00fa20f0 with the parser 00fa1310. |
| `00fa23e0` | FieldParseInt_23e0 | speculative | Same shape as 00fa20f0 with the parser 00fa1420. |
| `00fa25f0` | FieldParseChar_25f0 | speculative | Same shape as 00fa20f0 with the char-sized parser 00fa15c0 writing a char output. |
| `00fa4860` | CopyBoundedCStringAlloc | speculative | Two stack arguments: param_1 (passed on to 01017330) and a source record at param_2 (string pointer at +4, length at +8). Allocates length+1 bytes with 00d7b91… |
| `010888d0` | InitIStringMemberRecord | speculative | Clears this+0, +4, +8, +0xc, +0x18, +0x1c, +0x20 and +0x24 to zero, stores SoeUtil::IString<char> vftable at this+0x10 and points this+0x14 at DAT_01cb80c8. Se… |
| `01088c20` | InitIStringAt8RecordA | speculative | Fastcall: zeroes this+4, +0x10, +0x14, +0x40, +0x44, +0x4c, +0x50 and +0x54; stores SoeUtil::IString<char> vftable at this+8, points this+0xc at DAT_01cb8160 a… |
| `01088da0` | InitIStringAt8RecordB | speculative | Fastcall: zeroes this+0, +4, +0x10, +0x14, +0x1c, +0x20, +0x24 and +0x2c; stores SoeUtil::IString<char> vftable at this+8, points this+0xc at DAT_01cb81b0, and… |
| `01089140` | InitIStringAt0CRecord | speculative | Fastcall: zeroes this+0, +4, +8, +0x14, +0x18, +0x1c and +0x28; stores SoeUtil::IString<char> vftable at this+0xc, points this+0x10 at DAT_01cb8248, stores the… |
| `012a5a10` | AdjustStringPairDelta | speculative | Walks the 0x38-byte element array of the string-pair table at param_1 (begin +4, end +8). Copies each element's first string and compares it with the key strin… |
| `012a5e10` | AppendStringPair | speculative | Copies param_3 (value) and param_2 (key) into two temporary std::strings, builds a pair from them with 012a29e0 (key at pair+0, value at pair+0x1c), appends th… |
| `012a85c0` | ScalarDeletingDtor | speculative | Thiscall with one flag argument: calls 013c2230 on this (frees the buffer whose pointer is in the first word when it is nonzero), frees the object with _free w… |
| `012cd3c0` | StdStringFromInt | speculative | Formats the int param_2 with snprintf %d into a 1000-byte stack buffer, resets the std::string at param_1 to empty (size 0, capacity 15, first byte 0), and ass… |
| `014469c0` | StringGetCopyAt08 | speculative | Returns a copy of the std::string object at this+0x8 through the hidden return buffer param_2: initialises the buffer (size 0, capacity 15, first byte 0) and a… |
| `014481b0` | SetStringAt38FromTemp | speculative | Assigns the std::string passed by value on the stack (pointer and capacity in the stack arguments) into this+0x38, then frees the temporary's heap buffer when … |
| `01448230` | SetStringAt54FromTemp | speculative | Assigns the std::string passed by value on the stack into this+0x54, then frees the temporary's heap buffer when its capacity exceeds 15. |
| `004ca901` | — | unknown | Thiscall(this = holder whose first dword points to {dword length, ushort chars}, index, wide text): returns -1 when index is at or beyond the length, otherwise… |
| `004cac89` | — | unknown | Thiscall(this = holder, flag byte): walks the holder's ushort characters (length at +0, chars from +4), counting logical positions and adding one extra step fo… |
| `004cace1` | — | unknown | Thiscall(this = holder, flag byte): walks the holder's ushort characters, treating CR (0xd) and LF (10) pairs as one break when the flag is set and NUL as an e… |
| `00653a3d` | — | unknown | Two arguments: appends the string arg2 through 004e4e15, replaces characters 0x2e, 0x2f, 0x5c and 0x24 with 0x5f, 0x3a, 0x3a and 0x5f through 004c50f8, appends… |
| `00726270` | — | unknown | thiscall with two stack arguments (key, C string). Searches the chain at this+0xc for bucket key % 100 for a node whose +0x18 equals key. If the key is already… |
| `007a5da0` | — | unknown | Constructor taking 17 stack arguments: stores scalar dwords and bytes at +0x10..+0x67, an IString<char> at +0x1c assigned from param_5 through 00402ce0, an ISt… |
| `0081b710` | — | unknown | Destructor body for the two IString<char> members set up by 0081b680. Stores the IString vtable at this+0x10, releases the buffer at this+0x14 through the refe… |
| `0098deb0` | — | unknown | Forwards this and the supplied wide-character source to 00703620, which assigns a NUL-terminated UTF-16 string into the IString object. |
| `0098e1f0` | — | unknown | Calls the wide formatter with %llu and the two dwords at param_2, then returns the output string data pointer at param_1+8. |
| `0098e220` | — | unknown | Calls the wide formatter with the static format string at 017703e0 and the supplied dword, then returns the output string data pointer at param_1+8. |
| `009a0c10` | — | unknown | Obtains two temporary strings, compares their bytes through the first mismatch or NUL, returns the signed less-than result for the mismatching bytes, and relea… |
| `00d37470` | — | unknown | Cdecl with three arguments: fills a 2048-byte stack buffer with one delimited field from the text argument via 006fbe20 (size 0x800, delimiters param_3, trim 1… |
| `00d7bb00` | — | unknown | Cdecl-style, one argument (char pointer). Assigns one of two constant strings (DAT_0176cfcc when the argument is non-empty, else DAT_0176cfd0) into a local Str… |
| `00d7bc40` | — | unknown | Cdecl-style, two arguments (first an undefined dword passed through, second a char pointer). Same local-string pattern as StringFixed2048_PrefixCopySinkA (00d7… |
| `00d7bd80` | — | unknown | Cdecl-style, two arguments (first an undefined dword, second a char pointer). Same local-string pattern as 00d7bb00 and 00d7bc40 with the same prefix selection… |
| `00d7bee0` | — | unknown | Cdecl-style, one stack argument (object pointer). Assigns the char pointer at object+4 into a local StringFixed<2048>, copies it into a pool block of length 0 … |
| `00d7c010` | — | unknown | Cdecl-style, two stack arguments (a dword passed through and an object pointer). Assigns the char pointer at object+4 into a local StringFixed<2048>, copies it… |
| `010b7ea0` | — | unknown | Cdecl-style with the text pointer and an int* output (stack args). Skips leading characters classified as space via _iswctype, accepts an optional '-' (accumul… |
| `0130fe00` | — | unknown | Cdecl. The first stack argument (param_1) is not read by the decompiled body. The second (param_2) is measured as a C string; the call site in 013210e0 passes … |
| `013cae70` | — | unknown | One-time initialiser (cdecl, no args) for static string tables, guarded by flag DAT_01cbdd84 (set to 1 at the end). Fills int-keyed string maps through 013ca7f… |
| `014600d0` | — | unknown | Clears the output vector at param_1 (+4, +8, +0xc), scans the string at param_2 (begin at +4, length at +0x14) and splits it on the byte at *param_3: at each d… |

### `lib.t4`

**(no class)** (97)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e52b10` | KeyedGet_6c283fc8 | speculative | Thiscall with one stack arg (ret 4). Calls 00e608d0 on [p+0xc]; calls 00e57f70(arg1, 0x6c283fc8) and passes the result with the constant 0x6c283fc8 to 00e57190… |
| `00e52b60` | KeyedGet_48aa1d09 | speculative | Same shape as 00e52b10 with key 0x48aa1d09: calls 00e608d0, converts arg1 via 00e57f70 with that key, passes the result to 00e57190 with the key, returns it; o… |
| `00e52bb0` | KeyedGet_a33fa6e1 | speculative | Same shape as 00e52b10 with key 0xa33fa6e1: calls 00e608d0, converts arg1 via 00e57f70 with that key, passes the result to 00e57190, returns it; on the flagged… |
| `00e52c00` | KeyedGet_eaedf26f | speculative | Same shape as 00e52b10 with key 0xeaedf26f: calls 00e608d0, converts arg1 via 00e57f70 with that key, passes the result to 00e57190, returns it; on the flagged… |
| `00e52c50` | KeyedGet_a88d4ecd | speculative | Same shape as 00e52b10 with key 0xa88d4ecd: calls 00e608d0, converts arg1 via 00e57f70 with that key, passes the result to 00e57190, returns it; on the flagged… |
| `00e52ca0` | KeyedGet_a2e5f03d | speculative | Same shape as 00e52b10 with key 0xa2e5f03d: calls 00e608d0, converts arg1 via 00e57f70 with that key, passes the result to 00e57190, returns it; on the flagged… |
| `00e54050` | Wrapper_00e54050 | speculative | Stack args (7). Builds two 20-byte temporaries with 00e68d80(arg,1) and calls 00e52810 with them and the remaining args, returns its result; the temporaries ar… |
| `00e54110` | Wrapper_00e54110 | speculative | Same shape as 00e54050 forwarding to 00e52950 with the same temporaries; returns its result. |
| `00e541d0` | Wrapper_00e541d0 | speculative | Same shape as 00e54050 forwarding to 00e529f0; returns its result. |
| `00e54290` | Wrapper_00e54290 | speculative | One-arg wrapper: builds one 20-byte temporary with 00e68d80(arg,1), calls 00e52a90 on it, destroys the temporary with 00e68fb0 and returns the result. |
| `00e54310` | Wrapper_00e54310 | speculative | One-arg wrapper: builds a 20-byte temporary with 00e68d80(arg,1), calls 00e52b10 on it, destroys the temporary and returns the result. |
| `00e54390` | Wrapper_00e54390 | speculative | One-arg wrapper forwarding to 00e52b60 with a 20-byte temporary; returns the result. |
| `00e54410` | Wrapper_00e54410 | speculative | One-arg wrapper forwarding to 00e52bb0 with a 20-byte temporary; returns the result. |
| `00e54490` | Wrapper_00e54490 | speculative | One-arg wrapper forwarding to 00e52c00 with a 20-byte temporary; returns the result. |
| `00e54510` | Wrapper_00e54510 | speculative | One-arg wrapper forwarding to 00e52c50 with a 20-byte temporary; returns the result. |
| `00e54590` | Wrapper_00e54590 | speculative | One-arg wrapper forwarding to 00e52ca0 with a 20-byte temporary; returns the result. |
| `00e54610` | Wrapper_00e54610 | speculative | Two-arg wrapper: builds two 20-byte temporaries with 00e68d80, calls 00e528b0 with them and the remaining args, destroys the temporaries and returns the result. |
| `00e546d0` | Wrapper_00e546d0 | speculative | One-arg wrapper forwarding to 00e52ad0 with a 20-byte temporary; returns the result. |
| `00e54750` | CopyResultToHeap_00e6d490 | speculative | Stack args (out ptr, len ptr, arg3..arg6). Builds a 20-byte temporary from arg3, calls 00e6d490 with the temporaries and arg4..arg6, then copies the produced s… |
| `00e54850` | CopyResultToHeap_00e6d670 | speculative | Same copy-to-heap shape as 00e54750 with producer 00e6d670 taking four args (arg4..arg7). |
| `00e54950` | CopyResultToHeap_00e6df30 | speculative | Same copy-to-heap shape with producer 00e6df30 called as (temp, temp, arg4, 0). |
| `00e54a50` | CopyResultToHeap_00e6cc90 | speculative | Copy-to-heap shape; producer is 00e6e230 when arg7 is below 1, else 00e6cc90 with arg7; result copied to a new block of length+2 and NUL-terminated. |
| `00e54b80` | CopyResultToHeap_00e6cad0 | speculative | Copy-to-heap shape with producer 00e6cad0 called with arg4 and arg5; result copied to new block of length+2, NUL-terminated. |
| `00e54c80` | CopyResultToHeap_00e6cf90 | speculative | Copy-to-heap shape with producer 00e6cf90 called with arg4; result copied to new block of length+2, NUL-terminated. |
| `00e54dd0` | Array20FreeBuffer | speculative | Fastcall: when this+0xc is non-null, destroys all elements via 00e54d90 and frees the buffer at this+0xc. |
| `00e54ef0` | Array20PushBack | speculative | Thiscall with one arg (value). When count at +4 equals capacity at +8 it grows via 00e54df0 (capacity doubling, or 1 when empty); then copy-constructs the valu… |
| `00e55850` | StoreGlobal_01bfdacc | speculative | Stdcall one-arg: stores the argument into DAT_01bfdacc and returns 1. |
| `00e560b0` | FieldGetDword_0 | speculative | Fastcall: returns the dword at [ECX] (mov eax,[ecx]; ret). |
| `00e56ca0` | Array24EndPtr | speculative | Fastcall: returns this + 0x74 + (dword at this+0x70) * 0x24, the one-past-end address of an inline array of 0x24-byte elements with count at +0x70. |
| `00e56cb0` | Array24PeekNextDword | speculative | Fastcall: returns the dword at this + 0x70 + (count+1) * 0x24, reading past the current element index with count at +0x70 and stride 0x24. |
| `00e56fc0` | StackFramePush | speculative | Fastcall on a counted array (count at +0, nine dwords per entry): increments the count unless it would exceed 9 (then returns the 00e56ee0 sentinel); otherwise… |
| `00e57190` | HashBucketSetKey | speculative | Thiscall with a char* key and a uint hash: when the key is non-empty it prepares it with 00e660f0, resolves it with 00e66580 into a temporary, selects the buck… |
| `00e572c0` | Array24ElemAt | speculative | Thiscall with an index: returns this + 0x74 + index*0x24 when 0 <= index <= count (count at +0x70); otherwise returns the 00e56ee0 sentinel. |
| `00e572e0` | LinkedListClear | speculative | Fastcall on a singly linked list (head at +0, node next at +4, element pointer at node+0): for each node calls the element vtable slot 0 with flag 1 when non-n… |
| `00e57330` | LinkedListClear | speculative | Same body as 00e572e0: clears a singly linked list by destroying each element with flag 1 and freeing each node. |
| `00e57380` | LinkedListClear | speculative | Same body as 00e572e0: clears a singly linked list by destroying each element with flag 1 and freeing each node. |
| `00e51d80` | — | unknown | Switch on param_2 (0..14; jump table at 0xe51f84 read with dis). Cases 0-5 push 0xc..0x11 and call 00e51ab0 with param_4. Case 6 calls 00e51540(param_1,param_4… |
| `00e51fc0` | — | unknown | Stack-argument function (caller cleans; arg1 = struct p). If arg3 equals the dword at p+0x18 it stores that value at p+0x1c and tail-jumps to 00e50880(p, p+0x2… |
| `00e52260` | — | unknown | Fastcall (ECX = p, saved in ESI). Calls 00e55630 with ECX=p+0x14. If p+0x10 is non-null and the byte at (p+0x10)+0x30 is non-zero (checked via 00e596b0), it ta… |
| `00e52350` | — | unknown | Thiscall with two stack args (ret 8). If p+0x10 is non-null and 00e59790(p+0x10) returns non-zero and the dword at [p+0xc]+8 is non-zero, it calls 00e5e770 wit… |
| `00e52640` | — | unknown | Fastcall (ECX = p, saved in ESI). Calls 00e608d0 with ECX=p+0xc and 00e666d0 with ECX=p+0x14+0x54. If p+0x10 is non-null and the byte at (p+0x10)+0x30 is non-z… |
| `00e52680` | — | unknown | Thiscall (ECX = p). Calls 00e608d0, then calls 00e55900(arg1,arg2) and keeps its result. If p+0x10 is non-null and the flag check via 00e596b0 passes, it calls… |
| `00e52710` | — | unknown | Thiscall with two stack args. Calls 00e608d0, then returns the result of 00e566b0(arg1,arg2). If p+0x10 is non-null and 00e596b0 passes it calls 00e60680(arg1,… |
| `00e52810` | — | unknown | Thiscall with seven stack args. Calls 00e608d0, converts arg1 with 00e57f70 and 00e563f0 to an int pointer, copies that int to a local, calls 00e55990(&local, … |
| `00e528b0` | — | unknown | Same shape as 00e52810: calls 00e608d0, converts arg1 through 00e57f70 and 00e563f0 to a local int, forwards it with the remaining args to 00e55b20 and returns… |
| `00e52950` | — | unknown | Same shape as 00e52810: calls 00e608d0, converts arg1 through 00e57f70 and 00e563f0 to a local int, forwards it with the remaining args to 00e56760 and returns… |
| `00e529f0` | — | unknown | Same shape as 00e52810: calls 00e608d0, converts arg1 through 00e57f70 and 00e563f0 to a local int, forwards it with the remaining args to 00e55c10 and returns… |
| `00e52a90` | — | unknown | Thiscall with one stack arg. Calls 00e608d0, returns the result of 00e56910(arg1); on the p+0x10 flagged path it calls 00e5e300(arg1) first. |
| `00e52ad0` | — | unknown | Same shape as 00e52a90: calls 00e608d0, returns the result of 00e56a80(arg1); on the p+0x10 flagged path it calls 00e5e180(arg1). |
| `00e53f70` | — | unknown | Stack args (char** out, size_t* len, arg3). Calls 00e68d50 and 00e52680(local, arg3), then 00e69d70 for the length. If length is below 1 it stores NULL in *out… |
| `00e54d90` | — | unknown | Fastcall (ECX = this). For each of the count dwords at this+4 it calls the element vtable slot 0 with flag 0 (destruct without free), elements are 0x14 bytes s… |
| `00e54df0` | — | unknown | Fastcall. Uses count at this+8 times 0x14 as a new byte size; when the buffer at this+0xc is empty it stores that size there; otherwise it constructs the exist… |
| `00e54f80` | — | unknown | Builds three 20-byte element vectors through 00e54ef0 (one push per argument), calls 00e6ef20(arg1, vector, local) to produce a string, copies the result into … |
| `00e552c0` | — | unknown | No args, void: calls 00e6f650 and returns. It is called from eight EH unwind funclets, so it is a cleanup step of an unidentified object. |
| `00e55310` | — | unknown | Thiscall (ECX/param_1 = output buffer, param_2 = object pointer): calls 00e71e90(0,0,1), then calls the virtual at vtable slot +0xa4 of *param_2 with (param_1,… |
| `00e55380` | — | unknown | No args: returns &DAT_0175b400 (empty string) when 00e70ff0 is non-zero, otherwise returns the pointer from 00e706b0(0). |
| `00e55400` | — | unknown | Fastcall (ECX = p): when the dword at p+4 is non-zero and 00e70ff0 returns zero, returns 00e70680(0); otherwise returns &DAT_0175b400. |
| `00e555d0` | — | unknown | Thiscall with a flag byte: calls 00e5fe80, then frees this through 00e60fc0 when bit 0 is set; returns this. |
| `00e555f0` | — | unknown | Thiscall with a flag byte: calls 00e608b0, then frees this through 00e60fc0 when bit 0 is set; returns this. |
| `00e55610` | — | unknown | Thiscall with a flag byte: calls 00e65470, then frees this through 00e60fc0 when bit 0 is set; returns this. |
| `00e55630` | — | unknown | No args, void: calls 00e58c20, 00e665e0 and 00e78280 in that order. |
| `00e556b0` | — | unknown | One stack arg: calls 00e66580(arg); returns &DAT_01b86174 when that is null, returns the result of 00e66250() when the node dword at +0x18 equals 3, otherwise … |
| `00e556f0` | — | unknown | Thiscall with three args: computes a flag from the dword at this+0x1c (non-zero and 00e59790 returns non-zero gives 1, else 0), then calls 00e60950 on the obje… |
| `00e55780` | — | unknown | Thiscall with (arg2, char* arg3). When arg3 is non-empty it calls 00e660f0(arg3,0), builds a 16-byte temporary via 00e66d10(temp,arg2), calls 00e60910 and 00e6… |
| `00e55860` | — | unknown | Stdcall one-arg: stores the argument into DAT_01bfdac8 and returns 1. |
| `00e55900` | — | unknown | Thiscall (this, arg1, arg2): when arg1 is non-zero it stores arg2 at this+0x20, calls vtable slot 1 of *this with arg1, calls 00e78fe0, calls 00e666d0 when thi… |
| `00e55990` | — | unknown | Thiscall with eight args: calls 00e78270 and 00e58730 on arg2; when the result is non-null it calls 00e586b0 and 00e58620, then if arg3 is non-zero runs a key … |
| `00e55b20` | — | unknown | Thiscall with seven args: calls 00e78270 and 00e58730 on arg2; when the result is non-null calls 00e586b0 and 00e58620, and when arg3 is non-zero computes 00e5… |
| `00e55c10` | — | unknown | Thiscall with seven args: same outer structure as 00e55b20; when arg3 is zero it calls 00e65d00 on the empty string and 00e58da0, then reads the flag at this+0… |
| `00e55e80` | — | unknown | Two-arg function: returns 0 when 00e70ff0 reports empty; otherwise calls 00e57fa0(arg1), computes a key through 00e708d0 and 00e7aa80, looks it up with 00e65c9… |
| `00e55ea1` | — | unknown | Five-arg function (uint arg2 is returned shifted when not found): calls 00e57fa0(arg1), 00e708d0 and 00e7aa80, looks up with 00e65c90; when found calls the vir… |
| `00e55f10` | — | unknown | Fastcall (ECX = p): when the dword at p+0x2c is zero it tail-jumps to the vtable slot +0xc of *(p+4), otherwise to vtable slot +8 of *(p+4). |
| `00e55f30` | — | unknown | Thiscall with three args: when p+0x2c is zero it calls vtable slot +8 of *(p+4); otherwise it builds a temporary T4ConstString from 00e58700 and 00e6f520, call… |
| `00e56010` | — | unknown | Fastcall (ECX = p): when the dword at p+0x2c is zero it returns the result of vtable slot +4 of *(p+4); otherwise it returns 0 in the low byte. |
| `00e56030` | — | unknown | Fastcall (ECX = p): calls 00e76700, sets the dword at p+0x20 to zero and returns p. |
| `00e560c0` | — | unknown | Fastcall (ECX = p): when the dword at [p] is non-zero it calls 00e76740 and decrements [p]. |
| `00e56100` | — | unknown | Fastcall (ECX = p): calls 00e76740 and sets the dword at [p] to zero. |
| `00e562c0` | — | unknown | One stack arg: 00e660f0(arg,0), resolves a key with 00e66580 into a temporary; null gives &DAT_01b86174, type 3 gives the 00e66250() result, otherwise node+0x1… |
| `00e56360` | — | unknown | One stack arg, void: resolves the argument through 00e66580 (same null, type 3 and node+0x10 cases as 00e562c0) and passes the result to 00e58730. |
| `00e563b0` | — | unknown | Thiscall with one uint arg: selects bucket (arg AND ((1 shl (dword at +0x5c AND 31)) - 1)) of the table at this+0x60, then returns &DAT_01b86174 for an empty b… |
| `00e563f0` | — | unknown | Thiscall with a char* key: returns &DAT_01b86174 for a null or empty key; otherwise 00e660f0(key,0), resolves through 00e66580, falls back through 00e66cf0 and… |
| `00e56570` | — | unknown | Thiscall (this, int* key): returns 0 when *key is negative; when the byte at this+0x28 is zero returns 00e58730(key); otherwise resolves through 00e59110 and 0… |
| `00e56690` | — | unknown | Fastcall with one stack arg: returns the dword at the pointer from 00e56570(arg), or 0 when that pointer is null. |
| `00e566b0` | — | unknown | Thiscall with two stack args (arg2, arg3): when arg3 is non-zero calls 00e55e80(arg3,arg2) and, on success, 00e58080(arg2) and 00e60910; otherwise calls 00e609… |
| `00e56ee0` | — | unknown | No args: sets bit 0 of DAT_01bfdaf8 once and clears _DAT_01bfdaf4 on first use; returns the address of the static sentinel DAT_01bfdad4. |
| `00e56f60` | — | unknown | Fastcall: returns whether its argument equals the value returned by 00e56ee0(). |
| `00e56f80` | — | unknown | Thiscall with one arg: calls 00e75b00(arg, i) for i in 0..9, then stores arg at this+0x16c and copies the dword at arg+0x10 into this+0x24. |
| `00e57000` | — | unknown | Fastcall on the counted nine-dword array: returns the last entry pointer (this + count*9 - 8) when count is at least 1, otherwise the 00e56ee0 sentinel. |
| `00e570e0` | — | unknown | Thiscall with one out-pointer arg: reads the top frame entry (byte at +4 is an operator char, dword at +8 copied to *out), calls 00e58730 on *out, tests the ch… |
| `00e57540` | — | unknown | Fastcall: destroys a singly linked list (same body as 00e572e0), then zeroes head, +4 and +8. Called only from one EH unwind funclet. |
| `00e57550` | — | unknown | Fastcall: same linked-list destruction body as 00e572e0. Called only from one EH unwind funclet. |
| `00e57560` | — | unknown | Fastcall: same linked-list destruction body as 00e572e0. Called only from one EH unwind funclet. |
| `00e57590` | — | unknown | No args, void: zeroes a local and calls 00e572e0 (linked-list clear); called from three EH unwind funclets. |
| `00e57610` | — | unknown | No args, void: zeroes a local and calls 00e57330 (linked-list clear); called from three EH unwind funclets. |
| `00e57690` | — | unknown | No args, void: zeroes a local and calls 00e57380 (linked-list clear); called from three EH unwind funclets. |
| `00e57d90` | — | unknown | Thiscall with one arg: calls 00e80060(arg), loops while 00e80770 returns zero (calling 00e80e30 and 00e801e0(0) each time), calls 00e800a0 and returns the arg. |
| `00e57ee0` | — | unknown | Thiscall with one arg: same shape as 00e57d90 but the loop calls 00e80f40 instead of 00e80e30; calls 00e80060(arg), waits on 00e80770 with 00e801e0(0), then ca… |

### `lib.t4.base`

**T4Base** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e576f0` | T4Base::T4Base | confirmed | Thiscall with three stack args: stores T4Base::vftable at [this], sets this+0x20 from DAT_017649d4 and zeroes the fields at +4..+0x1c, stores arg1 at +0x24 and… |
| `00e57b30` | T4Base::~T4Base | confirmed | Fastcall non-deleting destructor body: stores T4Base::vftable, calls 00e58c20 and 00e76740, clears this+0x70, calls the list clears 00e572e0, 00e57330 and 00e5… |
| `00e57cb0` | T4Base::~T4Base_Deleting | confirmed | Thiscall with a flag byte (ret 4): calls 00e57b30 (T4Base destructor body), frees this through 00e60fc0 when bit 0 is set, returns this. |
| `00e57930` | — | unknown | Thiscall with one int arg: stores T4Base::vftable at [this], sets this+0x20 from DAT_017649d4, zeroes +4..+0x1c, stores 8 at +0x24, clears the byte at +0x28, c… |

**T4SystemConfigDefault** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e57400` | T4SystemConfigDefault::Attach | likely | Thiscall (ECX = this, one arg): stores the arg at this+0xc; when the arg is non-null and its dword at +0xc is non-zero it calls 00e60950 with a flag from this+… |

### `lib.t4.locale`

**(no class)** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e53230` | LocaleIdToCodeUpper | confirmed | Single stack argument: switch on the id minus 1 (range 0..19, verified with dis of the jump table) returning upper-case locale code strings: 1 ZH_CN, 2 DE_DE, … |
| `00e53310` | LocaleIdToCodeLower | confirmed | Same switch shape as 00e53230 (id minus 1, 0..19) returning lower-case locale code strings (zh_cn, de_de, fr_fr, ...); other ids return NULL. |
| `00e6d490` | FormatTimeOfDay | likely | cdecl, stack args: output object (arg2) receives the text, hour (arg3), minute (arg4, negative or 60+ values are normalised by 60) and a 12-hour flag byte (arg… |
| `00e6d670` | FormatTimeOfDaySeconds | likely | cdecl, stack args: output object (arg2), hour (arg3), minute (arg4), second (arg5) and a 12-hour flag byte (arg6); resets the receiver with 00e69320. Seconds a… |
| `00e6cad0` | FormatMonthName | speculative | cdecl (plain ret), four stack args; arg3 is the month number, which must be 1..12 (otherwise returns -1). Makes a temporary SoeString (00e68d50), formats arg4 … |
| `00e533f0` | — | unknown | Void, one char* arg. Copies up to 63 bytes of the argument, lower-cased with _tolower, into a 64-byte local, then compares it with locale codes (de_de, de, en_… |
| `00e6cc90` | — | unknown | cdecl (plain ret), six stack args; returns 0 on success or -1. Validates arg4 (day 1..31) and arg3 (month 1..12) and then formats arg5 with the '%d'-style form… |
| `00e6cf90` | — | unknown | cdecl (plain ret), ECX = receiver, stack arg3 = weekday 1..6 (index weekday-1; values above 6 return -1). Resets the receiver with 00e69320 and appends the wee… |
| `00e6d0d0` | — | unknown | cdecl, no args; returns AL = ',' (0x2c) when the locale tag DE_DE, FR_FR, IT_IT or RU_RU matches (00e69ba0 at position 0), else '.' (0x2e). |
| `00e6d130` | — | unknown | cdecl, no args; returns AL. Tests tags in order: DE_DE, ES_ES, ES_MX, RU_RU return '.' (0x2e); FR_FR returns ' ' (0x20); KO_KR returns 0 (the expression (-1 & … |
| `00e6d1c0` | — | unknown | cdecl, stack args (value int, grouping flag byte); receiver ECX is reset with 00e69320. INT_MIN and INT_MAX are appended as the literal words 'INT_MIN'/'INT_MA… |
| `00e6d350` | — | unknown | cdecl, stack args: receiver ECX reset with 00e69320; double value (arg3), precision count (arg4) and an int argument (arg5). A negative precision is treated as… |
| `00e6d950` | — | unknown | cdecl, stack args: output object (arg2), a duration in seconds (arg3) and a flag byte (arg4). Splits the duration into days ((arg3 / 86400) % 365), hours (/360… |
| `00e6df30` | — | unknown | cdecl, one stack arg. Divides the argument by 60 (signed multiply by 0x10624dd3, shift 6, sign correction) and tail-jumps to 00e6d950 with the quotient as its … |
| `00e6e230` | — | unknown | cdecl, five stack args: output object (arg2) and three numbers (arg3, arg4, arg5); receiver ECX reset with 00e69320. The separator character comes from the tag… |

### `lib.t4.stream`

**T4OutStrStream** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e55490` | T4OutStrStream::~T4OutStrStream | confirmed | Fastcall: reads the vbase offset at [[ECX]+4] and stores T4OutStrStream::vftable at ECX plus that offset, then calls 00e73cd0 (T4BaseStream destructor). |
| `00e56260` | T4OutStrStream::~T4OutStrStream_Deleting | confirmed | Thiscall (ECX = secondary subobject) with a flag byte: adjusts this by -4 to the primary object, stores T4OutStrStream::vftable there, calls 00e73cd0, frees th… |

### `lib.t4.string`

**T4ConstString** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e55210` | T4ConstString::~T4ConstString | confirmed | Fastcall: stores T4ConstString::vftable at [ECX], calls 00e6f540 and returns. |
| `00e55240` | T4ConstString::~T4ConstString_Deleting | confirmed | Thiscall with a flag byte (ret 4): stores T4ConstString::vftable, calls 00e6f540, and frees this through 00e60fc0 when bit 0 is set; returns this. |

**T4String** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e551f0` | T4String::~T4String_Deleting | confirmed | Thiscall with a flag byte (ret 4): calls 00e6f540 (base destructor body) and frees this through 00e60fc0 when bit 0 of the flag is set; returns this. |
| `00e6e9d0` | T4String::T4String | confirmed | thiscall, ECX = the object under construction (kept in ESI, returned in EAX), one stack arg (char* text), ret 4. Constructs the base with 00e68e60(0,0,1), stor… |
| `00e6ea40` | T4String::T4StringLen | confirmed | thiscall, two stack args (text char*, length), ret 8, returns EAX = the object. Constructs the base with 00e68e60(0,0,1), stores T4String::vftable, then copies… |
| `00e6f520` | T4String::T4StringFromCStr | confirmed | thiscall, ECX = the object, stack args (char* text, one more dword that is not used), ret 8, returns EAX = the object. Calls 00e68d80(text, 0) with ECX = the o… |
| `00e6f540` | T4String::DtorBody | confirmed | fastcall, ECX = object, no stack args, no return value. Stores T4String::vftable in the first dword. When 00e6fc90 says the flag 0x40000000 (not owned) is clea… |
| `00e6f5c0` | T4String::NewShallow | confirmed | cdecl, one stack arg (char* text), returns EAX = new T4String or 0. Allocates 0x14 bytes through 00e611f0 with the tag 'T4String::Shallow', file . 4string.cpp … |
| `00e6e950` | T4String::CopyConstruct | likely | thiscall, ECX = destination object (kept in ESI and returned in EAX), one stack arg = source object (kept in EDI), ret 4. Constructs the base with 00e68e60(0,0… |

**T4StringBuffer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e6f650` | T4StringBuffer::DtorBody | confirmed | fastcall, ECX = object, no stack args, no return value. Stores T4StringBuffer::vftable in the first dword. When 00e6fc90 says the flag 0x40000000 is clear it f… |
| `00e6f6d0` | T4StringBuffer::NewShallow | confirmed | cdecl, one stack arg (source object pointer), returns EAX = new T4StringBuffer or 0. Allocates 0xc bytes through 00e611f0 with the tag 'T4StringBuffer::Shallow… |

**T4StringHeap** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e6f8a0` | T4StringHeap::AddBlock | confirmed | fastcall, ECX = pool handle (+0 first block pointer, +4 current node, +8 used count). With no block yet, allocates 0x220c bytes through 00e60d50 with the tag '… |
| `00e6fab0` | T4StringHeap::FreeBlock | confirmed | thiscall, ECX = pool, one stack arg (node pointer), ret 4, returns DAT_0185f358 (0). A non-NULL node that 00e6f860 finds inside the pool is pushed on the free … |
| `00e6fb00` | T4StringHeap::Reallocate | confirmed | thiscall, ECX = pool, two stack args (pointer, new size), ret 8, returns the block pointer. If the pointer is not a pool node (00e6f860 gives NULL) it returns … |
| `00e6fc10` | T4StringHeap::DtorBody | confirmed | fastcall, ECX = pool, no stack args, no return value. Stores T4StringHeap::vftable, and when the in-use count at +0xc is positive and the log stream at +0x1c i… |
| `00e6f790` | — | unknown | fastcall, ECX = pool object (fields +0xc CURRENT, +0x10 MAX, +0x14 TOTAL), no stack args, no return value. Formats the pool summary text with 00e58090 into a 2… |

**T4SyTLAllocator** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e6ed50` | T4SyTLAllocator::Reallocate | likely | fastcall, ECX = allocator (count at +4, capacity at +8, block at +0xc), no stack args, returns EAX = 0 on success or -1. Returns 0 at once for capacity 0. Othe… |
| `00e6ee80` | T4SyTLAllocator::PushElement | speculative | thiscall, ECX = allocator (count at +4, capacity at +8, block at +0xc), one stack arg (source element pointer), ret 4. When count equals capacity it sets capac… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e6ecc0` | ReleaseElementBlock | speculative | fastcall, ECX = owner (count +4, capacity +8, block +0xc), no stack args, returns EAX = 0 or -1. When the block is non-NULL it calls 00e54d90 (returns -1 on fa… |
| `00e6fba0` | FreePoolBlockChain | speculative | fastcall, ECX = pool handle (first dword = head of a block chain at +8 links), no stack args, plain ret. Walks the chain from the head, freeing each block with… |
| `00e56760` | — | unknown | Thiscall with eight args: when arg2 dword is non-negative and arg3 non-zero it builds a temporary T4String (vftable T4String::vftable) and calls 00e55e80 with … |
| `00e56910` | — | unknown | Thiscall with one int arg: calls 00e586b0; for arg 0 it creates an empty string from &DAT_0175b400 via 00e65d00 and 00e58da0, then 00e60950 with a flag from th… |
| `00e56a80` | — | unknown | Thiscall with one arg: calls 00e586b0; for non-zero arg converts it with 00e57f70(arg,0) and 00e65d00, passes to 00e58da0 and returns 0; for zero arg it builds… |
| `00e6bfc0` | — | unknown | stdcall-style: one stack arg (char*) read first with ECX not used at entry, ret 4 (callee pops 4). A NULL argument returns -1. Otherwise it builds two temporar… |
| `00e6ef20` | — | unknown | thiscall-style: ECX = owner (block pointer at +0x14 used to fetch a service through 00e7cc80), stack args param_2 (forwarded to the service method) and param_3… |
| `00e6f0a0` | — | unknown | thiscall, ECX = handle whose first dword points at the element block (count at +4, block at +0xc), one stack arg (param_2), no return value. Walks the elements… |
| `00e6f110` | — | unknown | stdcall-style: one stack arg (pointer to a pair whose first dword is the list head), ret 4 (callee pops 4). Walks a singly linked list of nodes (next at +0x10,… |
| `00e6f2f0` | — | unknown | thiscall, ECX = this (count +4, capacity +8, block +0xc), one stack arg (param_2), plain ret (the caller pops the argument). Releases the element block like Re… |
| `00e6f3d0` | — | unknown | thiscall, ECX = this (count +4, capacity +8, block +0xc), two stack args (param_2, param_3; the caller pops them, plain ret). Releases the element block as Rel… |

### `lib.t4.strutil`

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e52cf0` | CharIsDigit | speculative | Cdecl-style one-byte argument: returns 1 when the argument is ASCII (>= 0) and _isdigit returns non-zero, else 0. |
| `00e52d50` | CharIsAlnumOrNonAscii | speculative | One-byte argument: returns 0 when the argument is ASCII and _isalnum returns 0, otherwise returns 1 (so non-ASCII bytes return 1). |
| `00e52d80` | CharToLowerAscii | speculative | One-byte argument: returns 0 when the argument is negative (non-ASCII), otherwise returns _tolower of the argument. |
| `00e52db0` | CharInSymbolSet | speculative | One-byte argument: switch returning 1 for 0x24, 0x25, 0x26, 0x2a, 0x2b, 0x3c, 0x3d, 0x3e, 0x40, 0x5e and 0x7e, else 0. |
| `00e52ed0` | CharIsAsciiAlnum | speculative | One-byte argument: switch returns 1 for the ASCII digits 0x30-0x39, uppercase 0x41-0x5a and lowercase 0x61-0x7a, else 0. |
| `00e52f50` | ParseArgPrefixDecimal | speculative | Thiscall-style with (char* text, int* out). Matches the case-insensitive prefix a,r,g using _tolower; then requires 00e67560 to accept the next byte; then accu… |
| `00e53010` | ScanSkipWordDashToSymbol | speculative | Cursor function on int* (advances *p). Loops: skips non-ASCII bytes and ASCII alnum or '-'; returns the first other ASCII byte (sign-extended byte in EAX) and … |
| `00e53070` | ScanSkipDigits | speculative | Cursor function on int* (advances *p): skips ASCII digits via _isdigit; returns the byte at the cursor (non-ASCII returns as is). |
| `00e530a0` | ScanSkipWordSpaceDash | speculative | Cursor function on int* (advances *p): skips ASCII alnum, whitespace (_isspace), '-' and non-ASCII bytes; returns the first other ASCII byte, cursor left on it. |
| `00e530f0` | ScanSkipSpaces | speculative | Cursor function on int* (advances *p): skips ASCII whitespace via _isspace and returns the byte at the cursor (non-ASCII returns as is). |
| `00e538e0` | CharMapEscape | speculative | One-byte argument switch: 0x21 and 0x2a return 0x24; 0x23 returns 0x5c; 0x3a returns 0x3a; any other byte returns 0. |
| `00e53a20` | ScanBracketTagEnd | speculative | Stack args (text, int* consumed). If the text starts with ':' or backslash, skips a run of ASCII digits via 00e53070 and reports the consumed length. If it sta… |
| `00e53b80` | ScanToTerminatorSet | speculative | Void, cursor arg (int* p). Repeatedly calls 00e530a0 on the cursor and advances it until the returned byte is NUL, ',', '{', '}' or ')'; leaves the cursor on t… |
| `00e53d90` | ScanWordOrDollarStart | speculative | Bool, cursor arg: calls 00e53b80, optionally skips a comma then 00e530f0; returns false for NUL, '{', '}' or ')'; returns false for non-ASCII bytes; returns tr… |
| `00e53e10` | ScanSkipToTerminatorLen | speculative | Stack args (cursor value, int* len). Calls 00e53b80 on a local copy of the cursor and stores the consumed byte count (new cursor minus start) into *len. |
| `00e53e40` | ScanTokenNextChar | speculative | Stack args (char* text, int* len). If the first byte is ASCII, not alnum and not '-', advances one byte and skips spaces with 00e530f0; otherwise scans with 00… |
| `00e53f20` | ScanWordStartLen | speculative | Stack args (cursor, int* len). Calls 00e53d90 on a local cursor and stores the consumed byte count into *len. |
| `00e57cd0` | — | unknown | No args (returns the result in EAX): when 00e70ff0 reports non-empty and 00e70b30 returns DAT_0185f358 it reads the bytes of 00e70680(0) and classifies high-bi… |
| `00e57e20` | — | unknown | No args (returns in EAX): same head as 00e57cd0 (00e70ff0, 00e70b30 compared with DAT_0185f358, then byte scan of 00e70680(0) with high-bit classification); th… |

### `lib.t4.tagapi`

**T4Defs** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e52000` | T4Defs::~T4Defs | confirmed | Fastcall: stores T4Defs::vftable at [ECX] and returns. It has no flag test and frees nothing. |
| `00e52010` | T4Defs::~T4Defs_Deleting | confirmed | Stdcall-style with one flag byte argument (ret 4) and this in ECX: stores T4Defs::vftable at [this]; when bit 0 of the flag is set it calls _free(this) through… |

**t4_tag::T4** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e52120` | t4_tag::T4::T4 | confirmed | Fastcall ctor: stores t4_tag::T4::vftable at [this], sets this+4 to 8, zeroes this+8 through this+0x14, calls 00e58460 and 00e581e0, returns this. |
| `00e52190` | t4_tag::T4::~T4 | confirmed | Fastcall non-deleting destructor body with an SEH frame: stores t4_tag::T4::vftable; when this+0x14 is non-null calls its vtable slot 0 with flag 1; calls 00e5… |
| `00e523f0` | t4_tag::T4::T4_CreateRuntimeEngine | confirmed | Thiscall ctor with three stack args: stores t4_tag::T4::vftable, this+4=arg1, zeroes this+8..0x14, calls 00e58460 and 00e581e0, allocates 0x230 bytes with tag … |
| `00e524e0` | t4_tag::T4::~T4_Deleting | confirmed | Thiscall with one flag byte argument (ret 4): calls the destructor body 00e52190; when bit 0 of the flag is set it calls _free(this); returns this. |
| `00e52500` | t4_tag::T4::T4_CreateWithLookup | confirmed | Thiscall ctor with four stack args: stores t4_tag::T4::vftable, clamps args 3 and 4 to >=0, allocates the 0x230-byte engine (tag T4RuntimeEngine, line 0x53) an… |

### `lib.t4.trace`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e56cc0` | T4DumpTreeXml | confirmed | Formats T4 stack and element tree markers with _sprintf (stack level/max, element level, object, closing tags and entering-next-level text); its output calls g… |

### `lib.t4macro`

**T4FileManager** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e83a00` | T4FileManager::T4FileManager | confirmed | fastcall, ECX = this: calls 00e7f690, zeroes the dwords at [this+4] and [this+8], stores T4FileManager::vftable at [this], returns this in EAX. |
| `00e83a20` | T4FileManager::~T4FileManager | confirmed | fastcall, ECX = this: stores T4FileManager::vftable at [this]; when [this+8] (FILE*) is nonnull closes it with _fclose and clears it; when [this+4] is nonzero … |
| `00e83ca0` | T4FileManager::ScalarDeletingDtor | confirmed | Virtual slot [0] of T4FileManager::vftable (thiscall, one stack argument = flag byte). Calls the destructor body 00e83a20, then when bit 0 of the flag is set f… |
| `00e83cc0` | T4FileManager::T4FileManager_Open | confirmed | thiscall, ECX = this, two stack arguments (path, mode; ret 8): calls 00e7f690, stores T4FileManager::vftable at [this], sets [this+8] to _fopen(path, mode), co… |
| `00e83b10` | T4FileManager::Slot1_Close | likely | Virtual slot [1] of T4FileManager::vftable (fastcall, ECX = this). If [this+8] (FILE*) is nonnull, returns _fclose of it after clearing [this+8]; otherwise ret… |
| `00e83b40` | T4FileManager::Slot2_Error | likely | Virtual slot [2] of T4FileManager::vftable (fastcall, ECX = this). If [this+8] (FILE*) is nonnull returns _ferror of it; otherwise returns -1. |
| `00e83b60` | T4FileManager::Slot3_ReadLine | likely | Virtual slot [3] of T4FileManager::vftable (thiscall, two stack arguments = buffer and count). If [this+8] (FILE*) is nonnull returns the result of 00d8b641 (f… |
| `00e83bc0` | T4FileManager::Slot8_Seek | likely | Virtual slot [8] of T4FileManager::vftable (thiscall, two stack arguments = offset and whence). If [this+8] (FILE*) is nonnull returns _fseek of the stream wit… |
| `00e83bf0` | T4FileManager::Slot9_Tell | likely | Virtual slot [9] of T4FileManager::vftable (fastcall, ECX = this, no stack arguments). If [this+8] (FILE*) is nonnull returns _ftell of it as a long; otherwise… |
| `00e83c10` | T4FileManager::Slot10_Write | likely | Virtual slot [10] of T4FileManager::vftable (thiscall, three stack arguments = buffer, size, count). If [this+8] (FILE*) is nonnull returns _fwrite of the buff… |
| `00e83c40` | T4FileManager::Slot11_Flush | likely | Virtual slot [11] of T4FileManager::vftable (fastcall, ECX = this, plain ret). If [this+8] (FILE*) is nonnull calls _fflush on it; returns nothing. |
| `00e83d40` | T4FileManager::Slot6_OpenOrNull | likely | Virtual slot [6] of T4FileManager::vftable as read from the binary, with two stack arguments (path, mode; ret 8) and no use of its ECX as an object. Allocates … |
| `00e83b90` | T4FileManager::Slot7_ReadBlock | speculative | Virtual slot [7] of T4FileManager::vftable (thiscall, three stack arguments). If [this+8] (FILE*) is nonnull returns the result of 00d8ae0b called with the thr… |
| `00e83c50` | T4FileManager::Slot5_SeekThenBlock | speculative | Virtual slot [5] of T4FileManager::vftable (thiscall, arguments: offset, buffer, count). If [this+8] (FILE*) is nonnull: calls _fseek on the stream with the of… |

**T4LoggingFormatter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e84f20` | T4LoggingFormatter::ScalarDeletingDtor | confirmed | Virtual slot [0] of T4LoggingFormatter::vftable (thiscall, one stack argument = flag byte): stores T4LoggingFormatter::vftable at [this] and, when bit 0 of the… |
| `00e85d20` | T4LoggingFormatter::ScalarDeletingDtorViaJSON | confirmed | Virtual slot [0] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument = flag byte): stores T4LoggingFormatter::vftable at [this] (the base vtable),… |

**T4LoggingFormatterJSON** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e85540` | T4LoggingFormatterJSON::Slot5_AppendSlotString | likely | Virtual slot [5] of T4LoggingFormatterJSON::vftable (thiscall, two stack arguments). When the second argument is nonzero appends ',"SLOT":"', then the text fro… |
| `00e85590` | T4LoggingFormatterJSON::Slot4_AppendSlotChar | likely | Virtual slot [4] of T4LoggingFormatterJSON::vftable (two stack arguments: output stream, byte). Appends ',"SLOT":"', then the byte through the table at DAT_018… |
| `00e85a10` | T4LoggingFormatterJSON::Slot20_AppendLevelInt | likely | Virtual slot [20] of T4LoggingFormatterJSON::vftable (thiscall, two stack arguments). Appends ',"LEVEL":"', formats the second argument with sprintf '%d' into … |
| `00e85b50` | T4LoggingFormatterJSON::Slot25_AppendNamedInt | likely | Virtual slot [25] of T4LoggingFormatterJSON::vftable (ret 0xc by its frame, three stack arguments). Appends the constant at 0x01860700, the name text (second a… |
| `00e854b0` | T4LoggingFormatterJSON::Slot1_AppendConst017a075c | speculative | Virtual slot [1] of T4LoggingFormatterJSON::vftable (ret 4, one stack argument = output stream, loaded into ECX). Appends the constant text at 0x017a075c with … |
| `00e854d0` | T4LoggingFormatterJSON::Slot2_AppendConst017704d4 | speculative | Virtual slot [2] of T4LoggingFormatterJSON::vftable (ret 4, one stack argument = output stream, loaded into ECX). Appends the constant text at 0x017704d4 with … |
| `00e85bf0` | T4LoggingFormatterJSON::Slot24_AppendNamedFormattedInt | speculative | Virtual slot [24] of T4LoggingFormatterJSON::vftable (three stack arguments). Same shape as Slot25_AppendNamedInt at 00e85b50, but the third argument is format… |
| `00e854f0` | — | unknown | two stack arguments (stream, value; ret 8, only the stream is used by the body). Appends the literal '"LOG_TYPE":"', then the text returned by 00e852c0(0x10000… |
| `00e855f0` | — | unknown | Virtual slot [6] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"ID":"', then the text from 00e57f70… |
| `00e85640` | — | unknown | Virtual slot [7] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"NAME":"', then the text from 00e57f… |
| `00e85690` | — | unknown | Virtual slot [13] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"ATTRIBUTE":"', then the text from … |
| `00e856e0` | — | unknown | Virtual slot [14] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is not -1 appends ',"COUNT":"', the decimal value throug… |
| `00e85730` | — | unknown | Virtual slot [15] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"GENDER":"', the decimal value thro… |
| `00e85780` | — | unknown | Virtual slot [16] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"TYPE":"', the decimal value throug… |
| `00e857d0` | — | unknown | Virtual slot [17] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is not -3 appends ',"HONOR":"', the decimal value throug… |
| `00e85820` | — | unknown | Virtual slot [18] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"AGE":"', the decimal value through… |
| `00e85870` | — | unknown | two stack arguments (param_1 unused, param_2 = first error node; ret 8). When the node is nonzero appends ',"ERROR_INFO":[' and, for each node in the chain (ne… |
| `00e85a80` | — | unknown | Virtual slot [21] of T4LoggingFormatterJSON::vftable (thiscall, one float argument). When the float differs from the global DAT_017649d4 appends ',"FACTION":"'… |
| `00e85ae0` | — | unknown | Virtual slot [26] of T4LoggingFormatterJSON::vftable (thiscall, a text argument and a bool byte). Appends the constant 0x01860700, the text argument, the const… |
| `00e85c90` | — | unknown | Virtual slot [22] of T4LoggingFormatterJSON::vftable (two stack arguments). Appends the constant at 0x01860700, the name text (second argument), the constant a… |
| `00e85e00` | — | unknown | Virtual slot [9] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"TEXT":"', then the escaped text of … |
| `00e85e50` | — | unknown | Virtual slot [8] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument = text pointer). When nonnull builds a SoeBuffer temporary from the text (00e… |
| `00e85f00` | — | unknown | Virtual slot [10] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"TRANSLATION":"', then the escaped … |
| `00e85f50` | — | unknown | Virtual slot [11] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"PATTERN":"', then the escaped text… |
| `00e85fa0` | — | unknown | Virtual slot [12] of T4LoggingFormatterJSON::vftable (thiscall, one stack argument). When the argument is nonzero appends ',"RESULT":"', then the escaped text … |
| `00e85ff0` | — | unknown | Virtual slot [27] of T4LoggingFormatterJSON::vftable (thiscall, two stack arguments: name text, T4 text object; ret 0xc by its frame). When the text object is … |
| `00e860d0` | — | unknown | thiscall with two stack arguments (name text, value; ret 0xc by its frame); ECX is not used by the body. Appends the constant at 0x01860700, the name text, the… |

**T4LoggingWriter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e84370` | T4LoggingWriter::T4LoggingWriter | confirmed | thiscall, ECX = this, two stack arguments (ret 8): stores T4LoggingWriter::vftable at [this], stores arg1 at [this+4] and arg2 at [this+8], sets [this+0xc] to … |
| `00e844d0` | T4LoggingWriter::~T4LoggingWriter | confirmed | fastcall, ECX = this: stores T4LoggingWriter::vftable at [this]; when [this+0x10] is nonnull releases it (virtual slot [1], then slot [0] with flag 1) and clea… |
| `00e84550` | T4LoggingWriter::ScalarDeletingDtor | confirmed | Virtual slot [0] of T4LoggingWriter::vftable (thiscall, one stack argument = flag byte): calls 00e844d0 (destructor body), and when bit 0 of the flag is set fr… |
| `00e845b0` | T4LoggingWriter::Slot1_WriteLine | likely | Virtual slot [1] of T4LoggingWriter::vftable (thiscall, ECX = writer, two stack arguments: message SoeBuffer*, BOM flag). If the message is nonempty (00e70ff0 … |

**T4Macro** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e86d10` | T4Macro::T4Macro | confirmed | thiscall, ECX = this, three stack arguments (id, name text, value): stores the id at [this+4], stores T4Macro::vftable at [this], builds the name in the SoeBuf… |
| `00e86d70` | T4Macro::~T4Macro | confirmed | fastcall, ECX = this: stores T4Macro::vftable at [this], releases the pointer array at [this+0x18] through its thunk to 00e86ae0 (PtrArrayReleaseAll), then cal… |

**T4MacroActor** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81c10` | T4MacroActor::T4MacroActor | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), stores T4MacroActor::vftable at [th… |
| `00e81c90` | T4MacroActor::Slot3_LookupIndex | likely | Virtual slot [3] of T4MacroActor::vftable (thiscall, no stack arguments). Calls 00e86df0 on this and passes the result to 00e68ae0; when that is nonzero it cal… |
| `00e81d40` | T4MacroActor::Slot5_ForwardToMember14 | speculative | Virtual slot [5] of T4MacroActor::vftable (thiscall, one stack argument, ret 4). If [this+0x2c] is 3 and [this+0x28] is >=0, calls virtual slot +8 of the objec… |

**T4MacroAgree** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81d80` | T4MacroAgree::T4MacroAgree | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroAgree::vftable a… |
| `00e81db0` | T4MacroAgree::Slot5_ForwardArgToMember14Slot2 | speculative | Virtual slot [5] of T4MacroAgree::vftable (thiscall, one stack argument, ret 4): calls virtual slot [2] (vtable+8) of the object at [this+0x14] with the argume… |
| `00e82f40` | T4MacroAgree::Slot3_ArgsToMembers | speculative | Virtual slot [3] of T4MacroAgree::vftable (fastcall, ECX = this, plain ret). Calls 00e86df0 and 00e68ae0 on the result; when that is nonzero calls 00e79700(1) … |

**T4MacroBase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e86dd0` | T4MacroBase::Slot1_GetMember14Field54 | speculative | Virtual slot [1] of T4MacroBase::vftable, shared with the derived T4Macro vtables (fastcall, ECX = this, no stack arguments, plain ret): returns the dword at +… |
| `00e86e10` | T4MacroBase::Slot0_GetNodeField0C | speculative | Virtual slot [0] of T4MacroBase::vftable (fastcall, ECX = this, no stack arguments). The body is not a destructor: returns the dword at +0xc of the object at [… |

**T4MacroCapitalize** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81f50` | T4MacroCapitalize::T4MacroCapitalize | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroCapitalize::vfta… |
| `00e83470` | T4MacroCapitalize::Slot5_CapOrUncapDispatch | confirmed | thiscall, one stack argument (ret 4 on every path). Resolves the first argument object (else DAT_01bfdd68); when its +8 pointer or the dword it points to is nu… |
| `00e81f90` | T4MacroCapitalize::Slot6_Forward796d0 | speculative | Virtual slot [6] of T4MacroCapitalize::vftable (fastcall, ECX = this, no stack arguments, plain ret). If [[this+0x14]+0x58] is nonzero, calls 00e796d0 with the… |
| `00e83440` | T4MacroCapitalize::Slot3_PassFirstArg | speculative | Virtual slot [3] of T4MacroCapitalize::vftable (fastcall, ECX = this, no stack arguments, returns 0). Calls 00e79560 with ECX = [this+0x14] and the value **[th… |

**T4MacroCase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec9270` | T4MacroCase::Ctor | likely | Thiscall with three stack arguments, ret 0xc. Calls 00e86d10 (T4Macro base constructor) with the three arguments, stores the third stack argument at [this+0x28… |
| `00ec92c0` | T4MacroCase::CallSubobject28WithArg2 | speculative | Virtual slot [11] of T4MacroCase (13 bytes, no stack arguments): loads ECX from [this+0x28], pushes 2, calls 00eb67f0, returns 0 in EAX. |

**T4MacroDefiniteArticle** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec9310` | T4MacroDefiniteArticle::Ctor | likely | Thiscall with three stack arguments, ret 0xc. Calls 00e86d10 (T4Macro base constructor) with the three arguments, stores the third stack argument at [this+0x28… |
| `00ec9360` | T4MacroDefiniteArticle::CallSubobject28WithArg3 | speculative | Virtual slot [11] of T4MacroDefiniteArticle (13 bytes, no stack arguments): loads ECX from [this+0x28], pushes 3, calls 00eb67f0, returns 0 in EAX. |
| `00ec9370` | T4MacroDefiniteArticle::TailCallSubobject28 | speculative | Virtual slot [12] of T4MacroDefiniteArticle (8 bytes): loads ECX from [this+0x28] and jumps (tail jump, no return of its own) to 00eb7700. |

**T4MacroDemonstrative** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec9490` | T4MacroDemonstrative::Ctor | likely | Thiscall with four stack arguments, ret 0x10. Calls 00e86d10 (T4Macro base constructor) with the first three arguments, stores the third at [this+0x28], sets [… |
| `00ec94e0` | T4MacroDemonstrative::CallSubobject28WithArg3 | speculative | Virtual slot [11] of T4MacroDemonstrative (13 bytes, no stack arguments): loads ECX from [this+0x28], pushes 3, calls 00eb67f0, returns 0 in EAX. |
| `00ec94f0` | T4MacroDemonstrative::AppendTextThenForward | speculative | Virtual slot [12] of T4MacroDemonstrative (38 bytes): thiscall with one stack argument, ret 4. Calls 00e74e10 with ECX=the argument (a stream) and two stack ar… |

**T4MacroGender** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e819a0` | T4MacroGender::T4MacroGender | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroGender::vftable … |
| `00e82b30` | T4MacroGender::Slot3_SetStateFromCode | likely | Virtual slot [3] of T4MacroGender::vftable (fastcall, ECX = this, no stack arguments, plain ret). Calls 00e79560 with ECX = [this+0x14] and the argument value … |
| `00e819f0` | T4MacroGender::ForwardArgToMember14Slot2 | speculative | Virtual slot [5] of T4MacroGender::vftable (thiscall, one stack argument, ret 4): calls virtual slot [2] (vtable+8) of the object at [this+0x14], passing the a… |

**T4MacroGenderAndRole** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81a40` | T4MacroGenderAndRole::T4MacroGenderAndRole | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), clears the byte at [this+0x28] and … |
| `00e82bb0` | T4MacroGenderAndRole::Slot2_ValidateArguments | confirmed | Virtual slot [2] of T4MacroGenderAndRole::vftable (fastcall, ECX = this, plain ret). Calls 00e871c0 as a guard; then takes the first argument object (at [this+… |
| `00e81ab0` | T4MacroGenderAndRole::ForwardSlot0xC4 | speculative | Virtual slot [5] of T4MacroGenderAndRole::vftable (thiscall, one stack argument, ret 4). If byte [this+0x28] is zero, calls 00e74250 with ECX = the argument. O… |
| `00e82df0` | — | unknown | Virtual slot [3] of T4MacroGenderAndRole::vftable (fastcall, ECX = this, no stack arguments). Stores the first argument object (at [this+0x1c], else DAT_01bfdd… |

**T4MacroIndefiniteArticle** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec93b0` | T4MacroIndefiniteArticle::Ctor | likely | Thiscall with four stack arguments, ret 0x10. Calls 00e86d10 (T4Macro base constructor) with the first three arguments, stores the third at [this+0x28], sets [… |
| `00ec9400` | T4MacroIndefiniteArticle::ChooseSubobjectArg2Or0 | speculative | Virtual slot [11] of T4MacroIndefiniteArticle (53 bytes, ECX=this, no stack arguments). If the byte at [this+0x34] is zero and 00e78e90(ECX=[this+0x28]) minus … |
| `00ec9440` | T4MacroIndefiniteArticle::ForwardFlagAndArg | speculative | Virtual slot [12] of T4MacroIndefiniteArticle (21 bytes): thiscall with one stack argument, ret 4. Pushes the zero-extended byte at [this+0x34] and the stack a… |

**T4MacroNoun::MacroNounBase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e822e0` | T4MacroNoun::MacroNounBase::Slot6_Forward796d0 | speculative | Virtual slot [6] of T4MacroNoun::MacroNounBase::vftable, shared by T4MacroNoun::MacroPhrase and T4MacroNoun::MacroPlayerName (slot [6] of each) (fastcall, ECX … |

**T4MacroNoun::MacroPhrase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e82310` | T4MacroNoun::MacroPhrase::MacroPhrase | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroNoun::MacroPhras… |
| `00e82340` | T4MacroNoun::MacroPhrase::Slot3_ForwardThisPlus18ToMember30 | speculative | Virtual slot [3] of T4MacroNoun::MacroPhrase::vftable (fastcall-style, ECX = this, no stack arguments, plain ret). Calls virtual +0x30 of the object at [this+0… |
| `00e82360` | T4MacroNoun::MacroPhrase::Slot5_ForwardArgToMember14Slot2 | speculative | Virtual slot [5] of T4MacroNoun::MacroPhrase::vftable (thiscall, one stack argument, ret 4): calls virtual slot +8 of the object at [this+0x14] with the argume… |

**T4MacroNoun::MacroPlayerName** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e823c0` | T4MacroNoun::MacroPlayerName::MacroPlayerName | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroNoun::MacroPlaye… |
| `00e823f0` | T4MacroNoun::MacroPlayerName::Slot3_ReadStateThenForward34 | speculative | Virtual slot [3] of T4MacroNoun::MacroPlayerName::vftable (fastcall-style, ECX = this, no stack arguments, plain ret). If [this+0x20] is nonzero, calls 00e7971… |
| `00e82430` | T4MacroNoun::MacroPlayerName::Slot5_ForwardArgToMember14Slo… | speculative | Virtual slot [5] of T4MacroNoun::MacroPlayerName::vftable (thiscall, one stack argument, ret 4): calls virtual slot +8 of the object at [this+0x14] with the ar… |

**T4MacroNumber** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81ff0` | T4MacroNumber::T4MacroNumber | confirmed | thiscall, ECX = this, three stack arguments plus a byte flag (ret 0x10): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), stores the flag b… |
| `00e82020` | T4MacroNumber::Slot5_EmitViaT4OutStrStream | likely | Virtual slot [5] of T4MacroNumber::vftable (thiscall, one stack argument, ret 4). Calls virtual +0xc of the member at [this+0x14] with the argument (ECX = memb… |
| `00e83530` | — | unknown | Virtual slot [3] of T4MacroNumber::vftable (fastcall, ECX = this, no stack arguments, returns 0). Calls 00e79560 with ECX unchanged and the value **[this+0x1c]… |

**T4MacroPerson** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81ea0` | T4MacroPerson::T4MacroPerson | confirmed | thiscall, ECX = this, three stack arguments: calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), clears the byte at [this+0x28] and the dword … |
| `00e83060` | T4MacroPerson::Slot2_ValidateArguments | confirmed | Virtual slot [2] of T4MacroPerson::vftable (fastcall, ECX = this, no stack arguments). Calls the guard 00e871c0, then takes the first argument object (at [this… |
| `00e81f10` | T4MacroPerson::Slot5_ForwardSlot0xC4OrStreamReset | speculative | Virtual slot [5] of T4MacroPerson::vftable (thiscall, one stack argument, ret 4). If byte [this+0x28] is nonzero it loads [this+0x2c], reads the dword at +8 of… |
| `00e83220` | — | unknown | Virtual slot [3] of T4MacroPerson::vftable (fastcall, ECX = this, no stack arguments, returns 0). Looks up keys 0xa88d4ecd, 0x6c283fc8 and 0x48aa1d09 in table … |

**T4MacroPluralize** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e82140` | T4MacroPluralize::T4MacroPluralize | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroPluralize::vftab… |
| `00e82170` | T4MacroPluralize::Slot5_ForwardArgToMember14Slot2 | speculative | Virtual slot [5] of T4MacroPluralize::vftable (thiscall, one stack argument, ret 4): calls virtual slot +8 of the object at [this+0x14] with the argument and r… |
| `00e83560` | T4MacroPluralize::Slot3_PassFirstArgSetState2 | speculative | Virtual slot [3] of T4MacroPluralize::vftable (fastcall, ECX = this, no stack arguments, returns 0). Calls 00e79560 with the first argument value (**[this+0x1c… |

**T4MacroPossessive** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec9550` | T4MacroPossessive::Ctor | likely | Thiscall with four stack arguments, ret 0x10. Calls 00e86d10 (T4Macro base constructor) with the first three arguments, stores the third at [this+0x28], sets [… |
| `00ec9ab0` | T4MacroPossessive::CallHelperThenSubobjectArg0 | speculative | Virtual slot [11] of T4MacroPossessive (22 bytes, ECX=this, no stack arguments): calls 00ec9850 with ECX=this, then loads ECX=[this+0x28] and calls 00eb67f0 wi… |
| `00ec9ad0` | T4MacroPossessive::CallSubobjectWithTwoChildren | speculative | Virtual slot [12] of T4MacroPossessive (89 bytes): thiscall, one stack argument, ret 4. Selects two children from the array at [this+0x1c] by the indices at [t… |

**T4MacroPronoun** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec95e0` | T4MacroPronoun::Ctor | likely | Thiscall with three stack arguments, ret 0xc. Calls 00e86d10 (T4Macro base constructor) with the three arguments, stores the third argument at [this+0x28], set… |
| `00ec9620` | T4MacroPronoun::EmitViaSubobjectWhenFlag | speculative | Virtual slot [5] of T4MacroPronoun (53 bytes): thiscall with one stack argument, ret 4. Calls 00eb9eb0 with ECX=[this+0x28] and the argument, then 00e86df0 wit… |

**T4MacroSeparable** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec96a0` | T4MacroSeparable::Ctor | likely | Thiscall with three stack arguments, ret 0xc. Calls 00e86d10 (T4Macro base constructor) with the three arguments, stores the third at [this+0x28], sets [this+0… |
| `00ec96e0` | T4MacroSeparable::EmitViaSubobjectWhenFlag | speculative | Virtual slot [5] of T4MacroSeparable (53 bytes): thiscall with one stack argument, ret 4. Calls 00eb6480 with ECX=[this+0x28] and the argument, then 00e86df0 w… |
| `00ec9b30` | T4MacroSeparable::CountCheckThenEmitChild | speculative | Virtual slot [8] of T4MacroSeparable (67 bytes, __fastcall, ECX=this, no stack arguments). Reads the byte count at [this+0x19]: returns 0x5c67 when it is 0 and… |

**T4MacroUpperLower** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e821d0` | T4MacroUpperLower::T4MacroUpperLower | confirmed | thiscall, ECX = this, three stack arguments (ret 0xc): calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroUpperLower::vfta… |
| `00e82200` | T4MacroUpperLower::Slot5_ForwardArgThenCodeEorF | speculative | Virtual slot [5] of T4MacroUpperLower::vftable (thiscall, one stack argument, ret 4). Calls virtual +8 of the member at [this+0x14] with the argument, then cal… |
| `00e82250` | T4MacroUpperLower::Slot6_Forward796d0 | speculative | Virtual slot [6] of T4MacroUpperLower::vftable (fastcall, ECX = this, no stack arguments, plain ret). If [[this+0x14]+0x58] is nonzero, calls 00e796d0 with the… |
| `00e835b0` | T4MacroUpperLower::Slot3_PassFirstArg | speculative | Virtual slot [3] of T4MacroUpperLower::vftable (fastcall, ECX = this, no stack arguments, returns 0). Calls 00e79560 with the value **[this+0x1c] when [this+0x… |

**T4MacroVerb** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81e10` | T4MacroVerb::T4MacroVerb | confirmed | thiscall, ECX = this, three stack arguments: calls the T4Macro base constructor 00e86d10(this, arg1, arg2, arg3), then stores T4MacroVerb::vftable at [this]; r… |
| `00e81e40` | T4MacroVerb::Slot5_ForwardArgToMember14Slot0x20 | speculative | Virtual slot [5] of T4MacroVerb::vftable (thiscall, one stack argument, ret 4): calls virtual slot +0x20 of the object at [this+0x14] with the argument and ret… |
| `00e82fd0` | T4MacroVerb::Slot3_LookupClassifyForward | speculative | Virtual slot [3] of T4MacroVerb::vftable (fastcall, ECX = this, plain ret). Calls 00e79700(1) on [this+0x14] when 00e68ae0 of 00e86df0 is nonzero. Looks up key… |

**T4Macro_2_0** (53)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e81740` | T4Macro_2_0::~T4Macro_2_0 | confirmed | Stores T4Macro_2_0::vftable at [ecx] and tail-jumps to 00e86d70 (base destructor body). Does not free the object. Byte-identical to 00e817b0. |
| `00e81750` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Stores T4Macro_2_0::vftable at [esi], calls 00e86d70 (base destructor body), and when bit 0 of the stack flag argument is set frees the object with 00e60fc0; r… |
| `00e817b0` | T4Macro_2_0::~T4Macro_2_0 | confirmed | Stores T4Macro_2_0::vftable at [ecx] and tail-jumps to 00e86d70 (base destructor body). Does not free the object. Byte-identical to 00e81740. |
| `00e817c0` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroBase | confirmed | Same body as 00e81750: stores T4Macro_2_0::vftable at [esi], calls 00e86d70, frees through 00e60fc0 when bit 0 of the flag argument is set, returns this (ret 4… |
| `00e81a10` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroGender | confirmed | Stores T4Macro_2_0::vftable at [esi], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; returns… |
| `00e81a80` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroGenderAndRole | confirmed | Stores T4Macro_2_0::vftable at [esi], calls 00e86d70 (base destructor body) and, when bit 0 of the flag argument is set, frees the object with 00e60fc0; return… |
| `00e81c60` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroActor | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e81de0` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroAgree | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e81e70` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroVerb | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e81ee0` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroPerson | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e81fc0` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroCapitalize | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e82110` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroNumber | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e821a0` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroPluralize | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e82280` | T4Macro_2_0::ScalarDeletingDtorViaT4MacroUpperLower | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e82390` | T4Macro_2_0::ScalarDeletingDtorViaMacroPhrase | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e82460` | T4Macro_2_0::ScalarDeletingDtorViaMacroPlayerName | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00e835e0` | T4Macro_2_0::ScalarDeletingDtorViaMacroNounBase | confirmed | Stores T4Macro_2_0::vftable at [this], calls 00e86d70 (base destructor body), and when bit 0 of the flag argument is set frees the object with 00e60fc0; return… |
| `00ec3b90` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Stores T4Macro_2_0::vftable (0185f6bc) into this, calls 00e86d70 (base cleanup), and when bit 0 of its byte argument is set calls 00e60fc0(this) to free the st… |
| `00ec4b00` | T4Macro_2_0::ScalarDeletingDtorDup1 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec4b30` | T4Macro_2_0::ScalarDeletingDtorDup2 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec4e60` | T4Macro_2_0::ScalarDeletingDtorDup3 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec4f00` | T4Macro_2_0::ScalarDeletingDtorDup4 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec4fb0` | T4Macro_2_0::ScalarDeletingDtorDup5 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec5060` | T4Macro_2_0::ScalarDeletingDtorDup6 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec5110` | T4Macro_2_0::ScalarDeletingDtorDup7 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec51b0` | T4Macro_2_0::ScalarDeletingDtorDup8 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec5250` | T4Macro_2_0::ScalarDeletingDtorDup9 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec52c0` | T4Macro_2_0::ScalarDeletingDtorDup10 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec5b40` | T4Macro_2_0::EhUnwindBaseCleanup | confirmed | Stores T4Macro_2_0::vftable into the dword at its argument, then tail-jumps to 00e86d70 (base cleanup). Bytes: c7 01 imm32; e9 rel32 (jmp 00e86d70). No frees. |
| `00ec5b90` | T4Macro_2_0::ScalarDeletingDtorDup11 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec5e10` | T4Macro_2_0::EhUnwindBaseCleanupDup | confirmed | Stores T4Macro_2_0::vftable into the dword at its argument, then tail-jumps to 00e86d70 (base cleanup). Same 11-byte body as 00ec5b40. |
| `00ec79b0` | T4Macro_2_0::ScalarDeletingDtorDup12 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this through 00e60fc0 when bit 0 of the stack … |
| `00ec79e0` | T4Macro_2_0::ScalarDeletingDtorDup13 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec7b00` | T4Macro_2_0::ScalarDeletingDtorDup14 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec7bf0` | T4Macro_2_0::ScalarDeletingDtorDup15 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec7d10` | T4Macro_2_0::ScalarDeletingDtorDup16 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec7f40` | T4Macro_2_0::ScalarDeletingDtorDup17 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec8110` | T4Macro_2_0::ScalarDeletingDtorDup18 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec82b0` | T4Macro_2_0::ScalarDeletingDtorDup19 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec84b0` | T4Macro_2_0::ScalarDeletingDtorDup20 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec8620` | T4Macro_2_0::ScalarDeletingDtorDup21 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec86a0` | T4Macro_2_0::ScalarDeletingDtorDup22 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec88d0` | T4Macro_2_0::ScalarDeletingDtorDup23 | confirmed | Identical body to T4Macro_2_0::ScalarDeletingDtor (00ec3b90): stores T4Macro_2_0::vftable, calls 00e86d70, frees this when bit 0 of the stack argument is set, … |
| `00ec91a0` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Thiscall, one stack argument, ret 4. Stores T4Macro_2_0::vftable (0185f6bc) at [this], calls 00e86d70 (T4Macro base destructor) with ECX=this, and when bit 0 o… |
| `00ec9240` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Byte-identical copy of 00ec91a0 (same 36 bytes, only the call displacements differ): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=this, and w… |
| `00ec92e0` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Third byte-equivalent copy of the T4Macro_2_0 scalar deleting destructor (36 bytes): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=this, and w… |
| `00ec9380` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Eighth copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, same code as 00ec91a0): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=thi… |
| `00ec9460` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Fourth byte-equivalent copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, only call displacements differ): stores T4Macro_2_0::vftable at [this], ca… |
| `00ec9520` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Fifth copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, same code as 00ec91a0): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=this… |
| `00ec95b0` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Sixth copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, same code as 00ec91a0): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=this… |
| `00ec9670` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Seventh copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, same code as 00ec91a0): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=th… |
| `00ec9730` | T4Macro_2_0::ScalarDeletingDtor | confirmed | Ninth copy of the T4Macro_2_0 scalar deleting destructor (36 bytes, same code as 00ec91a0): stores T4Macro_2_0::vftable at [this], calls 00e86d70 with ECX=this… |
| `00ec6050` | T4Macro_2_0::StoreVtableTailJump | speculative | Stores T4Macro_2_0::vftable into the dword at its argument (mov [ecx], imm32), then tail-jumps (jmp rel32) to a shared routine; no frees and no return value of… |

**T4Obj** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e83de0` | T4Obj::T4Obj | confirmed | fastcall, ECX = this: stores T4Obj::vftable at [this], calls 00e838c0 (zeroes the two dwords at [this+4] and [this+8] and returns this), returns this. |
| `00e83e10` | T4Obj::~T4Obj | confirmed | fastcall, ECX = this: stores T4Obj::vftable at [this] and calls 00e83640, which is an empty stub; returns nothing. Does not free the object. |
| `00e84190` | T4Obj::ScalarDeletingDtor | confirmed | Virtual slot [0] of T4Obj::vftable (thiscall, one stack argument = flag byte). Stores T4Obj::vftable at [this], calls 00e83640 (empty stub), and when bit 0 of … |
| `00e83e20` | T4Obj::Slot1_ZeroBitsetAt4 | likely | Virtual slot [1] of T4Obj::vftable (fastcall, ECX = this): add ecx,4 and jump to 00e83650, which zeroes the two dwords at this+4 and this+8. No return value. |
| `00e840e0` | — | unknown | Virtual slot [67] of T4Obj::vftable (fastcall, ECX = object, no stack arguments; shared with T4ObjMacro::vftable[67] and T4ObjGrammar::vftable[67]). Calls 00e5… |

**(no class)** (75)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00e82490` | CreateT4MacroForTokenCode | confirmed | thiscall: ECX = context object (kept in EDI and passed to each constructor as its third argument); stack arg1 = token record (ESI; dword +0 is the token code, … |
| `00e841e0` | LoadT4LogDll | confirmed | one stack argument (the path prefix text): returns 1 at once when the global object pointer at 0x01bfdc58 is set; otherwise frees any loaded module at 0x01bfdc… |
| `00e843d0` | CreateT4LoggingWriter | confirmed | Allocates 0x28 bytes through 00e611f0 (tracked allocation, tag T4LoggingWriter, file .\t4loggingwriter.cpp, line 0xf); when the allocation is nonzero it runs 0… |
| `00e852c0` | T4Logging::getLogTypeString | confirmed | no stack arguments; the type number comes in EAX. Returns the name for codes 0..0x22 and 1000 (HEADER_INFO, SET_PATTERN, USE_PATTERN, EXPAND, LOOKUP, USE_CONTE… |
| `00e85450` | T4Logging::getLogLevelString | confirmed | no stack arguments; the level number comes in EAX. Returns the string for 0..6: LOG_NONE, LOG_TRACE, LOG_DEBUG, LOG_INFO, LOG_WARN, LOG_ERROR, LOG_FATAL; any o… |
| `00e85d40` | CreateT4LoggingFormatterJSON | confirmed | no arguments: allocates 4 bytes through 00e611f0 (tracked allocation, tag T4LoggingFormatterJSON, file .\t4loggingformatter.cpp, line 0x3f); when nonzero store… |
| `00e83610` | CreateT4MacroFromLookup | likely | thiscall, ECX = this, one stack argument (ret 4). Calls 00e7bd20 with ECX = the global DAT_01bfdc34 and the stack argument, then calls the macro factory 00e824… |
| `00e83ab0` | CopyCStringTracked | likely | thiscall-free helper with one stack argument = source char* (ret 4). Measures the string, allocates strlen+1 bytes through 00e60d50 (tag 'T4FileManager', file … |
| `00e85040` | AppendFloatToStream | likely | one float stack argument: formats it with sprintf '%f' (as a double) into a 64-byte stack buffer, appends the text through 00e74e10 with count 0x1000000, and c… |
| `00e85100` | EscapeTextToOutStream | likely | thiscall, ECX = source text object (its text is 00e706b0(0)), one stack argument = output stream (ret 4). When the text is nonempty and contains any of backsla… |
| `00e86180` | SetGlobalPathPrefix | likely | one stack argument = source text; plain ret (the caller cleans the argument). When nonnull copies it into the global buffer at 0x01bfdc68 (including the termin… |
| `00e861e0` | GetLocaleDirPrefix | likely | no arguments: when the byte at 0x01bfdc68 is zero, copies the text './locale/' into the global buffer through 00e86180 (SetGlobalPathPrefix); returns the addre… |
| `00e86ae0` | PtrArrayReleaseAll | likely | fastcall, ECX = pointer array (count byte at +1, data pointer at +4): frees each nonnull element with 00e60fc0, destroys the element block through _eh_vector_d… |
| `00e86c30` | T4MacroArgArrayCopyFrom | likely | thiscall, ECX = destination array, one stack argument = source array (ret 4): releases the destination (00e86ae0), grows it to the source count (00e867e0), the… |
| `006b0087` | InitListHeadAndReturnThis | speculative | Thiscall with no arguments. Saves ESI, sets ESI to ECX (this), calls 006ac70a with ECX = this, then returns this in EAX. |
| `006b03ba` | ChunkChain8K_CtorArgs | speculative | Thiscall with two stack arguments (ret 8). Stores vtable 017cdb90 at this+0, constructs the member at this+0x24 with 006ac903, stores arg1 at +4, zeroes +8 and… |
| `006b0412` | ChunkChain8K_Ctor | speculative | Thiscall with no arguments. Stores vtable 017cdb90, constructs the member at this+0x24 with 006ac903, zeroes +4, +8, +0x1c and +0x20, sets +0x18 to 1, allocate… |
| `006b0461` | ChunkChain8K_Dtor | speculative | Thiscall with no arguments. Stores vtable 017cdb90, then frees each block of the chain starting at this+0xc, reading the next link at block+0x2008 before each … |
| `006b049b` | ChunkChain8K_DerivedCtorArgs | speculative | Thiscall with two stack arguments. Calls base constructor 006b0412, sets +0x50 = 1, +0x54 = &DAT_01bc7e28, +0x44 = -1, +0x64 = 0 and +0x3c = 0, then calls 006a… |
| `006b051a` | SetField4AndCall006af53f | speculative | Thiscall with one stack argument (ret 4). Stores the argument at this+4, then calls 006af53f with no arguments. |
| `006b0850` | ResetCursorFromOwner | speculative | Thiscall with no arguments. Calls 006af15a(0) and 006ad0dc(this+4, 1), zeroes this+0x40 and stores 006aa900() at +0x4c. If the owner at [this] is set, this+4 i… |
| `006b090e` | TrySetCursorByIndex | speculative | Thiscall with one stack argument (the index). Returns 0 unless this+0x40 is 0, the record at this+0x3c is not -1, this+0x44 is not -1, and the index is below t… |
| `006b0ea1` | Wrapper_006afc94 | speculative | Forwards its three stack arguments and ECX to 006afc94 as (arg1, arg2, arg3, ECX), ECX being the last argument. Plain ret, so the caller cleans the stack. |
| `006b0ec0` | ChunkChain8K_AltCtor | speculative | Thiscall with no arguments. Stores vtable 017cdb94, calls 006aca66 with ECX = this+0x24, zeroes +4, +8, +0x1c and +0x20, sets +0x18 to 1, allocates 0x200c byte… |
| `006b1019` | ChunkChain8K_AltDtor | speculative | Thiscall with no arguments. Stores vtable 017cdb94, then for each block from this+0xc: reads the next link at block+0x2008, calls 006ab01a(), and frees the blo… |
| `006b1053` | FillElem40Range | speculative | Cdecl-shaped; uses its first three stack arguments (pointer, count, value) and sets an SEH frame with 00d83d3c. While count is nonzero it calls 006afcdb(elemen… |
| `006b10bd` | Wrapper_006afcef | speculative | Cdecl-shaped wrapper. Calls 006afcef(arg1, arg2, arg3, arg4, X, X), where X is ECX on entry with its low byte cleared by a stosb (a zero bool local, shown as 0… |
| `006b10e5` | Wrapper_006afd32 | speculative | Cdecl-shaped wrapper with four stack arguments. Calls 006afd32(arg1, arg2, arg3, arg4, arg2, X), where X is ECX with its low byte cleared (shown as 0 by Ghidra… |
| `006b1121` | StackInit_Stride14 | speculative | Thiscall with three stack arguments (A, B, C). Only when this+0x44 is -1: frees the buffer at +0x64 if set, sets +0x68 to -1, stores A at +0, calls 006aba3c(3)… |
| `006b11cd` | ChunkChain8K_AltDerivedCtorArgs | speculative | Thiscall with two stack arguments. Calls base constructor 006b0ec0, sets +0x50 = 1, +0x54 = &DAT_01bc7e28, +0x44 = -1, +0x64 = 0 and +0x3c = 0, then calls 006a… |
| `006b1207` | ChunkChain8K_AltDerivedDtor | speculative | Thiscall with no arguments. Calls 006af15a(0), frees the buffers at this+0x3c and this+0x64 when set, calls 004d83ba, then calls base destructor 006b1019. |
| `006b124c` | SetField4AndCall006b0850 | speculative | Thiscall with one stack argument. Stores it at this+4 and calls 006b0850 with no arguments. |
| `006b125b` | Wrapper_006b1053 | speculative | Cdecl-shaped wrapper. Calls 006b1053(arg1, arg2, arg3, arg4, X, X), where X is ECX with its low byte cleared (shown as 0 by Ghidra). |
| `006b1283` | Wrapper_006b10bd | speculative | Cdecl-shaped wrapper with three stack arguments. Calls 006b10bd(arg1, arg2, arg3, arg2, arg2, X), where X is ECX with its low byte cleared (shown as 0 by Ghidr… |
| `006b12c4` | VectorClearAndFree | speculative | Thiscall with no arguments. If the begin pointer at this+4 is nonzero, calls 006b0ea1(begin, this+8 value, this) and frees begin with _free. Then zeroes this+4… |
| `006b12f1` | Elem40FillReturnEnd | speculative | Thiscall with three stack arguments (base, count, value; ret 0xc). Calls 006b125b(base, count, value, ECX) and returns base + count*0x28. |
| `006b1313` | ChunkChain8K_Stack14Ctor | speculative | Thiscall with three stack arguments. Calls base constructor 006b0412, sets +0x50 = 1, +0x54 = &DAT_01bc7e28, +0x44 = -1, +0x64 = 0 and +0x3c = 0, calls 006b112… |
| `006b1351` | StackInit_Stride1c | speculative | Thiscall with three stack arguments (A, B, C). Only when this+0x44 is -1: frees +0x64 if set, sets +0x68 to -1, stores A at +0, frees the buffer at +0x3c if se… |
| `006b13fe` | ChunkChain8K_AltStack1cCtor | speculative | Thiscall with three stack arguments. Calls base constructor 006b0ec0, sets +0x50 = 1, +0x54 = &DAT_01bc7e28, +0x44 = -1, +0x64 = 0 and +0x3c = 0, calls 006b135… |
| `006b143c` | Wrapper_006b10e5 | speculative | Thiscall with three stack arguments (ret 0xc). Pushes ECX as the fourth argument and calls 006b10e5(arg1, arg2, arg3, ECX). |
| `006b1454` | CtorSixListHeads | speculative | Thiscall with no arguments. Stores vtable 017cdb98, calls 005db243(), calls 006ac70a six times on sub-objects, calls 004fb0b9 once, zeroes +0xc, +0x28, +0x2c, … |
| `006b14c9` | ResetMemberContainers | speculative | Thiscall with no arguments. Calls 006b12c4 on the member at this+0x80, 0051b342 on this+0x70, 006af040 on this+0x64, and 006af040 on other members (six calls i… |
| `006b1791` | ChoiceProductEnumerate | speculative | Takes a node pointer (second argument) and returns 1 only when its +8 dword has tag 0x50000000. For such a node it counts child entries (006aa02f, 006aa045), m… |
| `00e81af0` | DispatchCodeRange13To29 | speculative | fastcall, ECX = this (saved in ESI). Calls 00e86de0 for a code, then for codes 0x13..0x29 (default returns 0): 0x13/0x19/0x1f/0x25 set ECX = [this+0x14] and ta… |
| `00e81b90` | MapCodeRangeToGroup | speculative | fastcall (no stack args). Calls 00e86de0 for a code and maps it: 0x10,0x13..0x18 return 1; 0x11,0x19..0x1e return 2; 0x12,0x1f..0x24 return 3; 0x25..0x29 retur… |
| `00e82960` | ArrayPtrAtIndexOrDefault | speculative | thiscall, ECX = this, one stack argument = index (ret 4). If the pointer at [this+4] is nonzero and index is below the byte at [this+1], returns the dword at [… |
| `00e846a0` | T4ConstStringSeekArg | speculative | thiscall, ECX = this, one stack argument (ret 4): initialises an inner text object from the argument with 00e6f520(..., 0), stores T4ConstString::vftable into … |
| `00e84730` | T4ConstStringSeekBuffer0 | speculative | thiscall, one stack argument = SoeBuffer object (loaded into ECX; ret 4). Takes its text pointer 00e706b0(0), initialises an inner text object from it through … |
| `00e86470` | ClassifyNodeModeToByte | speculative | fastcall, ECX = this (the node at [this+4] may be null). Computes a 16-bit mode from [this+0xc] OR the flag dword of the node at [this+4] (or 1 when that point… |
| `00e86540` | NodeByte15IfFlag2 | speculative | fastcall getter, ECX = this; loads the node at [this+4] and its flags (dword at +4, or 0 when the node is null) OR-ed with [this+0xc]; when bit 0x2 is set retu… |
| `00e86700` | PtrArraySetAt | speculative | thiscall, ECX = this, stack args: index, value (ret 8). When the index is at or above the count byte [this+1] returns 0; otherwise stores the value at [[this+4… |
| `00e86950` | PtrArrayPushBack | speculative | thiscall, ECX = array object (capacity byte at +0, count byte at +1, data pointer at +4), one stack argument = value (ret 4). When count >= capacity first grow… |
| `00e86990` | PtrArrayInsertAt | speculative | thiscall, ECX = array object (same layout as PtrArrayPushBack), stack args: index, value (ret 8). Returns 0 when index is above the count; grows through 00e867… |
| `00e86de0` | DerefPlus4Dword | speculative | fastcall, ECX = object: loads the pointer at [this+4] and returns the dword it points to (mov eax,[ecx+4]; mov eax,[eax]; ret). Six bytes, no stack arguments. |
| `00e86df0` | CallE57fa0WithThisPlus8 | speculative | fastcall, ECX = object: pushes this+8 and calls 00e57fa0 (one stack argument, cdecl-style cleanup by add esp,4); returns the result of 00e57fa0 in EAX. |
| `00e86e30` | NodeFlagsCountOrIndex | speculative | fastcall, ECX = this: reads the flags dword of the node at [this+4] ([node+0x10]). When bit 0 is clear: returns 1 if any bit of 0x2007e is set; when the flags … |
| `00e86ea0` | NodeFlagsStatusCode | speculative | fastcall, ECX = this: reads the flags dword at [[this+4]+0x10]. When bit 0x200000 is clear: returns 1 if any bit of 0x2007e is set, 0 if bit 0 is set, and when… |
| `00ec9760` | T4Macro_SharedSlot3TestThenEmit | speculative | Shared virtual slot [3] of nine T4 macro vtables (T4Macro_DE_DE, T4MacroNounPhrase, T4MacroCase, T4MacroDefiniteArticle, T4MacroIndefiniteArticle, T4MacroDemon… |
| `00ec97b0` | T4Macro_SharedSlot9CodeToSubArg | speculative | Shared virtual slot [9] of the nine T4 macro vtables that also share slot [3] (00ec9760); ECX=this, no stack arguments. Calls 00e86de0 and dispatches on its re… |
| `00ec9850` | T4Macro_MacroCodeToSubArg123 | speculative | Non-virtual method (ECX=this): calls 00e86de0 and dispatches on its result for codes 0xbd9..0xbf0 (offset index bytes at 00ec98a0 and jump table at 00ec9894, b… |
| `00ec98c0` | T4Macro_SharedSlot5ChildStreamEmit | speculative | Shared virtual slot [5] of six T4 macro vtables (NounPhrase, Case, DefiniteArticle, IndefiniteArticle, Demonstrative, Possessive); thiscall, one stack argument… |
| `00ec9b80` | T4Macro_SharedSlot8TemplateExpand | speculative | Shared virtual slot [8] of several T4 macro vtables (show lists DE_DE, NounPhrase, Case, DefiniteArticle, IndefiniteArticle and more); __fastcall, ECX=this, no… |
| `00e82990` | — | unknown | thiscall, ECX = this, one stack argument (ret 4). Reads the dword at [[this+4]+4] (0 when [this+4] is null) and tests bit 0x20000 of ([this+0xc] \| that dword)… |
| `00e847c0` | — | unknown | two stack arguments: calls 00e84730 (the buffer seek on ECX), then if the buffer is empty (00e70ff0) returns the second argument; otherwise returns 00e70190(0)… |
| `00e84820` | — | unknown | thiscall-style, first stack argument = text pointer, second = flag byte (ret 8 by its frame). Builds a T4ConstString temporary from the text with 00e6f520 and … |
| `00e849c0` | — | unknown | ECX = buffer; stack arg 2 (ret 8 by frame): when 00e70ff0 says the buffer is empty returns the low byte of arg 2; otherwise calls 00e84820 with the text pointe… |
| `00e84a00` | — | unknown | two stack arguments are popped (ret 8; the body does not read them in the export). Builds two T4ConstString temporaries from the text pointer 00e706b0(0) with … |
| `00e84b30` | — | unknown | thiscall (ECX is passed to the buffer tests); stack arguments: the output pair pointer (param_2) and one more dword, ret 8. Scans the text buffer line by line … |
| `00e84d00` | — | unknown | no arguments (returns 1 on success, 0 on failure): takes a file service object from 00e7fa70, opens the path text 00e706b0(0) through its virtual +0x18 with th… |
| `00e84fa0` | — | unknown | One stack argument = int, read after the frame; formats it with sprintf '%d' into a 64-byte stack buffer and appends the text through 00e74e10 with count 0x100… |
| `00e84ff0` | — | unknown | one stack argument = int; formats it with sprintf using the format string at the global pointer PTR_DAT_01856698 into a 64-byte stack buffer, appends the text … |
| `00e85d70` | — | unknown | no stack arguments; ECX = the source text object passed to 00e85100 as a temporary. Creates a T4StringBuffer temporary (00e71e90(0,0,1), vtable T4StringBuffer:… |
| `00e86570` | — | unknown | fastcall, ECX = expansion state (node at [this+4], mode word at [this+0xc]); partial read: clears the output SoeBuffer (00e72550), computes a 16-bit mode from … |
| `00e867e0` | — | unknown | thiscall, ECX = array object, one stack argument = new element count (ret 4): refuses counts above 10 (returns 0). Otherwise allocates count*4+4 bytes through … |
| `00ec9a30` | — | unknown | Shared virtual slot [10] of five T4 macro vtables (NounPhrase, Case, DefiniteArticle, IndefiniteArticle and one more; show lists the full set); thiscall, two s… |

### `lib.t4macro.de_de`

**T4Locale_DE_DE** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00eca150` | — | unknown | Virtual slot [11] of T4Locale_DE_DE (1084 bytes; thiscall with two stack arguments, returns a node pointer). Calls 00e7bd20(arg2) to get the record puVar2; whe… |

### `lib.t4macro.es_es`

**T4Macro_ES_ES::ArticleMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5bc0` | T4Macro_ES_ES::ArticleMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the byte at this+0x28 and the byte at this+0x2c to 0, stores T4Macro_ES_ES::ArticleMacr… |

**T4Macro_ES_ES::DemonstrativeAdjectiveMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6cb0` | T4Macro_ES_ES::DemonstrativeAdjectiveMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::DemonstrativeAdjectiveMacro::vftable, calls 00e86… |

**T4Macro_ES_ES::DemonstrativePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6970` | T4Macro_ES_ES::DemonstrativePronounMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::DemonstrativePronounMacro::vftable, calls 00e86de… |

**T4Macro_ES_ES::IndefiniteAdjectiveMacro** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec71f0` | T4Macro_ES_ES::IndefiniteAdjectiveMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::IndefiniteAdjectiveMacro::vftable and returns thi… |
| `00ec8e90` | T4Macro_ES_ES::IndefiniteAdjectiveMacro::EmitForm | confirmed | Virtual slot [5] of T4Macro_ES_ES::IndefiniteAdjectiveMacro (fastcall, ECX=this). Builds a local T4OutStrStream (vftable 018560d0) and calls vtable slot +0x24 … |

**T4Macro_ES_ES::IndefinitePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6f30` | T4Macro_ES_ES::IndefinitePronounMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::IndefinitePronounMacro::vftable and returns this … |

**T4Macro_ES_ES::InterrogativePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6790` | T4Macro_ES_ES::InterrogativePronounMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::InterrogativePronounMacro::vftable and returns th… |

**T4Macro_ES_ES::ObjectPronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6060` | T4Macro_ES_ES::ObjectPronounMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, sets the dword at this+0x30 to 2, stores T4M… |

**T4Macro_ES_ES::PossessiveAdjectiveMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec63f0` | T4Macro_ES_ES::PossessiveAdjectiveMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets bytes at this+0x28 and this+0x2c to 0, sets the dword at this+0x30 to 2, stores T4Macro_ES_ES::PossessiveAdjective… |

**T4Macro_ES_ES::PossessivePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec6150` | T4Macro_ES_ES::PossessivePronounMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, sets the dword at this+0x30 to 2, stores T4M… |

**T4Macro_ES_ES::PrepositionMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec7640` | T4Macro_ES_ES::PrepositionMacro::Ctor | confirmed | Calls 00e86d10 as the base initialiser, sets the bytes at this+0x28 and this+0x2c to 0, stores T4Macro_ES_ES::PrepositionMacro::vftable and returns this (ret 0… |

**T4Macro_ES_ES::SubjectPronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5e20` | T4Macro_ES_ES::SubjectPronounMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the byte at this+0x28 and at this+0x2c to 0, sets the dword at this+0x30 to 2, stores T… |

**(no class)** (87)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5ed0` | T4MacroEs_EmitNosotrosForms | likely | Three-argument wrapper: builds a four-pointer table (nosotros, Nosotros, nosotras, Nosotras) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec5f60` | T4MacroEs_EmitVosotrosForms | likely | Three-argument wrapper: builds a four-pointer table (vosotros, Vosotros, vosotras, Vosotras) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec6000` | T4MacroEs_EmitEllosForms | likely | Three-argument wrapper: builds a four-pointer table (ellos, Ellos, ellas, Ellas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec62a0` | T4MacroEs_EmitTuyosForms | likely | Three-argument wrapper: builds a four-pointer table (tuyos, Tuyos, tuyas, Tuyas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec6340` | T4MacroEs_EmitSuyosForms | likely | Three-argument wrapper: builds a four-pointer table (suyos, Suyos, suyas, Suyas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec6640` | T4MacroEs_EmitNuestroForms | likely | Three-argument wrapper: builds a four-pointer table (nuestro, Nuestro, nuestra, Nuestra) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec6690` | T4MacroEs_EmitNuestrosForms | likely | Three-argument wrapper: builds a four-pointer table (nuestros, Nuestros, nuestras, Nuestras) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec66e0` | T4MacroEs_EmitVuestroForms | likely | Three-argument wrapper: builds a four-pointer table (vuestro, Vuestro, vuestra, Vuestra) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec6730` | T4MacroEs_EmitVuestrosForms | likely | Three-argument wrapper: builds a four-pointer table (vuestros, Vuestros, vuestras, Vuestras) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec6c60` | T4MacroEs_EmitAquelloForm | likely | Two-argument wrapper (50 bytes, ret 8): references the string aquello at 01868648 for its table and passes the table address in ECX to 00ebe9f0. |
| `00ec6d90` | T4MacroEs_Forms_018686a8 | likely | Three-argument wrapper: builds a four-pointer table (018686a8 estos, 018686b0 Estos, 018686b8 estas, 018686c0 Estas) on the stack, passes its address in ECX to… |
| `00ec6e80` | T4MacroEs_EmitAquelForms | likely | Three-argument wrapper: builds a four-pointer table (aquel, Aquel, aquella, Aquella) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec6ed0` | T4MacroEs_EmitAquellosForms | likely | Three-argument wrapper: builds a four-pointer table (aquellos, Aquellos, aquellas, Aquellas) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec6f60` | T4MacroEs_EmitAlgunoForms | likely | Three-argument wrapper: builds a four-pointer table (alguno, Alguno, alguna, alguna) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec6fb0` | T4MacroEs_EmitAlgunosForms | likely | Three-argument wrapper: builds a four-pointer table (algunos, Algunos, algunas, Algunas) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec7000` | T4MacroEs_EmitMuchoForms | likely | Three-argument wrapper: builds a four-pointer table (mucho, Mucho, mucha, Mucha) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec7050` | T4MacroEs_EmitMuchosForms | likely | Three-argument wrapper: builds a four-pointer table (muchos, Muchos, muchas, Muchas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec70a0` | T4MacroEs_EmitNingunoForms | likely | Three-argument wrapper: builds a four-pointer table (ninguno, Ninguno, ninguna, Ninguna) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec70f0` | T4MacroEs_EmitNingunosForms | likely | Three-argument wrapper: builds a four-pointer table (ningunos, Ningunos, ningunas, Ningunas) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec7190` | T4MacroEs_EmitPocosForms | likely | Three-argument wrapper: builds a four-pointer table (pocos, Pocos, pocas, Pocas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec7220` | T4MacroEs_EmitAlgunForms | likely | Three-argument wrapper: builds a four-pointer table (01868858 algún, 01868860 Algún, 01868868 alguna, 01868870 Alguna) on the stack, passes its address in ECX … |
| `00ec7270` | T4MacroEs_EmitAlgunosForms2 | likely | Three-argument wrapper: builds a four-pointer table (algunos, Algunos, algunas, Algunas) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec72c0` | T4MacroEs_EmitCiertoForms | likely | Three-argument wrapper: builds a four-pointer table (cierto, Cierto, cierta, Cierta) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec7310` | T4MacroEs_EmitCiertosForms | likely | Three-argument wrapper: builds a four-pointer table (ciertos, Ciertos, ciertas, Ciertas) on the stack, passes its address in ECX to 00ebe9f0 with the three sta… |
| `00ec7360` | T4MacroEs_EmitCualquierForms | likely | Three-argument wrapper: builds a four-pointer table (cualquier, Cualquier, cualquiera, Cualquiera) on the stack, passes its address in ECX to 00ebe9f0 with the… |
| `00ec73b0` | T4MacroEs_EmitCualesquierForms | likely | Three-argument wrapper: builds a four-pointer table (cualesquier, Cualesquier, cualesquiera, Cualesquiera) on the stack, passes its address in ECX to 00ebe9f0 … |
| `00ec7400` | T4MacroEs_EmitNingunForms | likely | Three-argument wrapper: builds a four-pointer table (ningún, Ningún, ninguna, Ninguna) on the stack, passes its address in ECX to 00ebe9f0 with the three stack… |
| `00ec7450` | T4MacroEs_EmitNingunosForms2 | likely | Three-argument wrapper: builds a four-pointer table (ningunos, Ningunos, ningunas, Ningunas) on the stack, passes its address in ECX to 00ebe9f0 with the three… |
| `00ec74f0` | T4MacroEs_EmitOtrosForms | likely | Three-argument wrapper: builds a four-pointer table (otros, Otros, otras, Otras) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec7590` | T4MacroEs_EmitTodosForms | likely | Three-argument wrapper: builds a four-pointer table (todos, Todos, todas, Todas) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec75e0` | T4MacroEs_EmitVariosForms | likely | Three-argument wrapper: builds a four-pointer table (varios, Varios, varias, Varias) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec76c0` | T4MacroEs_EmitALosForms | likely | Three-argument wrapper: builds a four-pointer table (a los, A los, a las, A las) on the stack, passes its address in ECX to 00ebe9f0 with the three stack argum… |
| `00ec7710` | T4MacroEs_EmitDeLaForms | likely | Three-argument wrapper: builds a four-pointer table (del, Del, de la, De la) on the stack, passes its address in ECX to 00ebe9f0 with the three stack arguments… |
| `00ec7760` | T4MacroEs_EmitDeLosForms | likely | Three-argument wrapper: builds a four-pointer table (de los, De los, de las, De las) on the stack, passes its address in ECX to 00ebe9f0 with the three stack a… |
| `00ec7a10` | T4MacroEs_PersonalPronounByField30 | likely | Thiscall: reads the code (00e86df0 with 00e68ae0), mode (00e78e90) and role (00e78f30 == 2). When the dword at this+0x30 is 0 it calls 00ebe9f0(out, predicate,… |
| `00ec8650` | T4MacroEs_EmitVariosForms2 | likely | Three-argument wrapper: builds the same four-pointer table as 00ec75e0 (varios, Varios, varias, Varias) on the stack, passes its address in ECX to 00ebe9f0 wit… |
| `00ec8900` | T4Macro_ES_ES::CreateMacroForToken | likely | Thiscall: gets a node from 00e7bd20(param_2); returns 0 when null. When node[2] is zero it calls 00e82490(node, param_2). Otherwise it switches on the token *n… |
| `00ec8d30` | T4MacroEs_EmitSubjectPronounByField18 | likely | Thiscall: builds a temporary SubjectPronounMacro with token 0xfa5 through 00e7b4e0 and 00ec5e20 (ctor path) on a stack object, reads the mode (00e78e90) and ro… |
| `00ec5c80` | T4MacroEs_Forms_01868218 | speculative | Three-argument wrapper: builds a four-pointer table (01868218 el, 0186821c El, 01868228 la, 0186822c La) on the stack, passes its address in ECX to 00ebe9f0 wi… |
| `00ec5cd0` | T4MacroEs_Forms_01868220 | speculative | Three-argument wrapper: builds a four-pointer table on the stack (01868220 los, 01868224 Los, 01868230 las, 01868234 Las), passes its address in ECX to 00ebe9f… |
| `00ec5d20` | T4MacroEs_Forms_01868238 | speculative | Three-argument wrapper: builds a four-pointer table (01868238 un, 0186823c Un, 01868250 una, 01868254 Una), passes its address in ECX to 00ebe9f0 with the thre… |
| `00ec5d70` | T4MacroEs_Forms_01868240 | speculative | Three-argument wrapper: builds a four-pointer table (01868240 unos, 01868248 Unos, 01868258 unas, 01868260 Unas), passes its address in ECX to 00ebe9f0 with th… |
| `00ec5f20` | T4MacroEs_EmitTuForms | speculative | Three-argument wrapper with two pointers built from immediates (01868328 tú, 0186832c Tú) placed in the stack table, passes the table address in ECX to 00ebe9f… |
| `00ec5fb0` | T4MacroEs_Forms_01868268 | speculative | Three-argument wrapper: builds a four-pointer table (01868268 el, 0186826c El-accented, 01868270 ella, 01868278 Ella) on the stack, passes its address in ECX t… |
| `00ec60a0` | T4MacroEs_Forms_01868330 | speculative | Three-argument wrapper: builds a four-pointer table (01868330 lo, 01868334 Lo, 01868338 la, 0186833c La) on the stack, passes its address in ECX to 00ebe9f0 wi… |
| `00ec60f0` | T4MacroEs_Forms_01868340 | speculative | Three-argument wrapper: builds a four-pointer table (01868340 los, 01868344 Los, 01868348 las, 0186834c Las) on the stack, passes its address in ECX to 00ebe9f… |
| `00ec6200` | T4MacroEs_Forms_01868370 | speculative | Three-argument wrapper: builds a four-pointer table (01868370 míos, 01868378 Míos, 01868380 mías, 01868388 Mías) on the stack, passes its address in ECX to 00e… |
| `00ec6250` | T4MacroEs_Forms_01868350 | speculative | Three-argument wrapper: builds a four-pointer table (01868350 mío, 01868358 Mío, 01868360 mía, 01868368 Mía) on the stack, passes its address in ECX to 00ebe9f… |
| `00ec62f0` | T4MacroEs_Forms_01868390 | speculative | Three-argument wrapper: builds a four-pointer table (01868390 tuyo, 01868398 Tuyo, 018683a0 tuya, 018683a8 Tuya) on the stack, passes its address in ECX to 00e… |
| `00ec6390` | T4MacroEs_Forms_018683d0 | speculative | Three-argument wrapper: builds a four-pointer table (018683d0 suyo, 018683d8 Suyo, 018683e0 suya, 018683e8 Suya) on the stack, passes its address in ECX to 00e… |
| `00ec6840` | T4MacroEs_Forms_01868500 | speculative | Three-argument wrapper: builds a four-pointer table (01868500 cuánto, 01868508 Cuánto, 01868510 cuánta, 01868518 Cuánta) on the stack, passes its address in EC… |
| `00ec6890` | T4MacroEs_Forms_01868520 | speculative | Three-argument wrapper: builds a four-pointer table (01868520 cuántos, 0186852c Cuántos, 01868538 cuántas, 01868544 Cuántas) on the stack, passes its address i… |
| `00ec6920` | T4MacroEs_Forms_01868560 | speculative | Two-argument wrapper: builds a two-pointer table (01868560 quiénes, 0186856c Quiénes) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6a00` | T4MacroEs_Forms_01868578 | speculative | Three-argument wrapper: builds a four-pointer table (01868578 éste, 01868580 Éste, 01868588 ésta, 01868590 Ésta) on the stack, passes its address in ECX to 00e… |
| `00ec6a50` | T4MacroEs_Forms_018685b8 | speculative | Three-argument wrapper: builds a four-pointer table (018685b8 éstos, 018685c0 Éstos, 018685c8 éstas, 018685d0 Éstas) on the stack, passes its address in ECX to… |
| `00ec6aa0` | T4MacroEs_Forms_01868598 | speculative | Two-argument wrapper: builds a two-pointer table (01868598 esto, 018685a0 Esto) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6ae0` | T4MacroEs_Forms_018685e0 | speculative | Three-argument wrapper: builds a four-pointer table (018685e0 ése, 018685e8 Ése, 018685f0 esa, 018685f8 Esa) on the stack, passes its address in ECX to 00ebe9f… |
| `00ec6b30` | T4MacroEs_Forms_01868600 | speculative | Three-argument wrapper: builds a four-pointer table (01868600 ésos, 01868608 Ésos, 01868610 esas, 01868618 Esas) on the stack, passes its address in ECX to 00e… |
| `00ec6b80` | T4MacroEs_Forms_018685D8 | speculative | Two-argument wrapper: builds a two-pointer table (018685d8 eso, 018685dc Eso) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6bc0` | T4MacroEs_Forms_01868620 | speculative | Three-argument wrapper: builds a four-pointer table (01868620 aquél, 01868628 Aquél, 01868630 aquélla, 0186863c Aquélla) on the stack, passes its address in EC… |
| `00ec6d40` | T4MacroEs_Forms_01868688 | speculative | Three-argument wrapper: builds a four-pointer table (01868688 este, 01868690 Este, 01868698 esta, 018686a0 Esta) on the stack, passes its address in ECX to 00e… |
| `00ec6de0` | T4MacroEs_Forms_018686c8 | speculative | Three-argument wrapper: builds a four-pointer table (018686c8 ese, 018686cc Ese, 018686d0 esa, 018686d4 Esa) on the stack, passes its address in ECX to 00ebe9f… |
| `00ec6e30` | T4MacroEs_Forms_018686d8 | speculative | Three-argument wrapper: builds a four-pointer table (018686d8 esos, 018686e0 Esos, 018686e8 esas, 018686f0 Esas) on the stack, passes its address in ECX to 00e… |
| `00ec7140` | T4MacroEs_Forms_01868818 | speculative | Three-argument wrapper: builds a four-pointer table (01868818 poco, 01868820 Poco, 01868828 poca, 01868830 Poca) on the stack, passes its address in ECX to 00e… |
| `00ec74a0` | T4MacroEs_Forms_01868990 | speculative | Three-argument wrapper: builds a four-pointer table (01868990 otro, 01868998 Otro, 018689a0 otra, 018689a8 Otra) on the stack, passes its address in ECX to 00e… |
| `00ec7540` | T4MacroEs_Forms_018689d0 | speculative | Three-argument wrapper: builds a four-pointer table (018689d0 todo, 018689d8 Todo, 018689e0 toda, 018689e8 Toda) on the stack, passes its address in ECX to 00e… |
| `00ec7670` | T4MacroEs_Forms_01868a30 | speculative | Three-argument wrapper: builds a four-pointer table (01868a30 al, 01868a34 Al, 01868a38 a la, 01868a40 A la) on the stack, passes its address in ECX to 00ebe9f… |
| `00ec77c0` | T4MacroEs_FlagSetThenTailVSlot8 | speculative | Fastcall: if the byte at this+0x2c is nonzero and the dword at member(this+0x14)+0x20 equals -1, calls 00e79710 with argument 2 on that member; then tail-jumps… |
| `00ec77f0` | T4MacroEs_ArticleFormByField30 | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 0x24 of this with it. Reads the code (00e86df0 with 00e68ae0), mode (00e78e90) and role (0… |
| `00ec7c20` | T4MacroEs_PossessiveFormByField30AndMode | speculative | Thiscall: reads the code (00e86df0 with 00e68ae0), mode (00e78e90, flag below 2) and role (00e78f30 == 2). Dispatches on the dword at this+0x30: 0 calls 00ec62… |
| `00ec7d40` | T4MacroEs_PossessiveFormByPersonAndNumber | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 0x24 of this with it. Reads the code (00e86df0 with 00e68ae0), the mode flag (below 2) and… |
| `00ec7f70` | T4MacroEs_InterrogativeFormByToken | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary, calls vslot 0x24 of this with it, reads the code (00e86df0 with 00e68ae0), mode (00e78e90, flag below 2) … |
| `00ec8140` | T4MacroEs_NeuterDemonstrativeByToken | speculative | Stdcall with one stack argument: reads the code (00e86df0 with 00e68ae0), mode (00e78e90, flag below 2) and role (00e78f30 == 2 and a second read of role). Com… |
| `00ec82e0` | T4MacroEs_DemonstrativeAdjectiveByToken | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 0x24 of this with it. Reads the code (00e86df0 with 00e68ae0), mode (00e78e90, flag below … |
| `00ec84e0` | T4MacroEs_QuantifierByToken | speculative | Stdcall with one stack argument: reads the code (00e86df0 with 00e68ae0), mode (00e78e90, flag below 2) and role (00e78f30 == 2), then switches on 00e86de0: 0x… |
| `00ec86d0` | T4MacroEs_PrepositionContractionByToken | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 0x24 of this with it. Reads the code (00e86df0 with 00e68ae0), mode (00e78e90), role (00e7… |
| `00ec64c0` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868410 mi, 01868414 Mi) on the stack, passes its address in ECX to 00ebe9f0 with the stack arguments, retur… |
| `00ec6500` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868418 mis, 0186841c Mis) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6540` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868420 tu, 01868424 Tu) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6580` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868428 tus, 0186842c Tus) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec65c0` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868430 su, 01868434 Su) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6600` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868438 sus, 0186843c Sus) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec67c0` | — | unknown | Two-argument wrapper: builds a two-pointer table (018684e0 cuál, 018684e4 Cuál) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6800` | — | unknown | Two-argument wrapper: builds a two-pointer table (018684f0 cuáles, 018684f8 Cuáles) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec68e0` | — | unknown | Two-argument wrapper: builds a two-pointer table (01868550 quién, 01868558 Quién) on the stack, passes its address in ECX to 00ebe9f0, returns with ret 8. |
| `00ec6c10` | — | unknown | Three-argument wrapper: builds a four-pointer table (01868658 aquéllo, 01868664 Aquéllo, 01868670 aquélla, 0186867c Aquélla) on the stack, passes its address i… |
| `00ec7b30` | — | unknown | Stdcall with one stack argument: reads the code (00e86df0 with 00e68ae0), reads the mode (00e78e90, discarded) and the role (00e78f30); calls 00ebe9f0(param_1,… |

### `lib.t4macro.fr_fr`

**T4Macro_FR_FR::MacroAdjectivePhrase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec4f30` | T4Macro_FR_FR::MacroAdjectivePhrase::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, sets the dword at this+0x30 to… |

**T4Macro_FR_FR::MacroArticlePhrase** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec4e90` | T4Macro_FR_FR::MacroArticlePhrase::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, stores T4Macro_FR_FR::MacroArt… |
| `00ec4ed0` | T4Macro_FR_FR::MacroArticlePhrase::Slot5_ForwardToMember2C | speculative | Thiscall with one stack argument (ret 4): loads the object at this+0x2c into ECX and calls 00eb32d0 with that object and the stack argument; returns 0 (xor eax… |

**T4Macro_FR_FR::MacroDemonstrativePronoun** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5140` | T4Macro_FR_FR::MacroDemonstrativePronoun::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, stores T4Macro_FR_FR::MacroDem… |

**T4Macro_FR_FR::MacroInterrogativePronoun** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec51e0` | T4Macro_FR_FR::MacroInterrogativePronoun::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, stores T4Macro_FR_FR::MacroInt… |

**T4Macro_FR_FR::MacroPersonalPronoun** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5090` | T4Macro_FR_FR::MacroPersonalPronoun::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, sets the dword at this+0x30 to… |

**T4Macro_FR_FR::MacroPossessivePronoun** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec4fe0` | T4Macro_FR_FR::MacroPossessivePronoun::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, sets the dword at this+0x30 to… |

**T4Macro_FR_FR::MacroRelativePronoun** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5280` | T4Macro_FR_FR::MacroRelativePronoun::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, stores param_4 at this+0x2c, sets the byte at this+0x28 to 0, stores T4Macro_FR_FR::MacroRel… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec5680` | T4Macro_FR_FR::CreateMacroForToken | likely | Thiscall: gets a node from 00e7bd20(param_2); when the node is null it returns 0. When node[2] is zero it calls 00e82490(node, param_2) (base macro path). Othe… |
| `00ec4f80` | T4MacroMember2C_ForwardToEb3640 | speculative | Thiscall with one stack argument: loads the member at this+0x2c into ECX and calls 00eb3640 with (stack argument, this+0x30 value); returns 0 (ret 4). Bytes: 8… |
| `00ec5030` | T4MacroMember2C_ForwardToEb16f0 | speculative | Thiscall with one stack argument: loads the member at this+0x2c into ECX and calls 00eb16f0 with (stack argument, this+0x30 value); returns 0 (ret 4). Bytes: 8… |
| `00ec50e0` | T4MacroMember2C_ForwardToEb1500 | speculative | Thiscall with one stack argument: loads the member at this+0x2c into ECX and calls 00eb1500 with (stack argument, this+0x30 value); returns 0 (ret 4). Bytes: 8… |
| `00ec5180` | T4MacroMember2C_ForwardOneArgToEb1570 | speculative | Thiscall with one stack argument: loads the member at this+0x2c into ECX, pushes the stack argument and calls 00eb1570; returns 0 (ret 4). Bytes: 8b 44 24 04; … |
| `00ec5220` | T4MacroMember2C_ForwardOneArgToEb15c0 | speculative | Thiscall with one stack argument: loads the member at this+0x2c into ECX, pushes the stack argument and calls 00eb15c0; returns 0 (ret 4). Bytes: 8b 44 24 04; … |
| `00ec52f0` | T4MacroFr_TwoVirtualPredicatesThenMember2C | speculative | Fastcall: calls virtual slot 0x20 of this; if zero calls slot 0x28; if that is also zero it reads the code from 00e86df0, passes it through 00e68ae0 and 00e797… |
| `00ec5340` | T4Macro_SetMemberStateFromToken7e4to7eb | speculative | Fastcall: reads the token from 00e86de0 and switches on it. Tokens 0x7e4, 0x7e5, 0x7e6, 0x7e7, 0x7e8, 0x7e9, 0x7ea and 0x7eb each write flag bytes (+0x69, +0x6… |
| `00ec54d0` | T4Macro_TokenToPtrTable7dcto7e3 | speculative | Reads the token from 00e86de0. For 0x7dc..0x7e3 it stores a table address into *param_1: PTR_DAT_01b8a224 (0x7dc), PTR_DAT_01b8a2c0 (0x7dd), PTR_DAT_01b8a2d4 (… |
| `00ec5590` | T4Macro_TokenToPtrTable7d5to7da | speculative | Reads the token from 00e86de0. For 0x7d5, 0x7d6, 0x7d7, 0x7d8, 0x7d9 and 0x7da it stores the address of PTR_DAT_01b8a260, PTR_DAT_01b8a270, PTR_DAT_01b8a280, P… |
| `00ec5960` | T4Macro_FR_FR_EmitMemberTextWithSeparator | speculative | Thiscall: if the member at this+0x2c has a nonzero +0x3c field it builds a 100-unit T4OutStrStream temporary, calls vslot 2 of the member with it, then picks a… |
| `00ec5ad0` | T4MacroFr_PassMember30AddressToEc54d0 | speculative | Fastcall: pushes the address this+0x30 and calls 00ec54d0 with it as the stack argument; returns with no stack cleanup (ret). Bytes: 8d 41 30; 50; e8 call 00ec… |
| `00ec5ae0` | T4MacroFr_PassMember30AddressToEc5590 | speculative | Fastcall: pushes the address this+0x30 and calls 00ec5590 with it as the stack argument; returns with no stack cleanup (ret). Bytes: 8d 41 30; 50; e8 call 00ec… |
| `00ec5af0` | T4MacroFr_PassMember30AddressToEc5620 | speculative | Fastcall: pushes the address this+0x30 and calls 00ec5620 with it as the stack argument; returns with no stack cleanup (ret). Bytes: 8d 41 30; 50; e8 call 00ec… |
| `00ec5b00` | T4MacroFr_VisitChild0OnMember2C | speculative | Fastcall: when this+0x1c and the count byte at this+0x19 are nonzero it pushes child 0 (dword at [this+0x1c]); otherwise it pushes the default object address 0… |
| `00ec5b30` | T4MacroFr_ForwardStackArgToEmitMemberText | speculative | Thiscall with one stack argument: forwards the argument to 00ec5960 with ECX unchanged, then returns 0 (ret 4). Bytes: 8b 44 24 04; 50; e8 call 00ec5960; 33 c0… |
| `00ec5620` | — | unknown | Thiscall with an output pointer argument: reads the token from 00e86de0. Token 0x7ec sets the dword at this+0x2c+0x20 to 2 and stores PTR_DAT_01b8a204 into *pa… |

### `lib.t4macro.grammar`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006afda0` | GrammarLexiconRegister | confirmed | No arguments. Calls 006af28c(0x24), then loops 36 times over a static table of string pairs (name, text). For each pair it builds string objects for both (004e… |
| `006b5892` | BuildMetaGrammarRules | confirmed | No arguments. Calls 0060016d, then interns 43 symbol names (004e4e6d and 004e4e15 with 005ef6c3; names such as ROOT_BLOCK, MACRO_SECTION, NAMED_ERROR) and 34 t… |

### `lib.t4macro.it_it`

**T4Macro_IT_IT::DefiniteArticleMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec35c0` | T4Macro_IT_IT::DefiniteArticleMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the byte at this+0x28 to 0, stores T4Macro_IT_IT::DefiniteArticleMacro::vftable into th… |

**T4Macro_IT_IT::IndefiniteArticleMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec3900` | — | unknown | Fastcall virtual slot 5: builds a 100-unit T4OutStrStream temporary and calls vslot 2 of param_1[5] with it. Reads 00e73e90(0) and 00ebea40, the code from 00e8… |

**T4Macro_IT_IT::ObjectPronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec3b00` | T4Macro_IT_IT::ObjectPronounMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the byte at this+0x28 to 0, stores T4Macro_IT_IT::ObjectPronounMacro::vftable into this… |

**T4Macro_IT_IT::SubjectPronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec33a0` | T4Macro_IT_IT::SubjectPronounMacro::Ctor | confirmed | Calls 00e86d10(param_2, param_3, param_4) as the base initialiser, sets the byte at this+0x28 to 0, stores T4Macro_IT_IT::SubjectPronounMacro::vftable into thi… |

**(no class)** (25)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec31c0` | T4MacroIt_FirstCharSOrZTest | speculative | Reads two characters with 00e73e90(0) and 00e73e90(1). Returns 0 for any first character other than S/s/Z/z. For Z/z returns 1. For S/s returns 0 when 00ebea40… |
| `00ec3260` | T4MacroIt_WriteArticleFormForPrefix | speculative | Writes C string param_2 (flag 0x1000000) and then, depending on the mode from 00e78e90 and role from 00e78f30 and the predicates 00ebea40 (first char of param_… |
| `00ec3430` | T4MacroIt_EmitFormFromObjectField18 | speculative | Thiscall: reads the code from 00e86df0 and its predicate 00e68ae0, mode from 00e78e90 and role from 00e78f30. Reads the int at the object pointed to by this+0x… |
| `00ec3550` | T4MacroIt_EmitSpaceUnlessApostropheBefore | speculative | Calls 00ebeb20. When that returns 0 and the object's length-like field at (vbase displacement +0xc) is nonzero, it reads the character at length-1 with 00e73e9… |
| `00ec3650` | T4MacroIt_EmitFormWithChildStream | speculative | Fastcall: builds a 100-unit T4OutStrStream temporary, calls vslot 2 of param_1[5] with it, then reads 00e73e90(0) and 00ebea40 of it, the code from 00e86df0 wi… |
| `00ec3bc0` | T4MacroIt_EmitFormWithFlagsOrSlot2C | speculative | Thiscall: reads the code from 00e86df0 and predicate 00e68ae0, role from 00e78f30 and mode from 00e78e90. When the int at the object pointed to by this+0x14, o… |
| `00ec49e0` | T4MacroIt_ForwardToEmitDroppingArg3B | speculative | Four-argument forwarder: calls 00ebe9f0(param_1, param_2, param_4) and drops param_3, then returns. |
| `00ec4a30` | T4MacroIt_WriteArticleByTokenAndInitialA | speculative | Fastcall: reads the first character of the word via 00e73e90(0), the code from 00e86df0 with predicate 00e68ae0, and switches on the dword at **(this+4), the t… |
| `00ec3ce0` | — | unknown | Takes four arguments; the third is a byte flag. Both branches of the flag test call 00ebe9f0(param_1, param_2, param_4) and return, so the flag has no effect. |
| `00ec3d60` | — | unknown | Four-argument forwarder: calls 00ebe9f0(param_1, param_2, param_4) and drops param_3, then returns. |
| `00ec3dd0` | — | unknown | Four-argument wrapper with a byte flag: builds a two-pointer temporary on the stack (0x018677c4 and 0x018677cc stored at [esp+0xc] and [esp+0x10]); the pseudoc… |
| `00ec3e50` | — | unknown | Four-argument wrapper with a byte flag: builds a two-pointer temporary at [esp+0xc] (0x018677ec, 0x018677f4) and the pseudocode then calls 00ebe9f0(param_1, pa… |
| `00ec3ed0` | — | unknown | Four-argument wrapper with a byte flag: builds a two-pointer temporary at [esp+0xc] (0x01867814, 0x0186781c) and the pseudocode then calls 00ebe9f0(param_1, pa… |
| `00ec3f50` | — | unknown | Four-argument wrapper with a byte flag: builds a two-pointer temporary at [esp+0xc] (0x0186784c nostri, 0x01867854 Nostri) and the pseudocode then calls 00ebe9… |
| `00ec3fd0` | — | unknown | Four-argument wrapper with a byte flag: builds a two-pointer temporary at [esp+0xc] (0x0186788c vostri, 0x01867894 Vostri) and the pseudocode then calls 00ebe9… |
| `00ec4050` | — | unknown | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 2 of param_1[5] with it. Reads the code from 00e86df0 with predicate 00e68ae0, role (00e78… |
| `00ec4210` | — | unknown | Thiscall: calls 00ebeb20; when it returns nonzero calls vslot 0x30 of this with no arguments, otherwise calls vslot 0x2c of this with param_2. Then calls 00e79… |
| `00ec4260` | — | unknown | Reads the code from 00e86df0 with predicate 00e68ae0, role (00e78f30 == 2) and mode (00e78e90). Calls 00ebe9f0(param_1, predicate, role==2) on both mode branch… |
| `00ec4310` | — | unknown | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 2 of param_1[5] with it. Reads the code from 00e86df0 with predicate 00e68ae0, role (00e78… |
| `00ec44a0` | — | unknown | Reads the code from 00e86df0 with predicate 00e68ae0, role (00e78f30 == 2) and mode (00e78e90). Calls 00ebe9f0(param_1, predicate, role==2) on both mode branch… |
| `00ec4550` | — | unknown | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 2 of param_1[5] with it. Reads the code (00e86df0 with 00e68ae0), role (00e78f30 == 2) and… |
| `00ec4760` | — | unknown | Fastcall: reads the code (00e86df0 with 00e68ae0) and role (00e78f30 == 2) and mode (00e78e90). Builds a 100-unit T4OutStrStream temporary and calls vslot 2 of… |
| `00ec4960` | — | unknown | Four-argument function with a byte flag as the third argument: both branches of the flag test call 00ebe9f0(param_1, param_2, param_4) and return, so the flag … |
| `00ec4b60` | — | unknown | Fastcall: reads the code (00e86df0 with 00e68ae0), role (00e78f30 == 2) and mode (00e78e90). Builds a 100-unit T4OutStrStream temporary and calls vslot 2 of pa… |
| `00ec4d10` | — | unknown | Fastcall: builds a 100-unit T4OutStrStream temporary and calls vslot 2 of param_1[5] with it. If the member at param_1[5]+0x3c is nonzero and 00e83fe0 is true … |

### `lib.t4macro.ja_jp`

**T4Macro_JA_JP::Phrase** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1dd0` | T4Macro_JA_JP::Phrase::Slot3_VisitSelectedChild | speculative | Takes child 0 of this+0x1c (or DAT_01bfdd68 when none). If the flag word of child 0 (object+0xc chain, bit 0x20000) is set it calls 00e79610 on child 0 and sel… |

**T4Macro_JA_JP::Title** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1e60` | T4Macro_JA_JP::Title::Slot3_VisitSelectedChild | speculative | Same code as 00ec1dd0: selects child 0 or 1 (1 after calling 00e79610 on child 0 when flag bit 0x20000 is set), calls 00e79560 on the selected child or DAT_01b… |

**T4Macro_JA_JP::VerbConjugation** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1c00` | — | unknown | Reads child 0 (or the default object) and its flag word; when bit 0x20000 of child 0 is set it calls 00e79610 on child 0 and starts from child 1. It calls 00e7… |

**T4Macro_JA_JP::VerbStem** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1b50` | T4Macro_JA_JP::VerbStem::Slot3_MapFormCodeToNode | likely | Takes child 0 of this+0x1c (or the static default object DAT_01bfdd68 when the array is empty), calls 00e79560 on it, then calls 00e86de0 and maps the code 0x2… |

### `lib.t4macro.lexer`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006b0529` | DfaLexerStep_B | likely | Same maximal-munch DFA step and fallback token rules as 006b0093 (same 005e05d6, 005db353 and 005db588 calls; same 0x10000000, 0x20000000 and 0x30000000 action… |
| `006b0093` | — | unknown | Fastcall with this in ECX. Runs a table-driven maximal-munch DFA step: transitions with 005e05d6(state, character), remembers the last accepting rule (005db353… |

### `lib.t4macro.parser`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006b0995` | LrParseDriverRun | likely | Thiscall with one stack argument (a callback object). Returns 0xfffffffe when this+0x44 is -1 and on the error path, and 0 on accept (action 0x4fff). Loop over… |

### `lib.t4macro.sv_se`

**SV_SE_Grammar::DemonstrativePronoun** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1f10` | SV_SE_Grammar::DemonstrativePronoun::EhUnwindVtableReset | confirmed | Stores SV_SE_Grammar::DemonstrativePronoun::vftable (018676a0) into the dword at its argument (mov [ecx], imm32; ret); no calls. |
| `00ec1f20` | SV_SE_Grammar::DemonstrativePronoun::ScalarDeletingDtor | confirmed | Stores the DemonstrativePronoun vftable 018676a0 into this, and when bit 0 of its byte argument is set calls _free(this) through thunk 00d82463; returns this i… |

**SV_SE_Grammar::InterrogativePronoun** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1f60` | SV_SE_Grammar::InterrogativePronoun::EhUnwindVtableReset | confirmed | Stores SV_SE_Grammar::InterrogativePronoun::vftable (018676a8) into the dword at its argument; no calls (mov [ecx], imm32; ret). |
| `00ec1f70` | SV_SE_Grammar::InterrogativePronoun::ScalarDeletingDtor | confirmed | Stores the InterrogativePronoun vftable 018676a8 into this, and when bit 0 of its byte argument is set calls _free(this) through thunk 00d82463; returns this i… |

**T4Macro_SV_SE::DefiniteArticleMacro** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec2140` | T4Macro_SV_SE::DefiniteArticleMacro::Slot12_PredicateOnRole… | speculative | Adjusts this by the vbase displacement at *(vtable+4), calls 00eae070 on it (cVar1), calls 00e78f30 to get a role value and tests it with 00eae150; returns tru… |
| `00ec21c0` | T4Macro_SV_SE::DefiniteArticleMacro::Slot13_TailRewriteIfRo… | speculative | Tests role value 00e78f30 with 00eae150 and table DAT_0183d6f8, and with 00eae140 and tables DAT_01862414, DAT_018676b0, DAT_0183d6f8; returns when no test pas… |
| `00ec2d70` | T4Macro_SV_SE::DefiniteArticleMacro::Slot5_EmitArticleAndCh… | speculative | Gets role (00e78f30) and mode (00e78e90); when 00e798d0(1,0) is nonnegative writes a prefix string (DAT_0183d6fc when mode is not 1; when mode is 1 PTR_LAB_018… |

**T4Macro_SV_SE::DemonstrativePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec2500` | T4Macro_SV_SE::DemonstrativePronounMacro::Slot5_EmitFormByC… | likely | Gets mode from 00e78e90 and role from 00e78f30, then switches on token code 00e86de0: 0x1f50 calls 00eae3a0(param_2, mode, role), 0x1f51 calls 00eae4b0, 0x1f52… |

**T4Macro_SV_SE::IndefiniteArticleMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec1ff0` | T4Macro_SV_SE::IndefiniteArticleMacro::Slot5_EmitArticleThe… | likely | Calls 00e78e90 (mode) and, when this+0x14 points at an object whose +0x34 field is nonzero and 00e84030 returns 0, sets a flag. When mode is 1 and the flag is … |

**T4Macro_SV_SE::IndefinitePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec2740` | T4Macro_SV_SE::IndefinitePronounMacro::Slot5_EmitFormByCode… | likely | Gets mode (00e78e90) and role (00e78f30); switches on 00e86de0: 0x1f56 calls 00eae580(param_2, mode, role), 0x1f57 calls 00eae5d0, 0x1f58 calls 00eae620, 0x1f5… |

**T4Macro_SV_SE::InterrogativePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec2680` | — | unknown | Takes its output target from the first stack argument param_1 (ECX is not read). Gets mode from 00e78e90 and role from 00e78f30; switches on 00e86de0: 0x1f54 c… |

**T4Macro_SV_SE::ObjectPronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec23e0` | T4Macro_SV_SE::ObjectPronounMacro::Slot15_InitFormIdFromTab… | speculative | Calls 00eae2d0(1) and stores its result at this+0x34; returns. |

**T4Macro_SV_SE::PossessivePronounMacro** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec3060` | T4Macro_SV_SE::PossessivePronounMacro::Slot5_EmitPossessive… | likely | Gets mode from 00e78e90 and role from 00e78f30; when param_1[5]+0x40 is set those two are replaced by fields +8 and +0xc of that object. Chooses mode (the stor… |
| `00ec2400` | T4Macro_SV_SE::PossessivePronounMacro::Slot15_InitFormIdFro… | speculative | Calls 00eae2d0(3) and stores its result at this+0x34; returns. |

**T4Macro_SV_SE::PossessiveReflexivePronounMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec24e0` | T4Macro_SV_SE::PossessiveReflexivePronounMacro::Slot15_Init… | speculative | Calls 00eae2d0(4) and stores its result at this+0x34; returns. |

**T4Macro_SV_SE::SubjectPronounMacro** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec2280` | T4Macro_SV_SE::SubjectPronounMacro::Slot5_EmitFormWithFlags | speculative | Uses mode 2 when the byte at this+0x31 is nonzero, otherwise the result of 00e78e90; calls 00e78f30 for a role value and passes (param_2, this+0x34, mode, role… |
| `00ec22f0` | T4Macro_SV_SE::SubjectPronounMacro::Slot9_CacheTwoFlagsFrom… | speculative | Calls virtual slot 0x30 and discards the result; stores the byte from virtual slot 0x34 at this+0x30; stores the byte from virtual slot 0x38 at this+0x31; call… |
| `00ec2320` | T4Macro_SV_SE::SubjectPronounMacro::Slot12_TokenCodeToSelec… | speculative | Calls 00e86de0 for a token code and switches on it: 0x1f43, 0x1f46, 0x1f47, 0x1f4c, 0x1f4f call 00e79720(3); 0x1f44, 0x1f48, 0x1f4a, 0x1f4d call 00e79720(1); 0… |
| `00ec2380` | T4Macro_SV_SE::SubjectPronounMacro::Slot13_CodeNotEquals0x1… | speculative | Calls 00e86de0 and returns 0 when its code equals 0x1f47, otherwise returns 1 (bytes: call 00e86de0; cmp eax,0x1f47; mov al,1 path; ret). |
| `00ec23a0` | T4Macro_SV_SE::SubjectPronounMacro::Slot14_CodeInRange0x1f4… | speculative | Calls 00e86de0 and computes x = code - 0x1f4d. Returns 1 when x is 0 or 1 (unsigned); otherwise returns x with its low byte cleared, which is 0 for codes near … |
| `00ec23c0` | T4Macro_SV_SE::SubjectPronounMacro::Slot15_InitFormIdFromTa… | speculative | Calls 00eae2d0(0) and stores its result at this+0x34; returns. |

**(no class)** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ec28e0` | T4MacroSvSe_Slot8_ExpandArgumentsAndEmit | speculative | Shared expansion routine (vslot 8) for the SV_SE macros. Reads the argument count byte at this+0x19; with zero arguments it calls 00e86ea0 and returns 0x5c67 o… |
| `00ec2ce0` | T4MacroSvSe_Slot10_ChildFieldAt8 | speculative | Thiscall accessor: if the argument array pointer at this+0x1c is nonzero and the index param_2 is below the count byte at this+0x19, returns the dword at child… |
| `00ec2d10` | T4MacroSvSe_Slot11_SelectedChildHasFlag20000 | speculative | Fastcall predicate: takes the child at index this+0x28 (or a zero slot when the array is empty or the index is out of range). When that child is nonnull and ei… |
| `00ec1fb0` | — | unknown | Calls virtual slot 0x20 of this; if that returns 0 calls virtual slot 0x24; if that also returns 0 calls slot 0x3c of the object at this+0x14. Returns the firs… |
| `00ec2420` | — | unknown | Test A = 00e75020(DAT_0176ce84), a dword table of 0x67..0x6e; test B = 00e75020(DAT_0176ce74), a dword table of 0x6b..0x72. If A is false and B is true: writes… |

### `ui.locale`

**LanguageDataSource** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a85660` | LanguageDataSourceDtor | confirmed | Destructor body of the language data source: stores the three LanguageDataSource vtables and then runs the shared locale data-source destructor 00a855b0. |
| `00a85860` | LanguageDataSourceCtor | confirmed | Constructor of the BaseClient.Language data source: runs the shared locale constructor 00a857a0 with the owner pointer and the name "BaseClient.Language", then… |
| `00a858e0` | LanguageDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of LanguageDataSource: runs the destructor body 00a85660 and frees the object with _free when the delete flag bit … |
| `00a852a0` | — | unknown | Refreshes the language table (vtable slot [14]): triggers the underlying accessor (this+0x34 slot 3), then for every entry of the global language list (0076971… |

**LocaleDataSource** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a85140` | LocaleLanguageDataSource::GetColumnName | confirmed | Column-title getter (vtable slot [1] of both LocaleDataSource and LanguageDataSource): switches on the column index and returns the literal column names "ID" (… |
| `00a851d0` | LocaleLanguageDataSource::GetRowCount | confirmed | If the loaded flag at this+0x238 is clear, invokes the sibling LocaleDataSource method at vtable byte offset +0x38 (slot 14); then tail-jumps to 010c8f30 for t… |
| `00a85370` | LocaleLanguageDataSource::GetField | confirmed | Lua/UI field getter (vtable slot [4] of LocaleDataSource and LanguageDataSource): if this+0x238 is clear, invokes the LocaleDataSource method at the sibling su… |
| `00a855b0` | LocaleDataSourceDtor | confirmed | Destructor body of the locale data source: stores the three LocaleDataSource vtables, frees the int array owned by the member at param_1[0x95] (SoeUtil::Array<… |
| `00a85780` | LocaleDataSourceScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot [0]) of LocaleDataSource: runs the destructor body 00a855b0 and frees the object with _free when the delete flag bit 0 … |
| `00a85240` | — | unknown | Refreshes the locale table (vtable slot [14]): triggers the underlying table accessor through this+0x34 slot 3, walks the global locale list (00769710 -> +4 ->… |
| `00a856c0` | — | unknown | Constructor of the BaseClient.Locale data source: registers the name through 00922890(PTR_s_BaseClient.Locale), stores the three LocaleDataSource vtables, init… |
| `00a857a0` | — | unknown | Shared constructor body of the locale/language data sources: registers the caller-supplied name (param_3) through 00922890, stores the three LocaleDataSource v… |

**T4Macro_PT_PT::DoPrepositionArticleMacro** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ebf440` | T4Macro_PT_PT::DoPrepositionArticleMacro::Dispatch | likely | Ret 0x10, four stack arguments. Builds a 16-byte stack record of four pointers, 018665a0/a4/a8/ac when the char argument (4th) is nonzero, otherwise 01866590/9… |

**(no class)** (27)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0097b520` | IStringFormatGlobalTextKey | confirmed | Calls FUN_00402fd0(param_1, 'Global.Text.%d', param_2): formats the integer param_2 into the IString at param_1 as the key Global.Text.<n>. Returns nothing. |
| `0097b940` | IStringFormatGlobalTextKeyAtPlus10 | confirmed | Calls FUN_00402fd0(this+0x10, 'Global.Text.%d', param_2): formats the localisation key Global.Text.<param_2> into the IString at this+0x10. Returns nothing. |
| `0097bca0` | ExpandGlobalTextIdOrLog | confirmed | Builds the key Global.Text.<param_2> in a StringFixed temporary, assigns the static text at DAT_0175b400, and looks the key up with FUN_00e54290. When the look… |
| `00c34880` | FormatTimeAgo | confirmed | Selects minute, hour, day or year units from an elapsed count, formats the localized unit and relative-time strings, and handles values under one minute separa… |
| `00d04b60` | LanguageIdToCode | confirmed | Switch on the index (0..0xb): returns the code string "Invalid" for 0, then "de", "en", "es", "fr", "it", "ja", "ko", "pt", "ru", "sv", "zh" for 1..11; any oth… |
| `00979fc0` | LocaleNameForLanguageId | likely | Maps a numeric language id to a locale string: 1 = zh_CN, 2 = de_DE, 3 = fr_FR, 4 = en_GB, 5 = ja_JP, 6 = ko_KR, 7 = zh_TW, 9 = es_ES, 10 = it_IT, 12 = ru_RU, … |
| `0097afe0` | IStringFormatWithLocaleNameFour | likely | Selects the underscore-form locale name for the language id at this+0xc (same id table as 00979fc0, default en_US), passes it with four varargs (param_3..param… |
| `0097b0e0` | IStringFormatWithLocaleNameFourB | likely | Same body as 0097afe0 with FUN_00e54a50: locale name from the id at this+0xc (default en_US), four varargs (param_3..param_6), temporary text assigned with FUN… |
| `0097b1e0` | IStringFormatWithLocaleNameOne | likely | Selects the underscore-form locale name for the language id at this+0xc (default en_US), passes it with one vararg (param_3) to FUN_00e54c80, assigns the retur… |
| `0097b2d0` | IStringFormatWithLocaleNameTwo | likely | Selects the underscore-form locale name for the language id at this+0xc (default en_US), passes it with two varargs (param_3, param_4) to FUN_00e54b80, assigns… |
| `0097b3d0` | IStringFormatWithLocaleNameOneB | likely | Selects the underscore-form locale name for the language id at this+0xc (default en_US), passes it with one vararg (param_3) to FUN_00e54950, assigns the retur… |
| `0097be40` | StringLookupByIdWithFallback | likely | Calls the provider at this+8 through its vtable slot 0xcc; when that returns zero it calls FUN_00e52260, then 0097bca0(param_2, param_3) and, when that fails, … |
| `0097bf40` | ExpandTextWithRoleNodes | likely | Creates an empty IString temporary and calls the provider at this+8 through vtable slot 0xcc. When that returns zero it calls FUN_00e52260, walks the linked li… |
| `0097c090` | ExpandTextWithRoleArray | likely | Calls the provider at this+8 through vtable slot 0xcc. When that returns zero it calls FUN_00e52260, runs 0097b960 (RoleTemplateTextBuild) on each of the param… |
| `0097c170` | StringLookupByIdToOut | likely | Initialises the IString at param_1 as empty (vtable, static empty buffer, length and capacity zero), then calls 0097be40 with (param_2, param_1). The receiver … |
| `0097c1e0` | ExpandTextWithRoleArrayToIString | likely | Initialises the IString param_2 as empty (vtable, static empty buffer, zero length and capacity), then calls 0097c090 with (param_2, param_3, param_4, the addr… |
| `0097c280` | PlayerNameTemplateExpand | likely | On first use (flag bit 0 of DAT_01be5244 clear) it sets the flag, queries the object from FUN_00769710()+4 through the vtable slot 0x14 with the name 'PlayerNa… |
| `00d04bf0` | IndexedEntryOrEmpty | likely | Returns the pointer stored at table 0x183d754 element index*2 (8-byte records) for 1 <= index <= 0xea, else the empty string at 0x175b400. |
| `00d04e80` | ParseLocaleTag | likely | Parses a short tag (under 6 chars) for '_' (0x5f) or '-' (0x2d): with no separator it resolves the whole tag through 00d04d70 (language) and then 00d04db0 (cou… |
| `009a0770` | FormatStringLocalized | speculative | Builds a large temporary wide string, appends the input string, formats the result with FUN_0099F6E0, then destroys both temporary strings. |
| `009a0830` | FormatStringWithFallback | speculative | Selects one of two static wide strings based on the input byte, performs string lookup/formatting, then releases temporary wide strings. |
| `00d04d70` | TableFind11ByString | speculative | Searches the 11-entry table at 0x183d700 with 00d04c10 and the predicate at 00d04d10 against param_1; when a record is found it compares the string at its +4 w… |
| `00d04db0` | TableFind234ByString | speculative | Searches the 234-entry table at 0x183d758 with 00d04c90 and the predicate at 00d04d40 against param_1; when a record is found it compares the string at its +4 … |
| `0097a180` | — | unknown | Calls LocaleNameForLanguageId (00979fc0) with the dword at this+0xc and the param_2 flag, discarding the returned string. Returns nothing. |
| `0097ae80` | — | unknown | Initialises the IString param_2 as empty, selects the underscore-form locale name for the language id at this+0xc with the same id table as 00979fc0 (default e… |
| `0097c260` | — | unknown | Calls 0097c090 with (param_2, param_3, param_4, the address of its stack argument block) and returns nothing; param_1 is unused. |
| `00d04e00` | — | unknown | Formats into the IString at param_3 with the code from 00d04b60 (the language index in param_1) and a country-table entry: when param_4 is non-zero it formats … |

## Source shards

[[S0041|S0041]] (13), [[S0113|S0113]] (1), [[S0130|S0130]] (34), [[S0151|S0151]] (4), [[S0154|S0154]] (1), [[S0171|S0171]] (9), [[S0174|S0174]] (23), [[S0195|S0195]] (3), [[S0233 008ec2a0-008f0cc0|S0233]] (3), [[S0261 00976740-0097c910|S0261]] (18), [[S0264 00989810-0098eae0|S0264]] (5), [[S0267 0099ca10-009a1c60|S0267]] (3), [[S0308 00a81470-00a87b30|S0308]] (12), [[S0342 00b39c10-00b3f310|S0342]] (1), [[S0368|S0368]] (2), [[S0376 00be5b30-00bea020|S0376]] (2), [[S0389 00c34110-00c39a40|S0389]] (1), [[S0395|S0395]] (1), [[S0427 00d01e40-00d07440|S0427]] (6), [[S0439|S0439]] (5), [[S0450|S0450]] (1), [[S0452|S0452]] (15), [[S0453|S0453]] (3), [[S0489|S0489]] (140), [[S0493|S0493]] (35), [[S0497|S0497]] (154), [[S0509|S0509]] (209), [[S0510|S0510]] (36), [[S0543|S0543]] (25), [[S0582|S0582]] (13), [[S0591|S0591]] (1), [[S0592|S0592]] (1), [[S0675|S0675]] (5), [[S0682|S0682]] (1), [[S0685|S0685]] (10), [[S0689|S0689]] (2), [[S0694|S0694]] (5), [[S0729|S0729]] (9), [[S0752|S0752]] (6), [[S0757|S0757]] (1), [[S0809 0066ca4d-0147b010|S0809]] (1)
