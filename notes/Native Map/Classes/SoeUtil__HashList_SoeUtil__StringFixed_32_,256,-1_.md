# Class: SoeUtil::HashList<SoeUtil::StringFixed<32>,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00981780` | `HashList_StringFixed32_Ctor` | 🟢 | ctor | Constructor: installs the HashListMap vtable, zeroes head/tail/count and the count field at +0x10, memsets the 256 bucket heads at +0x14 (0x400 bytes) to zero, then writes the HashList<StringFixed<32>,256,-1> vtable and… |
