# Class: PetBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b50610` | `PetBasePacket::PetBasePacket ? ⚠rename` | 🟡 | ctor | Default constructor of the pet base packet object: writes BasePacket::vftable at *this, stores the pet base opcode 0x35 at +4, rewrites *this with PetBasePacket::vftable and zeroes the subtype byte at +8. The subtype-ca… |
| `00b50670` | `— ⚠reject` | ⚪ | ctor | Pet base-packet constructor taking the subtype: writes BasePacket::vftable, stores the pet base opcode 0x35 at +4, writes PetBasePacket::vftable and saves the subtype argument at +8. All 35 named pet packet classes call… |
| `00b506e0` | `PetBasePacket::~PetBasePacket` | 🟢 | dtor | Destructor body of the pet base packet: rewrites the pointer at *this with BasePacket::vftable and returns; no members are released. 119 MSVC unwind funclets call it, so it is the shared cleanup target for partially con… |
| `00b52db0` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
