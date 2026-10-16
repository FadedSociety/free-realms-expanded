# Class: FactoryPacketUpdateBlueprint

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d93b0` | `FactoryPacketUpdateBlueprint::~FactoryPacketUpdateBlueprint` | 🟢 | dtor | Destructor body: destroys the member at this+0x10 via FUN_01058e00, a folded member-destructor body that tears down two 10-bucket maps (member+0x60 and member+0x9c) and a SoeUtil::List<EffectTag,-1> member (member+0x50)… |
| `009db2d0` | `FactoryPacketUpdateBlueprint::FactoryPacketUpdateBlueprint` | ✅ | ctor | Constructor for FactoryPacketUpdateBlueprint: calls FUN_009d90d0(0x11), installs FactoryPacketUpdateBlueprint::vftable and runs the member initialiser FUN_01058d00. |
| `009db330` | `FactoryPacketUpdateBlueprint::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009d93b0, then frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdateBlueprint::vftable (dword at 0x01810560 is this address). |
