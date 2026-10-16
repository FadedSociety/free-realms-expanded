# Class: ProxiedObject

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009706d0` | `ProxiedObject::SubobjectAt20` | ✅ | virtual | Returns the input pointer plus 0x20 bytes. |
| `009706e0` | `ProxiedObject::ReturnFirstStackArgument` | ✅ | virtual | Loads the dword at [ESP+4], returns it in EAX, and executes ret 8; ECX is untouched. |
| `009706f0` | `ProxiedObject::GetField48` | ✅ | virtual | Reads and returns the dword at this+0x48. |
| `00970700` | `ProxiedObject::SubobjectAt4C` | ✅ | virtual | Returns the input pointer plus 0x4c bytes. |
| `00beff10` | `ProxiedObject::SetTransform` | ✅ | virtual | Copies four 32-bit values from the input transform into object offsets 0x20 through 0x2c. |
| `00beff50` | `ProxiedObject::SetOverheadValue` | ✅ | virtual | Stores the supplied value at offset 0x4c, then obtains overhead element set type 2 and sends the value through virtual slot +0x18. |
| `00bf00e0` | `ProxiedObject::Ctor` | ✅ | ctor | Initializes object fields, installs ProxiedObject vtable, initializes a shared string, sets a color default, and conditionally creates an overhead element set from client state. |
| `00bf01e0` | `ProxiedObject::GetTransformData ? ⚠rename` | 🟡 | virtual | Returns the address of the object's data at offset 0x38. |
| `00bf0240` | `ProxiedObject::Dtor` | ✅ | dtor | Restores the ProxiedObject vtable and decrements the owned shared string reference, invoking its release slot when the count expires. |
| `00bf0330` | `ProxiedObject::ScalarDeletingDtor` | ✅ | dtor | Calls the ProxiedObject destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
| `00bf04f0` | `ProxiedObject::UpdateOverheadChatBubble ⚠downgrade` | 🟢 | virtual | When an overhead set exists, computes timing and color parameters, allocates an OverHeadChatBubbleElement, creates it, then updates existing elements in reverse order. |
| `00bf07c0` | `ProxiedObject::SpawnOverheadEffect` | ✅ | virtual | If an overhead set exists, derives effect parameters from object and input fields, allocates a 0x140-byte effect, creates it through FUN_0096f540, then releases the temporary reference. |
| `00bf0a30` | `ProxiedObject::SpawnOverheadEffectVariant` | ✅ | virtual | If an overhead set exists, computes effect parameters from input state, allocates a 0x140-byte object, creates it through FUN_0096f540, and releases the temporary reference. |
| `00bf0c10` | `ProxiedObject::SpawnOverheadEffectWithParams` | ✅ | virtual | If an overhead set exists, allocates a 0x140-byte effect object, prepares fixed parameter words, calls FUN_0096f540, and releases the resulting reference. |
| `00bf0d20` | `ProxiedObject::SpawnTimedOverheadEffect ? ⚠rename` | 🟡 | virtual | If an overhead set exists, computes a scaled duration, allocates a 0x140-byte effect object, calls FUN_0096f540, and releases the resulting reference. |
