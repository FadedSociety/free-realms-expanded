# Class: SoeUtil::StringFixed<char,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00703300` | `FixedStringAllocate ?` | 🟡 | method | Returns the object's inline string buffer for small requests and allocates external storage when it is insufficient. |
| `00703330` | `FixedStringReleaseBuffer ?` | 🟡 | virtual | Releases a dynamic string buffer while leaving the object's inline buffer intact. |
| `00703540` | `FixedString32Constructor ?` | 🟡 | ctor | Initializes the fixed string's vtable, empty state, and inline buffer representation. |
