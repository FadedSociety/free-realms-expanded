# Class: GameCommerce::UramManager::UramRequestTracker

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68220` | `— ⚠reject` | ⚪ | ctor | Stores request type and two constructor arguments, installs the tracker vtable, initializes four linked-list endpoints and counters, then obtains a synchronization object. |
| `00d682f0` | `UramRequestTracker::Dtor ?` | 🟡 | dtor | Stores the request tracker vtable pointer and returns without other work. |
| `00d6a030` | `GameCommerce::UramManager::UramRequestTracker::WriteVtableAndFreeWhenFlagSet → …` | ✅ | virtual | Writes the UramRequestTracker vtable, frees this only when flag bit 0 is set, and otherwise returns this; it does not call a destructor body. |
