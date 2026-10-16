# Class: SocialSharePacketGetProviderStatusResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adf140` | `SocialSharePacketGetProviderStatusResponse::SocialSharePacketGetProviderStatusR…` | ✅ | ctor | Packet constructor: calls the family base constructor 00ade960 with sub-opcode 2, installs SocialSharePacketGetProviderStatusResponse::vftable at +0, and initialises the two embedded SoeUtil::IString<char> members at +0… |
| `00adf1c0` | `SocialSharePacketGetProviderStatusResponse::~SocialSharePacketGetProviderStatus…` | ✅ | dtor | Destructor body: releases the second embedded IString at +0x1c (shared buffer at +0x20 when length +0x28 is positive), then the first IString at +0xc (buffer +0x10, length +0x18), and finally rewrites the object vtable … |
| `00adf490` | `SocialSharePacketGetProviderStatusResponse::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): calls the destructor body 00adf1c0 on this, then, when bit 0 of the flag argument is set, calls _free(this), and finally returns this in EAX. |
