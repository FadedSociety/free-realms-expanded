# Class: BaseBugSubmissionPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f8800` | `BaseBugSubmissionPacket::Ctor ⚠downgrade` | 🟢 | ctor | Installs BasePacket then BaseBugSubmissionPacket vtables, writes packet identifier 0x53 at +4, and stores the argument at +8. |
| `009f8870` | `BaseBugSubmissionPacket::Dtor` | ✅ | dtor | Resets the object's vtable first to BaseBugSubmissionPacket and then to BasePacket. |
| `009f88c0` | `BaseBugSubmissionPacket::ScalarDeletingDtor` | ✅ | dtor | Sets the derived and base packet vtables, conditionally frees the object when flag bit 0 is set, and returns its pointer otherwise. |
