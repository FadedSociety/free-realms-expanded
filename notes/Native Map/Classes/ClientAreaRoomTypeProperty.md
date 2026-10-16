# Class: ClientAreaRoomTypeProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be9130` | `ClientAreaRoomTypeProperty::Ctor` | ✅ | ctor | Stores ClientAreaProperty vtable, zeroes fields at +8, +0xc, +0x10, +0x14, +0x20 and the bytes at +0x28; stores its two arguments at +0x18 and +0x1c, the constant _DAT_0175b420 at +0x24, and finally installs ClientAreaR… |
| `00be91e0` | `ClientAreaRoomTypeProperty::DtorBody` | ✅ | dtor | Stores the ClientAreaRoomTypeProperty vtable at *this, calls 00ceaea0(0) (room-type removal hook per its accepted label) with an unwind frame, then stores the ClientAreaProperty vtable at *this and returns. |
| `00be9240` | `— ⚠reject` | ⚪ | virtual | Calls vtable slot +0x28 (slot 10, the type-name getter) on this and computes a one-at-a-time hash of the returned string (h = (c + h) * 0x401; h ^= h >> 6 per character, up to 0x31 characters, then h = ((h*9) >> 11 ^ h*… |
| `00be97f0` | `ClientAreaRoomTypeProperty::GetDword38` | ✅ | virtual | Returns the dword at this+0x38 (mov eax,[ecx+0x38]; ret). |
| `00be9800` | `ClientAreaRoomTypeProperty::SetDword38` | ✅ | virtual | Stores the dword argument at this+0x38; returns void (ret 4 shape per the pseudocode). |
| `00be9810` | `ClientAreaRoomTypeProperty::GetTypeName` | ✅ | virtual | Returns a pointer to the literal string 'RoomType' (mov eax, offset; ret). |
| `00be9820` | `ClientAreaRoomTypeProperty::ScalarDtor` | 🟢 | dtor | Calls 00be91e0 (room-type destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00ceae00` | `ClientAreaRoomTypeProperty::WriteRoomTypeId ?` | 🟡 | virtual | Writes the property name RoomTypeId and the field at object offset +0x38 through the supplied serializer helper. |
| `00ceae20` | `ClientAreaRoomTypeProperty::Log ?` | 🟡 | virtual | Obtains a formatted base description, writes the inherited field at index 7, and logs RoomTypeId from index 14. |
| `00ceae70` | `ClientAreaRoomTypeProperty::OnApply ?` | 🟡 | virtual | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the apply helper. |
| `00ceaea0` | `ClientAreaRoomTypeProperty::OnRemove ?` | 🟡 | virtual | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the remove helper. |
