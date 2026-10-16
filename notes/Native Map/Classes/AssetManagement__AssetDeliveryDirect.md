# Class: AssetManagement::AssetDeliveryDirect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0109dbc0` | `AssetDeliveryDirect::ReturnThis ?` | 🟡 | virtual | Virtual slot [13] of AssetDeliveryDirect (bytes 8b c1 c3): returns its this pointer unchanged. |
| `0109dbd0` | `AssetDeliveryDirect::SubobjectAt640 ?` | 🟡 | virtual | Virtual slot [17] of AssetDeliveryDirect (bytes 8d 81 40 06 00 00 c3): returns this+0x640 as the address of the subobject. |
| `0109dbe0` | `AssetDeliveryDirect::TestOrForwardToQueue ?` | 🟡 | virtual | Virtual slot [20] of AssetDeliveryDirect, one stack argument: returns 1 when 01093820 accepts the argument; otherwise returns the result of virtual slot [2] of the object at this+0x638 (the FileRequestQueue) called with… |
| `0109e360` | `AssetDeliveryDirect::ReleaseLoadersAndQueue ?` | 🟡 | method | If the queue pointer at this+0x638 is set: while the list head at this+0x62c is non-null, unlinks a node through 0109ddb0, calls 00fc4a80 on it and releases one reference (+4) of the object, invoking its vtable slot [1]… |
| `0109e3d0` | `AssetDeliveryDirect::InitFileLoaders` | 🟢 | virtual | Virtual slot [1] of AssetDeliveryDirect. Calls 0109e360 to release any previous queue and loaders, allocates 0xb0 bytes and constructs a FileRequestQueue (0109e2e0) stored at +0x638, calls virtual slot [2] of this, then… |
| `0109e4d0` | `— ⚠reject` | ⚪ | virtual | Virtual slot [2] of AssetDeliveryDirect. Takes the lock 00fc45f0 when the counter at +0x628 is positive. When the pair at +0x9978/+0x997c is set and 008bb840 accepts the id at +0x61c, it calls 01538790 instead of forwar… |
| `0109e570` | `AssetDeliveryDirect::ClearItemRefs ?` | 🟡 | virtual | Virtual slot [19] of AssetDeliveryDirect, one stack argument: calls 010937d0 on this with the argument, then calls 0109e140 on the FileRequestQueue at this+0x638 with the same argument. |
