# Class: ActorGroupRequestor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007993d0` | `ActorGroupRequestorDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores ActorGroupRequestor::vftable at +0. If the pointer at +0x1c is non-NULL: when its count at +4 is 1 it calls the vtable slot at +0x3c of the object at this+4 with it; then it decrements the … |
| `007994b0` | `ActorGroupRequestorSetFlag28` | 🟢 | virtual | Virtual slot [4] of ActorGroupRequestor (bytes c6 41 28 01 c3): stores 1 into the byte at this+0x28. |
| `007994c0` | `ActorGroupRequestorGetFlag28` | 🟢 | virtual | Virtual slot [5] of ActorGroupRequestor (bytes 8a 41 28 c3): returns the byte at this+0x28 in AL. |
| `007995d0` | `ActorGroupRequestorScalarDtor` | ✅ | dtor | Virtual slot [0] of ActorGroupRequestor: calls 007993d0 on this, frees the block through _free when bit 0 of its flag argument is set, returns this. |
| `00799700` | `— ⚠reject` | ⚪ | ctor | Thiscall with five stack arguments. Stores AssetManagement::AssetRequestInterface::AssetRequester::vftable and the owner pointer arg1 at +4, then ActorGroupRequestor::vftable; sets the IString<char> member at +8 (vtable… |
| `007997a0` | `ActorGroupRequestorOnAssetLoadFailed` | ✅ | virtual | Virtual slot [2] of ActorGroupRequestor, thiscall with one stack argument (asset name). Formats 'Could not load asset %s for actor group requestor %s' with the argument and the requestor's name at +0xc through 00402ff0,… |
| `0079a880` | `ActorGroupRequestorAcceptAsset` | 🟢 | virtual | Virtual slot [1] of ActorGroupRequestor, thiscall with two stack arguments. When arg3 is 0, dynamic-casts arg2 from AssetManagement::Asset to ActorGroupDefinitionAsset through 00d8d382, stores the result at this+0x1c wi… |
| `0079c210` | `ActorGroupRequestorCreateAsset` | 🟢 | virtual | Virtual slot [3] of ActorGroupRequestor, two stack arguments. When arg2 is non-zero returns 0. Otherwise allocates 0x124 bytes through 00d833a6; when the allocation is non-NULL constructs an ActorGroupDefinitionAsset in… |
