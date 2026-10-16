# Class: ClientItem

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b1d80` | `— ⚠reject` | ⚪ | ctor | Constructor: calls FUN_0103dc60 with this in ECX, then writes ClientItem::vftable at offset 0, clears bytes at +0xc and +0x31, and zeroes dwords at +0x3c, +0x40, +0x44, +0x48, +0x4c, +0x50, +0x54, and +0x58. Returns thi… |
| `008b1e20` | `ClientItem::~ClientItem` | ✅ | dtor | Destructor body: stores ClientItem::vftable into *this and then runs the base-object cleanup at 0103dd90 (which resets ItemInstance::vftable, frees the pointer at base+0x28 and stores ItemRecord::vftable). It does not f… |
| `008b1e70` | `ClientItem::ScalarDeletingDestructor` | ✅ | dtor | Deleting destructor: stores ClientItem::vftable, runs the base cleanup at 0103dd90, then calls operator delete (_free) on this when bit 0 of the flag argument is set, and returns this. |
