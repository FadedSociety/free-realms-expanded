# Class: RepeatingActivityBaseAdminPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ada120` | `— ⚠reject` | ⚪ | ctor | Base constructor for the admin subset (sub-ids 6-10): runs 00ad9620 with the caller's activity type, stores arguments at +0xc, +0x10, +0x18 and +0x1c, then installs RepeatingActivityBaseAdminPacket::vftable; the two hig… |
| `00ada1a0` | `RepeatingActivityBaseAdminPacket::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityBaseAdminPacket: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adc270` | `— ⚠reject` | ⚪ | method | Writes the repeating-activity header with 00adbe60, appends the two 4-byte fields at +0xc and +0x10, then serialises the trailing member at +0x18 through FUN_008fb810. |
