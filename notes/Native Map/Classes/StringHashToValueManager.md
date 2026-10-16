# Class: StringHashToValueManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00922f70` | `StringHashToValueManager_Ctor` | ✅ | ctor | Constructor: stores the BaseDefinitionManager vtable then the StringHashToValueManager vtable at this, calls 00906740 on the sub-object at this+4 (its vtable reference is HashList<double,2048,1024>), then calls 0091e840… |
| `00922fe0` | `StringHashToValueManager_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls 01049790 on this (the body that stores the StringHashToValueManager and BaseDefinitionManager vtables), then when bit 0 of the second argument is set frees the object through 00401c00 w… |
| `01049520` | `— ⚠reject` | ⚪ | virtual | Thiscall, one stack arg (C-string prefix). Calls the virtual at slot +8 of *this, then loads "<prefix>StringHashToValue.txt" through the file reader (00fbda90, 00fbdaf0, 00fbe130, 00fbe250 with 0x8000-byte lines). For e… |
| `01049750` | `— ⚠reject` | ⚪ | virtual | Fastcall, ECX = manager. Repeatedly calls 008bf3a0 on the head of the list at this+0xc while it is non-null (008bf3a0 is a list-node removal, accepted label unknown), then repeatedly calls 008e4650 on the head of the li… |
| `01049790` | `StringHashToValueManager::DtorBody` | ✅ | dtor | Fastcall, ECX = this. Stores StringHashToValueManager::vftable (01805cb0) at *this, calls 01049750 (ClearTables), then calls 00917e70 and 008f1610 (member cleanups), and finally stores BaseDefinitionManager::vftable (01… |
