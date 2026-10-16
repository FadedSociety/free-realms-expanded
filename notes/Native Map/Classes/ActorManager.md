# Class: ActorManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0079c460` | `ActorManagerGetFloat1060` | 🟢 | virtual | Virtual slot [8] of ActorManager (bytes d9 81 60 10 00 00; c3): loads the float at this+0x1060 onto the x87 stack and returns it. |
| `0079c470` | `ActorManagerGetByte1064` | 🟢 | virtual | Virtual slot [7] of ActorManager (bytes 8a 81 64 10 00 00; c3): returns the byte at this+0x1064 in AL. |
| `0079c7c0` | `ActorManagerFindById` | 🟢 | virtual | Virtual slot [4] of ActorManager, thiscall with one stack argument (32-bit id). Hashes the id with & 0x3ff into the 1024-bucket table at this+0x44, walks the chain through +0x1c8 and returns the first entry whose id at … |
| `0079c7f0` | `ActorManagerFindByIdAlt` | 🟢 | virtual | Virtual slot [3] of ActorManager, byte-identical to 0079c7c0 (ActorManagerFindById): hashes the 32-bit id argument with & 0x3ff into the table at this+0x44, walks +0x1c8 and returns the matching entry or 0. |
| `0079c820` | `ActorManagerRecordDrawnActor` | ✅ | virtual | Virtual slot [5] of ActorManager, thiscall with one stack argument (actor). If the actor's +8 link is non-NULL and the link's +4 word is non-NULL it counts a 'skipped' entry at this+0x24. Otherwise, when the actor's +0x… |
| `0079c950` | `ActorManagerSetFloat1060Clamped` | 🟢 | virtual | Virtual slot [9] of ActorManager, thiscall with one float argument. Raises the argument to the floor DAT_017edf3c when it is not greater than the floor, then stores it at this+0x1060. |
| `0079cd90` | `ActorManagerRemoveById` | 🟢 | virtual | Virtual slot [2] of ActorManager, thiscall with one stack argument (id). Finds the entry through this vtable's slot [4] (ActorManagerFindById). When found: unlinks it from its id bucket (0079c690) and from the list (007… |
| `0079d450` | `ActorManagerDtorBody` | ✅ | dtor | Fastcall (ecx = this). Stores ActorManager::vftable at +0, runs 0079cdf0 (drains the actor list into the pool), releases the list block at +0x1054 through 006fb900 or the manager's slot [2], frees the block at +0x1044 t… |
| `0079d5e0` | `ActorManagerSetByte1064` | 🟢 | virtual | Virtual slot [6] of ActorManager (bytes: store of the byte argument): writes the byte argument to this+0x1064 and returns nothing. |
| `0079d5f0` | `ActorManagerCreateEntry` | 🟢 | virtual | Virtual slot [1] of ActorManager, thiscall with one stack argument (request block). Increments the id counter at +0x30 (skipping 0), allocates an entry through 0079cc50, constructs it through 0078aa90, links it at the l… |
