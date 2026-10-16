# Class: HousingPacketFixtureItemList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aae640` | `HousingPacketFixtureItemList::ReleaseGuidArray` | 🟢 | method | Releases the embedded SoeUtil::Array<StrongType<unsigned __int64,SoeUtilSignatureCharacterGuid,0>,0,1> member at object+0x14: resets the member vtable, clears the capacity at +0x1c, frees the buffer at +0x18 with 8-byte… |
| `00ab3ce0` | `HousingPacketFixtureItemList::Ctor` | ✅ | ctor | Constructs subtype 43, stores two arguments, installs the packet vtable and initializes an embedded list. |
| `00ab3d60` | `HousingPacketFixtureItemList::Dtor` | 🟢 | dtor | Destroys the embedded list, then restores the BaseHousingPacket and BasePacket vtables during cleanup. |
| `00ab7360` | `HousingPacketFixtureItemList::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, then conditionally frees the object when the deleting flag's low bit is set. |
