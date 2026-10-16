# Class: T4ContextList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00e75be0` | `T4ContextList::Resize` | ✅ | method | thiscall, new count in stack arg 1: when the current 16-bit count at +0 is below the new count, it (re)allocates the pointer array at +4 (size count*4 plus a header; tracked allocation 00e612d0 with the T4ContextList::D… |
| `00e75e00` | `T4ContextList::Append` | 🟢 | method | fastcall (ECX = list): reads the 16-bit capacity at +0 and the 16-bit used count at +2; when capacity is not above used it grows the list through T4ContextList::Resize (00e75be0) to a computed size (in steps of four ent… |
