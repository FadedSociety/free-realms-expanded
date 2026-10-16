# Class: DeferredPlayListLoader

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00946110` | `DeferredPlayListLoader::PendingListInit` | 🟢 | init | Initialises an embedded SoeUtil::List<DeferredPlayListLoader::LoadPendingEntry,4>: stores the -1 variant vtable and then overwrites the same dword with the ,4 variant, zeroes this+4, +8, +0xc, +0x10, +0x14 and the byte … |
| `009461a0` | `DeferredPlayListLoader::DeferredPlayListLoader` | ✅ | ctor | Constructor: calls the SoundDefinitionAssetLoader base constructor 008a2530 on this with its stack argument, installs DeferredPlayListLoader::vftable at +0 and +8, calls 00946110 on the member at this+0x10, zeroes this+… |
