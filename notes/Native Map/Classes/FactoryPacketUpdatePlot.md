# Class: FactoryPacketUpdatePlot

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d98a0` | `FactoryPacketUpdatePlot::FactoryPacketUpdatePlot` | ✅ | ctor | Constructor for FactoryPacketUpdatePlot: calls FUN_009d90d0(0x1c), installs FactoryPacketUpdatePlot::vftable and runs the member initialiser FUN_01059590 (HashListMap keyed by StrongType<u64,FactoryFoundationGuid> in th… |
| `009d9900` | `FactoryPacketUpdatePlot::~FactoryPacketUpdatePlot` | 🟢 | dtor | Calls FUN_010596b0 on the embedded plot-data member, then rewrites this object?s vtable to BaseFactoryPacket::vftable and BasePacket::vftable. FUN_010596b0 destroys the foundation and misfortune maps, placement-node arr… |
| `009db720` | `FactoryPacketUpdatePlot::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: calls the class destructor body 009d9900 (which destroys the placement-node map member) and frees the object when (param_2 and 1) is set. Slot 0 of FactoryPacketUpdatePlot::vftable (dword at … |
