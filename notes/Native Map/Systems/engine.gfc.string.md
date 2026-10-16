# System: engine.gfc.string

36 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0041d2e3` | `GStringAccessWrapper::ScalarDeletingDtor` | GStringAccessWrapper | Scalar deleting destructor bound by the GStringAccessWrapper vtable; delegates cleanup to its GDataAccessWrapper base destructor. | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `007054c0` | `GCStringCopyConstructorClosure` | GCString | Copies a GCString value through the GFC string copy-construction closure. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007054d0` | `GStringCopyConstructorClosure` | GString | Copies a GString value through the GFC string copy-construction closure. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `008c5b90` | `BaseClient::StringCodeLookup` | BaseClient | Virtual slot 4 of BaseClient, shared with PlayClient (inherited implementation, named for BaseClient). Maps a string code to a value and logs unmapped codes. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `008c5c50` | `BaseClient::StringCodeLookupSlot3` | BaseClient | Virtual slot 3 of BaseClient, shared with PlayClient (inherited implementation). Maps a string code to a value through a different callback slot. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `008c5d10` | `BaseClient::StringCodeFind` | BaseClient | Virtual slot 5 of BaseClient, shared with PlayClient (inherited implementation): non-logging read of the same string-code table. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `0093c3a0` | `InitIStringFields → SoeUtil::IString<char>::Ctor` |  | Initialize an inline IString object. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0041b41f` | `GStringAccessWrapper::ApplySlotValue` | GStringAccessWrapper | Virtual slot 8 implementation shared by GStringAccessWrapper vtables. | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `0044a33e` | `GStringObjectActionDispatcher` |  | Lifecycle operation dispatcher for GStringObject: initialization, copy construction, member release and scalar deletion. The class connection comes from the initializer and copy-initializer callees. | [[Native Map/Shards/S0015 00447e1a-0044e19f|S0015]] |
| `0093c410` | `CopyThenInitIString` |  | Initialize a string-bearing record from an existing first-field value. | [[Native Map/Shards/S0248 009392c0-0093d740|S0248]] |
| `00a18f90` | `DestroyMemberAtOffset8AndMaybeFree` |  | Deleting wrapper for an object with a member at +8; the enclosing type is unidentified, and the member role is inferred from the destructor callee. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a19000` | `LuaParamArrayAndIString_Dtor` |  | Composite destructor body for an embedded Lua-parameter array and IString; the body does not test a deleting flag or free the outer object. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00a1ad40` | `DestroyLuaParamArrayAndIStringAndMaybeFree` |  | Scalar deleting wrapper around the composite array-and-IString destructor body; owner class is not identified. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00b70a00` | `ReleaseEmbeddedIStringBuffer` |  | Shared cleanup for the string member embedded at +0x20 in the camera-related objects passed by 00b70a70 and 00b71320. | [[Native Map/Shards/S0352 00b6cd80-00b71320|S0352]] |
| `00bd5520` | `HashList256_Insert` |  | Inserts a CodeStringMapping node into the 256-bucket list built by the loader 00bd55d0. Same insert pattern as HashListMap8_Insert with a different node layout. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00c2f690` | `CollapseAndResizeWideString` |  | String normalization used during item-search indexing. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c2f6f0` | `LowercaseWideStringWithCapacity` |  | Case normalization used by the search index. | [[Native Map/Shards/S0388 00c2df40-00c34100|S0388]] |
| `00c90910` | `FormatValueAndReturnField8` |  | Format a value into a string output and return a stored field. | [[Native Map/Shards/S0406 00c8f830-00c95560|S0406]] |
| `00cc56b0` | `FormatIntegerIntoBuffer` |  | Formats one integer into the supplied string buffer and returns its stored length or status field. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟡 speculative (13)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0047b56c` | `UpdateGlobalString ? ⚠rename` |  | Updates shared string state through two string helpers and performs cleanup; the meaning of the global and arguments remains unresolved. | [[Native Map/Shards/S0025 00477947-0047ca0c|S0025]] |
| `008bb840` | `CStringEqual ?` |  | Compares the null-terminated string at this+4 byte by byte with param_2 and returns whether both reach the same terminating byte. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `0090bff0` | `IString::IString ?` | SoeUtil::IString<char> | Initializes an empty shared string. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090c060` | `IStringCopyCtor ? ⚠rename` | SoeUtil::IString<char> | Copies the leading source dword, initializes an empty IString at the destination's second field, and clears its trailing length and capacity. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090eca0` | `IString::IString ?` | SoeUtil::IString<char> | Initializes an empty string field. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090ed10` | `IStringCopyCtor ?` | SoeUtil::IString<char> | Copies the source first dword, initializes an empty IString subobject, and returns the destination pointer. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090f190` | `IString::IString ?` | SoeUtil::IString<char> | Initializes an empty string field. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090f200` | `IStringCopyCtor ? ⚠rename` | SoeUtil::IString<char> | Copies the source first dword, initializes an empty IString subobject, and returns the destination pointer. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090f670` | `IString::IString ?` | SoeUtil::IString<char> | Initializes an empty string field. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `0090f6e0` | `IStringCopyCtor ?` | SoeUtil::IString<char> | Copies the source first dword, initializes an empty IString subobject, and returns the destination pointer. | [[Native Map/Shards/S0239 0090a890-009105b0|S0239]] |
| `00a19d00` | `Wrapper_00a19000 ?` |  | Compiler cleanup wrapper around 00a19000; the containing temporary's owner is outside this shard. | [[Native Map/Shards/S0289 00a18a90-00a1dd30|S0289]] |
| `00bd5440` | `NodeInitWordAndIString ?` |  | Thiscall (ECX = node): stores the first stack arg at +0, sets an SoeUtil::IString<char> vftable at +4 with the shared empty buffer at +8 and zeros at +0xc and +0x10, calls 00402ce0 (IStringAssign) with the second stack … | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00bd54b0` | `NodeCtorFromPointers ?` |  | Thiscall (ECX = node base): calls 00bd5440 with the dword at *param_3 and the pointer *param_4 as its two stack args, stores the dword at *param_2 into this+0x1c, and returns this. | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |

## ⚪ unknown (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0040df4e` | `— ⚠reject` |  | Coerces a typed value object into a GString destination, choosing the cheap narrow copy when the source is already a GString; the embedded type name 'p<c GString>' anchors the target type. | [[Native Map/Shards/S0003 0040affb-004103c5|S0003]] |
| `00ab6210` | `—` |  | Adds 0x2864 to ECX, then jumps to the shared string-assignment implementation without its own assignment body. | [[Native Map/Shards/S0317 00ab2c80-00ab7380|S0317]] |
| `00bd55d0` | `— ⚠reject` |  | Loads the CodeStringMapping table (string key to id) from its text resource. Called from PlayClient::Initialize (0092b170). | [[Native Map/Shards/S0372 00bd37e0-00bd5d40|S0372]] |
| `00c1f880` | `— ⚠reject` |  | Forms a combined string from an argument and a global string; the exact user-facing role is unknown. | [[Native Map/Shards/S0386 00c1f190-00c26aa0|S0386]] |

