# Class: SoeUtil::HashList<GodBeamDefinition,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff110` | `HashList_GodBeamDefinition_Construct` | ✅ | ctor | Saves this, calls the base constructor 009fefe0 with this in ECX, then stores SoeUtil::HashList<GodBeamDefinition,8,1>::vftable at *this and returns this. |
| `009ff170` | `HashList_GodBeamDefinition_ScalarDeletingDestructor` | ✅ | dtor | Calls 009fe520 with this in ECX to tear down the container; if flag bit 0 is set, calls _free(this). |
