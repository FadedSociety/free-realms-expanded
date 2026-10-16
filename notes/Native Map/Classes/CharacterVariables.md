# Class: CharacterVariables

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008b2c80` | `CharacterVariables::CharacterVariables` | ✅ | ctor | Installs CharacterVariables::vftable, zeroes fields +4, +8, +0xc, +0x10 and the 0x40-byte block starting at +0x14 via _memset, and zeroes +4 last. Returns this. |
| `008b2cf0` | `CharacterVariables::~CharacterVariables` | ✅ | dtor | Stores CharacterVariables::vftable into *this and returns; it frees nothing and is called from six MSVC unwind funclets (0158da84, 0158db44, 01597440, 01597778, 016393e3, 01639503). |
| `008b2d40` | `CharacterVariables::ScalarDeletingDestructor` | ✅ | dtor | Stores CharacterVariables::vftable into *this and, when bit 0 of the flag argument is set, calls operator delete on this; returns this. |
