# Class: ClientUpdatePacketNotifyPlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008cf5e0` | `ClientUpdatePacketNotifyPlayer::ClientUpdatePacketNotifyPlayer` | ✅ | ctor | Packet constructor: runs the client-update family constructor FUN_0089d180 with subtype 0x17 (23), installs ClientUpdatePacketNotifyPlayer::vftable, then initializes one embedded SoeUtil::IString member whose vtable sit… |
| `008cf650` | `ClientUpdatePacketNotifyPlayer::~ClientUpdatePacketNotifyPlayer` | 🟢 | dtor | Destructor body: releases the embedded SoeUtil::IString at +0xc (buffer +0x10, capacity +0x18) through the locked refcount and virtual slot +8, then resets the vtable to BaseClientUpdatePacket::vftable and again to Base… |
| `008dc920` | `ClientUpdatePacketNotifyPlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008cf650 (labelled as ClientUpdatePacketNotifyPlayer destructor) and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
