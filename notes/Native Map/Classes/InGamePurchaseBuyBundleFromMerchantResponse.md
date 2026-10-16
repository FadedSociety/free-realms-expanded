# Class: InGamePurchaseBuyBundleFromMerchantResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2a3b0` | `— ⚠reject` | ⚪ | ctor | Initializes the purchase base with subtype 0x2c, writes the response vtable, zeros its data fields, and initializes its string member. |
| `00b2a440` | `BuyBundleFromMerchantResponse::Dtor` | ✅ | dtor | Restores response and string vtables, releases the string buffer if unshared, then restores BasePacket. |
| `00b2c2c0` | `BuyBundleFromMerchantResponse::ScalarDeletingDtor` | ✅ | virtual | Calls the response destructor and conditionally frees the packet object. |
