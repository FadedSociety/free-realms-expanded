# Class: DBMInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `013c29c0` | `DBMInterface::~DBMInterface` | ✅ | dtor | Destructor body (fastcall): re-stores the DBMInterface vtable at +0 and returns. Nothing is freed here. |
| `013c29d0` | `DBMInterface::SetWordAt4 ?` | 🟡 | virtual | Virtual slot [10] of DBMInterface: stores the argument word at this+4 and returns (bytes 8b 44 24 04 / 89 41 04 / c2 04 00). |
| `013c29e0` | `DBMInterface::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0] of DBMInterface (bytes f6 44 24 04 01 / 56 / 8b f1 / c7 06 ... / 74 09 / 56 / e8 ... / c3): stores the DBMInterface vtable, calls _free(this) only when bit 0 of the argument is set, and returns this. |
