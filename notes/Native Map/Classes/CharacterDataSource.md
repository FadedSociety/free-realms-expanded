# Class: CharacterDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bfca90` | `CharacterDataSource::FieldNameForIndex` | ✅ | virtual | Maps integer indices 0 through 0x1b to fixed field-name strings for identity, wallet, progression, membership, demographics, and social-profile entries; returns <error> for other values. |
| `00bfcfe0` | `CharacterDataSource::GetAboutMeValue` | ✅ | virtual | Switches on the requested property, reads player statistics or durations, formats localized labels, and writes the result through the supplied output interface. |
| `00bfdd20` | `CharacterDataSource::DtorBody` | ✅ | dtor | Restores three CharacterDataSource vtable pointers, releases seven owned IString buffers by decrementing their refcounts, then destroys the embedded GuiUserDataTable holder. |
| `00bfdf20` | `CharacterDataSource::Ctor` | ✅ | ctor | Stores the passed owner pointer, installs three CharacterDataSource vtables, clears a flag, and initializes seven empty IString members. |
| `00bfe070` | `CharacterDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls 00bfdd20 to destroy the object and calls _free(this) only when flag bit 0 is set; otherwise returns this. |
