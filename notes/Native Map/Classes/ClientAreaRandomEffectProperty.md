# Class: ClientAreaRandomEffectProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bee090` | `ClientAreaRandomEffectProperty::CopyFrom` | ✅ | virtual | Copies the incoming property's string through the virtual string getter, compares it with the local key, and copies scalar fields when keys match. |
| `00ced1f0` | `ClientAreaRandomEffectProperty::ResetAreaCounter ? ⚠rename` | 🟡 | virtual | Writes constant 100 to the owning object's field at offset 0x1b4. |
| `00ced210` | `ClientAreaRandomEffectProperty::UpdateTransform` | ✅ | virtual | When the area is active, copies four transform words from the owner into fields 0x70-0x7c and calls the effect-node motion/transform update routine. |
| `00ced260` | `StoreDwordAt54 → StoreFloatAt54` | 🟢 | method | Stores the supplied 32-bit value at object offset 0x54. |
| `00ced270` | `StoreDwordAt58` | 🟢 | method | Stores the supplied 32-bit value at object offset 0x58. |
| `00ced280` | `StoreDwordAt5C → StoreFloatAt5C` | 🟢 | method | Stores the supplied 32-bit value at object offset 0x5c. |
| `00ced290` | `StoreDwordAt60 → StoreFloatAt60` | 🟢 | method | Stores the supplied 32-bit value at object offset 0x60. |
| `00ced2a0` | `StoreByteAt64` | 🟢 | method | Stores the supplied byte at object offset 0x64. |
| `00ced2b0` | `StoreByteAt65` | 🟢 | method | Stores the supplied byte at object offset 0x65. |
| `00ced2c0` | `StoreByteAt66` | 🟢 | method | Stores the supplied byte at object offset 0x66. |
| `00ced2d0` | `StoreInvertedByteAt67` | 🟢 | method | Stores the logical inverse of the supplied byte as a boolean at object offset 0x67. |
| `00ced330` | `ClientAreaRandomEffectProperty::WriteFields` | ✅ | virtual | Writes the base property values, effect ID, distance and frequency bounds, camera/ground/random-Y/area-relative flags, nondefault rotations, and nondefault scale. |
| `00ced510` | `ClientAreaRandomEffectProperty::ReadFields` | ✅ | virtual | Reads effect ID, minimum/maximum distance and frequency, placement flags and rotations; reads scale and passes it to a virtual setter, then invokes another property helper. |
