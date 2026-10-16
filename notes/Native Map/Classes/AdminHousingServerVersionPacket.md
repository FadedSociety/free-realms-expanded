# Class: AdminHousingServerVersionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad040` | `AdminHousingServerVersionPacket::ctor` | ✅ | ctor | Constructor of the admin housing server-version packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x39 (57, base opcode 127), stamps AdminHousingServerVersionPacket::vftable at +0 and co… |
| `00aad0b0` | `AdminHousingServerVersionPacket::~AdminHousingServerVersionPacket` | 🟢 | dtor | Destructor of the admin housing server-version packet: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement on the buffer, release through vtable slot +8) and then resets the object's vta… |
| `00ab0500` | `AdminHousingServerVersionPacketScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
