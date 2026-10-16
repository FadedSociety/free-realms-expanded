# Class: HeadshotProxy

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc5cf0` | `HeadshotProxy::SetNestedReadyFlag ? ⚠rename` | 🟡 | virtual | Stores one at offset +0x100 in the object referenced by this+0x10, then consumes one 4-byte stack argument. |
| `00cc5d00` | `HeadshotProxy::~HeadshotProxy` | ✅ | dtor | Restores HeadshotProxy vtables, calls 00cc5c20 to release and clear field +0x20, frees the buffer at +0x10 after 0072d130, then calls the VisibleProxy destructor. |
| `00cc5d80` | `HeadshotProxy::DispatchWhenDelegateReady ⚠downgrade` | 🟢 | virtual | Calls the delegate at this+0xc vtable slot +0x10 with this+0x20; when nonzero and 0076c590 also succeeds, calls 00705200 twice, 00764850 once, then invokes vtable slot 0 on the object at this+0xbc with fields +0x18 and … |
| `00cc5e10` | `HeadshotProxy::ScalarDeletingDestructor` | ✅ | dtor | Calls the HeadshotProxy destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |
| `00cc5e30` | `HeadshotProxy::ReleaseAndClearField20` | ✅ | virtual | Calls 00cc5c20 with this, clears this+0x20, and returns while consuming one 4-byte stack argument. |
| `00cc5e50` | `HeadshotProxy::ClearField20` | ✅ | virtual | Stores zero at this+0x20 and returns while consuming one 4-byte stack argument. |
| `00cc5e60` | `HeadshotProxy::HeadshotProxy` | ✅ | ctor | Constructs the VisibleProxy base, installs HeadshotProxy vtables, stores supplied context fields, allocates and initializes two render-related objects, processes appearance records, and applies HairTint and EyeTint valu… |
