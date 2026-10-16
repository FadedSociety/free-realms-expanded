# Class: FlashManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d2040` | `FlashManager::FlashManager` | ✅ | ctor | Constructor of the Flash movie manager: installs FlashManagerInterface::vftable (interface base), then overrides it with FlashManager::vftable, installs the SoeUtil::Map<unsigned int,FlashMovie*> vtable at +4 and zeroes… |
| `008d20a0` | `FlashManager::ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor (slot [0] of the Flash movie manager vtable): runs the out-of-line destructor body FUN_00d56540 and frees the object through the CRT _free thunk when bit 0 of the flag argument is set. |
| `00d55790` | `FlashManager::GetInt64At10 ?` | 🟡 | virtual | Returns the 64-bit value stored at this+0x10 (eax low dword, edx high dword) as an undefined8 return. |
| `00d557a0` | `FlashManager::GetInt64At18 ?` | 🟡 | virtual | Returns the 64-bit value stored at this+0x18 (eax low dword, edx high dword). |
| `00d557b0` | `FlashManager::ZeroPairs10To1C ?` | 🟡 | virtual | Writes zero to the four dwords at this+0x10, +0x14, +0x18 and +0x1c, clearing two 64-bit fields. |
| `00d55cb0` | `FlashManager::LookupByKey ?` | 🟡 | virtual | Searches a binary search tree rooted at this+0x8 for the unsigned key argument (node key at +0x4; the +0xc child holds keys below the node key (taken when the node key is greater than the target) and the +0x10 child hol… |
| `00d55cf0` | `FlashManager::GetField0C ?` | 🟡 | virtual | Returns the 32-bit value at this+0xc (mov eax,[ecx+0xc]; ret). |
| `00d56540` | `FlashManager::Dtor` | ✅ | dtor | Sets FlashManager::vftable and the map vtable, walks the tree from its leftmost node deleting each value through vtable slot 0 with flag 1, then destroys tree nodes with TreeDestroyPostOrder (008ac620) on both children … |
| `00d56830` | `FlashManager::RemoveByKey` | 🟢 | virtual | Finds the tree node with key param_2 under the root at this+8; when found, deletes the value pointer's object through its vtable slot 0 with flag 1 if non-null, erases the node with TreeEraseNode (00d56330), and frees t… |
| `00d56b60` | `FlashManager::GetOrCreateMovie` | 🟢 | virtual | Computes a key from its name argument with FUN_00705b00, searches the manager tree at this+8, and on a miss allocates and constructs a FlashMovie (FUN_00d56890), inserts it through TreeInsertKeyValue (00d56670), allocat… |
