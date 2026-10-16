# Class: InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b1fe70` | `InGamePurchaseUpdateMerchantSellVftableRestore` | ✅ | dtor | Stores InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase::vftable at [param_1] and then BasePacket::vftable; those two stores are the whole body, and the only caller is an EH unwind funclet. |
| `00b23a10` | `InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase::Ctor` | 🟢 | ctor | Constructor: calls FUN_00b22e10 with sub-opcode 0x2d (45), writes its own vftable and zeroes this+0x18 and this+0x1c. |
| `00b23a70` | `InGamePurchaseUpdateMerchantSellBundleQuantityAvailableForPurchase::ScalarDelet…` | ✅ | dtor | Scalar deleting destructor: writes its own vftable, seeds try level -1, writes BasePacket::vftable, and frees this via _free when the flags low bit is set. |
