# Class: InventoryPacketPreviewStyleCard

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bd58c0` | `InventoryPacketPreviewStyleCard::InventoryPacketPreviewStyleCard` | ✅ | ctor | Thiscall ctor (ECX = packet): calls the BaseInventoryPacket ctor 00a10400 with stack arg 0xb (sub-type 11), then stores InventoryPacketPreviewStyleCard::vftable at +0 and returns this. |
| `00bd5c10` | `InventoryPacketPreviewStyleCard::ScalarDeletingDtor` | ✅ | dtor | Thiscall scalar deleting destructor (virtual slot [0] of the preview-style-card packet): stores BaseInventoryPacket::vftable then BasePacket::vftable at +0, then calls _free(this) (thunk 00d82463) when bit 0 of the flag… |
