# Class: SoeUtil::List<FactoryNpcTooltipInfo::FactoryNpcTooltipEffectInfo,10>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e1d70` | `TooltipEffectListCtor` | ✅ | ctor | Clears list links/count, installs fixed-capacity list vtable and initializes storage. |
| `009e2760` | `TooltipEffectListDtor` | ✅ | dtor | Destroys end nodes/backing block, clears pointers/counts and restores base-list vtable. |
| `009e2840` | `ThisAdjustor_009e26b0` | ✅ | virtual | Adds 0x10 to ECX then jumps to 009e26b0; this-adjustor thunk, not target body. |
| `009e2850` | `ThisAdjustor_009e2700` | ✅ | virtual | Adds 0x10 to ECX then jumps to 009e2700; this-adjustor thunk, not target body. |
| `009e33f0` | `TooltipEffectListScalarDeletingDtor` | ✅ | dtor | Calls list destructor, then frees this when deleting flag set; otherwise returns this. |
