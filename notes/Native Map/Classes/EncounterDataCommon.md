# Class: EncounterDataCommon

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a69b0` | `EncounterDataCommon::EncounterDataCommon` | ✅ | ctor | Copy constructor for an EncounterDataCommon object (a BasePacket-derived encounter data record): installs BasePacket::vftable, copies the packet id word from the source at +4 into +4, installs EncounterDataCommon::vftab… |
| `008a6ae0` | `EncounterDataCommon::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: restores BasePacket::vftable, conditionally calls _free(this) when param_2 bit 0 is set, and returns this. |
