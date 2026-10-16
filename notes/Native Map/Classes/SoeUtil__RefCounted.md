# Class: SoeUtil::RefCounted

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071ac50` | `RefCountedVtableReset ?` | 🟡 | function | Stores the RefCounted vtable pointer at object offset zero and returns. |
| `0071b520` | `RefCounted::Destructor` | ✅ | dtor | Installs the RefCounted base vtable at object offset zero and returns. |
| `0071b590` | `RefCountedScalarDeletingDtor` | ✅ | dtor | Installs RefCounted vtable and frees object when the deleting flag bit is set. |
| `0071b950` | `SoeUtil::RefCountedImplicit::RefCountedImplicit → SoeUtil::RefCountedImplicit::…` | 🟢 | ctor | Installs RefCounted vtable, initializes count to zero then one, and installs RefCountedImplicit vtable. |
| `00bbd0f0` | `RefCounted::~RefCounted` | ✅ | dtor | Restores the ExpressionNodeBase and RefCounted vtables; if flag bit 0 is set, frees the receiver with alignment 0, then restores the SEH frame. |
| `00bbd160` | `RefCounted::~RefCounted` | ✅ | dtor | Restores the IRequirement, ExpressionNodeBase, and RefCounted vtables in sequence, then returns. |
| `00bbd2f0` | `RefCounted::~RefCounted` | ✅ | dtor | Restores the IRequirement, ExpressionNodeBase, and RefCounted vtables; if flag bit 0 is set, frees the receiver with alignment 0, then restores the SEH frame. |
