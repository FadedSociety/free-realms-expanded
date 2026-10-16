# Class: CCNamesDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00722c60` | `CCNamesDefinition::~CCNamesDefinition` | ✅ | dtor | Installs CCNamesDefinition::vftable, then rewrites the vtable of the embedded IString at +0x14 and releases that string buffer when it is owned (capacity field +0x20 positive). |
| `00722cd0` | `CCNamesDefinition_ScalarDeletingDtor` | 🟢 | dtor | Calls the CCNamesDefinition destructor body 00722c60 for this and calls _free when bit 0 of the flag byte is set; returns this. |
| `00722cf0` | `CCNamesDefinition::CCNamesDefinition` | ✅ | ctor | Installs CCNamesDefinition::vftable, clears +4..+0x10, constructs the embedded IString at +0x14 (vtable plus static empty buffer), releases any previous buffer, stores the static empty string again and clears +0x24. |
| `00723660` | `CCNamesDefinition_ScalarDeletingDtor_B` | 🟢 | dtor | Under an SEH frame calls the CCNamesDefinition destructor body 00722c60 for this and calls _free when bit 0 of the flag byte is set. |
| `00725540` | `CCNamesDefinition_ParseFields` | ✅ | method | Scans one delimited [:...:] token from the row with 006fb560, formats it into the embedded string at +0x14 as [:%s:], skips the following delimited token and tokenizes the remaining columns into +0x24. |
