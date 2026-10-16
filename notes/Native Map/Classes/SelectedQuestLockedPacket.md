# Class: SelectedQuestLockedPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79f00` | `SelectedQuestLockedPacket::Construct` | ✅ | ctor | Constructs a SelectedQuestLockedPacket: calls FUN_00a86640(0xd) (BaseUiPacket base constructor with subtype 13), stores the vtable and the byte argument at +0xc. |
| `00c7a5b0` | `SelectedQuestLockedPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
