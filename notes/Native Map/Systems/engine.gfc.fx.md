# System: engine.gfc.fx

66 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (42)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405a32` | `GDrawFx::GDrawFx` | GDrawFx | Base class of the GFC draw-effect objects drawn by the client's display layer. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405ab0` | `GFillFx::GFillFx` | GFillFx | Fill-effect drawing object (solid fills) of the GFC display layer. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405b36` | `GBlitFx::GBlitFx` | GBlitFx | Blit-effect drawing object (image blits) of the GFC display layer. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `0040e68c` | `GLineFx::GLineFx` | GLineFx | Constructor of the line-drawing effect object; the vtable names GLineFx::vftable at 017fde04/017fde0c and the base constructor is the GDrawFx one already mapped in S0001. | [[Native Map/Shards/S0003 0040affb-004103c5|S0003]] |
| `004186c9` | `GLineFx::GLineFx` | GLineFx | Construction of the line-effect variant of the GFC display layer (the base class ctor GDrawFx::GDrawFx at 00405a32 was mapped in S0001); the effect kind 1 matches the kind values of GFillFx (2) and GBlitFx (3). | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00418700` | `GFillFx::GFillFx` | GFillFx | Construction of the solid-fill effect of the GFC display layer; the effect kind 2 and field block match the GLineFx (1) and GBlitFx (3) constructors. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `0041873b` | `GBlitFx::GBlitFx` | GBlitFx | Construction of the image-blit effect of the GFC display layer; the 0xFFFFFF default is consistent with an initial colour/alpha value for blitting. | [[Native Map/Shards/S0005 00414a48-00419846|S0005]] |
| `00419c67` | `CtorVtables175D5E8` |  | Creates a composite fill/line/blit drawing object from a description struct (the drawing primitive set of the display layer). | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `0043d5a3` | `ConfigureBlitEffect → ConfigureGBlitFx` |  | Configures the explicitly named GBlitFx object; the caller-visible feature represented by its state bits is unknown. | [[Native Map/Shards/S0012 00437372-0043dc9a|S0012]] |
| `004595b0` | `GFlashFx::GFlashFx` | GFlashFx | Default constructor of the GFlashFx draw-effect object of the GFC display layer. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004595f2` | `GFlashFx::Slot2Thunk5017F8` | GFlashFx | Virtual method of the RTTI-named GFlashFx class; GDrawFx carries the identical slot-2 body at 00405a71, so this is the fx-family emission rather than a free helper. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004801e5` | `GDrawFx::WriteGlobalAndOptionalArgument` | GDrawFx | Slot-1 implementation of GDrawFx; the role of DAT_01af81c4 is not established. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480270` | `RegisterGDrawFx` | GDrawFx | Startup registration of the GDrawFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480302` | `GLineFx::WriteGlobalAndOptionalArgument` | GLineFx | Slot-1 implementation of GLineFx; the role of DAT_01af82a4 is not established. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048038d` | `RegisterGLineFx` | GLineFx | Startup registration of the GLineFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004803c6` | `GLineFx::CopyFieldsCheckedPlus` | GLineFx | Slot-64 field-copy override for GLineFx. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480431` | `GFlashFx::WriteGlobalAndOptionalArgument` | GFlashFx | Slot-1 implementation of GFlashFx; the role of DAT_01af8414 is not established. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004804bc` | `RegisterGFlashFx` | GFlashFx | Startup registration of the GFlashFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480536` | `GFillFx::WriteGlobalAndOptionalArgument` | GFillFx | Slot-1 implementation of GFillFx; the role of DAT_01af84c4 is not established. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004805c1` | `RegisterGFillFx` | GFillFx | Startup registration of the GFillFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004805fa` | `GFillFx::CopyFieldsAndBlockChecked` | GFillFx | Slot-64 field-copy override for GFillFx. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480689` | `GBlitFx::WriteGlobalAndOptionalArgument` | GBlitFx | Slot-1 implementation of GBlitFx; the role of DAT_01af861c is not established. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480714` | `RegisterGBlitFx` | GBlitFx | Startup registration of the GBlitFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048074d` | `GBlitFx::CopyFieldsChecked` | GBlitFx | Slot-64 field-copy override for GBlitFx. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048084f` | `RegisterGTextFx` | GTextFx | Startup registration of the GTextFx drawing-effect class. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048c9f4` | `LoadColorCombineOps` |  | read-side accessor used by the draw-state property loaders to fetch the colour-combine operations for a colour-combine state object. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048ca8f` | `StoreColorCombineXY` |  | Writes values under keys x and y. This contradicts the earlier ColorOp/AlphaOp interpretation; callers include StoreFxBase and StoreColorKeyFxProps, so schema intent is unresolved. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048de8a` | `LoadBlitDrawState` |  | read-side property loader for the blit draw-state structure (mirror flag, colour, destination/source offsets and source size) used when a blit state is deserialised from a property bag. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048df9c` | `StoreBlitDrawState` |  | write-side property saver for the blit draw-state structure. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e0b4` | `GDrawFx::CopyCtor` | GDrawFx | copy-construction path of the GDrawFx effect base used when an effect is cloned from an existing effect object. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e0ee` | `LoadFxBase → GDrawFx::LoadFxBase` |  | common base-field loader shared by every effect property loader in this shard (line/fill/bitmap/text effects chain it first). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e15b` | `StoreFxBase → GDrawFx::StoreFxBase` |  | common base-field saver shared by every effect property store function in this shard. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e1c7` | `LoadLineFxProps → GLineFx::LoadLineFxProps` |  | read-side property loader of the line-effect draw state (antialias flag, two colours and the dashed-line pattern). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e2af` | `StoreLineFxProps → GLineFx::StoreLineFxProps` |  | write-side property saver of the line-effect draw state. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e441` | `LoadFlashFxProps` |  | read-side property loader of the Flash-movie effect (its scale/field data plus the movie name stored at +0x24). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e489` | `StoreFlashFxProps → GFlashFx::StoreFlashFxProps` |  | write-side property saver of the Flash-movie effect. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e4cf` | `LoadFillFxProps → GFillFx::LoadFillFxProps` |  | read-side property loader of the fill-effect draw state (antialias flag, foreground/background colours, hatch pattern). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e5d8` | `StoreFillFxProps → GFillFx::StoreFillFxProps` |  | write-side property saver of the fill-effect draw state. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e6d8` | `LoadColorKeyFxProps → GBlitFx::LoadColorKeyFxProps` |  | read-side property loader of the effect that blends a colour through a colour-key mask (bitmap colour keying). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e7c2` | `StoreColorKeyFxProps → GBlitFx::StoreColorKeyFxProps` |  | write-side property saver of the colour-key blending effect. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e8a8` | `LoadTextFxProps` |  | read-side property loader of the text-effect draw state (text colour, background colour and the two combine operations). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0048e992` | `StoreTextFxProps` |  | write-side property saver of the text-effect draw state. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00419d87` | `ScalarDeletingDtor` |  | Deleting destructor for the object whose destructor body is FUN_00410182; reached through a vtable slot. | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `004595d4` | `GFlashFxCallTable0` |  | Call-table emission of the GFlashFx translation unit (identical pattern at 00405a53/00405ad8/00405b65). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004595df` | `GFlashFxCallTable1` |  | Second call-table emission of the GFlashFx translation unit (pattern of 00405a5e). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045960b` | `GFlashFx::GFlashFx_2` | GFlashFx | Second RTTI-confirmed GFlashFx constructor. The role of param_2 passed to 004d9a26 is not established by this body. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004802d4` | `FxCopyBaseFieldsUnchecked` |  | Base-field copy used by the unchecked Fx copy operations. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004803f7` | `FxCopyFieldsPlusUnchecked` |  | Unchecked field copy for the derived Fx record. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048050e` | `FxCopyField24Unchecked` |  | Unchecked field copy for the flash-effect record. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048063d` | `FxCopyFieldsAndBlockUnchecked` |  | Unchecked field copy for the fill-effect record; one of the modes handled by the 00406913 dispatcher. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480784` | `FxCopyFields2Unchecked` |  | Unchecked field copy for the blit-effect record; used by the mode dispatcher and other drawing callers. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480888` | `CtorVtables1769298` |  | Constructor for a text-effect drawing object whose vtables sit directly after the GTextFx registration block. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |

## 🟡 speculative (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00405a53` | `GDrawFxCallTable0 ?` |  | Effect-object virtual invocation through the object's own table pointer (GDrawFx family; identical emissions at 00405ad8 and 00405b65). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405a5e` | `GDrawFxCallTable1 ?` |  | Effect-object virtual invocation through the object's table pointer (identical emission at 00405ae3 and 00405b70). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405a94` | `FxDeleteHelper ?` |  | Deleting-destructor helper in the effect-object translation unit. | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405ad8` | `GFillFxCallTable0 ?` |  | Effect-object virtual invocation for fill effects (emission of the same pattern as 00405a53). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405ae3` | `GFillFxCallTable1 ?` |  | Effect-object virtual invocation for fill effects (emission of the same pattern as 00405a5e). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405b65` | `GBlitFxCallTable0 ?` |  | Effect-object virtual invocation for blit effects (emission of the same pattern as 00405a53). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `00405b70` | `GBlitFxCallTable1 ?` |  | Effect-object virtual invocation for blit effects (emission of the same pattern as 00405a5e). | [[Native Map/Shards/S0001 00401040-00406b84|S0001]] |
| `0040d6e2` | `GDrawFx::VectorDeletingDtorThunk ? ⚠rename` | GDrawFx | Slot [0] of GDrawFx::vftable (secondary-vtable entry 017fa410) needs this adjustor so deleting through the secondary interface frees the right object. | [[Native Map/Shards/S0003 0040affb-004103c5|S0003]] |

## ⚪ unknown (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0041d3a0` | `— ⚠reject` |  | Constructor of the fourth fx sub-object that the composite draw object (00419C67/0041D672) embeds at +0x104; same defaults appear at composite offsets +0x11C..+0x13C. | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `0041d672` | `— ⚠reject` |  | Creates the display layer's default fill/line/blit drawing object with the client's stock styles. | [[Native Map/Shards/S0006 00419961-0041e544|S0006]] |
| `004802a3` | `— ⚠reject` | GDrawFx | Slot-64 field-copy override for GDrawFx. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `004804ef` | `— ⚠reject` | GFlashFx | Slot-64 field-copy override for GFlashFx. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `00480747` | `— ⚠reject` |  | Returns a pointer associated by its caller with an FX property descriptor; the property meaning is not established in this body. | [[Native Map/Shards/S0026 0047ca5e-004823b9|S0026]] |
| `0048e38d` | `— ⚠reject` | GFlashFx | the GFlashFx construction/destruction dispatcher of the flash-effect class (creates on the stack or heap, destroys through the virtual deleting destructor). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |

