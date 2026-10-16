# Class: SoeGems::ObjectFactory<IRequirement,int>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bbd4d0` | `ObjectFactory::InitRequirementFactory` | ✅ | ctor | Installs the factory and requirement-to-pool map vtables, clears the 0x20000-byte pool map, initializes the embedded creator and pool maps on their subobjects, and stores the supplied factory field. |
| `00bbd580` | `— ⚠reject` | ⚪ | method | Scans up to 0x400 pool buckets, invokes each encountered node's virtual slot 0 with flag 1, then clears the creator map through 00bb9d00. |
