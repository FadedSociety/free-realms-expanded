# Class: RepeatingActivityClearRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ad9fd0` | `RepeatingActivityClearRequest::RepeatingActivityClearRequest` | ✅ | ctor | Runs base constructor 00ad9620 with activity type 4, installs RepeatingActivityClearRequest::vftable and stores three arguments at +0x10, +0x14 and +0x18 (the id/context values mirrored by the serialiser 00adc160). |
| `00ada040` | `RepeatingActivityClearRequest::ScalarDeletingDtor` | ✅ | dtor | Virtual deleting destructor for RepeatingActivityClearRequest: resets the vtable to BasePacket::vftable inside an SEH frame and frees the object when the flag argument has bit 0 set. |
| `00adc160` | `RepeatingActivityClearRequest_Serialize` | 🟢 | method | Writes the repeating-activity header, copies min(remaining output capacity, 4) bytes from the dword at object +0x18, advances the cursor by the copied length, then serializes the member at +0x10 through 008fb810. |
