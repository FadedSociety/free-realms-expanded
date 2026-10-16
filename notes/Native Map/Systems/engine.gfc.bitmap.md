# System: engine.gfc.bitmap

48 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00482804` | `RegisterGTiledBitmapFontClass` |  | Static class-registration initializer for the GFC class GTiledBitmapFont (the tiled bitmap font class whose name string sits next to its vtable at 017695E8). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |

## 🟢 likely (19)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00486164` | `CtorVtables1769BA0` |  | Initializes the object with the GObjectBase constructor, installs vtable block 01769BA0/01769B98, and zeroes eight dwords at +0x38 through +0x58. The owning class name is unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004861d6` | `CtorVtables1769BA0_FromSize` |  | Constructor variant with the same vtable block as 00486164; after member initialization it calls the image creator 0047CA5E with format and dimensions. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `0048621e` | `CtorVtables1769BA0_FromPair` |  | Constructor variant with the same vtable block as 00486164; forwards a two-dword size pair to image creator 0047CA5E. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486328` | `DtorVtables1769BA0` |  | Destructor for the object initialized by constructors 00486164, 004861D6, 0048621E, 00486267, 004862A7, and 004862E7; the class name is unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486c7e` | `RangedValuesAllEqual` |  | Uniformity test over a run of frames/ids used by the 004A6856/004A68A8/004B4344/004B464E code paths. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `0048741e` | `ConvertAndStorePixel` |  | Pixel conversion/store routine. It does not dereference param_5 as a pointer. Its 0x211555 behavior differs from 004877f0, which packs RGB without copying source bit 31 into bit 15; the three routines should not be trea… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00487b20` | `ConvertAndStorePixelWithFastPaths` |  | Pixel conversion/store routine with fused branches for 0x211555, 0x214444 and 0x618888 before its general packing path. Its shared format dispatch does not make its 0x211555 behavior equivalent to 004877f0. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `004882dd` | `BitmapGetPixel` |  | Reads a raw pixel value using the descriptor fields and format tags. It extracts the stored value without converting it to a canonical colour. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `004884a4` | `BitmapSetPixelRaw` |  | Stores a caller-supplied value in the selected pixel format without performing the colour conversion handled by 00488686. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00488686` | `BitmapSetPixel` |  | The plain 'set a pixel' entry of the bitmap class, combining address computation with colour conversion; its single recorded caller FUN_0049e240 (outside this shard) is the only connection to higher-level drawing code i… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00488a1f` | `BitmapSetPixelWithAlphaPlane` |  | Writes the supplied colour to a pixel and, when flag 0x800000 is set, writes its high byte to the separate byte plane addressed by the code. The identity of descriptor fields used as that plane base remains unresolved. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00488ff6` | `BitmapSetPixelAlpha` |  | Writes alpha-related data for the selected pixel format. The 0x214444 branch only masks the stored word with 0xfff; this function does not establish why that branch omits param_4. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048b14e` | `PcxHeaderRead` |  | PCX image header deserialiser: the field sizes (4 bytes, 6 words, 48-byte EGA palette, 2 bytes, 4 words, 54-byte filler) reproduce the 128-byte PCX header exactly, which is what identifies the format. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048b287` | `PcxHeaderWrite` |  | PCX header writer paired with 0048b14e by the matching 128-byte field layout. This function does not establish which higher-level image class consumes the header. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048bb24` | `PpmHeaderWrite` |  | PPM header writer. The max-value field is written as decimal text, not as one raw byte; initial magic/newline write results are ignored. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048c220` | `PackPixelToDepth` |  | Packs a colour value into a buffer for supported bit depths. The helper is called by the TGA table-entry writer in this shard; a relation to PCX or BMP writers is not established here. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048c2de` | `TgaHeaderRead` |  | Reads the 18-byte TGA header layout and skips the ID field only after the final descriptor-byte read succeeds. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048c3d3` | `TgaHeaderWrite` |  | TGA header writer paired with 0048c2de. The recorded save path is 0048c8ca; its optional color-map helper is 0048c4f5, while image-data handling is a separate call to 0048c619. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048c4f5` | `TgaWriteColorMapEntries` |  | Likely TGA colour-map entry writer: the TGA role comes from its only recorded caller 0048c8ca and the header offsets used there. That caller initializes colour-map type to zero, so its conditional call is skipped on the… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |

