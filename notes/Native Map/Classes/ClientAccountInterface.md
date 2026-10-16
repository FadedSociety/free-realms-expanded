# Class: ClientAccountInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071b350` | `ReplaceRefCountedMemberA260` | ✅ | virtual | Releases old object at offset 0xa260, stores the new pointer, increments its reference count, and returns one. |
| `0071bfd0` | `ClientAccountInterface::CreateParticleDefinitionAsset` | ✅ | virtual | Allocates 0x68 bytes, gets asset context from manager, and constructs ParticleDefinitionAsset with requester pointer and enabled flag. |
| `0071cb20` | `ClientAccountInterface::Destructor` | ✅ | dtor | Installs interface vtable, destroys two inline parameter arrays and ClientAccountDataManager, then tears down AssetRequester state. |
| `0071cec0` | `ClientAccountInterface::Constructor` | ✅ | ctor | Installs AssetRequester and ClientAccountInterface vtables, constructs ClientAccountDataManager and two inline arrays, stores inputs, and initializes selection state. |
| `0071d010` | `ClientAccountInterface::ScalarDeletingDtor` | ✅ | dtor | Calls interface destructor and frees object when deleting flag bit is set. |
| `0071d030` | `ClientAccountInterface::BuildItemData` | ✅ | method | Looks up customization data and dispatches SetItemData, SetItemIcon, SetItemLabel, and SetItemHighlight UI events with parameter arrays. |
| `0071f030` | `ClientAccountInterface::ChangeItem` | ✅ | method | Switch on slotId (param_3) 1..9 = skintone, hairstyle, haircolor, eyecolor, facialhair, facepaint, wings, clothes, head/face-type: stores itemId into this+0x9f7c..this+0x9f9c, resolves the mapping entry through the mana… |
| `0071fd60` | `ClientAccountInterface::BuildCategoryItems` | ✅ | method | Walks categories 1..9; for each it fetches the current mapping/definition row from this+0x10, builds a fixed 7-parameter ClientLuaParam array through 0070b470/007071f0/0070c580 (SoeUtil::Array<ClientLuaParam,1,1> vtable… |
