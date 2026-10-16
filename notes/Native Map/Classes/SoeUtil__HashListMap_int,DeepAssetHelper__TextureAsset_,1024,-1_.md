# Class: SoeUtil::HashListMap<int,DeepAssetHelper::TextureAsset*,1024,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8af90` | `HashListTextureAssetDtorBody` | ✅ | dtor | Sets the HashListMap<int,DeepAssetHelper::TextureAsset*,1024,-1> vptr at *param_1, then repeatedly calls 00a88a10 (HashListUnlinkNode) with the current head at param_1[2] while it is non-null. |
| `00a8b140` | `HashListTextureAssetScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of HashListMap<int,DeepAssetHelper::TextureAsset*,1024,-1>: stores the vptr at *param_1, then while the head at param_1[2] is non-null calls 00a88a10 (HashListUnlinkNode) with it, and calls _f… |
| `00a8b930` | `HashListTextureAssetDtorFrame` | ✅ | dtor | Destructor body with an MSVC EH frame: reads the head at param_1[2], stores the HashListMap<int,DeepAssetHelper::TextureAsset*,1024,-1> vptr at *param_1, then repeatedly calls 00a88a10 (HashListUnlinkNode) with the curr… |
