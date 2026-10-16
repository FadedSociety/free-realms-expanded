# Class: Target

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089c640` | `Target::TargetCtor` | ✅ | ctor | Constructor of Target: installs Target::vftable, clears the field at +4 and forwards param_2 to FUN_0101b250. |
| `0089c6e0` | `TargetScalarDtor` | 🟢 | dtor | Scalar deleting destructor of Target: runs the destructor FUN_0101b370 and calls _free when bit 0 of param_2 is set. |
| `008b1b70` | `Target::Target` | ✅ | ctor | Constructor: writes Target::vftable at offset 0, zeroes this+4 and calls FUN_0101b910(param_2, param_3, param_4); returns this. No branches. |
| `00c01190` | `— ⚠reject` | ⚪ | callback | Parses three numeric coordinates, optional radius and boolean, constructs a Target object from the vector, calls 008c61e0, then destroys the temporary target and pushes the result. |
| `00c01320` | `— ⚠reject` | ⚪ | callback | Parses six coordinates and optional boolean, constructs two Target objects, calls 008c6170 with both, destroys both objects, and pushes the result. |
| `00c01520` | `— ⚠reject` | ⚪ | callback | Parses a GUID and optional radius and boolean, initializes a Target object with that GUID, calls 008c61e0, destroys the temporary target, and pushes the result. |
| `00c017f0` | `Target::ApplyGuidAndVector` | 🟢 | callback | Parses a GUID, three coordinates, and optional values, constructs a vector target plus GUID target, calls 008c6170, and destroys both temporaries. |
| `00c019d0` | `— ⚠reject` | ⚪ | callback | Parses three coordinates and a GUID plus an optional boolean, creates temporary targets for both values, calls 008c6170, and destroys them. |
