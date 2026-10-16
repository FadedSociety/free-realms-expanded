# Class: SoeGems::ObjectFactory<IRequirement,int>::BasePool

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb9e50` | `BasePool::~BasePool` | ✅ | dtor | Replaces the object's vtable with BasePool::vftable and returns; EH unwind funclets call this cleanup routine. |
| `00bb9e60` | `— ⚠reject` | ⚪ | dtor | Restores the BasePool vtable, calls the object's vtable slot +4 destructor, frees the pointer when flag bit zero is set, and returns it. |
