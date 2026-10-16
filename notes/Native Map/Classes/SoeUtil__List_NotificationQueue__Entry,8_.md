# Class: SoeUtil::List<NotificationQueue::Entry,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a966a0` | `FiniteListDestructorBody` | ✅ | dtor | Installs the bounded-list vtable, drains entries, recursively frees both tree branches, clears tree pointers, restores the base-list vtable, then drains again. |
| `00a97130` | `AdjustorThunk_To_00a97050 → SoeUtil::List<NotificationQueue::Entry,8>::Adjustor…` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00a97050; the target body pops one free entry. |
| `00a97140` | `AdjustorThunk_To_00a970c0 → SoeUtil::List<NotificationQueue::Entry,8>::Adjustor…` | ✅ | virtual | Adds 0x10 to ECX and jumps to 00a970c0; the target body pushes one entry onto the free list. |
| `00a97150` | `ListScalarDeletingDestructor → SoeUtil::List<NotificationQueue::Entry,8>::Scala…` | ✅ | dtor | Calls 00a966a0, tests delete flag bit 0, and frees this when that bit is set. |
| `00a97400` | `ListDestructorWrapper → SoeUtil::List<NotificationQueue::Entry,8>::DestructorCl…` | 🟢 | dtor | Forwards incoming ECX to 00a966a0 for list cleanup; raw bytes show ECX preserved across the call. |
| `00cb7190` | `NotificationQueueListCtor` | ✅ | ctor | Installs SoeUtil::List<NotificationQueue::Entry,-1>::vftable then SoeUtil::List<NotificationQueue::Entry,8>::vftable at receiver+0, zeroes receiver+4, +8, +0xc, then calls 00cb6d20 with ECX = receiver+0x10 to initialise… |
