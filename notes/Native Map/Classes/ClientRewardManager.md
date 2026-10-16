# Class: ClientRewardManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a0c3a0` | `ClientRewardManager_AddItemReceived` | 🟢 | method | Looks up an item definition by id param_3 in the hash table reached through FUN_00769710()+4 (bucket id&0x3ff, chain via +0xb0, key at +0xb4); on a miss logs the object 'AddItemReceived - Failed to lookup item definitio… |
| `00b88850` | `ClientRewardManager::FindReward ? ⚠rename` | 🟡 | method | Hash lookup by id: starts at bucket (param_2 % 10) of the table at this+0x14, walks the chain through node+0x1c and returns the first node whose key at node+0x18 equals param_2, otherwise null. |
| `00b88ec0` | `ClientRewardManager::ClientRewardManager` | ✅ | ctor | Constructor: runs the base constructor FUN_008cf6e0, installs ClientRewardManager::vftable, stores param_2 at +0x38 and zeroes the fields at +0x3c and +0x40. |
| `00b88f30` | `ClientRewardManager::~ClientRewardManager` | ✅ | dtor | Destructor body: reinstalls ClientRewardManager::vftable and runs the base destructor FUN_008cf7a0. |
| `00b890f0` | `ClientRewardManager::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 5 of ClientRewardManager: reinstalls the class vtable, runs the base destructor FUN_008cf7a0 and frees the manager through _free when bit 0 of param_2 is set. |
| `00b899f0` | `ClientRewardManager::PlayItemReceivedNotification` | ✅ | method | Item notification: when the notification kind param_10 equals 0x10 it looks up the reward definition in the hash table at (this+0x38)->+0x20c (buckets at +0x64, chain +0xb0, key +0xb4) and on success builds notification… |
