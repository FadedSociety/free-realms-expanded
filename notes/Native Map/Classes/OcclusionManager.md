# Class: OcclusionManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b6010` | `— ⚠reject` | ⚪ | ctor | Installs AssetManagement::AssetDeliveryHandlerInterface::vftable, stores param_2 into +4, overwrites *this with OcclusionManager::vftable, zeroes +0xc, +0x10, +0x14, +8 and +0x58..+0x64, clears 0x40 bytes from +0x18 via… |
| `008b60b0` | `OcclusionManager::ScalarDeletingDestructor` | 🟢 | dtor | Calls the destructor at 010b6ac0 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
