# Class: OverHeadChatBubbleElement

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bf0350` | `OverHeadChatBubbleElement::Ctor` | ✅ | ctor | Constructs the overhead base element and string array, initializes the chat bubble vtable, copies three constructor values, and sets timing defaults. |
| `00bf0450` | `OverHeadChatBubbleElement::Dtor` | ✅ | dtor | Restores the chat bubble vtable, destroys its fixed-string array and string member, then restores the OverHeadElement base vtable. |
| `00bf04d0` | `OverHeadChatBubbleElement::ScalarDeletingDtor` | ✅ | dtor | Calls the chat bubble destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
