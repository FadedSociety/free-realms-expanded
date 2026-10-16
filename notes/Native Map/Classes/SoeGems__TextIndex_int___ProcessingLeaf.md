# Class: SoeGems::TextIndex<int>::ProcessingLeaf

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b8e070` | `— ⚠reject` | ⚪ | ctor | Installs WeightedProvider and ProcessingLeaf vtables, sets provider index to -1, stores two supplied values, and clears another field. |
| `00b8e0f0` | `ProcessingLeaf::GetWeight ⚠downgrade` | 🟢 | virtual | Returns the dword at offset eight from the provider object referenced at this object's offset twelve. |
| `00b8e100` | `ProcessingLeaf::DtorBase` | ✅ | dtor | Restores ProcessingLeaf vtable, then restores WeightedProvider base vtable. |
| `00b8e150` | `ProcessingLeaf::Next` | ✅ | virtual | Copies one two-dword array element and two context dwords to output, advances cursor, and returns success; otherwise reports exhaustion. |
| `00b8e1a0` | `ProcessingLeaf::ScalarDeletingDtor` | ✅ | dtor | Restores ProcessingLeaf and WeightedProvider vtables, then conditionally frees the object. |
