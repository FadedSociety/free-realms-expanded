# Class: EncounterDetailsCommon

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a28430` | `EncounterDetailsCommon::Ctor` | 🟢 | ctor | Initializes participant and team lists, record fields and embedded objective state, then sets trailing flags. |
| `00a284d0` | `EncounterDetailsCommon::Dtor` | 🟢 | dtor | Destroys embedded objective state, clears team nodes and destroys nested participant list. |
| `00a28560` | `EncounterDetailsCommonDtorUnwind ?` | 🟡 | dtor | Creates unwind frame and calls EncounterDetailsCommon destructor on guarded temporary. |
| `00a285b0` | `EncounterDetailsCommon::CtorWithKey ?` | 🟡 | ctor | Constructs common detail record and stores supplied key at object offset 0x2d8. |
| `00a28e20` | `WriteEncounterDetailsCommonRecord` | 🟢 | function | Writes two identifier dwords, serializes teams through 00a27750 and participant-map lists through 00a27110, writes scalar fields and flags, emits nested objective/detail data through 00a28800, then writes trailing flags. |
| `00a29120` | `EncounterDetailsCommon::ReadRecord` | ✅ | function | Reads integers, nested team and participant lists, flags, objective data and trailing booleans from bounded stream. |
