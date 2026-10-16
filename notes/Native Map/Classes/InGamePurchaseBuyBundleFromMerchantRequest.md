# Class: InGamePurchaseBuyBundleFromMerchantRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2a290` | `— ⚠reject` | ⚪ | ctor | Initializes the purchase base with subtype 0x2b, writes the packet vtable, zeros request fields, and initializes a string member. |
| `00b2a320` | `BuyBundleFromMerchantRequest::Dtor` | ✅ | dtor | Restores the request and string vtables, releases the string buffer when its shared reference count reaches zero, then restores BasePacket. |
| `00b2c2a0` | `BuyBundleFromMerchantRequest::ScalarDeletingDtor` | ✅ | virtual | Calls the request destructor and conditionally frees the packet object. |
