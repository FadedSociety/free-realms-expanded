# Class: ContentPackManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010463c0` | `ContentPackManager::IsContentUnlocked` | ✅ | method | Thiscall, one stack arg (int id; returns a byte). If id < 1 returns 1. Otherwise indexes the 128-bucket table at this+0x20 by (id & 0x7f), walks the chain (next at node+0x1c, key at node+0), and returns the byte at node… |
| `01046410` | `ContentPackManager::SetFlagById` | ✅ | virtual | Thiscall, two stack args (uint id, char flag). Finds the node for id in the 128-bucket table at this+0x20 (bucket id & 0x7f, chain next at node+0x1c). If found and the byte at node+8 differs from flag, stores flag there… |
| `010469e0` | `ContentPackManager::LoadDefinitions` | ✅ | virtual | Thiscall. Builds the path: the directory string from 00725ee0 with a '/' appended when the last character is not a separator (006fc030), then "ContentPacks.txt" (00402ff0). Reads the file with 00fbf4a0. On failure logs … |
| `01046cf0` | `ContentPackManager::DeleteDefinitions` | ✅ | virtual | Fastcall, ECX = manager. Bytes: push esi; mov esi,ecx; lea ecx,[esi+0x14]; call 01046b70 (clears the ContentPack map at this+0x14); then logs to ContentPacks.log the count at this+0x1c through 00704db0 with the message … |
| `01047b80` | `ContentPackManager::DtorBody` | ✅ | dtor | Fastcall, ECX = manager. Stores ContentPackManager::vftable (0187f460) at *this, calls 01046b70 (clears the content-pack nodes), logs "ContentPackManager::DeleteDefinitions, Content pack manager has %d definitions after… |
| `01047d50` | `ContentPackManager::Ctor` | ✅ | ctor | Thiscall, one stack arg (name C string pointer, may be null). Stores BaseDefinitionManager::vftable and then ContentPackManager::vftable (0187f460) at *this, stores the IString vtable at +4, the empty-buffer pointer &DA… |
| `01047e00` | `ContentPackManager::ScalarDeletingDtor` | ✅ | dtor | Thiscall with a flag byte param_2. Calls 01047b80 (ContentPackManager::DtorBody), then if bit 0 of the flag is set calls 00401c00 with alignment 0 on the object. Returns the object pointer. |
| `01047e20` | `ContentPackManager::LoadDefinitionsFromBuffer` | ✅ | virtual | Thiscall, one stack arg (text buffer, C string). Parses the buffer line by line with 006fc270. Lines starting with '#' are header rows: the cells are read with 010479f0 into a string array, and the cell equal to the nam… |
