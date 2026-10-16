# Class: ClientObjectiveLookAt

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cf35e0` | `ClientObjectiveLookAt::Ctor` | ✅ | ctor | Clears base state, stores the objective id/type and target identifiers, installs the ClientObjectiveLookAt vtable, copies the supplied position vector and stores the scaled angle/range values. |
| `00cf36a0` | `ClientObjectiveLookAt::CheckCompletion` | ✅ | virtual | When the objective target mode is active, resolves the target or stored position, computes the camera-to-target vector, checks configured range and alignment thresholds, and calls 00cffba0 when alignment passes. |
| `00cf3890` | `ClientObjectiveLookAt::Ctor` | ✅ | ctor | Clears objective base fields, stores the objective id/type and target GUID, installs the ClientObjectiveLookAt vtable, stores the remaining constructor arguments, and scales one float by DAT_01b7dc74. |
