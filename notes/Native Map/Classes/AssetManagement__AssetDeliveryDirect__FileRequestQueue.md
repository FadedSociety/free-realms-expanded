# Class: AssetManagement::AssetDeliveryDirect::FileRequestQueue

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0109e1e0` | `FileRequestQueue::ContainsItemRef ?` | 🟡 | virtual | Virtual slot [2] of FileRequestQueue, one stack argument: under the lock 00fc45f0 searches the lists at +8 and +0x14 for a node whose word at +0x28 equals the argument, releases the lock, and returns 1 when found, else … |
| `0109e2e0` | `— ⚠reject` | ⚪ | ctor | Constructor: calls 0071b950, installs the FileRequestQueue vtable, zeroes six words at +8 through +0x1c, calls 00fc4580 with 4000 as the table size argument, and zeroes the word at +0xa8. |
| `0109e590` | `FileRequestQueue::FindItemByNameAndRef` | 🟢 | virtual | Virtual slot [3] of FileRequestQueue, two stack arguments: under the lock 00fc45f0 scans the lists at +8 and +0x14 for a node whose word at +0x28 equals the third argument and whose name at +0x1c equals the second argum… |
| `0109e890` | `— ⚠reject` | ⚪ | dtor | Destructor body of FileRequestQueue: installs the FileRequestQueue vtable, drains both lists through 0109e7c0, then installs the SoeUtil::RefCounted vtable 017e7d08 and runs the base teardown 00fc45a0. |
| `0109e900` | `AssetDeliveryDirect::FileRequestQueue::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the FileRequestQueue body 0109e890, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
