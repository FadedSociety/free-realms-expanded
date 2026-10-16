# Class: SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bd5cc0` | `HashListMap10_Slot1Helper ?` | 🟡 | virtual | Virtual slot [1] of the PlayerCustomizationData 10-bucket map (read from the binary): calls 006fb9c0, then calls vtable slot [1] of the object returned by 006fb860 when it is non-null, otherwise of the global DAT_01bdf2… |
| `00bd5d00` | `HashListMap10_FreeNode` | 🟢 | virtual | Virtual slot [2] of the PlayerCustomizationData 10-bucket map (read from the binary): frees its argument through 00401c00 (FreeMemory) with alignment 4; returns nothing. |
| `00bd9850` | `HashListMap_PlayerCustomizationData::~HashListMap_PlayerCustomizationData` | ✅ | dtor | Stores the SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1> vftable at object+0, then loops while the head pointer (+8) is non-null, calling 00bd8d70 on the head node to unlink and destroy it. |
| `00bd9880` | `HashListMap_PlayerCustomizationData::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (vtable slot 0 of SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>::vftable): stores the map vtable at object+0, then loops while the head pointer at +8 is non-null, calling 00b… |
| `00bd9920` | `HashListMap_PlayerCustomizationData::~HashListMap_PlayerCustomizationData` | ✅ | dtor | Destructor body: stores the map vtable at object+0 and drains the node list by repeatedly calling 00bd8d70 on the head pointer at +8 until it is null; it never frees the container itself (the deleting form is 00bd9880).… |
| `00bd9d70` | `HashListMap_PlayerCustomizationData::Ctor` | ✅ | ctor | Constructor body: stores SoeUtil::HashListMap<int,PlayerCustomizationData,10,-1>::vftable at object+0, zeroes the pointer/size/count fields at +4..+0x10, zeroes the ten bucket heads at +0x14..+0x3b, and finishes by stor… |
