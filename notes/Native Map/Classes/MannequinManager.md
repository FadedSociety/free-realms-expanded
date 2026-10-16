# Class: MannequinManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b87d70` | `MannequinManager::MannequinManager` | ✅ | ctor | Constructor: reads the service pointer from FUN_00769710 (+0x18) into +4, installs AssetManagement::AssetDeliveryHandlerInterface::vftable then MannequinManager::vftable, stores param_2 at +8, zeroes +0xc/+0x10/+0x14/+0… |
| `00b87e00` | `MannequinManager::FindEntry` | 🟢 | method | Hash lookup: starts from bucket (param_2 & 0xf) of the bucket table at this+0x1c, walks the chain via the next pointer at node+0x28 and returns the first node whose key at node+0x2c equals param_2, or null when the chai… |
| `00b87e30` | `MannequinManager::RemoveEntry` | 🟢 | method | Unlinks an entry from both containers: finds the node in the bucket chain at this+0x10 + (key&0xf)*4 by identity, splices it out (piVar1 = next), zeroes the node's chain/key fields, then unlinks the node from the doubly… |
| `00b88140` | `— ⚠reject` | ⚪ | method | Takes the record pointer stored at this+4; when non-null it unlinks the record from its containers via 00b87e30, releases its strings through 00b88050 and frees the memory with _free. |
| `00b881c0` | `MannequinManager::~MannequinManager` | ✅ | dtor | Destructor body: reinstalls MannequinManager::vftable, destroys the stored entry via 00b88140 and then runs the base teardown FUN_010935e0 (asset-delivery-handler base destructor). |
| `00b882f0` | `— ⚠reject` | ⚪ | virtual | Parses a mannequin definition text: builds a local IString from the byte buffer (data at +4, length at +8) appended byte by byte, then repeatedly, while the cursor starts with '#', tokenizes a numeric field (FUN_006fba4… |
| `00b885a0` | `MannequinManager::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of MannequinManager: runs the destructor body 00b881c0 and frees the manager object through _free when bit 0 of param_2 is set. |
