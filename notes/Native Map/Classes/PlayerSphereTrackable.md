# Class: PlayerSphereTrackable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfe0f0` | `PlayerSphereTrackable::Ctor` | ✅ | ctor | Stores PlayerSphereTrackable::vftable at the object start, then calls 01085190 with the stack-cookie value. |
| `00bfe340` | `PlayerSphereTrackable::CtorWithPayload` | ✅ | ctor | Calls 01085690 with arguments (param_2,1,0), stores param_3 in field +0x40, installs PlayerSphereTrackable::vftable, and calls 01084e80. |
| `00bfe3b0` | `PlayerSphereTrackable::ScalarDeletingDtor` | ✅ | dtor | Restores PlayerSphereTrackable::vftable, calls 01085190, and frees this only when flag bit 0 is set; otherwise returns this. |
| `00bfe410` | `— ⚠reject` | ⚪ | virtual | If the pointer at this+0x40 is non-null, copies eight dwords from that object into its +0x1820 block and returns that block; otherwise returns this+0x50. |
