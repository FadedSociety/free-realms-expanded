---
type: build-guide
section: World
updated: 2026-10-12
---

# Physics

Part of [[Build Guides]] › World. Related: [[Kart Racing]], [[Movement and Pathfinding]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

334 accepted functions: 106 confirmed, 84 likely, 91 speculative, 53 unknown. Network-linked (`wire`): 0.

Subsystems: `lib.bullet` (99), `physics.dynamics` (80), `physics.collision` (52), `physics.query` (31), `physics.shape` (31), `physics` (25), `physics.ship` (8), `physics.mesh` (8)

### `lib.bullet`

**SphereTriangleDetector** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0115bc90` | SphereTriangleDetector::getClosestPoints | confirmed | Virtual slot 1 of SphereTriangleDetector::vftable. Thiscall (ECX = detector) with stack arguments input (ClosestPointInput), output (Result*), debugDraw and sw… |
| `0115b820` | SphereTriangleDetector::Collide | likely | Thiscall with ECX = detector. Reads the sphere centre from the first stack argument, forms the triangle normal from the three 16-byte vertices of the triangle … |

**btActionInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff5d50` | btActionInterface::DtorBody | confirmed | Non-deleting destructor body (7 bytes): stores btActionInterface::vftable at [this] and returns. |
| `00ff5d60` | btActionInterface::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): stores the btActionInterface vtable at [this], tests bit 0 of the flag argument and frees this through 00d82463 when it … |

