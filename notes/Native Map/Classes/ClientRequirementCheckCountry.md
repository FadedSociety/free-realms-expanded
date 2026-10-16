# Class: ClientRequirementCheckCountry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ce7570` | `ClientRequirementCheckCountry::LoadValue ?` | 🟡 | virtual | Calls conversion helper on field +0x30 and stores its return at +0x60. |
| `00ce7860` | `ClientRequirementCheckCountry::Evaluate ?` | 🟡 | virtual | Reads country value from client context and compares it with loaded requirement value using the configured integer operator. |