## 🟡 speculative (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00486267` | `CtorVtables1769BA0_FromArg ? ⚠rename` |  | Constructor variant with the same vtable block as 00486164; after member initialization it calls image loader 0047CC29. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004862a7` | `CtorVtables1769BA0_FromHandle ? ⚠rename` |  | Constructor variant with the same vtable block as 00486164; after member initialization it calls 0047CA0C to bind an image from a drawable handle. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004862e7` | `CtorVtables1769BA0_FromSource ? ⚠downgrade` |  | Constructor variant with the same vtable block as 00486164; after member initialization it calls 004D9A26 with the supplied source object. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486714` | `GetRectPairFromField44_0C ? ⚠downgrade` |  | Accessor returning a two-dword rectangle corner (the +0xC pair of the bitmap header record at +0x44); used as a vtable slot by the classes whose tables contain 00486714/00486741 (e.g. 01764E90, 01769CF0, 0176B470). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486741` | `GetRectPairFromField44_04 ? ⚠downgrade` |  | Accessor returning the other two-dword rectangle corner (+0x4 pair) of the bitmap header record at +0x44; called from 008645C0. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004868d2` | `GetTotalFrameDataSize ? ⚠downgrade` |  | Size computation for the frame-sequence class, used when allocating or persisting the frame data. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `0048695f` | `FindFrameIndex ? ⚠downgrade` |  | Key-based frame lookup of the frame-sequence class (vtable slot 0176B514/0176B75C). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004869de` | `NextFrame ? ⚠downgrade` |  | Frame advance used by the animated bitmap/frame-sequence play loop (vtable slot 0176B520/0176B768). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486a4f` | `SetFrameImage ? ⚠downgrade` |  | Image assignment for one frame of the frame-sequence class (vtable slot 0176B534). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00d297c0` | `ResampleAreaBilinearRgba8Keyed ?` |  | Resamples a 4-byte-per-pixel image read from param_2 (row stride *(this+0x10)) into bytes written from param_3+1 (first byte *param_3 set to 0). Sample coordinates use ceil() via 00d843f0. Source pixels equal to the col… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2bb60` | `ResampleAreaBilinearRgba8 ?` |  | Same resampling shape as 00d297c0 without any colour-key test: every 4-byte source pixel contributes to the channel sums, interior windows are divided by width*height (fVar30 product), edge samples use two-tap linear we… | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |
| `00d2d960` | `SelectResampleRoutine ?` |  | Takes an int* descriptor and returns (EAX) one of twelve code addresses. It tests descriptor[2]==3, the byte flag at descriptor[0x12], the key word at descriptor[0x11] (zero or not), and whether descriptor[0],[1] equal … | [[Native Map/Shards/S0437 00d297c0-00d2f870|S0437]] |

## ⚪ unknown (16)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00486915` | `— ⚠reject` |  | Frame selection of the bitmap/frame-sequence class (vtable slot 0176B518/0176B760); it switches the active image to the selected frame. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486a14` | `— ⚠reject` |  | Frame rewind of the frame-sequence class (vtable slot 0176B524/0176B76C). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486abe` | `— ⚠reject` |  | Alternate key-based frame scan; on a match it invokes the object?s virtual slot +0x238. The owner and purpose of that slot are unresolved. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `00486e8c` | `— ⚠reject` |  | Pixel-to-luminance conversion of the GFC color-format family (the format word is the OR of the GFC_CLR_* flag bits seen at 01AF6A98/01AF6EF8 and the component widths 0x444/0x555/0x565/0x888/0x1555/0x4444). | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004870e5` | `— ⚠reject` |  | General pixel-format converter of the GFC image/color layer (the counterpart of the gray conversion 00486E8C); the format words and the bit-manipulation tables at 01AF6FE3/01AF6FF5 encode the GFC_CLR_* flag combinations. | [[Native Map/Shards/S0027 00482437-004870e5|S0027]] |
| `004877f0` | `— → BitmapConvertStorePixelValue` |  | Converts param_5 according to param_1 metadata, then stores the result at param_2 using the format tag in *param_1. For tag 0x210444 it performs a 0xfff masked word merge and falls through to the shared 0x7fff merge, so… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00488095` | `— ⚠reject` |  | Converts a colour value to the format selected by the descriptor at this+0x44 and returns the packed value without storing it. The relationship to the separate read/write routines is supported by matching format tags an… | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `004890e5` | `— → BitmapFillRect` |  | Loads the descriptor at this+0x44 and returns 0 if it or descriptor[9] is null. Orders the rectangle endpoints, clamps them to the descriptor bounds, and returns 0 for an empty clipped area. For descriptor[5] values 1, … | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `00489321` | `— ⚠reject` |  | Scans selected bitmap edges and stores the counts reached by the four flag-controlled walks. Whether callers interpret those values as margins or first-content coordinates is unknown. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048ac76` | `— ⚠reject` |  | BMP/DIB parsing and GDI surface-transfer path. The code does not establish a successful return: confirm whether the native routine really returns 0 after the transfer or whether Ghidra lost the success value. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048b48c` | `— → WritePlanarRleRows` |  | Transforms and writes row chunks; its caller and file-format framing are unresolved. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048ba3b` | `— ⚠reject` |  | PPM (netpbm) header parser; the 0x3350/0x3650 values are exactly the ASCII 'P3'/'P6' magics, and the seek-back positions the stream at the pixel payload for PpmDecodePixels. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048bc02` | `— → PpmDecodePixelsToSurface ⚠reject` |  | PPM pixel decoder with direct channel-byte copying. P6 short positive reads are accepted, and P3 values are truncated to bytes; maxval is not used to rescale samples. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048bd5e` | `— ⚠reject` |  | PPM pixel encoder. The P6 path emits binary RGB bytes and accepts positive partial row writes; the P3 path emits decimal channel values. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048bf0c` | `— ⚠reject` |  | Top-level PPM loading path. The staging-object initialization call 00486164 is not a lock; the loader decodes into a temporary 0x610888 surface before the destination format check and blit. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |
| `0048c04c` | `— ⚠reject` |  | Top-level P6 save path. The value 0xff is passed to the header writer as decimal maxval 255; 00486164 initializes a local object and is not a bitmap lock. | [[Native Map/Shards/S0028 0048741e-0048c4f5|S0028]] |

