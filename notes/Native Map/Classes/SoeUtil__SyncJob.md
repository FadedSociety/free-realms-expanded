# Class: SoeUtil::SyncJob

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083f770` | `SyncJobCtor` | 🟢 | ctor | __thiscall constructor: SEH frame, calls 0083f1e0 (SoeUtil::Job ctor) with the stack argument first, then stores the SoeUtil::SyncJob vtable into this, calls 00fc4ed0(0), and returns this. |
| `0083f7e0` | `SyncJobScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor: calls the Job destructor body 0083f180, then calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call: add esp,4; mov eax,esi; pop esi; ret 4, so it returns this. |
