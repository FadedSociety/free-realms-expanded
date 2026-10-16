# Class: ActorGroupDefinitionAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0079b510` | `ActorGroupDefinitionAssetCtor` | ✅ | ctor | Fastcall (ecx = this), returns this. Runs the base AssetConstructor 0071bb80, stores ActorGroupDefinitionAsset::vftable at +0, zeroes +0x28, calls 0079aa00 (ActorGroupEntry list constructor) and 00799cd0 (CustomTintEntr… |
| `0079ba50` | `ActorGroupDefinitionAsset::ParseActorSet` | ✅ | virtual | Virtual slot [3] of ActorGroupDefinitionAsset, thiscall with two stack arguments (reader object and element) and a stack-protected 0x8000+ frame. Reads named child elements and attributes through the unlabelled reader r… |
| `0079c180` | `ActorGroupDefinitionAssetDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores ActorGroupDefinitionAsset::vftable at +0, calls 0079a730 (the CustomTintEntry capacity-1 list destructor body) and 0079b420 (the ActorGroupEntry capacity-1 list destructor body) on its memb… |
| `0079c1f0` | `ActorGroupDefinitionAssetScalarDtor` | ✅ | dtor | Thiscall (ecx = this, one flag argument). Calls 0079c180 on this, frees this through _free when bit 0 of the flag is set, and returns this. |
