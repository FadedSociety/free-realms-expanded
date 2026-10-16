# Class: FlashLoaderJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00840a20` | `— ⚠reject` | ⚪ | ctor | __thiscall constructor: SEH frame, calls 0083f770 (SyncJobCtor) on this with argument 0 first, then stores the FlashLoaderJob vtable, the IString<char> vtable at +0x110 with DAT_01be196f at +0x114, zeroes +0x11c, +0x118… |
| `00840ab0` | `FlashLoaderJobCreateMovie` | 🟢 | virtual | Virtual slot 2 of FlashLoaderJob (26 bytes). Calls GFxWrap::CreateMovie with the movie name at this+0x114 and stores the returned movie at this+0x120. |
| `00840ad0` | `FlashLoaderJobDtorBody` | 🟢 | dtor | __fastcall destructor body (no free): stores the FlashLoaderJob vtable, calls GFxWrap::ReleaseMovie on this+0x120 when it is non-null, releases the IString<char> member at +0x110 (refcount word at +0x114 minus one; vfta… |
| `00840b80` | `FlashLoaderJobScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor (30 bytes): calls 00840ad0 (FlashLoaderJobDtorBody), then calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call: add esp,4; mov eax,esi; pop esi; ret 4, so it return… |
