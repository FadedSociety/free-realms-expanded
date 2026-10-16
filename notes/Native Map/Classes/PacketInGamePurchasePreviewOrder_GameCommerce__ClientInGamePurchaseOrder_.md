# Class: PacketInGamePurchasePreviewOrder<GameCommerce::ClientInGamePurchaseOrder>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2c610` | `PreviewOrder::Constructor` | ✅ | ctor | Initializes the purchase packet base with subtype 1, stores the supplied field at offset 0x18, and initializes the order member. |
| `00b2c680` | `PreviewOrder::ScalarDeletingDtor` | ✅ | virtual | Calls the preview-order destructor and conditionally frees the packet object. |
