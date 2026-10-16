# Class: SoeUtil::Array<DeepAssetHelper::TextureAsset*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071b750` | `TextureArrayChooseCapacity` | ✅ | virtual | Chooses capacity with 3/4 growth or 6/5 shrink thresholds, returns reusable storage or allocates four-byte pointers. |
| `0071b7f0` | `FreeTextureArrayStorage` | ✅ | virtual | Calls FreeMemory with alignment four on the supplied pointer. |
| `0071b810` | `TextureArrayVirtualOperation3` | ✅ | virtual | If the flag is nonzero, calls the second operand vtable entry at byte offset 0x0c. Otherwise swaps backing pointers at offset 4 and swaps the dwords at offsets 8 and 0x0c using xor operations. |
| `0071ba80` | `TexturePointerArrayDestructor` | ✅ | dtor | Installs texture-array vtable, clears used count, frees backing storage, and zeros pointer. |
| `0071bb40` | `TextureArrayScalarDeletingDtor` | ✅ | dtor | Clears texture-array count, frees storage, and frees object when deleting flag bit is set. |
