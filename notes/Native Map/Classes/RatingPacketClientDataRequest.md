# Class: RatingPacketClientDataRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00abd470` | `RatingPacketClientDataRequest::RatingPacketClientDataRequest` | ✅ | ctor | Calls FUN_00aa5ca0 with inner opcode 3, installs the request and IString vtables, initializes the string at +0xc, zeroes +0x10,+0x14,+0x18, then calls FUN_00abc200 for the embedded hash-list map. |
| `00abd4f0` | `RatingPacketClientDataRequest::ScalarDeletingDtor` | ✅ | dtor | Calls FUN_00abab40, frees this only when flag bit 0 is set, returns this in EAX and returns with ret 4. |
