# Class: VisibleProxy

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cc5a00` | `VisibleProxy::ForwardField10ToDelegate → VisibleProxy::TailCallMember10Process` | ✅ | virtual | Loads the pointer at this+0x10 into ECX and jumps directly to 0072cfe0. |
| `00cc5a70` | `VisibleProxy::VisibleProxy` | ✅ | ctor | Installs ActorHandler and BaseAttachmentGroupManagerHandler vtables, replaces both with VisibleProxy vtables, stores param_2 and param_3 at +8 and +0xc, and zeros dwords +0x20 through +0xb0. |
| `00cc5c50` | `— ⚠reject` | ⚪ | dtor | Restores the VisibleProxy vtables, releases the object stored at +8 through the delegate at +0xc vtable slot +8, clears +8, then calls 00798db0 for remaining member cleanup. |
| `00cc5cd0` | `VisibleProxy::ScalarDeletingDestructor` | ✅ | dtor | Calls the VisibleProxy destructor, frees this only when flag bit 0 is set, otherwise returns this, and consumes the 4-byte delete-flag argument. |
