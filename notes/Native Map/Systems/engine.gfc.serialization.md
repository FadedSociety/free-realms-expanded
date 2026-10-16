# System: engine.gfc.serialization

75 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004354b2` | `DtorUnlinkFromOwner` |  | Destruction of GInterface-derived data objects: unregister from the owning container through its slot +0x5c before the base-class destructor resets the vtables. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00482437` | `ReadStringStringFormatMembers` |  | Property-input handler in anonymous vtable slot [4][59]; reads members "String" and "StringFormat" into offsets +0x34 and +0x38. Owning class name is unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004824b5` | `CopyStringStringFormatMembers` |  | Copy-member handler in anonymous vtable slot [4][60], paired with 00482437; owning class name is unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486369` | `ReadGBitmap` |  | Bitmap read handler of the GFC bitmap class (its write counterpart is 00486586); the Header/ColorKey/Data/AlphaData member names define the GFC bitmap serialization contract. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486586` | `WriteGBitmap` |  | Bitmap write handler paired with the reader 00486369. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486ce2` | `ReadColorDataPlane` |  | Deserializer for the raw color-plane (texture) data of a GFC image object; the GFC_CLR_ORDER_ARGB default pins the pixel-format flags meaning. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `013c23f0` | `SerialAppendInteger` |  | Compact integer writer of the GFC-style serializer: the counterpart of SerialReadInteger (013c24d0). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c2640` | `SerialStartAppendContainer` |  | Begins writing an STL container into the serializer by writing its element count first. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c2690` | `SerialStartReadContainer` |  | Begins reading an STL container from the serializer by reading its element count first. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c27e0` | `SerialReadTimeValue` |  | Reads the two-word time value written by SerialAppendTimeValue (013c2790). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c2860` | `SerialAppendString` |  | Writes a std::string argument (length then bytes with the NUL) into the serializer. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c28d0` | `SerialReadString` |  | Reads a length-prefixed std::string written by SerialAppendString (013c2860). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟢 likely (14)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00403ddf` | `GSimpleClassIO::GSimpleClassIO` | GSimpleClassIO | GFC serialization adaptor that binds a class name plus read/write property handlers (GPropertyInput/GPropertyOutput/GString) for simple class load/save. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `004352c1` | `StringAccessWrapperResolve` |  | Lazy resolution of the string-accessor object inside the GStringAccessWrapper and invocation of one accessor method with a key argument; the class is independently anchored - this function is the last slot (+0x50) of th… | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00435394` | `GStringAccessInit` |  | Constructor path of a class that embeds the GStringAccess interface (secondary vtable at +0x18) used by the GFC data-access layer. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436c51` | `InitTreeHeader19Thunk` |  | Wraps FUN_0041bd1c (map-accepted as InitTreeHeader19, the MSVC red-black-tree header initialiser) over ECX; it does not install a data-access sub-vtable. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436c5d` | `InitTreeHeader1DThunk` |  | Wraps FUN_0041bd41 (map-accepted as InitTreeHeader1D) over ECX; not a sub-vtable installer. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436c75` | `InitContainerHeaderThunk1` |  | Wraps the container-header initializer FUN_0041f75d over ECX; not a sub-vtable installer. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436c81` | `InitContainerHeaderThunk2` |  | Wraps the container-header initializer FUN_0041f787 over ECX; not a sub-vtable installer. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436e5d` | `AccessObjectScalarDeletingDtor` |  | MSVC scalar deleting destructor of the data-access sub-object class whose constructor is 00436de0. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00482589` | `ReadPropertyAsNumberOrObject` |  | Property-read helper that converts a GFC tagged property value into either a 16-bit number or an interface object, for the class IO code that calls it. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00482889` | `ComposeStringFromMembers` |  | Builds and releases temporary GStrings while composing the members at +0x68 and +0x6C; no callers are recorded, so its owning class and higher-level role are unknown. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `009357a0` | `ReadBoundedU32` |  | Reads one bounded 32-bit value into caller-provided storage. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `00935830` | `SerializeRecordFieldsAndNestedList` |  | Serializes fixed fields, variable-length data, and a nested record list; the concrete record type is not established. | [[Native Map/Shards/S0247 00934740-00939200|S0247]] |
| `00a24750` | `ReadBoundedScalarRecord` |  | Parses a fixed scalar record shape; its callers use the helper while reading encounter data. | [[Native Map/Shards/S0291 00a244a0-00a29500|S0291]] |
| `013c2790` | `SerialAppendTimeValue ⚠downgrade` |  | Writes a two-word time value (low word then high word) through the integer writer. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟡 speculative (27)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0043147e` | `SetAccessValueSlot34Or1C ? ⚠downgrade` |  | Dispatcher over the two accessor paths: slot +0x34 of the resolved/created interface first, slot +0x1c of the pre-existing source object only as fallback. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004314dc` | `CallAccessSlot3C ?` |  | Mechanical accessor call: dispatch slot +0x3c of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `0043152b` | `CallAccessSlot38 ?` |  | Mechanical accessor call: dispatch slot +0x38 of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `0043157a` | `CallAccessSlot40 ?` |  | Mechanical accessor call: dispatch slot +0x40 of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004315c9` | `CallAccessSlot44 ?` |  | Mechanical accessor call: dispatch slot +0x44 of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431618` | `CallAccessSlot48 ?` |  | Mechanical accessor call: dispatch slot +0x48 of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431667` | `CallAccessSlot58 ?` |  | Mechanical accessor call: dispatch slot +0x58 of the interface resolved by FUN_004128ee with three value arguments; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004316b9` | `CallAccessSlot4C ?` |  | Mechanical accessor call: dispatch slot +0x4c of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431708` | `CallAccessSlot50 ?` |  | Mechanical accessor call: dispatch slot +0x50 of the interface resolved by FUN_004128ee for one value pair; read vs write is not anchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431a36` | `CallObjectSlot34 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2 (map-accepted as InitListenerRefs), one of a family that differs only in the slot id; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431a80` | `CallObjectSlot3C ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431ace` | `CallObjectSlot38 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431b1c` | `CallObjectSlot40 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431b6a` | `CallObjectSlot44 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431bb8` | `CallObjectSlot48 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431c06` | `CallObjectSlot4C ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00431c54` | `CallObjectSlot50 ?` |  | Mechanical slot dispatch on the object resolved by FUN_0041aaf2; object type, direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436c69` | `Wrapper_0041f717 ? → InitPooledListHeaderThunk` |  | Bare wrapper around FUN_0041f717; that helper's role is unmapped, so the wrapper's net effect is unknown. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00486dfe` | `AllocateColorPlaneFromSource ? ⚠downgrade` |  | Buffered-copy path of the color-plane reader that mirrors an existing source image's format/size (gate FUN_004c6e7c is the always-true stub). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00be1d60` | `StreamReadSixDwords ?` |  | Reads six dwords from a stream object into the output struct at +0, +4, +8, +0xc, +0x10 and +0x14. The stream has a cursor at +8, an end at +0xc and an error flag byte at +0x10: when fewer than 4 bytes remain it writes … | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be1e50` | `StreamReadTwoPairs_0x38 ?` |  | Reads two 8-byte values (two dwords each) from the stream at cursor +8 into the struct at +0x38/+0x3c and +0x40/+0x44, with the same bounds check and error flag as the reader at 00be1d60, then calls FUN_010831c0 with th… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be25e0` | `StreamReadShortAndByte ?` |  | Reads a 16-bit signed value from the stream cursor (+8) into this+4 and an 8-bit signed value from the next byte into this+8, with bounds checks against the stream end (+0xc) and error flag (+0x10): on exhaustion it wri… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be26c0` | `StreamReadShortAndByte_B ?` |  | Same body as 00be25e0: reads a 16-bit signed value from the stream cursor into this+4 and an 8-bit signed value into this+8, with the bounds checks and exhaustion handling (zero, error flag at stream+0x10, cursor set to… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be2ec0` | `StreamReadRecordByte_0xc_B ?` |  | Same body as 00be2e80: calls 00be25e0 on the record, then reads one signed byte from the stream cursor into record+0xc with the same exhaustion handling. | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be2f50` | `StreamReadRecord_0x18 ?` |  | Reads a record from a stream (second argument) into the output (first argument): calls 00be25e0 with the stream, then reads a signed byte into out+0xc, a dword into out+0x10, a dword into out+0x14 and a boolean byte int… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be5360` | `HashList20_ReadFromStream ?` |  | Clears the 20-bucket list through 00be2220, reads an entry count dword from the stream (stream argument: cursor +8, end +0xc, error flag +0x10; zero and error on exhaustion), then for each entry reads a dword key from t… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `013c26e0` | `SerialNoopReturnTrue ?` |  | Takes its arguments but does nothing: it sets the return byte to 1 (b0 01) and runs the stack cookie check (bytes 51 / a1 d8 39 b8 01 / 33 c4 / 89 04 24 / 8b 0c 24 / 33 cc / b0 01 / e8 / 59 / c3). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## ⚪ unknown (22)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `004312ed` | `— ⚠reject` |  | Mechanical accessor dispatch with probe/fallback/flush around slot +0x34 of the interface resolved by FUN_004128ee; neither direction (read vs write) nor value type is anchored, and the same slot ids appear as getter ac… | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004313b2` | `— ⚠reject` |  | Mechanical accessor dispatch around slot +0x3c with the probe/fallback/flush sequence; direction and value type are unanchored. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004353e2` | `— ⚠reject` |  | Create/destroy dispatch used for the data-access wrapper class family; the objects receive the derived vtable set above (the GDataAccessImpl base vtable is written only transiently), and the map pairs this factory with … | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `00436de0` | `— ⚠reject` |  | Constructor of the property/access object created by the control get-or-create helpers (004128ee/0041aaf2) when a control's data object is missing - the same vtable set emitted by the constructor at 00528f2f. | [[Native Map/Shards/S0011 004309d4-00436f31|S0011]] |
| `004824fe` | `— ⚠reject` |  | Size calculation in anonymous vtable slot [4][61], paired with 00482437 and 004824b5; returns the sum of its three size-helper results. Owning class name is unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00485930` | `— ⚠reject` |  | Central extractor that turns a GPropertyValue descriptor into the caller's destination buffer, used by the GFC property/serialization code; the switch documents the property value representation (inline small data, poin… | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00485e3f` | `— ⚠reject` |  | Deserializer for the GFC GBitmapInfoHeader block (bitmap pixel-format descriptor) consumed by the bitmap reader 00486369. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00485f9d` | `— ⚠reject` |  | Serializer counterpart of 00485E3F for the GBitmapInfoHeader block; called by the bitmap writer 00486586. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004867a5` | `— ⚠reject` |  | Writer for the frame-list properties Index and Bitmaps[].Frame. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486b31` | `— ⚠reject` |  | Writer that adds AnimateSpeed and AnimateSpeedList.FrameSpeed properties to the frame-list serialization. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `008bb880` | `— ⚠reject` |  | Compares two byte spans first by length, then by four-byte chunks and remaining bytes; returns zero for equal spans and a nonzero ordering/status value for differences. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00b0f550` | `—` |  | Reads two consecutive 16-bit values from a bounded byte buffer into output fields and marks failure when either read is out of bounds. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0f5f0` | `— ⚠reject` |  | Reads a length-prefixed integer sequence, advances the input pointer, then reads one byte into a boolean or marks the reader failed. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0f680` | `—` |  | Decodes integer, boolean, and 64-bit values from a bounded byte reader, writes zero defaults, and sets the failure flag when truncated. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b10580` | `—` |  | Calls the buffer reader and reads a 32-bit word into the output at offset 0x0c, setting failure state when data is absent. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00be1f30` | `— ⚠reject` |  | Reads a 32-byte record from a stream object (cursor at +8, end at +0xc, error flag byte at +0x10): two 8-byte pairs into +0 and +8, dwords into +0x10, a byte boolean into +0x14 (nonzero test), dwords into +0x18 and +0x1… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be2e80` | `— ⚠reject` |  | Fastcall-style reader: calls 00be25e0 on the record (second argument in ecx) with the stream as stack argument (reads a 16-bit value into record+4 and a byte into record+8), then reads one signed byte from the stream cu… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be2f10` | `— ⚠reject` |  | Calls 00be26c0 on the first argument (stream-side reader), then reads one dword from the stream cursor (stream+8, end at stream+0xc) into the second argument at +0xc, with the same exhaustion handling (zero, error flag … | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be3060` | `— ⚠reject` |  | Reads a record from a stream (second argument) into the output (first argument): three bounds-checked dwords into out+8, +0xc and +0x10, a call to 00be1e50 on the stream (two 8-byte pairs), a boolean byte into out+0x60,… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `00be53f0` | `— ⚠reject` |  | Drains the 16-bucket ordered list whose head is at the second argument +8 through 00be0a00; then reads an entry count dword from the stream (first argument: cursor +8, end +0xc, error flag +0x10; zero and error on exhau… | [[Native Map/Shards/S0375 00be0120-00be58f0|S0375]] |
| `013c24d0` | `— ⚠reject` |  | Reads an integer written by SerialAppendInteger (013c23f0). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c25b0` | `— ⚠reject` |  | Decodes one integer like SerialReadInteger (013c24d0) without the state check: in compact mode it reads the header byte at the read position (low nibble, count from bits 5-7, complement when bit 4 is set) and up to the … | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## Wire contract (client side) and server hypotheses

| addr | name | dir | packet | opcode | server hypothesis |
|---|---|---|---|---|---|
| `00a24750` | `ReadBoundedScalarRecord` | recv | BaseEncounterPacket |  |  |
