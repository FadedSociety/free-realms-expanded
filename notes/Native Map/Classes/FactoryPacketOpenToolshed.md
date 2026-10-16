# Class: FactoryPacketOpenToolshed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009de150` | `FactoryPacketOpenToolshed::FactoryPacketOpenToolshed` | ✅ | ctor | Constructor for FactoryPacketOpenToolshed: calls FUN_009d90d0(0x1a), installs FactoryPacketOpenToolshed::vftable, then initialises an embedded SoeUtil::IString<char> member at +0x0c (IString vtable, shared empty buffer … |
| `009de1c0` | `FactoryPacketOpenToolshed::~FactoryPacketOpenToolshed` | 🟢 | dtor | Destructor body: rewrites the embedded IString member vtable at +0x0c, releases the member's reference-counted buffer at param_1[4]-4 through vtable slot 2 when the length at +0x18 is positive, then rewrites the object … |
| `009df960` | `FactoryPacketOpenToolshed::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009de1c0 (IString member release plus vtable rewrites) and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketOpenToolshed::vftable (dword at… |
