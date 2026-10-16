# Class: RewardBundlePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b88c40` | `RewardBundlePacket::RewardBundlePacket` | ✅ | ctor | Constructor of the subtype-1 reward bundle packet: runs the base constructor with subtype 1 (00b88b40), installs RewardBundlePacket::vftable, stores the bundle object at +0x1c and clears the success byte at +0x20. |
| `00b88cb0` | `RewardBundlePacket_SetByte20` | ✅ | virtual | Ten-byte setter at vtable slot 1 of RewardBundlePacket: writes the byte argument to this+0x20 and returns. |
| `00b89010` | `RewardBundlePacket::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of RewardBundlePacket: runs the shared base destructor 00b88bc0 under an SEH frame and frees the packet through _free when bit 0 of param_2 is set. |
| `00b8a300` | `RewardBundlePacket_Read` | 🟢 | method | Entry point that reads a subtype-1 bundle packet: constructs the bounded reader over (param_2, param_2+param_3), calls the base header reader 00b89190, passes the bundle success byte (packet+0x20) to the bundle object's… |
