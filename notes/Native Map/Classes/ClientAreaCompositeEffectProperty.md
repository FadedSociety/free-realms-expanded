# Class: ClientAreaCompositeEffectProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cebe50` | `ClientAreaCompositeEffectProperty::WriteFields` | ✅ | virtual | Writes the base property values, effect definition ID, location offset, rotation, scale, floor tolerance, and broadcast flag to definition fields. |
| `00cebf90` | `ClientAreaCompositeEffectProperty::ReadFields` | ✅ | virtual | Reads effect definition ID, parses three location offsets and three rotations with atof, reads scale and floor tolerance, and parses Broadcast as 1/t/T. |
| `00cec1a0` | `ClientAreaCompositeEffectProperty::AcquireEffectReferences` | ✅ | virtual | Looks up the effect definition, increments an area-local counter, walks typed references, resolves each referenced object, and increments each resolved object's reference count. |
| `00cecaf0` | `ClientAreaCompositeEffectProperty::Release` | ✅ | virtual | Releases typed effect references, visits remaining registered objects and invokes their virtual cleanup when gates allow, then clears the reference handle and enabled bytes. |
| `00cecbc0` | `StoreAndDispatchVector4At50` | ✅ | virtual | Copies four input dwords to object offsets 0x50 through 0x5c, then recomputes and dispatches the composite-effect transform. |
| `00cecbf0` | `StoreAndDispatchVector4At60` | ✅ | virtual | Copies four input dwords to object offsets 0x60 through 0x6c, then recomputes and dispatches the composite-effect transform. |
| `00cecc20` | `SetScaleAboveThresholdAndDispatch` | ✅ | virtual | Stores scale at object offset 0x44 and recomputes the transform only when the input exceeds DAT_01762a30. |
| `00cece40` | `ClientAreaCompositeEffectProperty::Update` | ✅ | virtual | When active and not already updated, checks typed resource readiness, resolves the effect location, queues the local composite effect and recomputes transforms; when Broadcast is set, builds a PacketUpdateClientAreaComp… |
