# Class: PetCanGoLocationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b51b20` | `— ⚠reject` | ⚪ | ctor | Constructor: calls the pet base constructor 00b50670 with subtype 0x1a (26), stores PetCanGoLocationPacket::vftable and zeroes the byte at +0x20 (the flag byte the four-float + flag body ends with). |
| `00b51b80` | `— ⚠reject` | ⚪ | dtor | Destructor: resets the vtable pointer at *this to BasePacket::vftable and returns; the follow-location senders 00b5ce60/00b5fcf0 and two unwind funclets call it. |
| `00b53210` | `PetCanGoLocationPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: resets the vtable pointer at *this to BasePacket::vftable and calls _free(param_1) when bit 0 of the flag argument is set; returns the object pointer. The exe RTTI records this address as slo… |
| `00b5a530` | `PetCanGoLocationPacket::SerializeBody` | ✅ | method | Writes the common packet prefix, copies the 16-byte block at param_1+0x10, then writes the byte at param_1+0x20. |
