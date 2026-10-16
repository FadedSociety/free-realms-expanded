# Class: SocialSharePacketPublishRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adf280` | `SocialSharePacketPublishRequest::SocialSharePacketPublishRequest ? ⚠downgrade` | 🟡 | ctor | Packet constructor: calls the family base constructor 00ade960 with sub-opcode 3, installs SocialSharePacketPublishRequest::vftable at +0, and initialises the embedded SoeUtil::IString<char> members at +0x14 and +0x24 t… |
| `00adf300` | `SocialSharePacketPublishRequest::~SocialSharePacketPublishRequest` | ✅ | dtor | Destructor body: releases the embedded IString at +0x24 (shared buffer at +0x28 when length +0x30 is positive), then the IString at +0x14 (buffer +0x18, length +0x20), and finally rewrites the object vtable to BaseSocia… |
| `00adf4b0` | `SocialSharePacketPublishRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the destructor body 00adf300 on this, then, when bit 0 of the flag argument is set, calls _free(this), and finally returns this in EAX. |
