# Class: PacketClientFinishedLoading

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089b000` | `PacketClientFinishedLoadingCtor` | ✅ | ctor | Constructor of PacketClientFinishedLoading: installs BasePacket::vftable, stores opcode 10 at +4, then installs PacketClientFinishedLoading::vftable. |
| `0089b060` | `PacketClientFinishedLoadingDtor` | ✅ | dtor | Destructor body: installs PacketClientFinishedLoading::vftable then the BasePacket::vftable (inlined base destructor); no member cleanup. |
| `0089b0b0` | `PacketClientFinishedLoadingScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
