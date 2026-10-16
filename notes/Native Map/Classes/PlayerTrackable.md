# Class: PlayerTrackable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfe090` | `— ⚠reject` | ⚪ | ctor | Stores PlayerTrackable::vftable at the object start, then calls 01085190 with the stack-cookie value. |
| `00bfe210` | `PlayerTrackable::CtorWithPayload` | ✅ | ctor | Calls 01085690 with arguments (param_2,1,0), stores param_3 in field +0x40, installs PlayerTrackable::vftable, and calls 01084e80. |
| `00bfe280` | `PlayerTrackable::ScalarDeletingDtor` | ✅ | dtor | Restores PlayerTrackable::vftable, calls 01085190, and frees this only when flag bit 0 is set; otherwise returns this. |
| `00bfe2e0` | `— ⚠reject` | ⚪ | virtual | If the pointer at this+0x40 is non-null, copies eight dwords from that object into its +0x1820 block and returns that block; otherwise returns this+0x50. |
