# Class: SoeUtil::StringFixed<16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00709b60` | `FixedStringFree` | ✅ | virtual | Frees the supplied buffer with alignment zero unless it is the inline buffer at this+0x10. |
| `0070ab50` | `StringFixedScalarDeletingDtor → StringFixed16ScalarDeletingDtor` | ✅ | dtor | Releases the string member through 00709b80 and frees the object when the delete flag bit is set. |
