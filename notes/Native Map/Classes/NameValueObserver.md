# Class: NameValueObserver

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098dda0` | `NameValueObserver::Destructor` | ✅ | dtor | Installs the NameValueObserver vtable, releases two embedded IString values at +0x1c and +0x2c when owned, then calls base cleanup helper 00835610. |
| `0098de90` | `NameValueObserver::ScalarDeletingDestructor` | ✅ | dtor | Calls the NameValueObserver destructor body; frees this only when deleting flag bit 0 is set, otherwise returns this. |
| `0098df30` | `NameValueObserver::Constructor` | ✅ | ctor | Calls the base observer constructor, installs the NameValueObserver vtable, initializes two embedded IStrings, copies each nonnull input string, and registers the second string through 008356b0. |
| `0098e440` | `NameValueObserver::NotifyChanged` | ✅ | virtual | Builds a ClientLuaParam array containing the observer's key and, when available, a string value from GuiDsValue; invokes the registered callback through 009797a0 and destroys the temporary array. |
