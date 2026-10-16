# System: engine.gfc.font

113 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (17)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00450832` | `RegisterGFontDeviceFT2` |  | Registers the Scaleform FreeType2 font-device class using the dotted registry key shown in the binary string table. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450a32` | `RegisterGFontDescFT2` |  | Registers the FreeType2 font-descriptor class used by the FT2 font device. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452b79` | `RegisterGFCFontDeviceInterface` |  | Declares the class-implements-interface binding GFC_Draw_FontDevice -> GFC_Interface for the "Scaleform FreeType2 FontDevice" version 1.0.0 at startup. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00458ab5` | `GfcSystemFontScanFontDirectory` |  | The GFC System Font loader: scans the game Fonts/ directory, loads .ttf/.pfb faces into the font manager and fills the font-cache entries. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00490072` | `LoadFontInfo` |  | read-side font-description loader used when a font resource is deserialised from its property bag (callers 00452626 and 004a2a20 feed the loaded metrics into their font objects). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490322` | `StoreFontInfo` |  | write-side font-description saver used when a font resource is serialised back into its property bag (callers 00450a65 and 0049adbd). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `004905d8` | `LoadFontInfoEx` |  | read-side loader of the extended (typographic) font metrics object used when a font resource is deserialised from its property bag. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0049087f` | `StoreFontInfoEx` |  | write-side saver of the extended font metrics object. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490b29` | `LoadGlyphInfo` |  | read-side glyph metric loader used by the bitmap-glyph loader (caller 0049abe2) when a font glyph record is deserialised. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00cfff60` | `FindOrCreateGlyphTextureEntry` |  | Provides cached texture records for glyph lookup and font measurement. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d02bb0` | `FontBitmapPrimitiveCtor` | Deep::FontBitmapPrimitive | Constructs a Deep font bitmap primitive record. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d02c30` | `FontBitmap9GridPrimitiveCtor` | Deep::FontBitmap9GridPrimitive | Constructs a Deep 9-grid font bitmap primitive record. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d02de0` | `FontBitmapPrimitiveFillQuad` | Deep::FontBitmapPrimitive | Writes the quad vertex data of one font bitmap primitive; the 0xc69b89d8 slot call is the buffer lock and is not named. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d03360` | `FontBitmap9GridFillQuads` | Deep::FontBitmap9GridPrimitive | Fills a nine-cell (3x3 grid) font bitmap primitive buffer; grid meaning inferred from the nine-cell loop. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d03a90` | `FontPrimitiveBuildGlyphQuads` | Deep::FontPrimitive | Lays out and fills the quads for a text string in a Deep font primitive. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04320` | `GlyphTextureRowAlloc` |  | Packs glyph rectangles into a glyph texture, starting a new row when needed. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04650` | `GlyphBitmapRowAlloc` |  | Packs glyph rectangles into a 512 x 1024 glyph bitmap row by row. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## 🟢 likely (67)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00450923` | `CtorVtables1764070` |  | Construction of a font-layer object that shares the 01764070 vtable family with the font descriptor class. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045097e` | `ScalarDeletingDtorFontObj` |  | MSVC scalar deleting destructor for the font-object class whose cleanup routine is FUN_00526562. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450a65` | `SetFontInfoProperty` |  | Attempts to store this+0x24 under the FontInfo property on param_2; the return value reports only the initial validation result, not whether the property store succeeded. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450ab7` | `CtorVtables1764070_b` |  | Second construction path of the 01764070/01764068 class (the other is 00450923); the font-descriptor reading is context rather than body evidence. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450ad0` | `ReleaseFontFace` |  | Frees the FreeType face/stream owned by the font descriptor (called before re-assigning a new font). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450af3` | `LoadGlyphMetrics` |  | FreeType glyph load used by the FT2 font device to obtain glyph metrics for a character code. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450bdf` | `BlitMask1bppOr` |  | Glyph-mask compositing into a 1-bit destination surface of the font device's glyph cache. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450cec` | `BlitMask4bpp` |  | Glyph compositing into a 4-bit destination surface. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450dae` | `BlitMask8bpp` |  | Glyph compositing into an 8-bit destination surface. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450e2a` | `BlitMask16bpp` |  | Glyph compositing into a 16-bit destination surface. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450eac` | `BlitMask24bpp` |  | Glyph compositing into a 24-bit destination surface. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450f3c` | `BlitMask32bpp` |  | Glyph compositing into a 32-bit destination surface. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00450fc8` | `ScalePointAndAppend` |  | Appends a scaled 2-float point record to the blocked 8-byte-pair container used by the draw/scale code (sibling of the accepted ScalePairAndAppend 00454bb7, which uses the same global scale). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045100c` | `ScaleTwoPointsAndAppendA` |  | Appends a scaled two-point segment to the blocked 8-byte-pair container used by the draw code (FUN_004aaece and FUN_004544f8 are the pair-append family). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00451063` | `ScaleTwoPointsAndAppendB` |  | Second emission of the scaled two-point append used by the draw code. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004510ba` | `CtorVtables17641C8` |  | Construction of a font/GFx helper object (constructed from 00457bb7 in the font module). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045149b` | `FontKeyCompareName` |  | Compares a wide-string key. Font-map use is inferred from its lower-bound and map callers; this body contains no font-specific anchor. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00451f28` | `CtorCopyFontOwner` |  | Copies a font owner/provider reference into a font-layer object (constructed at 004531eb/0045321e). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00451f59` | `ReleaseFontOwnerRef` |  | The short x86 body makes the interface release and tail-calls the string cleanup helper. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004524c9` | `OffsetGridPoints` |  | Translates a batch of point records (glyph/text geometry) by an (x,y) offset. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045255a` | `CopySecondaryBase17642E4` |  | Secondary-base copy construction used by the font/GFx auxiliary object (caller 0045910d). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045257b` | `FontRecordCopyCtor` |  | Copies two wide strings and metric fields in a record used by the font subsystem; the font association is inferred from the surrounding copy/load paths, as no RTTI name is present. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452626` | `LoadFontInfoFromProperty` |  | Loads the FontInfo property value of param_2 into this object's +0x24 field after releasing this object's current FreeType face; the writer twin FUN_00490322 is used by the genuine store path 00450a65. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452676` | `FontDeviceCtor` |  | Construction of the Scaleform/FreeType2 font-device instance registered by 00452b79. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452724` | `FontDeviceDtor` |  | Destruction of the font-device instance, releasing its FreeType resources before the base teardown. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452ccd` | `ReleaseField60` |  | Teardown step releasing the font owner/provider reference at +0x60 (sibling of 00451f59). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452d41` | `CtorCopyFontOwnerFromPtr` |  | Copies a font owner/provider reference given indirectly (through a pointer-to-pointer) into a font-layer object. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452dad` | `PointerArrayGrowAndAllocSlab` |  | Grows the pointer table and allocates one 0x800-byte block per call. Caller 00453739 indexes 256 8-byte records per block, supporting the pair-record use; the allocator body does not identify a font glyph or mask role. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00453571` | `GfcFontCopyCtor` |  | Copy constructor of the same object type the font creator 0045471b builds (both store the PTR_FUN_017642f0 / PTR_LAB_017642e8 vtable pair and use the 0xe4-byte allocator 004509c0). | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `004535dd` | `GfcFontScalarDeletingDtor` |  | Delete path for the font object built by 0045471b/00452676; 004509dd is the conditional deallocation path, not the CRT _free thunk. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `004535f9` | `GfcFontGlyphDraw` |  | Draws one character/glyph of a GFC font through the 2D renderer; the bits resource tag marks the bitmap font path. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `0045471b` | `GfcFontCreate` |  | Font-object creation path called by font management code 0045760b/004577b6. The literal name and renderer setup are direct anchors; default DPI lookup occurs in 004549aa, not this function. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `00454b11` | `GfcFontSizeResolveStub` |  | Thin wrapper around the size resolver 004549aa. Raw call setup shows the local pair is passed by address and the resolver receiver remains in ECX; why this wrapper supplies a zero pair is unknown. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `00454bb7` | `ScalePairAndAppend` |  | One of the four function pointers passed by 00454bed to 006ba990, whose loop invokes the table entries. Its pair-append role is mechanical; the meaning of the produced pair is unknown. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `00457564` | `CtorVtable17644A0` |  | Table-based primary-base constructor called first by the font-manager constructor 00459198; its original class name is not recovered. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004575cc` | `CopyCtorVtable17644A0` |  | Copies the unnamed primary base as part of font-manager copy construction at 0045910d; its original class name is not recovered. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045760b` | `GfcSystemFontLookup` |  | Cached font creation: turns a font descriptor into the cache key and creates the font object on a miss. The name rests on the callee roles (FUN_0045471b is the font constructor, FUN_00454523 builds the '<name> <size>' k… | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004577b6` | `GfcSystemFontLookupDefaultSize` |  | Default/DPI size resolution ahead of the font-cache lookup; the qualifier reflects only that resolution - no device argument is read, the DPI comes from the screen DC, and the same logic is recorded for GfcFontSizeResol… | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004581ae` | `GfcSystemFontResizeRecords` |  | Resize helper observed on the font construction path (callers 00458ab5, 004be2b1). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `004588e0` | `GfcSystemFontAddEntry` |  | Font-cache insertion observed on the font construction path (caller 00458ab5); the cached key is the font descriptor and the value carries the face's records. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00458f8b` | `GfcSystemFontManagerCopyMap` |  | Map sub-object construction used by the font-manager copy constructor at 0045910d. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00458ffa` | `GfcSystemFontManagerDtor` |  | Font-manager destruction, called by the deleting destructor 0045917c and from catch paths. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045910d` | `GfcSystemFontManagerCopyCtor` |  | Copy-construction path used by the class dispatcher 00459221. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045917c` | `GfcSystemFontManagerScalarDtor` |  | Deleting destructor of the GFC font manager. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00459198` | `GfcSystemFontManagerCtor` |  | The font-manager object created by the dispatcher at 00459221 and populated by the directory scan 00458ab5. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00459221` | `GfcSystemFontManagerFactory` |  | Class-registry dispatcher for the GFC font manager (same shape as 00457bb7). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0048a620` | `GlyphLookupMetric` |  | Character-record value lookup with a cached default-record fallback and an optional virtual adjustment. The meanings of record fields +0x34 and +0x40 are unknown. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048a6e8` | `GlyphInfoGet` |  | Builds a nine-dword output record from a character record and an object at this+0x80. The character and atlas interpretations follow the field accesses and related routines; the fallback cache is not a cache of the last… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00490cc0` | `StoreGlyphInfo` |  | write-side saver of glyph metrics used by the bitmap-glyph saver (caller 0049acd5). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490e77` | `LookupSortedPairValue` |  | sorted pair-keyed value lookup; the two 16-bit key fields and the 8-byte entry layout are established by the code, while the domain (character-pair metrics or another pair table) is not. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490f3b` | `CachePointDistanceAndTestAboveEpsilon` |  | distance cache plus epsilon test over the two 2D points handed to it (callers 004a927c and 004a9849). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00490f7c` | `WrapPagedIndexLookup` |  | circular lookup into a paged 12-byte record array; only the layout and the wrap arithmetic are established here (caller 004aa19e). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491824` | `InitGlyphAtlasPages` |  | geometry initialiser whose written fields match the receiver read by the glyph-cache accessor FUN_004919AD (interface at +0x44 queried through +0x10C, cell sizes, +0xC8); the glyph-cache basis is that field match, not a… | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `004919ad` | `GetGlyphSlot` |  | primary glyph-cache accessor which rasterises on demand via the font device and returns the cached glyph block. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491ae2` | `GetGlyphSlotSizeA` |  | glyph size query of the glyph cache (first size component). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00cffc80` | `Initialize128SlotCache` |  | Initializes the bounded cache state used by the font-loading path. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cffd70` | `PagedCacheSetValue` |  | Adds a resolved glyph-cache value to the paged lookup table. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cffdd0` | `ComputeFixedPointSpan` |  | Computes a font-layout span in the caller's fixed-point coordinate system. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00cffe70` | `CreateGlyphTextureEntry` |  | Creates and registers one bounded glyph texture entry. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d00000` | `CreateGlyphTextureEntry` |  | Creates a cache record for a source texture's bounded dimensions. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d00470` | `MeasureWideTextWidth` |  | Measures a wide string using cached font glyph metrics. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d00eb0` | `RegisterFontTextureCache` |  | Registers a font texture cache and returns its zero-based owner-array index. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d012c0` | `PrepareWideBufferThenMeasure` |  | Provides a stack-buffered path into the adjacent wide-text measurement routine; the exact formatting inputs depend on the called helper. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d01380` | `TruncateWideStringToPixelWidth → SplitWideStringAtPixelWidth` |  | Applies a pixel-width limit to a wide string using cached glyph metrics. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d04230` | `ReleaseTwoRefcounted` |  | Drops the two references held by the glyph cache owner on teardown; reached from 00d023c0. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04280` | `LockFillAlphaRows` |  | Copies one glyph coverage block into a locked texture; reached only from 00d04320. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04540` | `GlyphTexturePairCreate` |  | Creates the two GPU textures used by the glyph cache owner built in 00d025d0. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## 🟡 speculative (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00452537` | `ReleaseField2C ?` |  | Mechanically releases the interface in +0x2C if non-null, clears that field, then performs a redundant null check. Font-object teardown attribution is unsupported by this body; there are no direct callers or class ancho… | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452b5d` | `ScalarDeletingDtorFontObj_b ? ⚠rename` |  | MSVC scalar deleting destructor for a second font-object class whose cleanup is FUN_0052f673. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00454b47` | `GfcFontSizeResolveStub2 ?` |  | Wrapper around the size resolver 004549aa; its parameter meanings come from that callee. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `00458f3d` | `GfcSystemFontManagerClearTree ? ⚠rename` |  | Destruction of the font cache contents in the GFC font manager (caller 00458ffa and an EH funclet). | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0048a98b` | `FontKernString ? ⚠rename` |  | Character-pair accumulation routine. Its initial values, mode-selected output lane and virtual pair callback are direct code behavior; the semantic meaning of param_5 and the accumulators is not established here. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00491713` | `InitSentinelStateFields ?` |  | Mechanical sentinel initializer with no recorded caller. Its fields match some reads in glyph-cache accessors 004919ad and 00491824, so a glyph-cache association is a hypothesis, not a local class identification. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00d017d0` | `ReadIndexedEntryAndFormat ? ⚠rename` |  | Formats a selected nonzero indexed entry through the stack-buffered text helper. | [[Native Map/Shards/S0426 00cfc4e0-00d01c70|S0426]] |
| `00d025d0` | `CtorHashSubobjectTexturePair ?` |  | Builds a font-side cache owner whose hash map sits at +0x20c; the owning class and the texture-cache role are not proven. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04400` | `SubmitSizedQuadIfEnabled ?` |  | Submits one sized draw item when the enable flag is set; the owning object is not proven. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04490` | `SubmitFixedQuadIfEnabled ?` |  | Submits one fixed-size draw item when the enable flag is set; the owning object is not proven. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d04600` | `Wrapper_00d04600 ?` |  | Bare forwarding wrapper used by the bitmap row allocator 00d04650. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

## ⚪ unknown (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00450865` | `— ⚠reject` |  | Maps internal font descriptor flags to the font device's style/quality flag word (used by the system-font creation path at 0045471b). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004508fb` | `— ⚠reject` |  | First destruction step of a GInterface-derived class (three of its four callers are action-4 destructor paths; the 00458ab5 caller sits inside the font-file enumeration loop). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004511f7` | `— ⚠reject` |  | Secondary-base construction step for the font-device object whose second vtable is 017642E4 (the same pattern the Win32 window classes use with their own secondary vtables). | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `004513d5` | `— ⚠reject` |  | Compares wide-string key data and numeric fields. Font-map use is inferred from surrounding comparators and map callers; this body contains no font-specific anchor. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00452762` | `— ⚠reject` |  | Geometry step of the font device's glyph draw: derives the source/destination rectangles (with format-specific rounding) before the mask is copied. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `0045293c` | `— ⚠reject` |  | Format-dispatch helper called by glyph drawing routine 004535f9; this body establishes mask-writer, byte-copy/expansion, and blit branches, while the glyph role comes from its caller. | [[Native Map/Shards/S0016 0044e1d3-0045302b|S0016]] |
| `00454523` | `— ⚠reject` |  | Produces the cache key font objects are stored under; called by the font manager 0045760b/004577b6 and the font creator path. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `004549aa` | `— ⚠reject` |  | Size/DPI resolution front end for text requests, used by the two wrapper entries 00454b11/00454b47. | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `00454bed` | `— ⚠reject` |  | Likely part of the GFC font/path processing flow: its table includes the coordinate-scaling function 00454bb7 and is consumed by 006ba990. The code passes callbacks for processing; it does not register or store that tab… | [[Native Map/Shards/S0017 004530c5-00457488|S0017]] |
| `0048f992` | `— ⚠reject` |  | folded-index table lookup whose only caller is FUN_0049174B; that caller's own role is unresolved, so the earlier glyph-coverage reading is not established. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `0049174b` | `— ⚠reject` |  | row writer driven by the run list at +0x3C into the buffer param_3; which quantities the merged +8/+0xC fields and the FUN_0048f992 conversion represent is not established, so no glyph/coverage reading is asserted. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `004918fa` | `— ⚠reject` |  | Page allocation or cleanup helper used by glyph-slot accessors; cleanup runs only when the page pointer was already non-null. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491966` | `— ⚠reject` |  | page teardown path of the glyph cache. | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491a43` | `— ⚠reject` |  | glyph size query of the glyph cache (second size component). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491b81` | `— ⚠reject` |  | bitmap-handle query of the glyph cache (selects between the two stored handles). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00491c2c` | `— ⚠reject` |  | second bitmap-handle query entry point of the glyph cache (used where an additional argument is supplied by the caller). | [[Native Map/Shards/S0029 0048c619-00491c2c|S0029]] |
| `00d023c0` | `— ⚠reject` |  | Destroys the font-side cache owner: its pointer array, the referenced block, the FontFace array and the hash map base; reached from the cache release callers 007913a0, 00791d50 and 00795700. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |
| `00d02d30` | `— ⚠reject` | Deep::FontPrimitive | Constructs a Deep font primitive record. | [[Native Map/Shards/S0427 00d01e40-00d07440|S0427]] |

