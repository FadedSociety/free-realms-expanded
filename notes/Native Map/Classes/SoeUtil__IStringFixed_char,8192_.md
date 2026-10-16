# Class: SoeUtil::IStringFixed<char,8192>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `004025e0` | `SoeUtil::IStringFixed8192Ctor` | ✅ | ctor | Constructor body for IStringFixed<char,8192>: performs the IString base initialization, then overwrites the vtable with the IStringFixed<char,8192> vtable. |
| `00402640` | `SoeUtil::IStringFixed8192GetBuffer` | 🟢 | virtual | Buffer-request entry for IStringFixed<char,8192>: requests under 0x2005 return the inline buffer at +0x10 with capacity 0x2004 and the non-heap flag; larger requests defer to the heap routine 00401cb0. |
| `00402670` | `SoeUtil::IStringFixed8192ReleaseBuffer` | 🟢 | virtual | Buffer-release entry for IStringFixed<char,8192>: the inline buffer at +0x10 is ignored; any other pointer is freed through the global free entry 00401c00 with alignment 0. |
| `00402690` | `SoeUtil::IStringFixed8192Dtor` | ✅ | dtor | Destructor body for IStringFixed<char,8192>: under an SEH frame rewrites the fixed vtable, releases the owned buffer when capacity is positive, resets to the static empty string, then stores the IString base vtable and … |
| `00402750` | `SoeUtil::IStringFixed8192DeletingDtor` | 🟢 | dtor | Scalar deleting destructor: runs the IStringFixed<char,8192> destructor body FUN_00402690, then frees the object with _free when the flag argument's low bit is set. |
| `00402ec0` | `SoeUtil::IStringFixed8192DtorThunk` | 🟢 | dtor | SEH-framed cleanup wrapper for the IStringFixed<char,8192> destructor body FUN_00402690; sets up and restores a scoped exception frame around the call. |
| `00ae4cd0` | `StringFixed8192Ctor` | ✅ | ctor | Initializes IString fields to fixed empty buffer, installs IStringFixed<char,8192> vtable, and copies supplied source string into the object. |
