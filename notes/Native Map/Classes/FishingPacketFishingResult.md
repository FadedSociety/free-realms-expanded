# Class: FishingPacketFishingResult

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b64ec0` | `FishingPacketFishingResult::FishingPacketFishingResult` | ✅ | ctor | Calls the fishing family base constructor FUN_00b60910 with sub-type 0xe (14), stores FishingPacketFishingResult::vftable, then default-constructs the packet's three SoeUtil::IString payload members at +0x28, +0x54 and … |
| `00b64f50` | `FishingPacketFishingResult::~FishingPacketFishingResult` | ✅ | dtor | Destroys the packet's three SoeUtil::IString payload members (at +0x64, +0x54 and +0x28), each time restoring the string vtable and releasing the shared buffer when its reference count drops to zero, then stores BaseFis… |
| `00b65d70` | `FishingPacketFishingResult::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls FUN_00b64f50 (which releases the packet's three IString members and restores the base vtables), frees the object with _free when bit 0 of the flag argument is set, and returns the point… |
