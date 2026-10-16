# Class: VehicleRefreshConfigsPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4d10` | `VehicleRefreshConfigsPacket::VehicleRefreshConfigsPacket` | ✅ | ctor | Constructor: calls FUN_008a4c40(3), installs VehicleRefreshConfigsPacket::vftable, then calls FUN_0101dcd0 with ECX pointing to the subobject at this+0xc inside an SEH frame. DAT_01b839d8 frame-cookie bookkeeping is not… |
| `008a4d70` | `VehicleRefreshConfigsPacket::~VehicleRefreshConfigsPacket` | 🟢 | dtor | Destructor body calls FUN_0101d850 with ECX pointing to the subobject at this+0xc, then restores BasePacket::vftable. Callers include 008b7f60, vehicle dispatcher 00944840 and EH cleanup 01593336. |
| `008b7f60` | `VehicleRefreshConfigsPacket::ScalarDeletingDtor` | ✅ | dtor | Calls VehicleRefreshConfigsPacket destructor, then frees the object when the deleting flag bit is set. |
