# System: engine.math

121 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (36)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0045a4b5` | `FastInvSqrt` |  | Quake-style reciprocal square root used by the draw maths of this translation unit. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `0045ab28` | `BuildOrthoMatrix2D` |  | 2-D projection matrix builder used by the draw layer; no caller is recorded in the export. | [[Native Map/Shards/S0018 004574f6-0045cca3|S0018]] |
| `00885b70` | `RandomLerpVec4` |  | Picks a random four-component value between two bounds with the same LCG as SizeRangeController::FillRandomRange (00884c90). | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00885bd0` | `InitLinearVec4Coeffs` |  | Sets up a linear four-component segment in the coefficient layout read by EvalCubicVec4 (00885b10): constant term, linear term, zero quadratic and cubic terms. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00885c60` | `Vec4MulAssign` |  | In-place componentwise multiply of a four-float vector. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886320` | `CacheVec4CubicBlock` |  | Caches a four-component linear or cubic segment (16 floats) for the owner's color table, rebuilt only when the flag is clear. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886b30` | `Vec4Lerp` |  | Componentwise linear interpolation of four-float vectors. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886b70` | `Vec4AddScalar` |  | Adds one scalar to each component of a four-float vector. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886ba0` | `SafeNormalizeVec4` |  | Normalises a four-float vector with rsqrt and zeroes non-finite lanes. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886bf0` | `RandomDirectionVec4` |  | Samples a random unit direction (sphere point) from the same LCG as SizeRangeController::FillRandomRange. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886f90` | `Vec4Sin` |  | Componentwise sine of a four-float vector. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00887900` | `EvalCubicTimesSin4` |  | Evaluates a four-component cubic segment at t and modulates each component by a sine of a scaled time plus a per-component phase. | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `008a9650` | `BlendIfLess4` |  | Vectorised select helper: copies mask where a<b, else zero; reached through a pointer table since no direct caller exists. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008b2e10` | `ComputeOrientationAngles` |  | Angle/orientation builder used by callers in the world code (008efe00, 008f0250, 00aab3b0, 00ab1e30): it converts two-component directions into a clamped three-angle orientation inside a pitch window. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |
| `008ba730` | `AddVector4AndApplyGlobals` |  | The vector addition and helper calls are visible; the inputs' domain meaning is unidentified. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008cc9c0` | `SphereIntersectsAabb` |  | Bounding-volume proximity test used when deciding whether an object touches a box volume (the export shows no callers, so the exact using system is unproven). | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `009588d0` | `ShuffleFloatVectorC9` |  | Fixed lane permutation decoded from the SHUFPS 0xc9 instruction. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009611d0` | `ComposeTransformFromContext` |  | A transform-composition path with a temporary projectile-parameter record; the caller and final operation do not establish a projectile gameplay role. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00961810` | `ComposeTransformMatrix` |  | Shared 4x4 transform-composition helper used by the two seat-transform accessors. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00961d40` | `BlendTransformRotationAndTranslation` |  | Shared transform utility used by the seat-attachment updater. | [[Native Map/Shards/S0256 009606e0-00965340|S0256]] |
| `00985a10` | `SetupFloat4PositionBlend` |  | Stores position and transform data, computes a float4 correction, and conditionally calls transform helpers. Its association with ControllerDressingRoom follows the related constructor 009858d0 and update method 00985ed… | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00985d70` | `InterpolateFloat4Sine` |  | Sinusoidal (spherical-type) blend of two orientations used by the dressing-room camera update 00985ed0 for the 0x10x/0x12x float4s; lane 1 is blended linearly. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `00a4fbb0` | `Vec2Subtract` |  | Two-lane float subtraction helper used by the ship-combat geometry code (caller 00a529b0 composes dot products with it). | [[Native Map/Shards/S0298 00a4b480-00a53380|S0298]] |
| `00a55b50` | `NormalizeVector2` |  | Transforms a two-component vector using its normalized scalar length; exact intended convention is uncertain. | [[Native Map/Shards/S0299 00a533a0-00a59140|S0299]] |
| `00a6f450` | `Quaternion_Slerp4` |  | Quaternion slerp used by the cinematic update and by the controller input path. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00aaaae0` | `AngleBetweenProjectedQuaternions` |  | Rotation-angle helper for the housing transform path: its only caller (00aad760) uses the result to build a rotation matrix limited by an angle parameter. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00ae8880` | `DivideVector4ByScalar` |  | Divides each component of a four-float vector by one scalar. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae9940` | `CopyQuaternionOrDefault` |  | Copies an object's quaternion-like value or the default constant. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00b4ad90` | `SubScalarFromVec4` |  | Vector4 arithmetic helper (no callers recorded, so it is reached through a function pointer or table). | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b52d40` | `BuildInterpolationSegment` |  | Closed-form builder for the quadratic position track that EvalInterpolationSegment (00b54040) later samples; used by the pet/movement code through callers 00b564d0, 00bf2bb0 and 00bf2e40. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b56130` | `WriteQuadraticVectorSamples` |  | Builds sampled points for the interpolation/render path called by 00b564d0. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00b76260` | `SegmentHitTest2D` |  | Call-site shim for the segment intersection test used by the soccer ball/effect code. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00bfc1c0` | `DistanceFromPointToSegment` |  | The seek-controller update calls this geometry helper while comparing intermediate and target positions. | [[Native Map/Shards/S0381 00bf9430-00bfcf30|S0381]] |
| `00c9aa00` | `VectorLength3` |  | Computes a three-dimensional vector magnitude. | [[Native Map/Shards/S0407 00c955b0-00c9baf0|S0407]] |
| `00cc1100` | `StepScalarTowardTarget` |  | Moves a scalar toward its target by a bounded step, with step sign selecting direction handling. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc43e0` | `ScaleSumsOfFourFloatComponents` |  | Computes the component-wise midpoint of two four-float vectors when the referenced constant is one half. | [[Native Map/Shards/S0415 00cc1a00-00cc6bc0|S0415]] |

## 🟡 speculative (64)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00705ac0` | `Copy16Bytes ?` |  | Copies four consecutive 32-bit words from the source pointer to the destination pointer. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00705ae0` | `AddVector4 ?` |  | Adds each of four source floats to the matching destination float and stores the four results. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00705c10` | `CopyEightBytes ?` |  | Copies two consecutive 32-bit words from source storage to destination storage. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007070d0` | `Copy16ByteRecordDuplicate ?` |  | Copies four consecutive 32-bit words from the source record to the destination record. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `007070f0` | `AddVector4Duplicate ?` |  | Adds corresponding components from two four-float vectors and writes the results into the output vector. | [[Native Map/Shards/S0143 00701a20-00707400|S0143]] |
| `00805b10` | `Vec4LerpThisToOut ?` |  | Thiscall with ret 4 and a hidden output pointer: out = from + (to - from) * t, where from = [ecx..ecx+0xF], to = [ecx+0x10..+0x1F], t = float at [ecx+0x20]. Writes four floats to out and returns out in EAX. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00805b40` | `Vec4SlerpThisToOut ?` |  | Thiscall with a hidden output pointer: computes theta = FUN_00d842c0(), and if DAT_017ebf0c is below theta it writes (sin((1-t)*theta)*from + sin(t*theta)*to) / sin(theta) (divps) into out; otherwise it copies to into o… | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00805cd0` | `Vec4LerpCdecl ?` |  | Cdecl: out at [ebp+8], a at [ebp+0xc], b at [ebp+0x10], t float at [ebp+0x14]. Writes out = a + (b - a) * t for four floats with SSE (subps, mulps, addps), returns out in EAX. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00809930` | `ScaleVec4OptionalClamp ?` |  | Thiscall: reads a float scalar at this[0] and a byte flag at this+4; writes out = scalar * in (componentwise, in = param_3) into the vector at *param_2. If the flag is nonzero the result is clamped with minps against th… | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00809980` | `Vec4SplatAndConstPair ?` |  | Thiscall with one float argument, ret 4 (bytes 8b c1 0f c6 c0 00 0f 29 00 ... 0f 29 40 10 c2 04 00): stores the argument splatted into this[0..3] and the float at DAT_01762a30 splatted into this[4..7]. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008099b0` | `Vec4MulAccumAndCopy ?` |  | Thiscall with two stack arguments (out pointer, input pointer), ret 8 (bytes 0f 28 0a ... 0f 59 c1 ... 0f 58 c1 ... c2 08 00): adds componentwise this[0..3] * in to this[4..7], then copies in to out. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008099e0` | `Vec4MaxSplatCdecl ?` |  | Cdecl with three stack arguments (out pointer, float scalar, input pointer), ret 0 (bytes ... 0f c6 c0 00 0f 5f c1 0f 29 00 ... c3): writes out = componentwise max(splat(scalar), in). | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00809ab0` | `Vec4ScaledMaxAccum ?` |  | Thiscall with out pointer param_2 and input param_3: builds four products of in[i] with the constants DAT_01cb54a0 and its neighbours, reduces them to their maximum with two shuffle+maxps steps, takes the max with the f… | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008830b0` | `CacheCubicCoeffs ?` |  | Thiscall (ECX = 0x14-byte cache block; flag byte at +0x10; stack arg = owner object). If the flag is zero, reads float table entries at owner+0x24 indexed by the shorts at +2, +6, +0xa and +0xe, computes four floats (a … | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00885b10` | `EvalCubicVec4 ?` |  | Thiscall, args: coefficient block (param_1, 16 floats), output (param_2, 4 floats) and parameter t (float). Writes out[k] = c0[k] + c1[k]*t + c2[k]*t*t + c3[k]*t*t*t for k in 0..3, where cK[k] is float param_1[4*K+k]. R… | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `00886010` | `BuildCubicVec4Segment ?` |  | Thiscall (ECX = 16-float output block), stack args: param_2, param_3, param_4 (float4 vectors) and param_5 (float scale). Writes the 16-float cubic block: block[0..3] = param_2; block[4..7] and block[8..11] are combinat… | [[Native Map/Shards/S0214 008830b0-00887ef0|S0214]] |
| `008bb690` | `Vector4NotEqual ?` |  | Compares four float components from param_3 and param_4 and returns true when the four-component equality mask is not 0xf. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008bb940` | `Vector4SelectLessComponents ?` |  | For each of four float components, writes the corresponding mask word from param_4 when param_2's component is less than param_3's, otherwise writes zero. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `00958850` | `SubtractScalarFromFourFloats ?` |  | Writes four output floats, each equal to scalar param_2 minus the corresponding input float. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009588b0` | `SseSqrtFourFloats ?` |  | Applies packed single-precision square root to one 128-bit input and stores the 128-bit result. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009588f0` | `ShuffleFloatVectorD2 ?` |  | Reorders four dwords so output lanes are input lanes [2,0,1,3]. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00958e60` | `SseSqrtFourFloatsVariant ?` |  | Applies packed single-precision square root to one 128-bit input and stores the result. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `009594a0` | `SseMinFourFloats ?` |  | Computes packed minimum of the two 128-bit float vectors and stores the four-lane result. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959a40` | `ShuffleFloatVectorC9Variant ?` |  | Reorders lanes in a 128-bit float vector using shuffle immediate 0xc9 and stores the vector. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `00959a60` | `ShuffleFloatVectorD2Variant ?` |  | Reorders four dwords so output words are selected as input [2,0,1,3]. | [[Native Map/Shards/S0254 00954ec0-0095a710|S0254]] |
| `0095ae30` | `Equal4Floats ?` |  | Four-component equality predicate. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095b880` | `MaxFourFloatLanes ?` |  | Broadcasts param_3 into four float lanes, computes maxps with the four lanes at param_2, and writes the result to param_1. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095b8b0` | `MinFourFloatLanes ?` |  | Broadcasts param_2 into four float lanes, computes minps with the four lanes at param_3, and writes the result to param_1. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095b970` | `MaskFloatLanesLessThan ?` |  | For each of four float lanes, writes the corresponding param_4 lane when param_2 is less than param_3, otherwise writes zero. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095b9a0` | `LerpFourFloats ?` |  | Writes four component-wise interpolations from param_2 toward param_3 using the scalar factor param_4. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095c1f0` | `SubtractFourFloats ?` |  | Subtracts four corresponding float components from the vector at param_1 using values at param_2 and writes the result in place. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `0095dcf0` | `MatrixToQuaternion ?` |  | Converts a rotation matrix to quaternion form. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `009859f0` | `AddFloat4 ? ⚠downgrade` |  | Four-component vector addition used by the dressing-room math path; it has no callers in the database, so it is most likely inlined at its use sites. | [[Native Map/Shards/S0263 00982f10-009897a0|S0263]] |
| `009c9eb0` | `AddScalarToFourFloats ?` |  | Adds the supplied float to each of the four consecutive float values at the destination pointer. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cb360` | `FloatLessThanPointedValue ?` |  | Returns whether the first float argument is less than the float pointed to by the second argument. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cb510` | `DivideFourFloatVectors ?` |  | Replicates one scalar into four lanes, divides the lanes by the supplied four-float vector, and stores the result. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009cb850` | `SumLinkedVector4SegmentLengths ?` |  | Returns zero when count is below two; otherwise follows the node link at float index +4 and sums Euclidean distances across four float components. | [[Native Map/Shards/S0275 009c8e90-009cdee0|S0275]] |
| `009ce360` | `SumAdjacentFourDwordDistances ?` |  | Returns zero for fewer than two entries; otherwise walks linked records and sums adjacent four-component Euclidean distances. | [[Native Map/Shards/S0276 009cdf80-009d3fe0|S0276]] |
| `00a6f6a0` | `Vec4_BlendTransformed_5420 ?` |  | Same as 00a6f600 with the source vector taken from DAT_01cb5420: transform through 00770e70 and write the weighted sum of the four components to param_2. | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00aa6180` | `FloatNotEqual ?` |  | Returns true when the float at param_1 differs from param_2. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa61b0` | `Vec3DivideComponentwise ?` |  | Three-component componentwise floating-point division. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa6200` | `Vec4MaskWhenLessEqual ?` |  | Four-lane comparison mask/select utility. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa8f20` | `Vec3DivideByScalar ?` |  | Three-component vector divided by one scalar. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00aa8fa0` | `Vec4MaskWhenLessEqual ?` |  | Four-lane comparison mask/select utility. | [[Native Map/Shards/S0314 00aa2560-00aa90c0|S0314]] |
| `00ae88a0` | `TransformVector4ByConstantMatrixA ? ⚠rename` |  | Applies one constant-derived transform to a Vector4. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00ae8930` | `TransformVector4ByConstantMatrixB ? ⚠rename` |  | Applies a second constant-derived transform to a Vector4. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |
| `00b001e0` | `SpringStepPair ?` |  | Non-member with two float* args and two float args; reads x=*param_1 and v=*param_2, computes v' = x*k + v (k = param_3) and x' = v'*dt + x (dt = param_4), multiplies by the float10 result of 00d84680() and stores x' in… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b00290` | `MoveTowardClamped ?` |  | Non-member with float* a (current), float* b (target), float speed and float dt. Computes diff = *b - *a. If \|diff\| is at most DAT_0181c4a8 it returns *b; if \|diff\| is below speed*dt it returns a + diff (reaches the… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b005c0` | `LinearFitSlopeIntercept ?` |  | Non-member: writes slope to *param_1 and intercept to *param_2 from the base point param_3 and the point list at param_4 (+4 data pointer of 4-float points, +8 count). Accumulates over the points the distances d from th… | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b4b280` | `ScaleVec4InPlace ? ⚠downgrade` |  | Vector scale helper used by the rendering/parameter code; no callers are recorded, so it is reached through a table or inlined sibling. | [[Native Map/Shards/S0344 00b44fa0-00b4b360|S0344]] |
| `00b4b770` | `Unpack32ToFloat4Curve ?` |  | Cdecl (param_1 at [ebp+8] is a float output array, param_2 at [ebp+0xc] a dword). Takes the four bytes of param_2, scales each by a constant (DAT_017f1ae4), and writes four floats: channel 0 as c*c*K + c*K' and channels… | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |
| `00b4b9b0` | `MatrixStoreAndCompose ?` |  | Thiscall (ECX = object; one stack pointer to a 16-float matrix). Copies the 16 floats of the argument to this+0x40..0x7c, copies 16 dwords from the pointer returned by 00739fc0(local_50, arg) to this+0..0x3c, computes a… | [[Native Map/Shards/S0345 00b4b3e0-00b50190|S0345]] |
| `00b54040` | `EvalInterpolationSegment ?` |  | Sampler paired with BuildInterpolationSegment (00b52d40) for the pet/movement interpolation used by callers 00bf2bb0 and 00bf2e40. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00be8f80` | `FloatEqualsFirst ?` |  | Loads the float at *param_1 (movaps of the 16 bytes at that address, low float used) and compares it with the float argument via comiss, returning 1 when they are equal, else 0. | [[Native Map/Shards/S0376 00be5b30-00bea020|S0376]] |
| `00cc1170` | `LerpScalarClamped ?` |  | Returns the second endpoint when the parameter reaches the upper threshold, the first endpoint when it is nonpositive, and otherwise returns linear interpolation. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc11b0` | `LerpScalarWithSignedDirection ?` |  | Uses one endpoint pair for a nonnegative parameter and the alternate pair for a negative parameter after converting its sign with the stored negative-zero constant; clamps at the endpoints and linearly interpolates insi… | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc1260` | `LerpVector3 ?` |  | Copies one endpoint vector when the interpolation parameter is outside the open unit interval; otherwise linearly interpolates three components and writes zero to output component four. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc16f0` | `AxisAngleToQuaternion ?` |  | Multiplies the first three input components by sin(angle times the stored conversion constant), then writes cos(angle times that constant) to output component four. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc1770` | `QuaternionMultiply ?` |  | Computes the three vector components and scalar component of the Hamilton product of two four-component quaternion inputs. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc18b0` | `AxisAngleToQuaternion ?` |  | Multiplies the first three input components by sin(angle times the stored conversion constant), then writes cos(angle times that constant) to output component four. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc6fd0` | `TransformPointWithVerticalOffset ?` |  | For non-null point and output pointers, passes point coordinates and a value derived from this+0x18 and a nested transform to FUN_007921a0, then copies four output words. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cd1730` | `Vec4MinScalar ?` |  | Cdecl. Stores to arg1 the component-wise minimum of the 4-float vector at arg2 and the float arg3 broadcast to all four lanes (movaps, shufps, minps, movaps). Returns void. | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd5350` | `ComposeMatricesAndQueryPosition ?` |  | Thiscall with arg2 (object pointer) and arg3 (float out[16]). Builds two 4x4 matrices through 00770e70 from the dwords at this+0x70 and this+0x74, multiplies them into a 16-float result copied to arg3[0..15], then write… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cd90c0` | `LerpFourFloatValues ?` |  | Computes four componentwise linear interpolations between two four-float values using the supplied weight. | [[Native Map/Shards/S0419 00cd5e90-00cdb1a0|S0419]] |

## ⚪ unknown (21)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00809a10` | `— ⚠reject` |  | Cdecl-style (returns its pointer argument): calls 00d84680 four times, each returning a float10 result, and stores the four results as floats into param_1[0..3]; returns param_1. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008b0e20` | `— ⚠reject` |  | Clamped integer difference used by timing/counting code. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008bf840` | `—` |  | Computes two 4D endpoints by subtracting and adding each of the first four input components scaled by input component 4, and stores both endpoints in the output. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c0390` | `—` |  | Returns true when the squared Euclidean distance between the first four float components is less than the square of the sum of the two fifth components. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c1e60` | `—` |  | Computes squared distance from a 4D point to the bounds formed by input components 0 through 7, then compares that distance with component 4 squared. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `008c4620` | `—` |  | Returns the four-component angle computed with fpatan from the negated second component and square root of scaled component squares. | [[Native Map/Shards/S0224 008bcf60-008c5290|S0224]] |
| `0095dc40` | `— ⚠reject` |  | Computes a dot-product projection of param_1 onto param_2, clamps the projection factor at zero, and returns the four-dimensional distance from param_1 to the projected point. | [[Native Map/Shards/S0255 0095a840-00960520|S0255]] |
| `00a6f600` | `— ⚠reject` |  | Initialises a vector from the data at DAT_01cb5430, transforms it through 00770e70 with the matrix argument param_2, and writes a weighted sum of the four transformed components (weights from DAT_01cb54c0, DAT_01cb54c4,… | [[Native Map/Shards/S0304 00a6c630-00a70790|S0304]] |
| `00aad760` | `— ⚠reject` |  | Rotation composition for the housing placement transform: its only caller 00ab1e30 is the shared fixture transform function referenced by the piece-027 note (15-degree rotation step, direction per action id). | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00afcc20` | `— ⚠reject` |  | Thiscall with one float* arg (ret 4). Calls 00790d00 and 00790d10 with ECX=this+0x70, then writes param_2[0] = float(this+0x50) / float(result of 00790d00) and param_2[1] = float(this+0x54) / float(result of 00790d10). | [[Native Map/Shards/S0331 00afb7a0-00b005c0|S0331]] |
| `00b0b060` | `— ⚠reject` |  | Computes squared differences across five float components and compares their sum with the first vector's fifth component squared. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b0b180` | `—` |  | Builds four SIMD comparisons for a 4D point against lower and upper bounds, then returns whether all lanes pass. | [[Native Map/Shards/S0334 00b0a370-00b10840|S0334]] |
| `00b758d0` | `— ⚠reject` |  | Line-of-sight/segment hit test used by the soccer ball and effect code (callers FUN_00b76260, FUN_00b7c6b0, FUN_00b7d210). | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |
| `00cb9450` | `— ⚠reject` |  | Thiscall (ECX = receiver p, param_2 = source object s): copies 8 floats from s+0x29c+0xc0 (or from the default produced by 00770410 when s+0x29c is zero) into p+0x80..0x9c; normalises three 4-float vectors at s+0x160, s… | [[Native Map/Shards/S0413 00cb6d20-00cbc7f0|S0413]] |
| `00cc1300` | `— ⚠reject` |  | Interpolates a scalar using the reciprocal of the duration returned by FUN_00d84680. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc1390` | `— ⚠reject` |  | Interpolates a three-component vector using the duration returned by FUN_00d84680. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cc18f0` | `— ⚠reject` |  | Rotates the input vector by quaternion conjugation and multiplication. | [[Native Map/Shards/S0414 00cbc8e0-00cc18f0|S0414]] |
| `00cd5550` | `— ⚠reject` |  | Fastcall (ECX = this). Copies a 16-float matrix from 00fe2700 into a local. When the byte at +0x78 is zero it resolves the handle pair at +0x28 and +0x2c (00769710, 00984960), composes that object's transform through 00… | [[Native Map/Shards/S0418 00cd1210-00cd5dd0|S0418]] |
| `00cf3570` | `—` |  | Returns whether the float at the second argument is strictly less than the first argument. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf45d0` | `— ⚠reject` |  | Advances one bounded interpolated value with randomized speed. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |
| `00cf5480` | `— ⚠reject` |  | Advances one of two bounded randomized values used by the adjacent runtime update. | [[Native Map/Shards/S0424 00cf1430-00cf68d0|S0424]] |

