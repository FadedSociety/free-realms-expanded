# Class: GuiDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00821dc0` | `GuiDataSource::NotifyChanged ? ⚠rename` | 🟡 | virtual | Pushes its argument and calls 0081f480 with ECX set to the global DAT_01be1090 (GUI tracking registry); 13 bytes, no return value. |
| `00822000` | `GuiDataSource::GetDword24 ?` | 🟡 | virtual | Returns the dword at this+0x24 (bytes 8b 41 24 c3: mov eax,[ecx+0x24]; ret). Shared virtual at slot 9 of many GuiDataSource-derived vtables. |
| `00822060` | `GuiDataSource::GetCachedSlot9 ?` | 🟡 | virtual | If the flag dword at this+0x1c is set returns the cached dword at this+0x18. Otherwise calls vtable slot 9 (+0x24, the 00822000 getter) on this, passes the result to 00705bc0, stores it at this+0x18, sets this+0x1c to 1… |
| `00822510` | `GuiDataSource::InOrderSlot6 ?` | 🟡 | virtual | Walks the tree at this+0xc in order (starting at the leftmost node, successor via +0x10 and parent +8) and calls vtable slot 6 (+0x18) of each node's first-dword object with no arguments. Returns nothing. |
| `00822810` | `GuiDataSource::DtorBody` | 🟢 | dtor | Sets GuiDataSource::vftable at this+0; calls 00822010 (drains the tree at this+0xc); when this+0x2c is positive, decrements the count at [this+0x24]-4 with a lock prefix and calls the IString vtable slot 2 at count zero… |
| `00822990` | `GuiDataSource::Ctor` | 🟢 | ctor | thiscall with one argument (a C string). Calls 0071b950 (RefCounted base constructor), stores GuiDataSource::vftable at this+0, SoeUtil::Map<int,GuiDataSourceObserver*,-1,0>::vftable at +8 with root +0xc, count +0x10 ze… |
| `00822a30` | `GuiDataSource::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor shape: calls the base teardown 00822810 on this, then calls _free (thunk 00d82463) on this only when bit 0 of the flag argument is set; returns this (bytes: push esi; mov esi,ecx; call 0082281… |
| `00822a50` | `GuiDataSource::AddIfAbsentNotify ?` | 🟡 | virtual | Stack argument obj (pointer). Looks obj up as the key in the Map embedded at this+8 (00822090); when not found, inserts a node holding obj through 00822910, then calls obj vtable slot 6 (+0x18) with no arguments. Callee… |
| `00822aa0` | `GuiDataSource::RemoveIfPresent ?` | 🟡 | virtual | Stack argument pointer used as the key: looks it up in the Map at this+8 (00822090); when found, erases the node (00822620), frees it through the Map vtable slot 3 (+0xc, 008221c0), and calls the node's object vtable sl… |
