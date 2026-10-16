# Class: SoeUtil::List<FriendData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7e320` | `— ⚠reject` | ⚪ | function | Initializes the pool, requests a pooled block and invokes its virtual constructor when available; otherwise allocates 0xe0 bytes aligned to 8. |
| `00c7e360` | `FriendData::Free` | ✅ | function | Passes the allocation pointer and alignment 8 to FUN_00401c00, then returns. |
| `00c80ee0` | `FriendDataListDtor` | ✅ | dtor | Restores the FriendData list vtable and repeatedly removes the head node through FUN_00c805b0. |
| `00c80f90` | `FriendDataListScalarDeletingDtor` | ✅ | dtor | Removes FriendData nodes until empty, then frees the list object only when flag bit 0 is set. |
