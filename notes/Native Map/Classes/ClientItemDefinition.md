# Class: ClientItemDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0090b720` | `ClientItemDefinition::ClientItemDefinition` | ✅ | ctor | Constructs BaseItemDefinition, initializes its ability list, two shared strings, and several scalar and pointer fields, then calls 00905c20. |
| `0090b820` | `ClientItemDefinition::~ClientItemDefinition` | ✅ | dtor | Destroys embedded strings, removes all ability-list nodes, calls the BaseItemDefinition destructor, and restores ClientItemDefinition's vtable during cleanup. |
| `0090b910` | `ClientItemDefinition::ScalarDeletingDtor` | ✅ | dtor | Calls ClientItemDefinition destruction, frees the object when flag bit 0 is set, and returns the original object pointer. |
