# Class: ClientAreaAdvJournalProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ceb810` | `ClientAreaAdvJournalProperty::LoadRegionOverride ?` | 🟡 | virtual | Reads RegionOverride from the property source and stores the supplied integer at object offset +0x38. |
| `00ceb840` | `ClientAreaAdvJournalProperty::Log ?` | 🟡 | virtual | Logs inherited property fields and the RegionOverride value at index 14. |
| `00ceb890` | `ClientAreaAdvJournalProperty::Apply ?` | 🟡 | virtual | For an active area object whose type and pointer fields are valid, checks a helper, then passes RegionOverride to a state tracking helper. |
| `00ceb8d0` | `ClientAreaAdvJournalProperty::Remove ?` | 🟡 | virtual | For an active area object whose type and pointer fields are valid, checks a helper, then removes RegionOverride through a state tracking helper. |
