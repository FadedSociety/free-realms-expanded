# System: crt.string

34 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00416aaf` | `_Eos` |  | Library function (Ghidra-matched VS2003-2008 basic_string<char>) used by every string assignment/erase/grow path in this build; not game logic. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00423532` | `std::basic_string::basic_string` |  | The C-string constructor every std::string construction in this library region funnels through (200+ callers, including the STL throw helpers). | [[Native Map/Shards/S0008 00422dcc-00426a69|S0008]] |
| `0099dde0` | `IStringFixed16Ctor` | SoeUtil::IStringFixed<unsigned_short,16> | Constructs the fixed-capacity UTF-16 string base object. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099de40` | `IStringFixed16Allocate` | SoeUtil::IStringFixed<unsigned_short,16> | Inline storage allocation callback shared with WideStringFixed<16>. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099de70` | `IStringFixed16FreeBuffer` | SoeUtil::WideStringFixed<16> | Shared fixed-string deallocation callback inherited by WideStringFixed<16>. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099de90` | `IStringFixed16Dtor` | SoeUtil::IStringFixed<unsigned_short,16> | Cleans up a fixed UTF-16 string and its shared IString base. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099dfe0` | `IStringFixed8Ctor` | SoeUtil::IStringFixed<unsigned_short,8> | Constructs a fixed-capacity UTF-16 string. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099e040` | `IStringFixed8Allocate` | SoeUtil::IStringFixed<unsigned_short,8> | Inline buffer allocation callback shared with WideStringFixed<8>. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099e070` | `IStringFixed8FreeBuffer` | SoeUtil::WideStringFixed<8> | Shared fixed-string deallocation callback. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099e6a0` | `IStringFixed16ScalarDeletingDtor` | SoeUtil::IStringFixed<unsigned_short,16> | Scalar deleting destructor for fixed UTF-16 string storage. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099e6c0` | `IStringFixed8ScalarDeletingDtor` | SoeUtil::IStringFixed<unsigned_short,8> | Scalar deleting destructor for the fixed UTF-16 string. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099e9f0` | `IStringFixed8192Allocate` | SoeUtil::IStringFixed<unsigned_short,81… | Inline allocation callback shared by this fixed-string template. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099ec00` | `IStringFixed8192ScalarDeletingDtor` | SoeUtil::IStringFixed<unsigned_short,81… | Scalar deleting destructor for the fixed UTF-16 string. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f540` | `WideStringFixed16Ctor` | SoeUtil::WideStringFixed<16> | Constructs the wide fixed-size string wrapper. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f5a0` | `WideStringFixed8Ctor` | SoeUtil::WideStringFixed<8> | Constructs the wide fixed-size string wrapper. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f620` | `WideStringFixed16::ScalarDeletingDtor` | SoeUtil::WideStringFixed<16> | Scalar deleting destructor identified by the slot-zero vtable and deleting-flag branch. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f680` | `WideStringFixed8::ScalarDeletingDtor` | SoeUtil::WideStringFixed<8> | Scalar deleting destructor identified by the slot-zero vtable and deleting-flag branch. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00d2f160` | `AtoiToOutParam` |  | CRT-style atoi used to parse integer text. | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## 🟢 likely (9)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00401a50` | `StrToInt64` |  | 64-bit integer parse used by the config reader chain (caller 00401d80 parses a file value). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00403a6f` | `MemCpyS` |  | Thin secure-copy wrapper used by callers in the 0x41xxxx region. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00825d80` | `WcsNCmpNoCase` |  | Case-insensitive wide string compare of n characters (MSVC wcsnicmp shape). | [[Native Map/Shards/S0197 00821940-00827640|S0197]] |
| `008979e0` | `StrRChr` |  | String helper used by the log-file name expander FUN_00898400 to split directory from file name; no callees. | [[Native Map/Shards/S0218 00896b90-0089d930|S0218]] |
| `0099e990` | `IStringFixed8192Ctor` | SoeUtil::IStringFixed<unsigned_short,81… | Constructs the fixed-capacity UTF-16 string object. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f070` | `IStringFixed8StackDtor` | SoeUtil::IStringFixed<unsigned_short,8> | Compiler-generated stack cleanup, reached from caller 009A0A60 and unwind funclets. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f100` | `IStringFixed16StackDtor` | SoeUtil::IStringFixed<unsigned_short,16> | Compiler-generated stack cleanup, reached from caller 009A56D0 and unwind funclets. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00c8f420` | `WideCaseInsensitiveCompare` |  | Case-insensitive wide string comparison helper. | [[Native Map/Shards/S0405 00c89e70-00c8f7c0|S0405]] |
| `0102a960` | `CrtAtoiSkipSpaces` |  | Integer field parser used by the record reader 0102abf0. | [[Native Map/Shards/S0565 01025bf0-0102b850|S0565]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007065d0` | `ParseIntegerToken ?` |  | Copies a token into a temporary buffer, parses prefixed hexadecimal with strtol, and otherwise parses decimal with atol. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `0099ea20` | `FreeBufferIfNotInline ?` |  | Fixed-string storage release callback. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f6e0` | `AppendWideString ?` |  | Generic wide-string append helper called from debug string formatting wrappers. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099f9e0` | `AppendWideFormattedString ?` |  | Generic wide-string formatting routine. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `009a0080` | `AppendWideFormat ?` |  | Variadic wide-string formatting wrapper. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099e090` | `— ⚠reject` | SoeUtil::IStringFixed<unsigned_short,8> | Destroys a fixed-capacity UTF-16 string. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099ea40` | `— ⚠reject` | SoeUtil::IStringFixed<unsigned_short,81… | Destroys fixed-capacity UTF-16 storage. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |

