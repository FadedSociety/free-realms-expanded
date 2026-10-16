# Class: ClientHousingPacketRequestPlayerHouses

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad310` | `ClientHousingPacketRequestPlayerHouses::~ClientHousingPacketRequestPlayerHouses` | 🟢 | dtor | Destructor of the request-player-houses packet: releases the SoeUtil::IString<char> member at +0x18 (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseHousingPacket:… |
| `00ab3f00` | `ClientHousingPacketRequestPlayerHouses::Ctor` | ✅ | ctor | Constructs subtype 14, installs the request packet vtable, initializes two words and assigns its default string member. |
| `00ab3f90` | `ClientHousingPacketRequestPlayerHouses::ScalarDeletingDtor` | ✅ | dtor | Calls the request packet destructor and conditionally frees the object when the deleting flag's low bit is set. |
| `00ac00c0` | `SendClientHousingPacketRequestPlayerHouses` | 🟢 | function | Transport wrapper for the 127/14 player-houses request: opens the scratch scope with FUN_008d4a40(0x10000000), emits the housing family header with FUN_00aba910 over the caller's packet, appends the 64-bit field at +0x1… |
