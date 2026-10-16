# Class: RewardNonBundledItem

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b88d10` | `RewardNonBundledItem::RewardNonBundledItem` | ✅ | ctor | Constructor of the subtype-2 non-bundled item packet: runs the base constructor with subtype 2 (00b88b40), installs RewardNonBundledItem::vftable, stores the arguments at +0x1c/+0x20, initializes an embedded object thro… |
| `00b88db0` | `RewardNonBundledItem::~RewardNonBundledItem` | 🟢 | dtor | Destructor body of the non-bundled item packet: reinstalls Loggable::vftable at this+0x24 (the secondary base subobject), then destroys the reward packet base through 00b88bc0. |
| `00b89070` | `RewardNonBundledItem::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of RewardNonBundledItem: runs the class destructor body 00b88db0 and frees the packet through _free when bit 0 of param_2 is set. |
| `00b891f0` | `RewardNonBundledItem_ReadBody` | 🟢 | method | Body reader for the subtype-2 item packet: calls the base header reader 00b89190, then reads four 32-bit words into the fields at +0x1c, +0x20 (plus an embedded read through FUN_008caaa0) and +0x30/+0x34, each with the … |
| `00b89590` | `RewardNonBundledItem_Read` | 🟢 | method | Entry point that reads a subtype-2 item packet body from a raw buffer: sets up the bounded reader (cursor = param_2, end = param_2+param_3, error byte local), calls the body reader 00b891f0, and returns 1 only when the … |
