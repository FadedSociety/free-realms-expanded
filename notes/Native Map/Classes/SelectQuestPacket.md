# Class: SelectQuestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c79e40` | `SelectQuestPacket::Construct` | ✅ | ctor | Constructs a SelectQuestPacket: calls FUN_00a86640(0xc) (BaseUiPacket base constructor with subtype 12), stores the SelectQuestPacket vtable and the quest id argument at +0xc. |
| `00c7a550` | `SelectQuestPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vtable to BasePacket::vftable and calls _free(this) when bit 0 of the flag argument is set; returns this. |
