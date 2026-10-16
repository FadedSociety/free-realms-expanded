# Class: ChatPacketDebugChat

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0f150` | `ChatPacketDebugChat::Dtor` | ✅ | dtor | Restores ChatPacketDebugChat and base packet vtables, cleaning up the debug-chat payload member. |
| `00b0f1c0` | `ChatPacketDebugChat::ScalarDeletingDtor` | ✅ | dtor | Calls the concrete debug-chat destructor and frees the object when the low flag bit is set. |
| `00b0f9d0` | `ChatPacketDebugChat::Ctor ⚠downgrade` | 🟢 | ctor | Initializes BaseChatPacket mode 3, installs the concrete vtable, initializes a subobject, and clears a trailing boolean. |