**btBoxBoxDetector** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0115ec20` | btBoxBoxDetector::ctor | confirmed | Thiscall constructor. Stores btBoxBoxDetector::vftable at this+0, then the two stack arguments at this+4 and this+8 (the box shapes that 0115ec60 reads later). |
| `0115ec60` | btBoxBoxDetector::getClosestPoints | confirmed | Virtual slot 1 of btBoxBoxDetector::vftable. Thiscall with ECX = detector, stack arguments the ClosestPointInput and the Result. Copies the input transforms, r… |

**btBoxShape** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcb110` | btBoxShape::btBoxShape | confirmed | Thiscall with one stack argument (pointer to three floats, the half extents). Calls 01113f00 (base constructor, not decoded), stores the btBoxShape vftable, se… |
| `00fcb300` | btBoxShape::SupportWithMargin | likely | Thiscall with two stack arguments (out vector param_2, direction param_3). Reads the three implicit dimensions at this+0x20..+0x28 and adds the margin obtained… |
| `00fcb3f0` | btBoxShape::SupportWithoutMargin | likely | Thiscall with two stack arguments (out vector param_2, direction param_3). For each axis takes the implicit dimension at this+0x20..+0x28, mirrored through DAT… |
| `00fcb460` | btBoxShape::BatchedSupportWithoutMargin | likely | Thiscall with three stack arguments (direction array param_2 with 16-byte stride, output array param_3, count param_4); returns nothing. For each direction, wr… |
| `00fcb6c0` | btBoxShape::SetMarginAdjustDims | likely | Thiscall with one stack argument (new margin). Reads the current margin three times through vtable slot 10, stores the argument at this+0x30, reads the margin … |
| `00fcb7c0` | btBoxShape::SetLocalScalingRescale | likely | Thiscall with one stack argument (new scale vector pointer). Reads the margin three times through vtable slot 10, saves the old scale (this+0x10..+0x18) and th… |
| `00fcb9b0` | btBoxShape::GetPlaneByIndex | likely | Thiscall with two stack arguments (output plane pointer param_2 of four dwords, index param_3). A switch over 0..5 writes a unit normal along one axis (constan… |
| `00fcbbe0` | btBoxShape::PointInsideWithMargin | likely | Thiscall with a point pointer (param_2, three floats) and a margin float (param_3). Returns 1 when, on each axis, the point lies between (DAT_017ea5a4 minus th… |
| `00fcbca0` | btBoxShape::GetName | likely | Thiscall with no arguments; returns the address of the static data at 017ff4e0 (bytes b8 e0 f4 7f 01 c3, i.e. mov eax, 0x017ff4e0; ret). The bytes at that addr… |
| `00fcbcc0` | btBoxShape::FaceNormalByIndex | likely | Two stack arguments (face index param_1 and output vector param_2); ECX is not used. For indices 0..5 writes a unit normal along x, y or z (components from 017… |
| `00fcb8b0` | btBoxShape::AabbFromSupportAndTransform | speculative | Thiscall with three stack arguments (an output vector param_2, an output vector param_3 and an index-like argument param_4). Calls vtable slot 28 (+0x70) with … |
| `00fcbb00` | — | unknown | Thiscall with two stack arguments (edge index param_2 and a second pointer param_3). A switch over edge indices 0..11 selects two vertex indices (uVar1, uVar2)… |

**btCollisionObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fca700` | btCollisionObject::StoreArgAtCcAndD4 | likely | Thiscall with one stack argument, returns 4 bytes. Stores the argument at this+0xcc and at this+0xd4. |

**btCollisionWorld::ClosestConvexResultCallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcac20` | btCollisionWorld::ClosestConvexResultCallback::ClosestConve… | confirmed | Thiscall with two stack arguments (pointers to two 16-byte vectors, copied to this+0x10..+0x1c and this+0x20..+0x2c). Sets the ConvexResultCallback vftable, se… |
| `00fcacd0` | — | unknown | Thiscall with two stack arguments (a local convex result pointer and a flag byte), returns the hit fraction (float at arg+0x30). Stores the object pointer at t… |

**btCollisionWorld::ClosestRayResultCallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fca920` | — | unknown | Thiscall with two stack arguments (pointers to two 16-byte vectors, copied to this+0x20..+0x2c and this+0x30..+0x3c). Sets the RayResultCallback vftable, zeroe… |
| `00fcaa40` | — | unknown | Thiscall with two stack arguments (a local ray result pointer and a flag byte), returns the hit fraction (float at arg+0x20). Stores the fraction at this+4 and… |

**btCollisionWorld::ContactResultCallback** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcae00` | btCollisionWorld::ContactResultCallback::~ContactResultCall… | confirmed | Fastcall with ECX = object. Stores the btCollisionWorld::ContactResultCallback vftable at this+0 and returns (no member cleanup). |
| `00fcae10` | btCollisionWorld::ContactResultCallback::FilterGroupMaskTest | confirmed | Thiscall with one stack argument (an object pointer whose 16-bit fields at +4 and +6 are read), returns 1 in AL when (this+6 AND arg+4) and (this+4 AND arg+6) … |
| `00fcae90` | btCollisionWorld::ContactResultCallback::DeletingDestructor | confirmed | Thiscall with one stack argument (flags byte), returns 4 bytes. Stores the ContactResultCallback vftable at this; when flags bit 0 is set calls _free on this (… |

**btCollisionWorld::ConvexResultCallback** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcabc0` | btCollisionWorld::ConvexResultCallback::~ConvexResultCallba… | confirmed | Fastcall with ECX = object. Stores the btCollisionWorld::ConvexResultCallback vftable at this+0 and returns (no member cleanup). |
| `00fcabf0` | btCollisionWorld::ConvexResultCallback::FilterGroupMaskTest | confirmed | Thiscall with one stack argument (an object pointer whose 16-bit fields at +4 and +6 are read), returns 1 in AL when (this+0xa AND arg+4) and (this+8 AND arg+6… |
| `00fcae70` | btCollisionWorld::ConvexResultCallback::DeletingDestructor | confirmed | Thiscall with one stack argument (flags byte), returns 4 bytes. Stores the ConvexResultCallback vftable at this; when flags bit 0 is set calls _free on this (t… |
| `00fcaf00` | btCollisionWorld::ConvexResultCallback::~ConvexResultCallba… | confirmed | Fastcall with ECX = object, inside an SEH frame (push -1, handler, fs:[0] link, cookie). Stores the btCollisionWorld::ConvexResultCallback vftable at this+0, r… |

**btCollisionWorld::RayResultCallback** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fca8a0` | btCollisionWorld::RayResultCallback::~RayResultCallback | confirmed | Fastcall with ECX = object. Stores the btCollisionWorld::RayResultCallback vftable at this+0 and returns (no member cleanup). |
| `00fca8f0` | btCollisionWorld::RayResultCallback::FilterGroupMaskTest | confirmed | Thiscall with one stack argument (an object pointer whose 16-bit fields at +4 and +6 are read), returns 1 in AL when (this+0xe AND arg+4) and (this+0xc AND arg… |
| `00fcae50` | btCollisionWorld::RayResultCallback::DeletingDestructor | confirmed | Thiscall with one stack argument (flags byte), returns 4 bytes. Stores the RayResultCallback vftable at this; when flags bit 0 is set calls _free on this (thro… |
| `00fcaeb0` | btCollisionWorld::RayResultCallback::~RayResultCallbackSEH | confirmed | Fastcall with ECX = object, inside an SEH frame (push -1, handler, fs:[0] link, cookie). Stores the btCollisionWorld::RayResultCallback vftable at this+0, rest… |

**btConvexInternalShape** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcafe0` | btConvexInternalShape::ForwardToSlot17 | confirmed | Fastcall with ECX = object (bytes 8b 01 8b 40 44 ff e0): loads the vtable, loads its entry at +0x44 (slot 17) and tail-jumps to it with ECX unchanged, so the c… |
| `00fcaff0` | btConvexInternalShape::AddressOfField10 | confirmed | Fastcall with ECX = object (bytes 8d 41 10 c3): returns this+0x10 in EAX. |
| `00fcb000` | btConvexInternalShape::SetFloatField30 | likely | Thiscall with one float stack argument, returns 4 bytes (bytes f3 0f 10 44 24 04 f3 0f 11 41 30 c2 04 00): stores the argument at this+0x30. |
| `00fcb010` | btConvexInternalShape::GetFloatField30 | likely | Fastcall with ECX = object (bytes d9 41 30 c3): loads the float at this+0x30 onto the x87 stack and returns it. |
| `00fcb0b0` | btConvexInternalShape::SerializeToData | likely | Thiscall with two stack arguments (a data buffer pointer and a serializer pointer). Calls 01113130 with the two arguments (base serialization, not decoded), co… |
| `00fcaf60` | — | unknown | Fastcall with ECX = object, inside an SEH frame. Stores the btConvexInternalShape vftable at this+0, sets the EH state, calls 01111620 with the /GS cookie valu… |
| `00fcb040` | — | unknown | Thiscall with one stack argument (flags byte), returns this. Stores the btConvexInternalShape vftable, calls 01111620 (cookie check) and, when flags bit 0 is s… |
| `00fcbed0` | — | unknown | Fastcall with ECX = object, inside an SEH frame. Stores the btConvexInternalShape vftable at this+0, sets the EH state, calls 01111620 with the /GS cookie valu… |

**btDefaultMotionState** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe1080` | btDefaultMotionState::ctor | confirmed | Constructor (ECX=this, two pointer arguments): stores btMotionState::vftable then btDefaultMotionState::vftable at +0; copies the 16-dword transform at param_2… |
| `00fe1660` | btDefaultMotionState::SetGraphicsFromCenterOfMass | likely | Vtable slot [2]: writes the graphics transform at this+0x10..+0x4c from the input transform param_2 multiplied by the centre-of-mass offset stored at this+0x50… |
| `00fe12a0` | — | unknown | Vtable slot [1] (thiscall, output transform pointer on the stack): calls 00fe0ef0 with ECX = this+0x50 to invert the centre-of-mass offset into a stack record,… |

**btDiscreteCollisionDetectorInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0115ec40` | btDiscreteCollisionDetectorInterface::DeletingDtor | confirmed | Scalar deleting destructor shape. Stores btDiscreteCollisionDetectorInterface::vftable at this+0 and, when the flag argument bit 0 is set, calls _free(this) th… |

**btGjkConvexCast** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0115eed0` | btGjkConvexCast::ctor | confirmed | Thiscall constructor with three stack arguments (ret 0xc). Stores btGjkConvexCast::vftable at this+0, the third argument at this+4, the first at this+8 and the… |
| `0115eef0` | btGjkConvexCast::DeletingDtor | confirmed | Scalar deleting destructor shape. Calls the base destructor body 0113edc0 on this, then when flag bit 0 is set calls _free(this) through the 00d82463 thunk, an… |
| `0115ef10` | btGjkConvexCast::calcTimeOfImpact | confirmed | Virtual slot 1 of btGjkConvexCast::vftable. Thiscall with two transform pairs and the CastResult pointer. Copies both transform sets into stack buffers, builds… |

**btIDebugDraw** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcbfa0` | btIDebugDraw::~btIDebugDraw | confirmed | Fastcall with ECX = object (bytes c7 01 e4 c1 87 01 c3): stores the btIDebugDraw vftable at this+0 and returns (no member cleanup). |
| `00fcc000` | btIDebugDraw::ForwardToSlot2 | likely | Thiscall with three stack arguments (param_2..param_4), returns 0x10 bytes (c2 10 00 in the bytes). Loads the vtable of the object, calls its entry at +8 (slot… |

**btMotionState** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe05b0` | btMotionState::~btMotionState | confirmed | Non-deleting destructor body (7 bytes): stores btMotionState::vftable at +0. |
| `00fe05c0` | btMotionState::DeletingDtor | confirmed | Scalar-deleting destructor (31 bytes): stores btMotionState::vftable at +0, then calls _free(this) when flag bit 0 is set. |
| `00fe1a00` | btMotionState::DeletingDtor | confirmed | Scalar-deleting destructor body (91 bytes, SEH frame): stores btMotionState::vftable at +0 and calls _free(this) when flag bit 0 is set. Stored at btDefaultMot… |
| `00fe1250` | btMotionState::~btMotionState | likely | Non-deleting destructor body with an SEH frame (67 bytes): stores btMotionState::vftable at +0 and returns; no free. |

**btOverlappingPairCallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fca860` | btOverlappingPairCallback::~btOverlappingPairCallback | confirmed | Fastcall with ECX = object. Stores the btOverlappingPairCallback vftable at this+0 and returns (no member cleanup). |
| `00fca870` | btOverlappingPairCallback::DeletingDestructor | confirmed | Thiscall with one stack argument (flags byte), returns 4 bytes. Stores the btOverlappingPairCallback vftable at this; when flags bit 0 is set calls _free on th… |

**btSphereShape** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fcbe20` | btSphereShape::btSphereShape | confirmed | Thiscall with one stack argument (radius), returns this. Calls 01113330 (base constructor, cookie-framed, not decoded), stores the btSphereShape vftable, sets … |
| `00fcbea0` | btSphereShape::GetName | confirmed | Thiscall with no arguments; returns the address of the literal "SPHERE" (bytes b8 d4 c1 87 01 c3). |
| `00fcbeb0` | btSphereShape::SetFloatField30 | likely | Thiscall with one float stack argument, returns 4 bytes (f3 0f 10 44 24 04 f3 0f 11 41 30 c2 04 00): stores the argument at this+0x30. |
| `00fcbec0` | btSphereShape::ProductField20Field10 | likely | Fastcall with ECX = object (bytes d9 41 20 d8 49 10 c3): loads the float at this+0x20, multiplies it by the float at this+0x10 and returns the product in ST(0). |

**btStridingMeshInterface** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde060` | btStridingMeshInterface::DeletingDtor | confirmed | Calls the btStridingMeshInterface destructor 01133a00 on this, then frees this when bit 0 of the stack argument is set. |

**btTriangleIndexVertexArray** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdf780` | btTriangleIndexVertexArray::ctor | confirmed | Constructor: stores btStridingMeshInterface::vftable then btTriangleIndexVertexArray::vftable at +0, sets +0x10, +0x14, +0x18 to the float constant at DAT_0175… |
| `00fdf850` | btTriangleIndexVertexArray::~btTriangleIndexVertexArray | confirmed | Scalar-deleting destructor: calls 0113d280 (the destructor body), then frees this through 01160c00 when flag bit 0 is set (30 bytes). |
| `00fdf870` | btTriangleIndexVertexArray::GetPartCount | likely | Vtable slot [6]: returns the dword at this+0x24, which is the element count of the part array that 00fdf880 appends to (4 bytes of code). |

**btVehicleRaycaster** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe3800` | btVehicleRaycaster::~btVehicleRaycaster | confirmed | Non-deleting destructor body (7 bytes): stores btVehicleRaycaster::vftable at +0. |
| `00fe3810` | btVehicleRaycaster::DeletingDtor | confirmed | Scalar-deleting destructor (31 bytes): stores btVehicleRaycaster::vftable at +0, then calls _free(this) when flag bit 0 is set. |
| `00fe4080` | btVehicleRaycaster::DeletingDtorDefaultSlot0 | confirmed | Scalar-deleting destructor body (91 bytes, SEH frame): stores btVehicleRaycaster::vftable at +0 and calls _free(this) when flag bit 0 is set. Stored at btDefau… |
| `00fe3950` | btVehicleRaycaster::ctor | likely | Constructor body (67 bytes, SEH frame): stores btVehicleRaycaster::vftable at +0 and returns. |

**(no class)** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0115c5f0` | ClipPolygonToRect | likely | Helper that clips a 2D polygon of four input vertices (x,y float pairs at param_1) against four half-planes, sign*x < param_2[0] and sign*y < param_2[1] for si… |
| `0115cca0` | BoxBoxContacts | likely | Free function (13 stack arguments) implementing the box-box separating-axis test. It projects 15 candidate axes (3 face normals of each box and 9 edge-edge cro… |
| `0115fc50` | ClipPolygonByPlane | likely | Sutherland-Hodgman clip of the polygon at param_1 (count at +4, 16-byte vertices at +0xc) against one plane with normal param_3 and offset param_4. Output goes… |
| `011601b0` | ClipFaceAgainstRefHull | likely | Clips the polygon at param_4 (Vec4 array, copied to a temporary through 01160bf0 and freed through 01160c00 on exit) against the planes of the faces connected … |
| `01160740` | ClipHullAgainstHull | likely | Selects the face of hull param_3 with the largest dot against the normal param_1 (scan over 0x38-byte faces at +0x30, count at +0x28). Builds that face's polyg… |
| `01161190` | TriangleNormal | likely | Writes the unit normal of the triangle formed by the points param_2, param_3, param_4 (16-byte vectors): normalised cross product of (p3 - p2) and (p4 - p3), w… |
| `01161750` | PerpendicularUnitVector | likely | Writes a unit vector perpendicular to the input (param_2): the cross product of the input with the Z axis (y, -x, 0) or with the Y axis (-z, 0, x), choosing th… |
| `011618b0` | TriangleSideTest | likely | Takes the triangle record at param_2 (three indices, scaled by 16 from the base param_1), builds its normal through 01161190 and returns 1 when dot(normal, par… |
| `01161a30` | TriangleEdgeNeighbourSlot | likely | Thiscall on a three-entry index array (ECX = param_1). Looks for the edge (param_2, param_3) in either orientation among the three cyclic edges. On a match it … |
| `01161dd0` | MaskedSupportVertexIndex | likely | Scans param_2 vertices (16-byte entries at param_1) and returns in EAX the index of the one with the largest dot product with direction param_3, considering on… |
| `011625d0` | SupportSweepSearch | likely | Repeatedly takes the masked support vertex (01161dd0) along a direction; when it finds a vertex whose flag is 3 (stored in the flag array at param_4+0xc) it re… |
| `01162970` | FindSupportTetrahedron | likely | Writes four vertex indices into the int array param_1: vertex A and vertex B from support queries along two fixed directions, a third vertex from the support o… |
| `00fdf880` | ArrayAppend32Tagged | speculative | Appends one 8-dword record (copied from the argument) to the array at this+0x24 (count +0x24, capacity +0x28, buffer +0x2c, owned +0x30), growing as in 00fdf64… |
| `0115f4e0` | TestSepAxisDepth | speculative | Fastcall helper. Projects two polyhedra onto an axis through 0113d5c0 twice, once per hull, giving min and max for each. Returns 0 when the intervals do not ov… |
| `01160cc0` | ThreePlaneIntersection | speculative | Solves three plane equations with normals param_2, param_3, param_4 (xyz) and offsets at index 4 of each, by Cramer's rule: the denominator is the triple produ… |
| `01161110` | PlanePointProject | speculative | Projects the point param_3 onto the plane whose normal is the xyz of param_2 with offset param_2[4]: out = p - n * (n.p + d). The normal is not normalised here. |
| `011612a0` | LinePairClosestPoints | speculative | Computes the common normal of the directions param_2 and param_4 (normalised cross product, stored in globals 01cba6a0..01cba6a8) and the separation along it, … |
| `011623c0` | MaxKeyElementAboveThreshold | speculative | Thiscall over a list (count at +4, pointer array at +0xc) of records with a float key at +0x20. It returns the record with the largest key (ties keep the earli… |
| `01162530` | OwnedBufferFree0C | speculative | Thiscall. If the pointer at this+0xc is nonzero, frees it through 01160c00 when the owned flag at this+0x10 is set; then zeroes this+0xc. |
| `01162580` | OwnedBufferFree0C | speculative | Thiscall. Identical to 01162530: frees the pointer at this+0xc through 01160c00 when the owned flag at this+0x10 is set, then zeroes this+0xc. |
| `01162fe0` | OwnedArrayReset | speculative | Thiscall. Frees the buffer at this+0xc through 01160c00 when it is nonzero and the owned flag at this+0x10 is set; then zeroes this+0xc, this+4 and this+8 and … |
| `01163090` | OwnedArrayReset | speculative | Thiscall. Identical in body to 01162fe0: frees the buffer at this+0xc through 01160c00 when nonzero and owned, then zeroes this+0xc, this+4, this+8 and sets th… |
| `01163350` | OwnedArrayReset | speculative | Thiscall. Identical in body to 01162fe0: frees the buffer at this+0xc through 01160c00 when nonzero and owned, then zeroes this+0xc, this+4, this+8 and sets th… |
| `01163440` | OwnedArrayReset | speculative | Thiscall. Identical in body to 01162fe0: frees the buffer at this+0xc through 01160c00 when nonzero and owned, then zeroes this+0xc, this+4, this+8 and sets th… |
| `0115c4b0` | — | unknown | Helper with six pointer arguments: pa, ua, pb, ub, alpha out, beta out. Computes k = ub.ua, c = 1 - ub.(pb-pa) and e = ua.(pb-pa), and d = 1 - k*k. When d exce… |
| `0115c7c0` | — | unknown | Chooses at most param_3 contact points from the n points in param_2 (x,y float pairs; n is passed as float bits in param_1, with n==1 and n==2 special-cased). … |
| `0115f590` | — | unknown | Searches for a separating axis between two convex polyhedra. Hull records have their face count at +0x28, face and vertex data at +0x30 (56-byte face stride) a… |
| `01160fc0` | — | unknown | Writes into param_1 a point on the line through param_3 with direction d = param_4 - param_3 (stored in the globals at 01cba680..01cba68c, w = 0). The scalar i… |
| `011622e0` | — | unknown | Takes a three-index triangle record (param_1). For each of its three cyclic edges it looks up link slots through 01161a30 and copies a slot value into a slot. … |
| `01163220` | — | unknown | Thiscall with one stack argument (esi = this). When this+0xc is nonzero it zeroes this+4, releases the buffer at this+0x14 through 01160c00 when nonzero and ow… |

### `physics`

**AppPhysics::CollisionObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0076b8a0` | ReturnThisPlus40 | likely | thiscall: returns this+0x40 (lea eax,[ecx+0x40]; ret). 4 bytes. |

**WaterBody** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0076b800` | WaterBodyCtor | confirmed | thiscall with ECX = object, one stack argument for the base constructor call and a 9-argument prototype. Calls 00fdd000 with the first seven arguments and 0 (p… |
| `0076f140` | WaterBodyScalarDeletingDtor | confirmed | thiscall with a stack flag argument: calls 00fddee0 (the destructor body, the same callee as Wrapper_0076b8e0) on this, then calls _free(this) only when bit 0 … |

**(no class)** (22)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ae91d0` | ResolveConvexHitEndpoint | confirmed | Queries convex hits from the current position toward the requested endpoint, averages returned hit vectors, projects and adjusts the endpoint, retries up to te… |
| `00aa9d10` | AabbStrictOverlap | likely | Four-lane SSE predicate on two float[8] min/max boxes: for every axis i it computes (param_2[i] < param_1[i+4]) && (param_1[i] < param_2[i+4]) through mask ari… |
| `00aa9eb0` | AabbInclusiveContains | likely | Four-lane SSE predicate on two float[8] min/max boxes: for every axis i it computes (param_1[i] <= param_2[i]) && (param_2[i+4] <= param_1[i+4]) through mask a… |
| `00aaed30` | TestTransformedBoundsOverlap | likely | Gets two object transforms, transforms their eight-word bounds, compares the resulting extents, and runs a geometry query when all four SIMD comparisons pass. |
| `00fd6f50` | GetField68 | speculative | Fastcall(this): returns the dword at this+0x68 (8b 41 68 c3). |
| `00fd6f60` | GetField58 | speculative | Fastcall(this): returns the dword at this+0x58 (8b 41 58 c3). |
| `00fe5e60` | Mat4FromPosAndBlock6c0 | speculative | Thiscall (ECX=this). Gets the matrix pointer via 00ffba10 on this+0x2c, then writes 16 floats to param_2: entries 0,1,2,4,5,6,8,9,10 copy matrix entries (some … |
| `00fe60a0` | Mat4FromPosAndBlock6cc | speculative | Same as 00fe5e60 except the vector is read from (*(this+0x1cc))+0x6cc..+0x6d4 instead of +0x6c0..+0x6c8: same 00ffba10 matrix pointer, same 16-float output lay… |
| `00fe9030` | Mat4FromRecord12AndBlock | speculative | Thiscall (ECX=this, args: index, float* out). Ensures the index exists in the 12-byte record array at [[this+0x1cc]+0x578] (count +0x57c, growth through 00fe69… |
| `00fe9320` | ResetFlagArray3AndDwordArray4 | speculative | Fastcall (ECX=this). Sets the dword at +0x204 to 3, sets the first three bytes of the byte array at +0x1f0 to 1 (growing it through 00869290 when needed), make… |
| `00fe95e0` | FloatTableCountBelow3 | speculative | Thiscall (ECX=this, float arg). Walks up to three float entries of the table at [[this+0x1cc]+0xa14] (count +0xa18, growth through 007d7250), returning the num… |
| `00fe9650` | FlagArraySetWithTrueCount | speculative | Thiscall (ECX=this, args: index, flag char). Grows the byte-flag array at +0x1f0 (count +0x1f4) to cover index when needed. If the flag is currently set and th… |
| `010ba630` | WorldQueryWithFlags0x143 | speculative | Calls 00769710 (global accessor), then 00fd4c20 with its seven stack arguments, the constant 0x143 and 0, and returns whether 00fd4c20 returned non-zero. |
| `010ba6e0` | ProbeSegmentWith0x143 | speculative | Stack arguments: a float* start point, a dword, and an int* output. Computes an end vector from the start point with two constant offsets, calls 00769710 then … |
| `00bf8940` | — | unknown | Thiscall(this, obj). Returns if obj is null or vcall +0x24 of obj is zero. Computes two sweep start/end points from this+0x74 and obj's transform (vcall +0x1c)… |
| `00fd81d0` | — | unknown | Thiscall(this, key): scans the list headed at this+0x1094 (links at +0x14), inspects child nodes at +8 (links at +0xc) comparing the dword at +4 with the argum… |
| `00fe8160` | — | unknown | Fastcall (ECX=this). When byte [[this+0x22c]+0x108c] is zero: ensures record index [this+0x108] in the 12-byte array at [[this+0x1cc]+0x77c] (growing with 009d… |
| `00fe8460` | — | unknown | Thiscall (ECX=this, args: index, float* out; ret 8). Calls 00ffbef0(index,0), 00ffba20(index) and 00ffbd50(index) (the last returns a per-index record with flo… |
| `00fe9710` | — | unknown | Thiscall (ECX=this, args: object pointer with id at +0x1c and flag at +0x74, float* vector, value). Scans the node list at this+0x210 for a node whose key equa… |
| `00fe9900` | — | unknown | Thiscall (ECX=this, one stack arg: a small code). Reads four float samples from 0x30-byte records of the array at [[this+0x1cc]+0x87c] (count +0x880, growth th… |
| `00fe9f10` | — | unknown | Thiscall (ECX=this, one stack arg stored at this+0x1e8). Scans the 0x30-byte records of [[this+0x1cc]+0x87c] (count +0x880) for the first whose float at +0 is … |
| `00fea360` | — | unknown | Thiscall returning bool (ECX=this, args: object pointer with fields at +0x1c, float key). Returns false at once when byte this+0x230 is set. Otherwise it takes… |

### `physics.collision`

**Actor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0077cad0` | — | unknown | Thiscall on an actor: reads the skeleton collision object at +0x29c and the actor collision data at +0x290, and when one is present without the other (or the s… |

**AppPhysics::BulletGhostObject** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdc4c0` | BulletGhostObject::ctor | confirmed | Constructs the btPairCachingGhostObject base through 0112d6f0, then stores the AppPhysics::BulletGhostObject vtable at offset 0. |
| `00fdc540` | BulletGhostObject::~BulletGhostObject | confirmed | Stores the BulletGhostObject vtable at offset 0 and calls the btPairCachingGhostObject destructor 0112d7b0. Does not free. |
| `00fdc600` | BulletGhostObject::DeletingDtor | confirmed | Stores the BulletGhostObject vtable, calls the btPairCachingGhostObject destructor 0112d7b0, and frees this when bit 0 of the stack argument is set. |
| `00fdc590` | BulletGhostObject::ForwardPairSlot6 | speculative | Virtual slot 6 of BulletGhostObject. Ignores ECX and reads the first stack argument: its inner object's owner pointer at +0xf8 is saved, 0112d830(arg1, arg2) i… |
| `00fdc5c0` | BulletGhostObject::ForwardPairSlot7 | speculative | Virtual slot 7 of BulletGhostObject. Reads the first stack argument's inner owner pointer at +0xf8, calls 0112cc60(arg1, arg2, arg3), then calls vtable slot 15… |

**AppPhysics::CollisionData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00775e00` | CollisionData::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag, ret 4). Calls the destructor body 00fdebf0 on the object, frees it through _free when flag bit 0 … |
| `00fdebc0` | CollisionData::ctor | confirmed | Stores its first argument at +4 (shape pointer), stores the CollisionData vtable at offset 0, stores its second argument at +8 and clears the flag bytes at +0x… |
| `00fdebf0` | CollisionData::~CollisionData | confirmed | Stores the CollisionData vtable, and if the shape at +4 is set calls its vtable slot 0 with argument 1 (scalar deleting destructor), then clears +4. |

**AppPhysics::CollisionDataAsset** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00775d70` | CollisionDataAssetCtor | confirmed | Fastcall constructor (ECX = object). Calls 0071bb80 on the object (the Asset base constructor), stores AppPhysics::CollisionDataAsset::vftable at +0, zeroes th… |
| `00775e20` | CollisionDataAsset::LoadBuffer | confirmed | Thiscall virtual slot [3] (ECX = asset, stack args: callback object pointer and a (data pointer, size) pair). Allocates a 0x10-byte AppPhysics::CollisionData o… |
| `00775ee0` | CollisionDataAssetDtor | confirmed | Fastcall destructor body (ECX = asset). Stores AppPhysics::CollisionDataAsset::vftable at +0; if the pointer at +0x28 (the CollisionData object) is non-null it… |
| `00775f50` | CollisionDataAsset::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag, ret 4). Calls the destructor body 00775ee0 on the object, frees it through _free when flag bit 0 … |

**AppPhysics::CollisionObject** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdcb40` | CollisionObject::~CollisionObject | confirmed | Stores the CollisionObject vtable, calls 00fd8160 when the field at +8 is nonzero, releases the object at +0xb4 through its vtable slot 1 with argument 1, rele… |
| `00fdce70` | CollisionObject::ctor | confirmed | Calls 0071b950 (RefCounted base constructor), stores the AppPhysics::CollisionObject vtable, zeroes +0x8, +0xc, +0x10, +0x80, +0xb0 and +0xb4, stores the param… |
| `00fdcf30` | CollisionObject::DeletingDtor | confirmed | Calls the CollisionObject destructor body 00fdcb40 on this and frees this when bit 0 of the stack argument is set. |
| `00fdcf50` | CollisionObject::SetPositionIfMoved | confirmed | Virtual slot 10, inherited by WaterBody, GhostTrigger, CameraFrustumSensor, GhostObject and RigidBody. If the squared distance between the 4-float argument and… |
| `00fdd1f0` | CollisionObject::CtorBulletObject | confirmed | Calls 0071b950 (RefCounted base), stores the CollisionObject vtable, sets fields +0x8/+0xc/+0x14..+0x20 and +0x30..+0x3c from the arguments and floats from the… |
| `00fdc820` | CollisionObject::SetFlagBit4 | speculative | Virtual slot 4. If the Bullet object at this+0xb4 is set, ORs 0x4 into its flags word at +0xd8. |
| `00fdc840` | CollisionObject::ClearFlagBit4 | speculative | Virtual slot 5. If the Bullet object at this+0xb4 is set, clears bit 0x4 of its flags word at +0xd8. |
| `00fdc860` | CollisionObject::IsFlagBit4Clear | speculative | Virtual slot 6, shared with RigidBody. Returns 1 when the Bullet object at this+0xb4 is set and bit 0x4 of its flags word at +0xd8 is clear; returns 0 otherwis… |
| `00fdc8e0` | CollisionObject::SetKinematicFlags | speculative | Virtual slot 7, inherited by WaterBody, GhostTrigger, CameraFrustumSensor, GhostObject and RigidBody. If the Bullet object at this+0xb4 is set, clears bit 0x1 … |
| `00fdc910` | CollisionObject::SetStaticFlags | speculative | Virtual slot 8, inherited by the same classes as slot 7. If the Bullet object at this+0xb4 is set, clears bit 0x2 and sets bit 0x1 in its flags word at +0xd8. |
| `00fdc940` | CollisionObject::GetFloatAtF0 | speculative | Virtual slot 12. Returns the float at +0xf0 of the Bullet object at this+0xb4, or 0.0 when that pointer is null. |
| `00fdc970` | CollisionObject::SetFloatAtF0 | speculative | Virtual slot 13. Stores its float argument at +0xf0 of the Bullet object at this+0xb4 when that pointer is set. |

**AppPhysics::GhostObject** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdd000` | GhostObject::ctor | confirmed | Constructs the CollisionObject base through 00fdce70, stores the AppPhysics::GhostObject vtable at offset 0, reads a value from the object at param_2+4 through… |
| `00fddee0` | GhostObject::~GhostObject | confirmed | Stores the GhostObject vtable, releases the debug records through 00fdd8c0 and 00fdd960, calls the CollisionObject destructor body 00fdcb40, and releases the s… |
| `00fddf90` | GhostObject::DeletingDtor | confirmed | Calls the GhostObject destructor body 00fddee0 on this and frees this when bit 0 of the stack argument is set. |

**AppPhysics::Trigger** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe32c0` | AppPhysics::Trigger::~Trigger | confirmed | Non-deleting destructor body (73 bytes): stores AppPhysics::Trigger::vftable then SoeUtil::RefCounted::vftable at +0. |
| `00fe33a0` | AppPhysics::Trigger::DeletingDtor | confirmed | Scalar-deleting destructor (97 bytes): stores the trigger vptr then the RefCounted vptr, then calls _free(this) when flag bit 0 is set. |
| `00fe3310` | — | unknown | Constructor (thiscall, four stack arguments): calls 0071b950 (RefCounted base), stores AppPhysics::Trigger::vftable, stores the arguments at +0x8, +0xc, +0x10,… |

**CameraFrustumSensor** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af1b90` | CameraFrustumSensor::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls GhostObject::~GhostObject (00fddee0) on this, then calls _free(this) when bit 0 of the flag argum… |
| `00af12d0` | — | unknown | Constructor (ECX = this, ten stack args). Calls GhostObject::ctor (00fdd000) with the first seven arguments and a zero flag, stores CameraFrustumSensor::vftabl… |

**btCollisionWorld::RayResultCallback** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fff8f0` | btCollisionWorld::RayResultCallback::DtorBody | confirmed | Fastcall (ECX=this, no stack arguments): stores btCollisionWorld::RayResultCallback::vftable at this+0 and returns. Frees nothing; it is the base-class destruc… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00cc4480` | TestOrientedBoxOverlap | likely | Builds centers and half extents from two four-component endpoint sets, rejects separated center projections on three axes and cross-product axes, and returns o… |
| `00fd80f0` | CollisionRegistryRemoveByKey100 | likely | Thiscall(this, obj): when obj+8 is non-null it calls the virtual at +0x60 of the world at this+8 with obj+0x100, clears obj+8, looks the key at obj+0xc up in t… |
| `00fd8160` | CollisionRegistryRemoveByKeyB4 | likely | Thiscall(this, obj): when obj+8 is non-null calls the virtual at +0x20 of the world at this+8 with obj+0xb4, clears obj+8, looks the key at obj+0xc up in the 1… |
| `00fdc660` | CollisionObject::SetProxyFilterGroup | likely | Stores param_2 at this+0x14. When the Bullet object at this+0xb4 is set and its +0xc8 pointer (broadphase proxy) is nonzero, writes the low 16 bits of param_2 … |
| `00fdc690` | CollisionObject::SetProxyFilterMask | likely | Stores param_2 at this+0x18. When the Bullet object at this+0xb4 is set and its +0xc8 proxy pointer is nonzero, writes the low 16 bits of param_2 to proxy+6. |
| `00fdcbe0` | CollisionObject::SyncCachedTransform | likely | If the Bullet object at this+0xb4 is set and its sub-object at +0xcc is set: calls vtable slot 1 of that sub-object with (bullet+0x10, this+0xe0, this+0xf0), t… |
| `00fdcce0` | CollisionObject::SetTransformMatrix | likely | Copies 16 dwords from param_2 into this+0x40 (a 4x4 float matrix). When the Bullet object at this+0xb4 is set, calls 00fca610 on the matrix, 00fca740 on a stac… |
| `00fdf050` | CollisionSpanCopySubmit | likely | Thiscall(this, src, n): when src is nonnull and n>0, selects the record at this+4 (creating one with operator new 0x60 and 00af18f0 when null; the target is th… |
| `00fdf120` | CollisionSpanSubmitNoCopy | likely | Thiscall(this, src, n): when src is nonnull and n>0, selects the record at this+4 (created as in 00fdf050 when null), submits the span src with size n via thun… |
| `00af1370` | Wrapper_00af1370 | speculative | Thiscall with no arguments (ECX = this). Sets up an MSVC EH frame (state -1), calls GhostObject::~GhostObject (00fddee0) on this, then unwinds the frame and re… |
| `00fd6e70` | CopyTransposedRotToObj100 | speculative | Stdcall(this, src): when this is non-null, calls 00fca610 with a 4x4 source to fill a local 16-dword matrix buffer and then calls 00fca740 to copy that block i… |
| `00fd6eb0` | NewObjectRegisterWithWorld | speculative | Thiscall(this, a, b, c, d): calls 00d833a6 (operator new of 4 bytes); when the allocation is non-null it calls 00fd3e40 with the four stack arguments and takes… |
| `00fe0c80` | CountManifoldsWithMask | speculative | Thiscall (param_1 = body pointer, param_2 = 16-bit mask): reads the dynamics world from DAT_01cb56fc+8, asks its dispatcher (world+0x18) for the manifold count… |
| `00fe0d30` | FindNthManifoldPartner | speculative | Thiscall (param_1 = body pointer, param_2 = 16-bit mask, param_3 = index): walks the same dynamics-world manifold list as 00fe0c80 and returns the partner body… |
| `00fe22e0` | CollisionGridBuildProcedural | speculative | Fastcall(this): allocates 800 vertices (12 bytes each, 0x2580 bytes) and 4446 16-bit indices (0x22bc bytes) for a 40x20 grid. Vertex positions come from float … |
| `00d3a7a0` | — | unknown | Thiscall reader: copies a 4-byte count from param_2 into this+4 and returns 0 unless 3 < count < 0x2711; allocates count*12 bytes at this+8 and copies that man… |
| `00fdfab0` | — | unknown | Thiscall (param_1 = owner, param_2 = payload pointer, param_3 = count): when count > 0 and payload is nonnull, reads the first dword, calls 006fc7b0 and 00fded… |
| `00fed630` | — | unknown | Fastcall (ECX = dispatcher, the int array at param_1). Returns when the manifold count (vtable +0x20 of the object at [ECX+0x18]) is not positive. For each pai… |

### `physics.dynamics`

**AppPhysics::DynamicsSystem** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdc050` | DynamicsSystem::DynamicsSystem | confirmed | Stores the AppPhysics::DynamicsSystem vtable at offset 0, zeroes the members, installs the vtables of the Array<VehiclePhysics*,0,1> member at +0x50, Array<Spe… |
| `00fdc190` | DynamicsSystem::~DynamicsSystem | confirmed | Stores the DynamicsSystem vtable, calls 00fdbf30 (destroy pool entries), releases the owned sub-objects at +0x8 through +0x2c (vtable slot 0 or 01160c00), free… |
| `00fdc380` | DynamicsSystem::DeletingDtor | confirmed | Calls the destructor body 00fdc190 on this and frees this when bit 0 of the stack argument is set. |

**AppPhysics::RigidBody** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fe0760` | AppPhysics::RigidBody::~RigidBody | confirmed | Destructor body (157 bytes): stores AppPhysics::RigidBody::vftable, clears the registry slot DAT_01cb5890[this+0x15c] when that index is nonnegative, zeroes +0… |
| `00fe1b70` | AppPhysics::RigidBody::DeletingDtor | confirmed | Scalar-deleting destructor: calls 00fe0760 (rigid body destructor body), then _free(this) when flag bit 0 is set (30 bytes). |
| `00fe3000` | AppPhysics::RigidBody::RigidBody | confirmed | Constructor (thiscall, four stack arguments): calls the collision-object base constructor 00fdce70, stores AppPhysics::RigidBody::vftable at +0, zeroes the own… |
| `00fe31b0` | AppPhysics::RigidBody::RigidBodyNamed | confirmed | Constructor variant (thiscall, seven stack arguments): calls 00fdce70, stores the RigidBody vptr, zeroes the owned pointers at +0x104/+0x108, fills +0x110..+0x… |
| `00fe2a00` | AppPhysics::RigidBody::CopyTransformTransposed | likely | Vtable slot [3] (fastcall): byte-identical to 00fe2700; copies the body's transposed rotation and origin into this+0x40..+0x7c, composing with 00fe02b0 when th… |
| `00fe0b90` | AppPhysics::RigidBody::SetFlagBit4 | speculative | Vtable slot [4]: ORs 4 into the dword at this+0x100 then +0xd8 (14 bytes). |
| `00fe0ba0` | AppPhysics::RigidBody::ClearFlagBit4 | speculative | Vtable slot [5]: clears bit 0x4 (AND with 0xfffffffb) of the dword at this+0x100 then +0xd8 (14 bytes). |
| `00fe2b50` | AppPhysics::RigidBody::SetTransformClearVelocity | speculative | Vtable slot [11] (thiscall, param_2 = 3x4 affine transform): builds the same local 4x4 record as 00fe2850, composes it through 00fe0ef0 and 00fe02b0 when the b… |

**AppPhysics::SphereRigidBodyPool** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fda300` | SphereRigidBodyPool::~SphereRigidBodyPool | confirmed | Stores the List<SphereRigidBodyPoolItem> vtable at this+4, then calls 00fd9ee0 on the embedded list at this+4 (remove all item nodes). Does not free. |
| `00fda350` | SphereRigidBodyPool::~SphereRigidBodyPool | confirmed | Byte-identical body to 00fda300: stores the item-list vtable at this+4 and calls 00fd9ee0 on this+4. Does not free. |
| `00fdb740` | SphereRigidBodyPool::DeletingDtor | confirmed | Stores the item-list vtable at this+4, calls 00fd9ee0 on that list, and frees this when bit 0 of the stack argument is set. |
| `00fda2a0` | SphereRigidBodyPool::ctor | likely | Stores the List<SphereRigidBodyPoolItem> vtable at this+4 and zeroes the head, tail and count at this+8, +0xc and +0x10. Returns this (EAX copy at entry). |
| `00fdb6e0` | SphereRigidBodyPool::ctor | likely | Stores the List<SphereRigidBodyPoolItem> vtable at this+4 and zeroes the head, tail and count at this+8, +0xc and +0x10. Returns this (EAX copy at entry). |

**GhostTrigger** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007bc040` | GhostTrigger_Ctor | confirmed | Constructor: runs the GhostObject base constructor 00fdd000 with seven of the arguments plus a trailing 0, stores GhostTrigger::vftable at +0 and the first arg… |
| `007bc160` | GhostTrigger_ScalarDeletingDtor | confirmed | Scalar deleting destructor (GhostTrigger vtable slot 0): runs the GhostObject destructor 00fddee0 on this, frees this with _free when flag bit 0 is set, and re… |
| `007bc0d0` | GhostObject_DtorBodyCall | likely | 66-byte destructor fragment: calls 00fddee0 (GhostObject::~GhostObject) once under an SEH frame, with no other member handling visible. |
| `007bc120` | — | unknown | 20-byte forwarder: tail-calls 007b49e0 with this and the two arguments; that helper walks the list at this+8 (next link at node+4) and calls virtual slot +4 of… |
| `007bc140` | — | unknown | 20-byte forwarder: calls 007b4a10 with this and the two arguments; that helper walks the list at this+8 (next link at node+4) and calls virtual slot +8 of each… |

**WaterBody** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ba840` | WaterBody_Event0E_FindAndCallvt20 | likely | If the event argument's type word at +0x20 equals 0xe it resolves an object by id: calls vtable slot +0x10 of the manager stored at this+0x110 with the id at e… |
| `007ba880` | WaterBody_Event0E_FindAndCallvt24 | likely | Same shape as 007ba840 (event type at +0x20 must be 0xe, id at +0x1c resolved through vtable +0x10 of this+0x110) but forwards to vtable slot +0x24 of the reso… |

**(no class)** (57)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fda940` | RegisterObjectInIdMap | likely | If the object at param_2 has +8 == 0: calls vtable slot 22 (+0x58) of the object at this+8 with the dword at object+0x100 and the words at object+0x14 and +0x1… |
| `00fdb600` | RegisterObjectInIdMapB | likely | If the object at param_2 has +8 == 0: increments the counter at this+0x10a0, stores the new id at object+0xc, inserts (id, object) through 00fda1e0 into the ma… |
| `00fdb800` | DynamicsSystemBuildBulletWorld | likely | Builds the Bullet simulation objects: arrays and pool buffers through 01160bf0 and 00fd01b0, btDefaultCollisionConfiguration (0112c0d0), btCollisionDispatcher … |
| `00fdbb30` | DynamicsSystemStep | likely | If the byte at this+0x108c is zero: converts the float argument with the scale constant at 0x017ebf0c, divides by the step length in this+0x10a4, takes the who… |
| `00fdbc20` | SphereRigidBodyPoolListAppendNew | likely | Allocates a node through vtable slot 2 (+8) of the list object (this), constructs it with 00fdb6e0 when the allocation is nonzero, sets node next (+0x14) to 0 … |
| `00fdbcb0` | SphereRigidBodyPoolListUnlink | likely | Unlinks node param_2 from the list (head at this+4, tail at +8, count at +0xc; node next at +0x14, prev at +0x18), stores the List<SphereRigidBodyPoolItem> vta… |
| `00fdbd40` | PoolEntryAcquireItem | likely | Finds the pool entry for the int key param_7 in the list at this+0x1094 (appending a new entry through 00fdbc20 if absent). Scans that entry's item list (head … |
| `00fe3fd0` | AxisAngleToQuaternion | likely | Thiscall (param_1 = quaternion output, param_2 = axis vector, param_3 = angle): half-angle a = angle * 0.5 (DAT_017649d4); scales the axis by sin(a)/\|axis\| i… |
| `00ffff00` | TwoBodyRowClampedImpulse | likely | One pointer argument (param_1), x87 float result. Reads the two-body row struct built by 00fffae0 (bodies at [0] and [1], point at [4..6], direction at [8..10]… |
| `00fdbb10` | DynamicsSystemCallSlot13 | speculative | Calls vtable slot 13 (+0x34) of the object at this+8 with no arguments. If the byte at this+0x108d is nonzero it tail-calls 00fdae80(this). |
| `00fdbf10` | PoolListClear | speculative | Loops while the list head at this+4 is nonzero, calling 00fdbcb0(this, head) to unlink and destroy each pool node. |
| `00fe0810` | CopyLinkVec140ToOffset130 | speculative | Thiscall: reads the object at this+0x100 and copies three dwords from it at +0x140, +0x144, +0x148 into this+0x130, +0x134, +0x138, then sets this+0x13c to the… |
| `00fe08a0` | LinkVec140SetWithZeroW | speculative | Thiscall (param_1, param_2 vec3): writes the three floats of param_2 to the object at this+0x100 offsets +0x140, +0x144, +0x148 and clears +0x14c (95 bytes). |
| `00fe0900` | CopyLinkVec150ToOffset140 | speculative | Thiscall: reads the object at this+0x100 and copies dwords +0x150, +0x154, +0x158 to this+0x140, +0x144, +0x148, then sets this+0x14c to DAT_0175b420 (1.0f) (1… |
| `00fe0990` | LinkVec150SetWithZeroW | speculative | Thiscall (param_1, param_2 vec3): writes the three floats of param_2 to the object at this+0x100 offsets +0x150, +0x154, +0x158 and clears +0x15c (95 bytes). |
| `00fe0a00` | Vec3PadToLocal4CallFunc011820a0 | speculative | Copies three dwords from the argument array into a local 4-dword record with the fourth word 0 and calls 011820a0 with that record (73 bytes). |
| `00fe0a50` | LinkDwordEC_Set | speculative | Thiscall (param_1, dword): stores the dword argument at the object at this+0x100 offset +0xec (23 bytes). |
| `00fe0a70` | Vec3PairCallFunc00fcff80 | speculative | Copies two 3-dword arrays into two local 4-dword records (w zeroed) and calls 00fcff80 with pointers to them (119 bytes). |
| `00fe0af0` | LinkVec140AddScaledVec | speculative | Thiscall (param_1, float vec3): adds, for each axis, the object at this+0x100 field (+0x170, +0x174, +0x178) times its scalar at +0x160 times the matching argu… |
| `00fe0c20` | LinkDwords1ECAnd1F0_Set | speculative | Thiscall (param_1, two dwords): stores the two dword arguments at the object at this+0x100 offsets +0x1ec and +0x1f0 (37 bytes). |
| `00fe0e60` | ResetLinkDwordE8AfterFunc011271d0 | speculative | Thiscall: calls 011271d0 with 1, then stores 0 at the object at this+0x100 offset +0xe8 (35 bytes). |
| `00fe1a60` | ConstructionRecordInit | speculative | Thiscall (param_1 = record, then mass, motion state, shape, and a 3-float vector pointer): fills the record: +0 mass, +4 motion state, +0x10..+0x4c an identity… |
| `00fe1b90` | RigidBodyObjectBuild | speculative | Thiscall with six arguments (ECX=this): stores the owner at this+0x10, creates a 0xe0-byte default motion state (00fd2330 + 00fe1080) at this+0x104, copies the… |
| `00fe20a0` | RigidBodyObjectBuildSimple | speculative | Thiscall with six stack arguments: stores param_2 at this+0x10, calls the virtual slot +4 of the object at param_2+4 to get this+0xb0; stores the mass and iner… |
| `00fe2700` | RigidBodyCopyTransformTransposed | speculative | Fastcall (this in ECX): reads 13 dwords of the body object at this+0x100 (rotation columns +0x10,+0x14,+0x18 / +0x20,+0x24,+0x28 / +0x30,+0x34,+0x38 and origin… |
| `00fe2850` | RigidBodySetTransformClearVelocity | speculative | Thiscall (param_2 = 3x4 affine transform read as 4-float rows, thiscall with one stack argument): builds a local 4x4 record from param_2 (rows with w zero); wh… |
| `00fe2ce0` | CopyBodyTransformToOut | speculative | Thiscall (param_2 = 16-dword output): writes the body at this+0x100 transposed rotation and origin into the output as a 4x4 (w words 0, 1.0 at output +0x3c) an… |
| `00fe2e30` | ObjectResetAndStringAssign | speculative | Thiscall (param_1 = object, param_2 = C string pointer): zeroes the vector at +0x160..+0x168, sets +0x16c to 1.0 (DAT_0175b420), zeroes the dwords at +0x110..+… |
| `00fe39c0` | ObjectGetQuatAt160 | speculative | Thiscall (output pointer): copies dwords +0x160, +0x164, +0x168 and +0x16c of the object to the output (42 bytes). |
| `00fe3a50` | OwnerResetAndSetTransform | speculative | Thiscall (param_1 = owner, param_2 = transform pointer): clears owner fields +0x48, +0x4c, +0x50, +0x54, +0x142, +0x1d0, +0x1dc, +0x1dd, +0x71..+0x74 and +0x15… |
| `00fe3b20` | TransformPtrOrDefault | speculative | Fastcall (this in ECX): when the dword at this+0x1c is nonzero returns the result of 00fe2700 (body transform copy); otherwise returns the address of the const… |
| `00fe3bb0` | ReturnByteAt10C | speculative | Fastcall (this in ECX): returns the byte at this+0x10c (7 bytes). |
| `00fedb90` | ChainBodyCall011825a0 | speculative | Thiscall with one float stack argument (ret 4). Builds a local record {0, arg/[this+0x11c], 0, 0}, then calls 011825a0 with the float argument and a pointer to… |
| `00fedc30` | Wrapper_Chain14_00fe2700 | speculative | Loads ECX from this+0x14 (the chassis RigidBody) and tail-jumps to 00fe2700 (8 bytes, mov ecx,[ecx+0x14]; jmp). No other work. |
| `00fedc40` | Wrapper_Chain14_00fe2850 | speculative | Loads ECX from this+0x14 (the chassis RigidBody) and tail-jumps to 00fe2850 (8 bytes, mov ecx,[ecx+0x14]; jmp). |
| `00fedc50` | Wrapper_Chain14_00fe0810 | speculative | Loads ECX from this+0x14 (the chassis RigidBody) and tail-jumps to 00fe0810 (8 bytes, mov ecx,[ecx+0x14]; jmp). |
| `00fedc60` | Wrapper_Chain14_00fe08a0 | speculative | Loads ECX from this+0x14 (the chassis RigidBody) and tail-jumps to 00fe08a0 (8 bytes, mov ecx,[ecx+0x14]; jmp). |
| `00fedc70` | Chain14_ReadFloat4Of00fe0900 | speculative | Loads ECX from this+0x14, calls 00fe0900 and returns the float at result+4 on the x87 stack (12 bytes: mov ecx,[ecx+0x14]; call; fld [eax+4]; ret). |
| `00fedc80` | Chain_SetVec150_YFromArg | speculative | Stack float argument p. Builds the local record {0, p, 0, 1.0} and calls 00fe0990 with ECX = the chassis RigidBody (this+0x14), which writes (0, p, 0) into the… |
| `00fedcd0` | Chain_SetVec140FromConstant | speculative | Copies the 16-byte constant at DAT_01cb53d0 into a local record and calls 00fe08a0 with ECX = the chassis RigidBody (this+0x14), which writes its xyz into the … |
| `00fedd00` | Chain_ZeroVec150 | speculative | Builds a zero 3-vector (w = DAT_0175b420) in a local record and calls 00fe0990 with ECX = the chassis RigidBody (this+0x14), clearing the Bullet body +0x150..+… |
| `00fede30` | Chain_OrFilterMaskWith2F8ThenVcall14 | speculative | Fastcall (ECX = this). Calls 00fdc690 (CollisionObject::SetProxyFilterMask) with the chassis+0x18 mask OR'd with this+0x2f8, then tail-calls the chassis vtable… |
| `00fede50` | Chain_ClearMaskBits2F8ThenVcall10 | speculative | Fastcall (ECX = this). Calls 00fdc690 with the chassis+0x18 mask AND NOT this+0x2f8, then tail-calls the chassis vtable +0x10. |
| `00fedec0` | RearmCounter5OnFloatKey | speculative | Thiscall with one float argument (ret 4). If the counter at +0xf4 is non-zero and the argument is at most +0xf0, decrements +0xf4 and returns. Otherwise stores… |
| `00fedf40` | Chain_SetVec140ConstThenZeroVec150 | speculative | Thiscall (ECX = this). With chassis = this+0x14, calls 00fe08a0 with the constant at DAT_01cb53d0 (sets the Bullet body +0x140 vector), then 00fe0990 with a ze… |
| `00fee900` | SetVec2At64IfMagnitudeAboveEps | speculative | Thiscall with two float arguments. When p2*p2 + p3*p3 exceeds DAT_017ebbb8 stores (p2, p3) at this+0x64/+0x68 and at +0x70/+0x74. Otherwise changes nothing (59… |
| `00fee990` | ChainPosePredicate | speculative | Fastcall (ECX = this). Returns 1 when the chassis record from 00fe2700 has +0x30, +0x34 and +0x38 inside their bounds and the vector from 00fe0810 has small co… |
| `00feea60` | ChainClampSlot34ThenSetTransform | speculative | Copies the 16-dword chassis record from 00fe2700 into a local. When the copy's +0x34 word is outside [DAT_017f1e40, DAT_017fe21c], stores DAT_0187c6cc there an… |
| `00fef100` | ResetChainStateAndVectors | speculative | Thiscall (ECX = this, one stack argument). Zeroes this+0x20, +0x24, +0x28 and +0x4c. Calls 00fe2850 with the argument on the chassis (this+0x14). Sets the chas… |
| `00fef350` | SetVec2At64FromRecordScalars | speculative | Thiscall with one float argument. When \|param_2\| exceeds DAT_017ebbb8, reads two scalars from the record returned by 00fe2700 (+0x20 and +0x28), multiplies t… |
| `00fff680` | BodyPointDirectionTerm | speculative | Thiscall(this, float* point, float* direction) returning float on x87 (float10). Computes the offset point - (this+0x40..0x48), crosses it with direction, mult… |
| `00fdbf30` | — | unknown | For each pool entry in the list at this+0x1094: for each item in the entry's item list (head at entry+8) it calls 00fd80f0 when the item's flag byte is set, de… |
| `00fef1b0` | — | unknown | Receiver ECX is passed on to 00fe2700. Copies the 16-dword record it returns into a local, adds the four floats of the argument to the copy's +0x30..+0x3c word… |
| `00fef210` | — | unknown | Receiver ECX is passed to 00fe2700 for the record copy. Builds a rotation about the axis (0, 1, 0) with angle param_1 through 00770e70 into a local, combines t… |
| `00fef2a0` | — | unknown | Receiver ECX is passed to 00fe2700. Builds a rotation about (0, 1, 0) with angle param_2 through 00770e70, combines the record copy through 0073a380, adds the … |
| `00fef920` | — | unknown | Copies the 16-dword record from 00fe2700 into a local. Sets the copy's +0x30 to DAT_017ed844 when outside [DAT_0181d5b8, DAT_017efd20], the same for +0x38, and… |
| `00fffae0` | — | unknown | Thiscall with five stack arguments (ret 0x14). Stores body A (param_2) and body B (param_3) at out+0 and +4, point at out+0x10..0x1c and direction at out+0x20.… |

### `physics.mesh`

**AppPhysics::GatherTrianglesConvexResultCallback** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fda3a0` | — | unknown | Virtual slot 2 of the GatherTriangles callback. Reads the body flags (+0x4, +0xc/+0xd) and the query mask at param_1+0x10, returns early on a filter match, oth… |

**AppPhysics::MeshData** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af18f0` | AppPhysics::MeshData::Ctor | confirmed | Constructor (fastcall, ECX = this). Stores AppPhysics::ShapeData::vftable and then AppPhysics::MeshData::vftable at [this], so the final vtable is MeshData. Ze… |
| `00af1970` | AppPhysics::MeshData::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls 00fdeac0 (MeshData destructor body), then calls _free(this) when bit 0 of the flag argument is se… |
| `00fde890` | MeshData::ReleaseBuffers | likely | If the flag byte at +0x41 is clear it sets it to 1; then, if the byte at +0x43 is set, frees the buffer at +0x50 through 01160c00, zeroes +0x50, subtracts the … |
| `00fde8d0` | — | unknown | Virtual slot 1 of MeshData. When the byte at +0x41 is set it creates a btConvexHullShape (01139010, 0xa0 bytes allocated) from +0xc/+0x8. Then, when the mesh s… |
| `00fdeac0` | — | unknown | Stores the MeshData vtable, releases the sub-shape at +0x48 through its vtable slot 0 with flag 1 when set, subtracts from the global counter DAT_01cb56f8, fre… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdf970` | CollisionMeshBuildTriangleArray | likely | Fastcall with this in ECX: allocates a 0x60-byte block (01160bf0, align 0x10), constructs a btTriangleIndexVertexArray through 00fdf780 and stores it at this+0… |
| `00fdfd90` | — | unknown | Thiscall with five stack arguments: if this+4 is null, allocates a 0x60 record (operator new) and constructs it through 00af18f0; then writes the record fields… |

### `physics.query`

**AppPhysics::GatherConvexHitsCallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fd4810` | GatherConvexHitsCallback::DeletingDestructor | confirmed | Scalar deleting destructor: stores the btCollisionWorld::ConvexResultCallback vftable at +0, calls _free on the object when flag bit 0 is set, and returns this. |
| `00fd9120` | — | unknown | Virtual slot 2 (addSingleResult shape): takes a local hit record (object at [0], local normal at [4..6], fraction at [8] or [0xc]); when the object at +0xf8 pa… |

**AppPhysics::GatherRayHitsCallback** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fd8c60` | GatherRayHitsCallback::AddSingleResult | likely | Virtual slot 2 (addSingleResult shape): takes a local hit record (object pointer at [0], local normal at [4..6], fraction at [8]); when the hit object's +0x14 … |

**AppPhysics::GatherTrianglesConvexResultCallback** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fd6e10` | GatherTrianglesConvexResultCallback::DeletingDestructor | confirmed | Scalar deleting destructor: stores the btCollisionWorld::ConvexResultCallback vftable at +0, calls _free on the object when flag bit 0 is set, and returns this. |

**AppPhysics::MyClosestConvexResultCallback** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fd4870` | MyClosestConvexResultCallback::DeletingDestructor | confirmed | Scalar deleting destructor: stores the btCollisionWorld::ConvexResultCallback vftable at +0, calls _free on the object when flag bit 0 is set, and returns this. |

**(no class)** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fd4030` | RayCastClosestMask100 | likely | Thiscall ray query: builds a ray-result callback with 00fd0b70 from the from and to pointers, the filter argument and a mask that is object+0x100 (0 when the o… |
| `00fd4290` | RayCastClosestMaskB4 | likely | Thiscall ray query identical to 00fd4030 except the mask is object+0xb4: builds a ray-result callback with 00fd0b70, calls the world virtual at +0x18 of this+8… |
| `00fd44f0` | ConvexCastFirstHitObjB4 | likely | Thiscall: builds a ray-result callback with 00fd0b70 (mask 0, filter 0xec7) from the two vector arguments, then casts the shape at object+0xb4 (with its fields… |
| `00fd48d0` | ConvexSweepClosestMask100 | likely | Thiscall convex sweep: calls the virtual at +0x14 of the shape at this+0x2c with a buffer built from the scalar argument, builds a callback with 00fd0e60 (mask… |
| `00fd4c20` | ConvexSweepClosestMaskB4 | likely | Thiscall convex sweep, the object-mask variant of 00fd48d0 that reads the mask from object+0xb4 and passes its tenth argument to 00fd0e60; sweeps the shape at … |
| `00fd52b0` | ConvexSweepObjShapeMatrices | likely | Thiscall convex sweep of the shape at param+0xb0 between two 4x4 matrix arguments: copies the matrix pieces into two transforms, builds a callback with 00fd0e6… |
| `00fd5660` | ConvexSweepCompoundChildren | likely | Thiscall closest convex sweep over the children of a compound shape: walks the child array (count at shape+0x10, child records at shape+0x18, child shape point… |
| `00fd6ac0` | ConvexQuerySingleObjShapeB4 | likely | Thiscall: builds a callback with 00fd0e60 (mask 0, filter 0xec7), calls the virtual at +0x14 of the shape at this+0x2c, then calls 01123d60 (an eight-argument … |
| `00fd6f70` | SphereContactTest | likely | Thiscall(this, radius, ...): when the world at this+8 is non-null, builds a btSphereShape with 00fcbe20 from the radius argument, copies a 4x4 block with 00fca… |
| `00fd8e20` | RayCastSortedHits | likely | Thiscall(this, hits*, ...): clears the hit count at hits+8, gathers ray hits with 00fd0990 into the 48-byte array at hits, casts the ray through the virtual at… |
| `00fd9210` | ConvexSweepSortedHits | likely | Thiscall(this, hits*, from*, to*, filter, ...): clears the hit count at hits+8, gathers convex hits with 00fd0ca0, sweeps the shape at this+0x2c with 01125840 … |
| `00fd78c0` | HeapSortHits48 | speculative | Heap sort of 48-byte hit records: arg1 is the array base, arg2 the count, arg3 a comparator called as cmp(a, b) through a stack call; sifts records down with 1… |
| `00fd7ab0` | HeapSortHits48 | speculative | Byte-identical duplicate of 00fd78c0 (template instance): heap sort of 48-byte hit records with a comparator callback argument. |
| `00fd84b0` | InsertionSortHits48 | speculative | Insertion-style sort of 48-byte hit records (array base, count, comparator in arg3): shifts records with 12-dword copies while the comparator (called through a… |
| `00fd87a0` | InsertionSortHits48 | speculative | Byte-identical duplicate of 00fd84b0 (template instance): insertion-style sort of 48-byte hit records with a comparator argument. |
| `00fda6a0` | ConvexSweepBoxGatherTriangles | speculative | Clears or checks the hit count at param_1+8 (007f9490 when negative), initialises a GatherTriangles callback through 00fd1000, builds a box shape from scaled b… |
| `010a5cb0` | AABBInitAndUnionTransformed | speculative | Thiscall (ECX = this, two stack args; returns the first arg): calls 0076c990 with ECX = the output box to write its default 32-byte value, then calls 00774cf0 … |
| `010a6570` | PoseBoxTestAnyBone | speculative | Thiscall (ECX = this, stack args: an object pointer, a vec4 pointer copied to a local, a quaternion-like vec4 pointer, which it normalises with rsqrt and a NaN… |
| `00fd4f70` | — | unknown | Thiscall convex sweep of the shape at param+0xb0 against the world with two fixed 6-float transform pairs that do not depend on the from and to vectors; the fr… |
| `00fd7f60` | — | unknown | Thiscall(this, a..h): forwards its eight stack arguments to 00fd5660 (compound convex sweep) with the object at this+0xb0 as the first (this) argument; returns… |
| `00fd83e0` | — | unknown | Thiscall(this, obj*, id*, point*, normal*): grows the 48-byte record array (data at +4, count at +8, capacity at +0xc) through 00aabca0 when count+1 exceeds ca… |
| `00fd86d0` | — | unknown | Thiscall(this, obj*, id*, point*, normal*): grows the 48-byte record array through 00aabe70 when count+1 exceeds capacity, writes a record at data+count*0x30 (… |
| `00fd9610` | — | unknown | Calls 00fd0ca0 to initialise the GatherConvexHits callback at param_2 from the two 16-byte inputs and the filter args, then calls convexSweepTest (01125840) wi… |
| `00fd99f0` | — | unknown | Same flow as ConvexSweepSortHits_A (00fd9610): 00fd0ca0 init, convexSweepTest (01125840) on world=*(param_3+0xb0), heap-sort of the hit records (0x30 bytes, ke… |
| `00ff24d0` | — | unknown | Float function of three pointers (point, segment start, segment end): builds the segment direction and the offset from its start, forms a dot-product projectio… |
| `010a5910` | — | unknown | Returns a bool from 4 float-pointer arguments: it computes the box centre and half-extents from the first argument (two 4-float vectors, scaled by the constant… |

### `physics.shape`

**AppPhysics::BoxData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde160` | BoxData::CreateBulletShape | confirmed | Virtual slot 1 of BoxData. Allocates 0x50 bytes through 01160bf0 and constructs a btBoxShape (00fcb110) from the three dwords at this+0x10..+0x18 copied to a s… |
| `00fde200` | BoxData::ctor | confirmed | Stores the ShapeData vtable and then the BoxData vtable at offset 0, sets the shape type field at +4 to 2, copies four dwords from the argument into +0x10..+0x… |
| `00fdf300` | AppPhysics::BoxData::BoxData | confirmed | Constructor body: stores AppPhysics::ShapeData::vftable then AppPhysics::BoxData::vftable at +0, sets the shape type word at +4 to 2, and writes +0x10..+0x1c a… |

**AppPhysics::CapsuleData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde400` | CapsuleData::ctor | confirmed | Stores the ShapeData vtable, sets the type field at +4 to 3, stores the CapsuleData vtable at offset 0 and copies the arguments: two 4-dword vectors into +0x10… |
| `00fde500` | CapsuleData::CreateBulletShape | confirmed | Virtual slot 1 of CapsuleData. Copies vectors from this+0x10..+0x20 and +0x30..+0x3c into stack buffers, allocates 0xa0 bytes through 01160bf0 and constructs a… |

**AppPhysics::CompoundData** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdf3a0` | AppPhysics::CompoundData::CreateBulletShape | confirmed | Vtable slot [1]: allocates a 0x60-byte block via 01160bf0 (aligned 0x10); if it succeeds, constructs a Bullet compound via 0113cbb0(1). Walks the child list fr… |
| `00fdfea0` | AppPhysics::CompoundData::CompoundData | confirmed | Constructor: stores ShapeData vptr, sets type word +4 to 5, stores CompoundData vptr at +0 and the child-list vptr at +8, zeroes list head +0xc, tail +0x10 and… |
| `00fdff10` | AppPhysics::CompoundData::~CompoundData | confirmed | Destructor body (93 bytes): stores the child-list vptr at +8, clears the child list through 00fdf720, then restores the ShapeData vptr at +0. |
| `00fdff70` | AppPhysics::CompoundData::DeletingDtor | confirmed | Scalar-deleting destructor: calls 00fdff10 (dtor body), then _free(this) when flag bit 0 is set (30 bytes). |

**AppPhysics::ConeData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde5d0` | ConeData::ctor | confirmed | Stores the ShapeData vtable, sets the type field at +4 to 4, stores the ConeData vtable at offset 0, and copies the two arguments to +8 and +0xc. |
| `00fde6a0` | ConeData::CreateBulletShape | confirmed | Virtual slot 1 of ConeData. Allocates through 01160bf0 and constructs a btConeShape (011358b0) from this+8 and this+0xc; returns the shape or 0. |

**AppPhysics::CylinderData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde720` | CylinderData::ctor | confirmed | Stores the ShapeData vtable, sets the type field at +4 to 6, stores the CylinderData vtable at offset 0 and copies a 4-dword vector from the argument into +0x1… |
| `00fde7f0` | CylinderData::CreateBulletShape | confirmed | Virtual slot 1 of CylinderData. Allocates through 01160bf0 and constructs a btCylinderShape (01135c10); returns the shape or 0. |

**AppPhysics::ShapeData** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af1130` | AppPhysics::ShapeData::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): calls ShapeData::~ShapeData (00fde150), then calls _free(this) when bit 0 of the flag argument is set, … |
| `00fde150` | ShapeData::~ShapeData | confirmed | Stores the ShapeData vtable (0181b8bc) at offset 0 and nothing else. |
| `00fdee70` | ShapeData::DeletingDtor | confirmed | Stores the ShapeData vtable at offset 0, runs the EH-free destructor tail (the free thunk 00d82463 when bit 0 of the stack argument is set) and returns this. |
| `00fdeed0` | ShapeData::DeletingDtor | confirmed | Stores the ShapeData vtable at offset 0, frees this through the free thunk 00d82463 when bit 0 of the stack argument is set, and returns this. |
| `00fdef30` | AppPhysics::ShapeData::~ShapeData | confirmed | Scalar-deleting destructor body (91 bytes): stores AppPhysics::ShapeData::vftable at +0, then calls _free(this) when bit 0 of the flag byte argument is set. |
| `00fdef90` | AppPhysics::ShapeData::~ShapeData | confirmed | Scalar-deleting destructor body (91 bytes): stores AppPhysics::ShapeData::vftable at +0, then calls _free(this) when bit 0 of the flag byte argument is set. |
| `00fdeff0` | AppPhysics::ShapeData::~ShapeData | confirmed | Scalar-deleting destructor body (91 bytes): stores AppPhysics::ShapeData::vftable at +0, then calls _free(this) when bit 0 of the flag byte argument is set. |
| `00fde280` | ShapeData::~ShapeData | likely | Stores the ShapeData vtable at offset 0 under an EH frame and returns. |
| `00fde340` | ShapeData::~ShapeData | likely | Stores the ShapeData vtable at offset 0 under an EH frame and returns. |
| `00fde4b0` | ShapeData::~ShapeData | likely | Stores the ShapeData vtable at offset 0 under an EH frame and returns. |
| `00fde650` | ShapeData::~ShapeData | likely | Stores the ShapeData vtable at offset 0 under an EH frame and returns. |
| `00fde7a0` | ShapeData::~ShapeData | likely | Stores the ShapeData vtable at offset 0 under an EH frame and returns. |

**AppPhysics::SphereData** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fde2d0` | SphereData::ctor | confirmed | Stores the ShapeData vtable, sets the shape type field at +4 to 1, stores the SphereData vtable at offset 0 and copies the argument to +8 (radius). |
| `00fde390` | SphereData::CreateBulletShape | confirmed | Virtual slot 1 of SphereData. Allocates 0x40 bytes through 01160bf0 (alignment 0x10) and constructs a btSphereShape (00fcbe20) with the radius at this+8; retur… |

**SoeUtil::List<AppPhysics::CompoundData::ChildShapeData,-1>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fdfe60` | ChildShapeListDtorBody | confirmed | Non-deleting destructor body: stores the list vptr at +0, then clears the list through 00fdf720 (11 bytes). |
| `00fdfe70` | ChildShapeListDeletingDtor | confirmed | Scalar-deleting destructor: stores the list vptr, clears the list through 00fdf720, then calls _free(this) when flag bit 0 is set (36 bytes). |
| `00fdf260` | ChildShapeListAllocNode | likely | Vtable slot [2]: calls 006fb9c0, then either the virtual slot +4 of the object returned by 006fb860 (or of DAT_01bdf280) with pushed arguments (0x60, 0x10), or… |
| `00fdf2a0` | ChildShapeListFreeNode | likely | Vtable slot [3]: calls 00401c00 with the node pointer argument and alignment 0x10 (a free with alignment, 18 bytes). |

### `physics.ship`

**AppPhysics::ShipPhysics** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff0e60` | ShipPhysics::ScalarDeletingDtor | confirmed | Scalar deleting destructor (slot [0]): calls 00ff0cb0 on this, tests bit 0 of the flag argument and, when it is set, calls _free on this; returns this. |
| `00fee000` | AppPhysics::ShipPhysics::BuildBodies | likely | Thiscall (ECX = ShipPhysics, two stack arguments). Releases any existing chassis RigidBody at this+0x14 (00fd80f0 unregister, then refcount at +4 and vtable +4… |
| `00ff0cb0` | — | unknown | Non-deleting destructor body: stores AppPhysics::ShipPhysics::vftable at [this], calls 00fdade0 on this, calls the virtuals at +0x40 and +0x60 of the object at… |
| `00ff1080` | — | unknown | Thiscall constructor: stores AppPhysics::ShipPhysics::vftable at [this], stores param_5 at +0x10, zeroes many fields and sets +0x34..+0x44 to the default const… |

**SoeUtil::List<ShipCombatPhysicsGearConfig,2>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ff0dc0` | ListShipCombatPhysicsGearConfig::Ctor | likely | Constructor: stores SoeUtil::List<ShipCombatPhysicsGearConfig,-1>::vftable and then SoeUtil::List<ShipCombatPhysicsGearConfig,2>::vftable at [this], zeroes the… |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fefa30` | ChassisSteerTargetStep | speculative | Thiscall (ECX = ship physics object, three stack arguments). Returns at once unless the chassis at this+0x14 passes its predicate (vtable +0x18). Computes a fo… |
| `00a67040` | — | unknown | Destructor of the ship physics/state struct: rewrites the SoeUtil::IString vtable at +0x1bc, releases the shared string buffer when the refcount at +0x1c8 drop… |
| `00ff0e80` | — | unknown | Copy constructor of a 0x1c0-byte state struct: copies dwords 0..0x2b from the source, calls 00964840 on the source at +0xb0, constructs the embedded gear list … |

## Source shards

[[S0163|S0163]] (3), [[S0165|S0165]] (5), [[S0166|S0166]] (1), [[S0177|S0177]] (7), [[S0302 00a62280-00a67470|S0302]] (1), [[S0315 00aa90e0-00aae6b0|S0315]] (2), [[S0316 00aae6e0-00ab2c20|S0316]] (1), [[S0326 00ae4fa0-00ae9b00|S0326]] (1), [[S0328|S0328]] (6), [[S0380 00bf5ff0-00bf9410|S0380]] (1), [[S0415 00cc1a00-00cc6bc0|S0415]] (1), [[S0440|S0440]] (1), [[S0549|S0549]] (44), [[S0551|S0551]] (31), [[S0552|S0552]] (70), [[S0553|S0553]] (68), [[S0554|S0554]] (12), [[S0555|S0555]] (25), [[S0556|S0556]] (8), [[S0558|S0558]] (4), [[S0587|S0587]] (3), [[S0591|S0591]] (2), [[S0620|S0620]] (37)
