# Class: SoundManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007bd9b0` | `SoundManager::AllocConstruct0x98` | ✅ | virtual | Virtual slot [5] (thiscall, two stack args, ret 8): allocates 0x98 bytes through allocator 00d833a6 and, if the result is non-null, runs constructor 007cf450 on it with (this, arg1, arg2); returns the constructed pointe… |
| `007bff80` | `SoundManager::CallKeyedGroupSlot0` | ✅ | virtual | Virtual slot [0] of SoundManager (thiscall, one stack arg, the event pointer P). Finds the first node in the 64-bucket table at this+0x85ac (bucket (P.+0x80) & 0x3f, node key at +0xc, chain next at +0x10) whose key equa… |
| `007c00e0` | `SoundManager::CallKeyedGroupSlot1` | ✅ | virtual | Virtual slot [1] of SoundManager (thiscall, one stack arg, the event pointer P). Same collection as SoundManager::CallKeyedGroupSlot0 (007bff80): the stored pointers of the 64-bucket side table nodes with key P.+0x80 go… |
