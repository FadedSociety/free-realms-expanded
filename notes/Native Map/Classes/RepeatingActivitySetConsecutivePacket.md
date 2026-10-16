# Class: RepeatingActivitySetConsecutivePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad99e0` | `RepeatingActivitySetConsecutivePacket::RepeatingActivitySetConsecutivePacket` | ✅ | ctor | Runs base constructor 00ad9620 with activity type 5, installs RepeatingActivitySetConsecutivePacket::vftable, then stores the two arguments at +0xc and +0x10. |
| `00ada0a0` | `RepeatingActivitySetConsecutivePacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivitySetConsecutivePacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adc1d0` | `RepeatingActivitySetConsecutivePacket_Serialize` | 🟢 | method | Writes the repeating-activity header with 00adbe60, then appends the two 4-byte fields at +0xc and +0x10. |
