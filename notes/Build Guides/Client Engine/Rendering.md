---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Rendering

Part of [[Build Guides]] › Client Engine. Related: [[Animation]], [[Assets and Pack Files]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

2163 accepted functions: 871 confirmed, 548 likely, 432 speculative, 312 unknown. Network-linked (`wire`): 1.

Subsystems: `engine.displayaction` (480), `engine.capture` (254), `engine.effect` (236), `render.particle` (161), `engine.render` (133), `render.emitter` (103), `engine.display` (102), `engine.screenshot` (99), `render.effect` (72), `engine.overhead` (66), `render.command` (59), `render.occlusion` (49), `lib.umbra.impl` (48), `render` (46), `lib.umbra` (43), `render.material` (31), `render.primitive` (25), `render.pipeline` (23), `render.sky` (19), `render.batch` (18), `lib.umbra.refcount` (16), `render.light` (15), `lib.umbra.services` (7), `lib.dpvs.model` (7), `render.target` (6), `lib.umbra.object` (6), `lib.umbra.commander` (6), `render.fog` (4), `render.layout` (4), `lib.dpvs.object` (4), `render.debug` (3), `render.path_debug` (3), `lib.umbra.model` (3), `render.entity` (2), `lib.umbra.camera` (2), `lib.dpvs.refcount` (2), `lib.dpvs.camera` (2), `lib.dpvs.cell` (2), `render.supersample` (1), `lib.umbra.info` (1)

_Large or library guide: only confirmed and likely functions are listed (1419 of 2163)._

### `engine.capture`

**AudioCapture::FRAudioCaptureCallbackHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010f3270` | AudioCapture::FRAudioCaptureCallbackHandler::FRAudioCapture… | confirmed | Thiscall constructor. Stores the AudioCapture::ICallback vftable, sets this[1] = 0, then stores the FRAudioCaptureCallbackHandler vftable (0188cd70) and this[2… |
| `010f34c0` | AudioCapture::FRAudioCaptureCallbackHandler::FRAudioCapture… | confirmed | Thiscall constructor, second form: stores the ICallback vftable (0188cd48) then the FRAudioCaptureCallbackHandler vftable (0188cd70), sets this[1] = param_2 an… |
| `010f3520` | AudioCapture::FRAudioCaptureCallbackHandler::~FRAudioCaptur… | confirmed | Thiscall scalar deleting destructor under an SEH frame: stores the FRAudioCaptureCallbackHandler vftable (0188cd70) then the ICallback vftable (0188cd48), and … |
| `010f32d0` | AudioCapture::FRAudioCaptureCallbackHandler::~FRAudioCaptur… | likely | Thiscall destructor body under an SEH frame (scope state -1, handler 0164d538). Stores the FRAudioCaptureCallbackHandler vftable (0188cd70) and then the AudioC… |

**AudioCapture::ICallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010f2cf0` | AudioCapture::ICallback::~ICallback | confirmed | Thiscall scalar deleting destructor: stores the AudioCapture::ICallback vftable (0188cd48) into this, and when bit 0 of the byte argument is set calls _free(th… |
| `010f2ce0` | AudioCapture::ICallback::ResetVptr | likely | Stores the AudioCapture::ICallback vftable (0188cd48) into the object at ECX (mov [ecx], imm32; ret). No other effect and no return value. |

**CaptureCommon::Buffer** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8df30` | CaptureCommon::Buffer::DtorBody | confirmed | Destructor body (does not free this): stores the Buffer vftable, frees the data block at this+0xc through _free (00d83ac6), then stores the IBuffer and IConstB… |
| `00f8dfa0` | CaptureCommon::Buffer::GetDataPtr | confirmed | Returns the dword at this+0xc (the data block pointer that the Buffer destructor body frees). |
| `00f8dfb0` | CaptureCommon::Buffer::GetDataPtr | confirmed | Returns the dword at this+0xc; same body as 00f8dfa0 installed at a second vtable slot. |
| `00f8dfc0` | CaptureCommon::Buffer::GetWordAt10 | confirmed | Returns the dword at this+0x10 (the value the Buffer constructor 00f8e3f0 stores from its size argument). |
| `00f8dfd0` | CaptureCommon::Buffer::GetWordAt14 | confirmed | Returns the dword at this+0x14 (the byte count stored by CopyIn and SetWordAt14). |
| `00f8dfe0` | CaptureCommon::Buffer::SetWordAt14 | confirmed | Stores arg1 as the dword at this+0x14 and returns. |
| `00f8dff0` | CaptureCommon::Buffer::CopyIn | confirmed | Copies arg2 bytes from the source pointer (arg1) into the data block at this+0xc with memcpy (00d826f0), then stores the byte count at this+0x14. |
| `00f8e1a0` | CaptureCommon::Buffer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the Buffer destructor body 00f8df30, then _free(this) only when bit 0 of the flag argument is set. |
| `00f8e3f0` | CaptureCommon::Buffer::Ctor | confirmed | Constructor body: stores the IConstBuffer, IBuffer and Buffer vftables, increments the global id counter DAT_01cb40b8 and stores it at +4, stores arg1 at +8, a… |
| `00f8e960` | CaptureCommon::Buffer::ReleaseToPool | likely | Virtual slot 1 of the Buffer: calls the pool at this+8 (ECX) through 00f8e7e0 with this as the buffer argument. |

**CaptureCommon::IBuffer** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8de60` | CaptureCommon::IBuffer::DtorBody | confirmed | Destructor body (does not free this): stores the IBuffer vftable at this, then the IConstBuffer vftable over it. |
| `00f8deb0` | CaptureCommon::IBuffer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the IBuffer vftable and then the IConstBuffer vftable at this, and calls _free(this) only when bit 0 of the flag argument is… |
| `00f8e1c0` | CaptureCommon::IBuffer::Ctor | confirmed | Constructor body (interface part): stores the IConstBuffer vftable at this, then the IBuffer vftable over it. |

**CaptureCommon::IConstBuffer** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8de30` | CaptureCommon::IConstBuffer::DtorBody | confirmed | Destructor body (does not free this): stores the CaptureCommon::IConstBuffer vftable at this. |
| `00f8de40` | CaptureCommon::IConstBuffer::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the IConstBuffer vftable at this and calls _free(this) only when bit 0 of the flag argument is set. |

**CaptureCommon::JobQueueCallbackHandlerT<VideoCapture::JPG::JPGEncodeJobRec>** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01007cd0` | JobQueueCallbackHandlerT_JPGEncodeJobRec::Ctor | confirmed | Stores the SoeUtil::JobQueueHandler vftable (0181ab94) at this+0, then the CaptureCommon::JobQueueCallbackHandlerT<VideoCapture::JPG::JPGEncodeJobRec> vftable … |
| `01007d40` | JobQueueCallbackHandlerT_JPGEncodeJobRec::DtorBody | confirmed | Stores the derived vftable (0187ceb0) at [ECX], calls 00fc4da0 (HandleClose) on the handle at this+8, then restores the SoeUtil::JobQueueHandler vftable (0181a… |
| `01007da0` | CaptureCommon::JobQueueCallbackHandlerT_JPGEncodeJobRec::Si… | confirmed | Virtual slot 3 of the JPEG encode job handler vtable (0187ceb0). Calls 00fc4db0 (ReleaseSemaphore label) on the object (the semaphore handle at this+8) with co… |
| `01007dc0` | CaptureCommon::JobQueueCallbackHandlerT_JPGEncodeJobRec::Sc… | confirmed | Calls 01007d40 (JPEG encode handler dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**CaptureCommon::JobQueueCallbackHandlerT<VideoCapture::JPG::JPGWriteJobRec>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01007e10` | JobQueueCallbackHandlerT_JPGWriteJobRec::Ctor | confirmed | Stores the SoeUtil::JobQueueHandler vftable at this+0, then the CaptureCommon::JobQueueCallbackHandlerT<VideoCapture::JPG::JPGWriteJobRec> vftable, stores the … |
| `01007e80` | JobQueueCallbackHandlerT_JPGWriteJobRec::DtorBody | confirmed | Stores the derived vftable of the JPGWriteJobRec handler at [ECX], calls 00fc4da0 (HandleClose) on the semaphore handle at this+8, then restores the SoeUtil::J… |
| `01007f00` | CaptureCommon::JobQueueCallbackHandlerT_JPGWriteJobRec::Sca… | likely | Calls 01007e80 (JPGWriteJobRec handler dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**CaptureCommon::JobT<VideoCapture::JPG::JPGEncodeJobRec>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01008da0` | JobT_JPGEncodeJobRec::Ctor | confirmed | Calls 0083f1e0 (JobCtor label, base constructor) on the new object, stores the CaptureCommon::JobT<VideoCapture::JPG::JPGEncodeJobRec> vftable (0187cee0) at th… |
| `01008e10` | CaptureCommon::JobT_JPGEncodeJobRec::DtorBody | confirmed | Stores the JobT vftable (0187cee0) at [ECX], releases the object at this+0x5c through its vtable slot +4 when non-null, then stores the SoeUtil::Job vftable (0… |
| `01008e90` | CaptureCommon::JobT_JPGEncodeJobRec::ScalarDeletingDtor | likely | Calls 01008e10 (JobT dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**CaptureCommon::JobT<VideoCapture::JPG::JPGWriteJobRec>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01008f00` | CaptureCommon::JobT_JPGWriteJobRec::Ctor | confirmed | Calls 0083f1e0 (JobCtor label, base constructor) on the new object, stores the CaptureCommon::JobT<VideoCapture::JPG::JPGWriteJobRec> vftable (0187cefc) at thi… |
| `01008fe0` | CaptureCommon::JobT_JPGWriteJobRec::ScalarDeletingDtor | likely | Calls 01008f70 (JobT write dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::DestFileDoneParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01005660` | VideoCapture::DestFileDoneParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the DestFileDoneParams vftable (0187cc40) at this+0, then copy-constructs an IString-layout member at this+8 from… |
| `010056d0` | VideoCapture::DestFileDoneParams::DtorBody | confirmed | Stores the DestFileDoneParams vftable at [ECX], calls 01001d70 (IString dtor body) on the member at this+8, then restores the FuncParams vftable (0187cbd0). |
| `01005730` | VideoCapture::DestFileDoneParams::ScalarDeletingDtor | confirmed | Calls 010056d0 (DestFileDoneParams dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::EncoderComp** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01001460` | VideoCapture::EncoderComp::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag. Stores VideoCapture::EncoderComp::vftable at this+0; if flag bit 0 is set calls _free (00d82463) on this. Returns … |

**VideoCapture::EncoderRes** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01001320` | VideoCapture::EncoderRes::Ctor | confirmed | Fastcall (ECX=this). Stores VideoCapture::EncoderRes::vftable at this+0, zeroes the dwords at this+4, +8 and +0xc, and clears the bytes at this+0x10 and +0x11. |
| `01001370` | VideoCapture::EncoderRes::DtorBody | confirmed | Fastcall (ECX=this). Stores VideoCapture::EncoderRes::vftable at this+0 and returns. Frees nothing. |
| `010013c0` | VideoCapture::EncoderRes::CopyCtor | confirmed | Thiscall with one pointer argument (the source record). Stores VideoCapture::EncoderRes::vftable at this+0, copies the dwords at source+4, +8 and +0xc, and cop… |
| `01001480` | VideoCapture::EncoderRes::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag. Stores VideoCapture::EncoderRes::vftable at this+0; if flag bit 0 is set calls _free (00d82463) on this. Returns t… |
| `010014a0` | EncoderResPreset | likely | Static function with one int argument (index). On first use fills a static table of 22 VideoCapture::EncoderRes records (vtable, id, width, height, two byte fl… |

**VideoCapture::EncoderStateChangeParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01003830` | VideoCapture::EncoderStateChangeParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) at this+0, stores two argument dwords at +4 and +8, then stores the EncoderStateChangeParams vftable (0187cbf0) at +0 … |
| `010038b0` | VideoCapture::EncoderStateChangeParams::DtorBody | confirmed | Stores the EncoderStateChangeParams vftable at [ECX], then restores the FuncParams vftable (0187cbd0). Nothing is freed. |
| `01003940` | VideoCapture::EncoderStateChangeParams::ScalarDeletingDtor | confirmed | Stores the EncoderStateChangeParams vftable, then the FuncParams vftable, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns th… |

**VideoCapture::EncoderThreadImpl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01005750` | VideoCapture::EncoderThreadImpl::Ctor | confirmed | Calls 00fc4cc0 (SoeUtil::Thread base ctor, stack 0x4000, priority 2), stores the EncoderThreadImpl vftable (0187cc48) at this+0, initialises four guards with s… |
| `01005bb0` | VideoCapture::EncoderThreadImpl::DtorBody | confirmed | Stores the EncoderThreadImpl vftable (0187cc48) at [ECX], then tears down members in reverse order: three empty stub calls, the deque destructor 010055d0 twice… |
| `01005cc0` | VideoCapture::EncoderThreadImpl::ScalarDeletingDtor | confirmed | Calls 01005bb0 (EncoderThreadImpl dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::EncoderThreadStateChangeParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010036e0` | VideoCapture::EncoderThreadStateChangeParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the EncoderThreadStateChangeParams vftable (0187cbe8) at this+0, and stores two argument dwords at this+4 and thi… |
| `01003750` | VideoCapture::EncoderThreadStateChangeParams::DtorBody | confirmed | Stores the EncoderThreadStateChangeParams vftable at [ECX], then restores the FuncParams vftable (0187cbd0). Nothing is freed. |
| `010037c0` | VideoCapture::EncoderThreadStateChangeParams::ScalarDeletin… | confirmed | Stores the EncoderThreadStateChangeParams vftable, then the FuncParams vftable, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Retu… |

**VideoCapture::FREncoderThreadCallbackHandler** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010f31d0` | OnVideoCaptureEvent | confirmed | Virtual slot [2] of the FREncoderThreadCallbackHandler vftable (0188cd58). If the stored callback at this+8 is non-null, maps param_3 to a trace name (1 Init, … |
| `010f3230` | OnVideoCaptureStatus | confirmed | Virtual slot [3] of the FREncoderThreadCallbackHandler vftable (0188cd58). If the callback at this+8 is set, computes __alldiv(64-bit value at param_2+0x10, di… |
| `010f33e0` | VideoCapture::FREncoderThreadCallbackHandler::FREncoderThre… | confirmed | Thiscall constructor. Stores the VideoCapture::IEncoderThreadCallback vftable (0187c92c), then the VideoCapture::FREncoderThreadCallbackHandler vftable (0188cd… |
| `010f3450` | VideoCapture::FREncoderThreadCallbackHandler::~FREncoderThr… | confirmed | Thiscall scalar deleting destructor under an SEH frame: stores the FREncoderThreadCallbackHandler vftable (0188cd58) then the IEncoderThreadCallback vftable (0… |
| `010f38a0` | OnVideoCaptureDestFileDone | confirmed | Virtual slot [4] of the FREncoderThreadCallbackHandler vftable (0188cd58), about 1.3 KB under an SEH frame. If this[1] is nonzero it scans the path in param_2 … |

**VideoCapture::FuncParams** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01003460` | VideoCapture::FuncParams::DtorBody | confirmed | Stores the VideoCapture::FuncParams vftable (0187cbd0) at [ECX] and returns. It does not free anything. |
| `01003470` | VideoCapture::FuncParams::ScalarDeletingDtor | confirmed | Stores the VideoCapture::FuncParams vftable at this+0; if flag bit 0 is set (if ((param_2 & 1) != 0)) calls _free on this. Returns this. |

**VideoCapture::IEncoder** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01006e60` | VideoCapture::IEncoder::DtorBody | confirmed | Stores the VideoCapture::IEncoder vftable at [ECX] and returns. Nothing else is done. |
| `01006e70` | VideoCapture::IEncoder::ScalarDeletingDtor | likely | Stores the IEncoder vftable at this+0, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::IEncoderThreadCallback** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010000e0` | IEncoderThreadCallback::DtorBody | confirmed | Fastcall (ECX=this). Stores VideoCapture::IEncoderThreadCallback::vftable at this+0 and returns. Frees nothing. |
| `010000f0` | IEncoderThreadCallback::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag. Stores VideoCapture::IEncoderThreadCallback::vftable at this+0; if flag bit 0 is set calls _free (00d82463) on thi… |

**VideoCapture::InitEncoderParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01003f60` | VideoCapture::InitEncoderParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the InitEncoderParams vftable (0187cc0c) at this+0, then copies 0x1a (26) dwords (0x68 bytes) from the source arg… |
| `01003fd0` | VideoCapture::InitEncoderParams::DtorBody | confirmed | Stores the InitEncoderParams vftable at [ECX], calls 010013a0 (bare return stub), then restores the FuncParams vftable (0187cbd0). |
| `01004030` | VideoCapture::InitEncoderParams::ScalarDeletingDtor | confirmed | Calls 01003fd0 (dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::ReinitEncoderParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010034b0` | VideoCapture::ReinitEncoderParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the ReinitEncoderParams vftable (0187cbd8) at this+0; copies 0x1a (26) dwords (0x68 bytes) from the source argume… |
| `01003520` | VideoCapture::ReinitEncoderParams::DtorBody | confirmed | Stores the ReinitEncoderParams vftable at [ECX], calls 010013a0 (a bare return stub), then restores the FuncParams vftable (0187cbd0) at [ECX]. |
| `01003590` | VideoCapture::ReinitEncoderParams::ScalarDeletingDtor | confirmed | Calls 01003520 (dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::StartEncoderParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01004c60` | VideoCapture::StartEncoderParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the StartEncoderParams vftable (0187cc38) at this+0. Sets up an embedded SoeUtil::IString<char> at this+4 (vftabl… |
| `01004ce0` | VideoCapture::StartEncoderParams::DtorBody | confirmed | Stores the StartEncoderParams vftable at this+0 and the IString vftable at +4. If the buffer count at this+0x10 is positive, atomically decrements the referenc… |
| `01004d70` | VideoCapture::StartEncoderParams::ScalarDeletingDtor | confirmed | Calls 01004ce0 (StartEncoderParams dtor body) on this, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |

**VideoCapture::StopEncoderParams** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010035b0` | VideoCapture::StopEncoderParams::Ctor | confirmed | Stores the FuncParams vftable (0187cbd0) then the StopEncoderParams vftable (0187cbe0) at this+0, and stores the argument dword at this+4. |
| `01003610` | VideoCapture::StopEncoderParams::DtorBody | confirmed | Stores the StopEncoderParams vftable at [ECX], then restores the FuncParams vftable (0187cbd0). Nothing is freed. |
| `01003670` | VideoCapture::StopEncoderParams::ScalarDeletingDtor | confirmed | Stores the StopEncoderParams vftable at this+0, then the FuncParams vftable, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns… |

**VideoCapture::VideoCaptureManagerImpl_SingleUse** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01000810` | VideoCaptureManagerImpl_SingleUse::Ctor | confirmed | Fastcall constructor with an SEH frame (ECX=this). Stores VideoCapture::IEncoderThreadCallback::vftable and then VideoCapture::VideoCaptureManagerImpl_SingleUs… |
| `01000ad0` | VideoCaptureManagerImpl_SingleUse::DtorBody | confirmed | Fastcall (ECX=this) with an SEH frame. Stores VideoCapture::VideoCaptureManagerImpl_SingleUse::vftable, frees the block at this+0x1a8 with 01003c30 and _free a… |
| `01000bf0` | VideoCaptureManagerImpl_SingleUse::ScalarDeletingDtor | confirmed | Virtual slot 0, thiscall with one byte flag. Calls the dtor body 01000ad0 on this, then if flag bit 0 is set calls _free (00d82463) on this. Returns this. |
| `01000a90` | VideoCaptureManagerImpl_SingleUse::Slot1SetDword21c | likely | Virtual slot 1, thiscall with two stack arguments (ret 8). Pushes the second argument and calls 01000910 with ECX = this, which stores it at +0x21c. |
| `01000aa0` | VideoCaptureManagerImpl_SingleUse::Slot2SetDwords220And224 | likely | Virtual slot 2, thiscall with four stack arguments (ret 0x10). Calls 01000a10 with the fourth argument when it is nonzero (stores +0x224), then calls 01000990 … |

**(no class)** (66)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f8e5c0` | CaptureCommon::PoolInit | confirmed | Constructor body for a capture buffer pool: stores a new numeric id (DAT_01cb40bc) at +0, the arg1 at +4 (buffer size used by acquire) and arg2 at +8 (buffer l… |
| `00f8e9b0` | CapturePoolDtorBody | confirmed | Pool destructor body: calls each buffer in the all-buffers list (+0x54, count +0x58) through its vtable slot 0 with flag 1 (scalar deleting destructor), releas… |
| `00f8e660` | CaptureCommon::PoolAcquire | likely | Takes the pool guard at this+0x10. Clears the output reference *arg1 through vtable slot 1 when set. If the free list (+0x68 count) is empty: returns -1 when t… |
| `00f8e7e0` | CaptureCommon::PoolRelease | likely | Takes the pool guard at this+0x10. If the buffer's refcount at +0x18 is zero it calls the buffer's vtable slot 6 with 0 (clears its length) and pushes the buff… |
| `00f8e8a0` | CaptureCommon::PoolAddRefAssign | likely | Takes the pool guard at this+0x10, increments the refcount at +0x18 of the buffer arg1, releases the previous pointer in arg2 through its vtable slot 1, stores… |
| `01000290` | DeleteOwnedObject_01000290 | likely | Fastcall (ECX=&field): if the pointer stored at *ECX is non-null, loads its vtable and calls the vtable slot 0 with argument 1 (the scalar deleting destructor,… |
| `01000750` | OwnedBlocksDtorBody | likely | Fastcall (ECX=this) with an SEH frame: if this+8 is non-null, calls 01003c30 on it, frees it with _free and zeroes this+8; if this+0 is non-null, calls 0100338… |
| `01000da0` | VideoCaptureManagerImpl_SingleUse::Create | likely | Fastcall (ECX = output slot pointer) with an SEH frame. Allocates 0x228 bytes with 00d833a6 (operator new). When the allocation succeeds it constructs the obje… |
| `010019b0` | EncoderResPresetSelect | likely | Cdecl-style function with three stack arguments (param_1 = width, param_2 = height, param_3 = index cap). First moves the cap down past presets whose flag byte… |
| `01001e30` | DumpEncoderCompFields | likely | Thiscall (ECX=this, one stream argument). Writes the text block EncoderComp_BEG, then the fcc line (field +4 with the name from 010068d0), minQuality (+8), max… |
| `01001ec0` | DumpEncoderCompToLocalString | likely | Void, no arguments. Builds a local SoeUtil::IString<char> (vtable, empty buffer), reserves DAT_01879fdc+1 bytes through 00401df0 when that is positive, calls 0… |
| `01001f80` | DumpEncoderResFields | likely | Thiscall (ECX=this, one stream argument). Writes EncoderRes_BEG, encRes (+4), width (+8), height (+0xc), wideScreen (byte +0x10 as true/false) and compressed (… |
| `01002020` | DumpEncoderResToLocalString | likely | Void, no arguments. Builds a local SoeUtil::IString<char>, reserves DAT_01879fdc+1 bytes through 00401df0 when positive, calls 01001f80 on it, then releases th… |
| `010020e0` | DumpEncoderPropsFields | likely | Thiscall (ECX = settings block, one stream argument). Writes EncoderProps_BEG, then about 26 'name=value' lines for the settings dwords (srcFmtVid, srcScan, en… |
| `010022d0` | DumpEncoderPropsToLocalString | likely | Void, no arguments. Builds a local SoeUtil::IString<char>, reserves DAT_01879fdc+1 bytes through 00401df0 when positive, calls 010020e0 (the EncoderProps dump)… |
| `01002390` | DumpEncoderLimitsFields | likely | Thiscall (ECX=this, one stream argument). Writes EncoderLimits_BEG, secDuration (+0), minPcntMemoryAvailable (+4), minDiskMBytesAvailable (+8) and EncoderLimit… |
| `010023f0` | DumpEncoderLimitsToLocalString | likely | Void, no arguments. Builds a local SoeUtil::IString<char>, reserves DAT_01879fdc+1 bytes through 00401df0 when positive, calls 01002390 on it, then releases th… |
| `010024b0` | DumpEncoderStatusFields | likely | Thiscall (ECX=this, one stream argument). Writes EncoderStatus_BEG, state (string via 01006620), frameCount (64-bit at +8), nsecElapsed (64-bit at +0x10), pcnt… |
| `010025f0` | DumpDestFileStatsFields | likely | Thiscall (ECX=this, one stream argument). Writes DestFileStatsBEG, filename (string at +4), width (+0x10), height (+0x14), size (64-bit +0x18), creationTime (+… |
| `01002920` | EncoderStart | likely | Thiscall (ECX=this, one argument = settings pointer). Returns -5 unless the state at +0x98 is 1, 2 or 5. Calls 01001a70 with ECX = the settings pointer and ret… |
| `01002990` | EncoderStop | likely | Fastcall (ECX=this). When the state at +0x98 is neither 0 nor 6: for states 3 and 4 calls virtual slot 9 (+0x24) of the object at +0x88; then calls virtual slo… |
| `010029f0` | EncoderAdvanceTo3 | likely | Thiscall (ECX=this, one argument). Returns -5 unless the state at +0x98 is 1, 2 or 5. Calls virtual slot 3 (+0xc) of the object at +0x88 with the argument; whe… |
| `01002a40` | EncoderStateTo5 | likely | Fastcall (ECX=this). When the state at +0x98 is 3 or 4 calls virtual slot 9 (+0x24) of the object at +0x88 and sets the state to 5 through 010027e0. Otherwise … |
| `01002a70` | EncoderFinishInState5 | likely | Thiscall (ECX=this, one argument) under a lock guard. Returns -5 (0xfffffffb) unless the state at +0x98 is 5; otherwise calls virtual slot 10 (+0x28) of the ob… |
| `01002b20` | EncoderForwardSlot4In34 | likely | Thiscall (ECX=this, one argument) under a lock guard. Returns -5 unless the state at +0x98 is 3 or 4; otherwise calls virtual slot 4 (+0x10) of the object at +… |
| `01002bd0` | EncoderForwardSlot6In34 | likely | Thiscall (ECX=this, one argument) under a lock guard. Returns -5 unless the state at +0x98 is 3 or 4; otherwise calls virtual slot 6 (+0x18) of the object at +… |
| `01002c80` | EncoderSubmitInState4 | likely | Thiscall (ECX=this, one argument) under a lock guard. Returns -1 unless the state at +0x98 is 4; otherwise calls virtual slot 8 (+0x20) of the object at +0x88 … |
| `01002d50` | EncoderAdvanceTo3Indirect | likely | Fastcall-like with one stack argument (ret not inferred from the shown pseudocode). Loads the object from [ECX], returns -5 unless its state at +0x98 is 1, 2 o… |
| `01002dd0` | EncoderStateTo5Indirect | likely | Loads the pointer at [ecx] into ESI. If the state word at +0x98 is 3 or 4, calls virtual slot +0x24 of the object at +0x88, then calls 010027e0 (GuardedSetDwor… |
| `01002e20` | ReleaseHeldObject | likely | Loads the pointer at [ecx]; if non-null, calls its vtable slot 0 with flag 1 (the scalar deleting-destructor pattern). It does not clear the holder field. |
| `01002e50` | EncoderStateInit | likely | Calls 00fc4580 with spin count 4000 on the object (ECX=this), which initialises the critical section at this+0. Then zeroes the dwords at +0x88 (backend), +0x9… |
| `01002ec0` | EncoderStateDtorBody | likely | Switches on state at +0x98: states 3 and 4 call 01002a40 then fall to 01002990; states 1, 2 and 5 call 01002990 only (jump table entries checked in bytes); oth… |
| `01002f60` | EncoderOfferGuardedSlot5 | likely | Stdcall-style with one stack argument (ret 4): the held object at *arg (ESI=this, EDI=arg). Calls its vtable slot +0x10 with no argument; if that returns 0 it … |
| `01003080` | EncoderStatusFill | likely | Zeroes eight dwords via 01001440, then writes into the out-record at arg: [0] = state from 01002850 (GuardedGetDword98), [2] and [3] = the 64-bit counter at th… |
| `01003140` | EncoderStateNewIntoHolder | likely | Allocates 0xa0 bytes with 00d833a6 (operator new). If the allocation is non-null, calls 01002e50 with the block as this and stores its result; otherwise stores… |
| `010031b0` | EncoderStateScalarDeletingDtor | likely | Calls 01002ec0 (encoder state dtor body) with this in ECX, then calls _free on this when flag bit 0 is set (if ((param_2 & 1) != 0)). Returns this. |
| `010031e0` | EncoderCheckThenForwardSlot7 | likely | Calls vtable slot +0x10 of the object at *param_2 (ECX=that object). If it returns 0, calls slot +4 of that object to release it, clears *param_2 and returns -… |
| `01003240` | HolderReplaceReleasing | likely | Stdcall-style with one stack argument (ret 4). If the holder at [ECX] is non-null, calls its vtable slot 0 with flag 1, then stores the argument into the holde… |
| `01003270` | EncoderStartBackend | likely | If the state at +0x98 is 0 or 6, calls 01001a70 on the settings block (arg); if that fails returns -1. When settings +0x38 equals 1: allocates 0x578 bytes with… |
| `01003380` | EncoderStateDestroyHeld | likely | If the pointer at [ECX] is non-null, calls 01002ec0 (encoder state dtor body) with it as this, then calls _free on the same block. |
| `01004060` | ThreadStateSetOneAndStart | likely | Enters the guard (00fc45f0), reads the state dword at this+0x3e0 and stores 1 there, leaves the guard (00fc4610). If the sink at this+0x3c4 is non-null, calls … |
| `01004e60` | GuardedQueuePushAndSignal | likely | Enters the guard at this+0x1d8 (00fc45f0), appends the 8-byte element at the argument to the deque at this+0x328 through 01004bf0, calls 00fc4db0 (ReleaseSemap… |
| `01004ef0` | GuardedQueuePopFront | likely | Enters the guard at this+0x1d8. If the element count at this+0x350 is zero, writes two zero dwords to the output pointer. Otherwise reads the 8-byte element at… |
| `01004fd0` | GuardedQueuePushNoSignal | likely | Stores ECX in EDI, enters the guard at this+0x260 (00fc45f0), appends the 8-byte element at the argument to the deque at this+0x35c through 01004bf0, and leave… |
| `01005050` | GuardedQueuePopFrontAt260 | likely | Enters the guard at this+0x260. If the element count at this+0x384 is zero, writes two zero dwords to the output pointer. Otherwise reads the 8-byte element at… |
| `01005130` | StateChangeMessagePost | likely | Reads the state of the object at this+0x3c0 (through 010028c0). If it is 0 or 6, starts the backend with 010033a0 (EncoderStartBackend wrapper) on the settings… |
| `010051f0` | StateChangeMessagePostRunning | likely | Reads the state of the object at this+0x3c0 through 010028c0. If it is 1, 2 or 5, calls 01002d40 (wrapper to 01002920) with the settings block at (arg+4)+8 and… |
| `010052c0` | GuardedStatusAdvanceAndPost | likely | Reads the state at this+0x3c0 through 010028c0 and returns unless it is 1, 2 or 5. Enters the guard (00fc45f0), reads limits at this+0x3d8 and this+0x3dc, and … |
| `01005430` | StateChangeMessagePostStop | likely | Reads the state through 010028c0. If it is 1, 2 or 5, calls 01002e10 (wrapper that tail-jumps to 01002990, EncoderStop), allocates 0x14 bytes, constructs an En… |
| `010054f0` | QueueDispatchTypes7to9 | likely | Pops elements from the second queue through 01005050 into a (type, payload) pair until the type is 0. For type 7 calls vtable slot +4 of the sink at this+0x3c4… |
| `010058d0` | ThreadStopRequest | likely | Enters the guard at this+0xc8 (00fc45f0). If the thread state at this+0x3e0 is 4 it leaves the guard and returns. Otherwise it sets the state to 3, leaves the … |
| `010059b0` | MessageBlockPostType1 | likely | Allocates 0x70 bytes with 00d833a6; if the allocation is non-null, constructs an InitEncoderParams copy of the argument in it through 01003f60. Then pushes an … |
| `01005a40` | MessageBlockPostType2 | likely | Allocates 0x70 bytes with 00d833a6; if non-null, constructs a ReinitEncoderParams copy of the argument through 010034b0. Pushes an 8-byte element (type word 2 … |
| `01005ad0` | MessageBlockPostType3 | likely | Allocates 0x14 bytes with 00d833a6; if non-null, constructs a StartEncoderParams from the string argument through 01004c60. Pushes an 8-byte element (type word… |
| `01005ba0` | EncoderPumpDispatchOnce | likely | Calls 010054f0 (QueueDispatchTypes7to9) to drain and dispatch the second queue, then calls 01003b70 (GuardedCopyAndNotifySink) on the same object. |
| `01005e60` | EncoderThreadRunLoop | likely | Pops the first queue through 01004ef0 and dispatches by the type word: 1 to 01005130, 2 to 010051f0, 3 to 010052c0, 4 to 01005ce0, 5 to 01005430. Each popped p… |
| `010064d0` | EncoderThreadNewIntoHolder | likely | Allocates 0x408 bytes with 00d833a6 (operator new). If the allocation is non-null, constructs an EncoderThreadImpl in it through 01005750 with the five stack a… |
| `01006600` | EncoderPumpDispatchOnceB | likely | Calls 010054f0 (QueueDispatchTypes7to9) and then 01003b70 (GuardedCopyAndNotifySink), the same body as 01005ba0 (EncoderPumpDispatchOnce). |
| `01007200` | BitmapInfoHeaderBuild | likely | Copies an 11-dword template at 0187cda4 into a local array, then sets the fields: [0] = 0x2c, [1] = width (+0x40), [2] = height (+0x44), a 16-bit planes value … |
| `01007270` | WaveFormatExBuild | likely | Writes an 18-byte PCM WAVEFORMATEX-style record to the output. When the code at this+0x1c is 1..4, the channel count (01006740 of +0x1c) and bits per sample (0… |
| `01007c10` | SemaphoreWaitLatched | likely | If the byte at this+0x18 is non-zero, returns 1. Otherwise calls 00fc4dd0 (SemaphoreWait) with the argument as the timeout, stores its byte result at this+0x18… |
| `01007c60` | SemaphoreWaitLatchedB | likely | Identical body to SemaphoreWaitLatched (01007c10): returns 1 when the byte at this+0x18 is set; otherwise calls 00fc4dd0 with the argument, stores the byte res… |
| `01008650` | GuardedSubmitObjectToBuffer | likely | Takes the object pointer from *arg (clearing the source). If the state at this+0x26c is 1 to 4: enters the guard (00fc45f0); calls the object's vtable slot +0x… |
| `010f3600` | InitVideoCaptureOutput | likely | Fastcall on this. Zeroes this[0..4], this[6], this[9] and this[10], sets this[5] = 3, writes the IString vftable (0175b24c) and the empty-string buffer 01cb9aa… |
| `010f37e0` | InitVideoCaptureOutputFromArgs | likely | Thiscall with six stack args (ret 0x18). Returns AL=0 unless param_2 == 8. Calls 010019b0(param_3, param_4, 3) and, when param_5 != 0, requires 010067c0(param_… |
| `010f4ac0` | WriteVideoInfoXml | likely | Fastcall, this = param_1. Builds an IString starting with '<VideoInfo> ', walks the tree at this+0x34 in order from its leftmost node and appends one '<%s>%s</… |

### `engine.display`

**Deep::AttachableEntity** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00765080` | Deep::AttachableEntity::Slot10ReattachAll | confirmed | Virtual method (thiscall, one stack argument): calls the base implementation 00764b70 on this with the argument, then walks the attachment list at this+0x18 (n… |
| `00766d80` | Deep::AttachableEntity::ctor | confirmed | Constructor body: calls 007648d0 (Deep::Entity constructor) on this, stores Deep::AttachableEntity::vftable at [this], calls 00766250 (the attachment list cons… |
| `00766e10` | Deep::AttachableEntity::~AttachableEntity | confirmed | Destructor body (fastcall): stores Deep::AttachableEntity::vftable, then while the attachment list head at this+0x18 is non-null calls slot [12] (+0x30) of eac… |
| `00766ea0` | Deep::AttachableEntity::ScalarDeletingDtor | confirmed | Scalar deleting destructor (30 bytes): calls 00766e10 (the AttachableEntity destructor body) on this, and if the low bit of the flag argument is set frees this… |
| `00764ed0` | Deep::AttachableEntity::UpdateAttachments | likely | Virtual method (thiscall, one stack argument, ret 4): if this+8 and this+4 are non-null it calls slot [1] of the object at this+8 with the argument; it calls t… |

**Deep::AttachableEntity::Attachment** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007651c0` | Deep::AttachableEntity::Attachment::Unlink | confirmed | Virtual method on an Attachment (fastcall, ECX = attachment, no stack arguments): unlinks it from its owner (owner = [attachment+4]) doubly linked list (next +… |

**Deep::Entity** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00764820` | Deep::Entity::ForwardChildSlot1 | confirmed | 33-byte virtual method: if this+8 and this+4 are both non-null it calls slot [1] (+4) of the object at this+8 with the stack argument; otherwise it does nothin… |
| `007648d0` | Deep::Entity::ctor | confirmed | Constructor body: zeroes the flag word at this+0x10 and +4, +8, +0xc, stores Deep::Entity::vftable at [this], sets flag bit 0 (and calls the global PTR_FUN_017… |
| `00764b30` | Deep::Entity::ClearFlag1AndNotify | confirmed | Virtual method: if bit 1 of the ushort at this+0x10 is set it clears it, calls 00735a30 when this+4 is non-null, then calls the virtual slot at +0x3c (vtable [… |
| `00764c50` | Deep::Entity::SetRefPtrAtC | confirmed | Virtual method (thiscall, one stack argument, ret 4): if the argument is non-null it increments its refcount at [arg+4] (LOCK) and calls its vtable slot [2] wi… |
| `00764e10` | Deep::Entity::~Entity | confirmed | Destructor body (called from 41 EH unwind funclets and from derived destructors): stores Deep::Entity::vftable, releases the refcounted pointer at this+0xc (de… |
| `00765440` | Deep::Entity::ScalarDeletingDtor | confirmed | Scalar deleting destructor (30 bytes): calls 00764e10 (Deep::Entity destructor body) on this, and if the low bit of the flag argument is set frees this with _f… |

**Engine::IDisplayActionHandler** (60)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0174b860` | Engine::IDisplayActionHandler::StaticInit_01cbd084 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd084, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174b9a0` | Engine::IDisplayActionHandler::StaticInit_01cbd078 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd078, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bad0` | Engine::IDisplayActionHandler::StaticInit_01cbd148 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd148, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bb20` | Engine::IDisplayActionHandler::StaticInit_01cbd188 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd188, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bb70` | Engine::IDisplayActionHandler::StaticInit_01cbd134 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd134, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bbc0` | Engine::IDisplayActionHandler::StaticInit_01cbd1cc | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd1cc, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bc10` | Engine::IDisplayActionHandler::StaticInit_01cbd128 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd128, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bc60` | Engine::IDisplayActionHandler::StaticInit_01cbd1d8 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd1d8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bcb0` | Engine::IDisplayActionHandler::StaticInit_01cbd158 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd158, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bd00` | Engine::IDisplayActionHandler::StaticInit_01cbd13c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd13c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bd50` | Engine::IDisplayActionHandler::StaticInit_01cbd12c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd12c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bda0` | Engine::IDisplayActionHandler::StaticInit_01cbd144 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd144, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bdf0` | Engine::IDisplayActionHandler::StaticInit_01cbd1b0 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd1b0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174be40` | Engine::IDisplayActionHandler::StaticInit_01cbd198 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd198, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174be90` | Engine::IDisplayActionHandler::StaticInit_01cbd17c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd17c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bee0` | Engine::IDisplayActionHandler::StaticInit_01cbd18c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd18c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bf30` | Engine::IDisplayActionHandler::StaticInit_01cbd16c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd16c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bf80` | Engine::IDisplayActionHandler::StaticInit_01cbd160 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd160, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174bfd0` | Engine::IDisplayActionHandler::StaticInit_01cbd1a0 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd1a0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c020` | Engine::IDisplayActionHandler::StaticInit_01cbd15c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd15c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c750` | Engine::IDisplayActionHandler::StaticInit_01cbd214 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd214, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c8e0` | Engine::IDisplayActionHandler::StaticInit_01cbd27c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd27c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c930` | Engine::IDisplayActionHandler::StaticInit_01cbd2e0 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd2e0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c980` | Engine::IDisplayActionHandler::StaticInit_01cbd2e8 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd2e8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174c9d0` | Engine::IDisplayActionHandler::StaticInit_01cbd28c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd28c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174cfc0` | Engine::IDisplayActionHandler::StaticInit_01cbd338 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd338, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d010` | Engine::IDisplayActionHandler::StaticInit_01cbd324 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd324, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d060` | Engine::IDisplayActionHandler::StaticInit_01cbd328 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd328, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d0b0` | Engine::IDisplayActionHandler::StaticInit_01cbd38c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd38c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d100` | Engine::IDisplayActionHandler::StaticInit_01cbd33c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd33c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d150` | Engine::IDisplayActionHandler::StaticInit_01cbd310 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd310, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174d1a0` | Engine::IDisplayActionHandler::StaticInit_01cbd334 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd334, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174dab0` | Engine::IDisplayActionHandler::StaticInit_01cbd44c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd44c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174dbf0` | Engine::IDisplayActionHandler::StaticInit_01cbd43c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd43c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174dce0` | Engine::IDisplayActionHandler::StaticInit_01cbd434 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd434, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e500` | Engine::IDisplayActionHandler::StaticInit_01cbd4c8 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd4c8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e5a0` | Engine::IDisplayActionHandler::StaticInit_01cbd560 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd560, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e5f0` | Engine::IDisplayActionHandler::StaticInit_01cbd564 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd564, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e640` | Engine::IDisplayActionHandler::StaticInit_01cbd52c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd52c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e690` | Engine::IDisplayActionHandler::StaticInit_01cbd540 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd540, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e6e0` | Engine::IDisplayActionHandler::StaticInit_01cbd524 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd524, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e730` | Engine::IDisplayActionHandler::StaticInit_01cbd528 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd528, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e780` | Engine::IDisplayActionHandler::StaticInit_01cbd518 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd518, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e870` | Engine::IDisplayActionHandler::StaticInit_01cbd584 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd584, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e8c0` | Engine::IDisplayActionHandler::StaticInit_01cbd5c4 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5c4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e910` | Engine::IDisplayActionHandler::StaticInit_01cbd5a0 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5a0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e960` | Engine::IDisplayActionHandler::StaticInit_01cbd57c | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd57c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174e9b0` | Engine::IDisplayActionHandler::StaticInit_01cbd594 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd594, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ea00` | Engine::IDisplayActionHandler::StaticInit_01cbd5b8 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5b8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ea50` | Engine::IDisplayActionHandler::StaticInit_01cbd578 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd578, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174eaa0` | Engine::IDisplayActionHandler::StaticInit_01cbd604 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd604, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174eaf0` | Engine::IDisplayActionHandler::StaticInit_01cbd5b4 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5b4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174eb40` | Engine::IDisplayActionHandler::StaticInit_01cbd588 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd588, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174eb90` | Engine::IDisplayActionHandler::StaticInit_01cbd5c0 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5c0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ebe0` | Engine::IDisplayActionHandler::StaticInit_01cbd5bc | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5bc, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ec30` | Engine::IDisplayActionHandler::StaticInit_01cbd5a8 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd5a8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ec80` | Engine::IDisplayActionHandler::StaticInit_01cbd598 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd598, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ecd0` | Engine::IDisplayActionHandler::StaticInit_01cbd600 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd600, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174ed70` | Engine::IDisplayActionHandler::StaticInit_01cbd618 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd618, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |
| `0174f090` | Engine::IDisplayActionHandler::StaticInit_01cbd664 | likely | Stores the Engine::IDisplayActionHandler vftable into the dword at 01cbd664, the vptr of a file-scope object; MSVC SEH frame with security cookie, no calls, re… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00765f30` | SoeUtil::List::AppendNewAttachment | likely | Fastcall on an attachment list (ECX = list): allocates a node through the list's vtable slot [2] (+8); when the allocation succeeds it sets the node's +4 to 0 … |

### `engine.displayaction`

**Chat::ChatChatInput** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01281a20` | Chat::ChatChatInput::ChatChatInput | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Chat::ChatChatInput::vftable into *param_1, returns this; when the second argument is nonz… |
| `01281a90` | Chat::ChatChatInput::GetName | confirmed | Returns the literal string SendChat (mov eax, imm32; ret). |
| `01281ab0` | Chat::ChatChatInput::GetLogTag | confirmed | Returns a pointer to the literal SendChat : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:156 (mov eax, imm32; ret). |
| `01281ac0` | Chat::ChatChatInput::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |

**Chat::ChatDisplayUChatTextDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01281730` | Chat::ChatDisplayUChatTextDisplayAction::ChatDisplayUChatTe… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::ChatDisplayUChatTextDisplayAction::vftable into *param_1, returns this; when the second a… |
| `012817c0` | Chat::ChatDisplayUChatTextDisplayAction::GetLogTag | confirmed | Returns a pointer to the literal kDisplayUChatTextDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:87 (mov eax, imm32; ret). |
| `012817d0` | Chat::ChatDisplayUChatTextDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |

**Chat::ChatGetConversationID** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01281c30` | Chat::ChatGetConversationID::ChatGetConversationID | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Chat::ChatGetConversationID::vftable into *param_1, returns this; when the second argument… |
| `01281ca0` | Chat::ChatGetConversationID::GetName | confirmed | Returns the literal string GetConversationID (mov eax, imm32; ret). |
| `01281cc0` | Chat::ChatGetConversationID::GetLogTag | confirmed | Returns a pointer to the literal GetConversationID : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:128 (mov eax, imm32; ret). |
| `01281cd0` | Chat::ChatGetConversationID::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |

**Chat::ChatSendTextToLobbyDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01281b30` | Chat::ChatSendTextToLobbyDisplayAction::ChatSendTextToLobby… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::ChatSendTextToLobbyDisplayAction::vftable into *param_1, returns this; when the second ar… |
| `01281bc0` | Chat::ChatSendTextToLobbyDisplayAction::GetLogTag | confirmed | Returns a pointer to the literal kSendTextToLobbyDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:30 (mov eax, imm32; ret). |
| `01281bd0` | Chat::ChatSendTextToLobbyDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |

**Chat::DisplayEasyChatTextDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012818d0` | Chat::DisplayEasyChatTextDisplayAction::DisplayEasyChatText… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Chat::DisplayEasyChatTextDisplayAction::vftable into *param_1, returns this; when the second ar… |
| `01281960` | Chat::DisplayEasyChatTextDisplayAction::GetLogTag | confirmed | Returns a pointer to the literal kDisplayEasyChatTextDisplayAction : 0 : ..\common\coreclient\engine\chat\chathandlers.cc:140 (mov eax, imm32; ret). |
| `01281970` | Chat::DisplayEasyChatTextDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set, returns this. |

**DisplayAction** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f8f60` | DisplayAction::ArgAt | confirmed | Thiscall with ECX = object and one index argument: when the mArgs begin at +8 is null or index >= (end at +0xc minus begin) / 12, calls 012f5a60 with the asser… |
| `012fa060` | DisplayAction::AddArg | confirmed | Stdcall with ECX = DisplayAction and one stack pointer to a 12-byte value (type id at +4) (ret 4): checks the type id minus 1 against the accepted set {1,2,3,6… |
| `012fa0c0` | DisplayAction::AddPropertySetArg | likely | Stdcall-style with one stack pointer (param_1, ret 4 not decoded): constructs a 12-byte local through 012fa910; if param_1 is null calls 012f5a60 with 'propert… |
| `012fa160` | DisplayAction::AddStringArg | likely | Stdcall (ret 4) with ECX = DisplayAction and one stack pointer to a std::string: constructs a 12-byte local through 012fa910, writes type tag 3 at the local's … |
| `012fa1f0` | DisplayAction::AddStringArgCStr | likely | Stdcall with one stack pointer to a C string: builds a temporary std::string from the C string, writes type tag 3 to a local through 01300680, copies the tempo… |
| `012fa2e0` | DisplayAction::AddTypeTag2Arg | likely | Stdcall with one stack dword, ESI = DisplayAction: builds a 12-byte local through 012fa910, writes type tag 2 at its +4 through 01300680(2), stores the argumen… |
| `012fa360` | DisplayAction::AddTypeTag1Arg | likely | Stdcall with one stack byte, ESI = DisplayAction: builds a 12-byte local through 012fa910, writes type tag 1 at its +4 through 01300680(1), stores the byte arg… |
| `012fa4f0` | DisplayAction::AddTypeTag6Arg | likely | Stdcall with one stack dword: builds a 12-byte local through 012fa910, writes type tag 6 through 01300680(6), fills the payload from the argument through 005f1… |
| `012fa570` | DisplayAction::AddTypeTag13ObjectArg | likely | Stdcall with one stack pointer: builds a 12-byte local through 012fa910, writes type tag 13 through 01300680(0xd), copies the argument into the local payload t… |
| `012fa630` | DisplayAction::AddTypeTag7Arg | likely | Stdcall with one stack dword: builds a 12-byte local through 012fa910, writes type tag 7 through 01300680(7), fills the payload from the argument through 012da… |

**Engine::AccountGetAccountInfoForDisplayDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e770` | Engine::AccountGetAccountInfoForDisplayDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountGetAccountInfoForDisplayDisplayAction::vftable into *this with EH state 0; when the int argum… |
| `0128e800` | Engine::AccountGetAccountInfoForDisplayDisplayAction::GetSo… | confirmed | Returns the address of the constant string kGetAccountInfoForDisplayDisplayAction : 0 : (accounthandlers.cc line 852); 6 bytes, mov eax,imm32 then ret. |
| `0128e810` | Engine::AccountGetAccountInfoForDisplayDisplayAction::Scala… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnReceiveFriendConfirmRequestUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128d7a0` | Engine::AccountOnReceiveFriendConfirmRequestUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendConfirmRequestUC::vftable into *this with EH state 0; when the int argument is… |
| `0128d830` | Engine::AccountOnReceiveFriendConfirmRequestUC::GetSourceTag | confirmed | Returns the address of the constant string kOnReceiveFriendConfirmRequestUC : 0 : (accounthandlers.cc line 86); 6 bytes, mov eax,imm32 then ret. |
| `0128d840` | Engine::AccountOnReceiveFriendConfirmRequestUC::ScalarDelet… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnReceiveFriendConfirmResponseUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128d8f0` | Engine::AccountOnReceiveFriendConfirmResponseUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendConfirmResponseUC::vftable into *this with EH state 0; when the int argument i… |
| `0128d980` | Engine::AccountOnReceiveFriendConfirmResponseUC::GetSourceT… | confirmed | Returns the address of the constant string kOnReceiveFriendConfirmResponseUC : 0 : (accounthandlers.cc line 132); 6 bytes, mov eax,imm32 then ret. |
| `0128d990` | Engine::AccountOnReceiveFriendConfirmResponseUC::ScalarDele… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnReceiveFriendLoginUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128da40` | Engine::AccountOnReceiveFriendLoginUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendLoginUC::vftable into *this with EH state 0; when the int argument is nonzero … |
| `0128dad0` | Engine::AccountOnReceiveFriendLoginUC::GetSourceTag | confirmed | Returns the address of the constant string kOnReceiveFriendLoginUC : 0 : (accounthandlers.cc line 229); 6 bytes, mov eax,imm32 then ret. |
| `0128dae0` | Engine::AccountOnReceiveFriendLoginUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnReceiveFriendLogoutUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128db90` | Engine::AccountOnReceiveFriendLogoutUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnReceiveFriendLogoutUC::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128dc20` | Engine::AccountOnReceiveFriendLogoutUC::GetSourceTag | confirmed | Returns the address of the constant string kOnReceiveFriendLogoutUC : 0 : (accounthandlers.cc line 256); 6 bytes, mov eax,imm32 then ret. |
| `0128dc30` | Engine::AccountOnReceiveFriendLogoutUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestAddFriendUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128dce0` | Engine::AccountOnRequestAddFriendUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestAddFriendUC::vftable into *this with EH state 0; when the int argument is nonzero ca… |
| `0128dd70` | Engine::AccountOnRequestAddFriendUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestAddFriendUC : 0 : (accounthandlers.cc line 283); 6 bytes, mov eax,imm32 then ret. |
| `0128dd80` | Engine::AccountOnRequestAddFriendUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestAddIgnoreUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128de30` | Engine::AccountOnRequestAddIgnoreUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestAddIgnoreUC::vftable into *this with EH state 0; when the int argument is nonzero ca… |
| `0128dec0` | Engine::AccountOnRequestAddIgnoreUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestAddIgnoreUC : 0 : (accounthandlers.cc line 397); 6 bytes, mov eax,imm32 then ret. |
| `0128ded0` | Engine::AccountOnRequestAddIgnoreUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestFriendConfirmUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128df80` | Engine::AccountOnRequestFriendConfirmUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestFriendConfirmUC::vftable into *this with EH state 0; when the int argument is nonzer… |
| `0128e010` | Engine::AccountOnRequestFriendConfirmUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestFriendConfirmUC : 0 : (accounthandlers.cc line 496); 6 bytes, mov eax,imm32 then ret. |
| `0128e020` | Engine::AccountOnRequestFriendConfirmUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestFriendStatusUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e0d0` | Engine::AccountOnRequestFriendStatusUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestFriendStatusUC::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128e160` | Engine::AccountOnRequestFriendStatusUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestFriendStatusUC : 0 : (accounthandlers.cc line 568); 6 bytes, mov eax,imm32 then ret. |
| `0128e170` | Engine::AccountOnRequestFriendStatusUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestIgnoreStatusUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e220` | Engine::AccountOnRequestIgnoreStatusUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestIgnoreStatusUC::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128e2b0` | Engine::AccountOnRequestIgnoreStatusUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestIgnoreStatusUC : 0 : (accounthandlers.cc line 609); 6 bytes, mov eax,imm32 then ret. |
| `0128e2c0` | Engine::AccountOnRequestIgnoreStatusUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestRemoveFriendUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e370` | Engine::AccountOnRequestRemoveFriendUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestRemoveFriendUC::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128e400` | Engine::AccountOnRequestRemoveFriendUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestRemoveFriendUC : 0 : (accounthandlers.cc line 646); 6 bytes, mov eax,imm32 then ret. |
| `0128e410` | Engine::AccountOnRequestRemoveFriendUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestRemoveIgnoreUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e4c0` | Engine::AccountOnRequestRemoveIgnoreUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestRemoveIgnoreUC::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128e550` | Engine::AccountOnRequestRemoveIgnoreUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestRemoveIgnoreUC : 0 : (accounthandlers.cc line 743); 6 bytes, mov eax,imm32 then ret. |
| `0128e560` | Engine::AccountOnRequestRemoveIgnoreUC::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountOnRequestSetFriendCommentUC** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e610` | Engine::AccountOnRequestSetFriendCommentUC::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountOnRequestSetFriendCommentUC::vftable into *this with EH state 0; when the int argument is non… |
| `0128e6a0` | Engine::AccountOnRequestSetFriendCommentUC::GetSourceTag | confirmed | Returns the address of the constant string kOnRequestSetFriendCommentUC : 0 : (accounthandlers.cc line 846); 6 bytes, mov eax,imm32 then ret. |
| `0128e6b0` | Engine::AccountOnRequestSetFriendCommentUC::ScalarDeletingD… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountSetAchievementListDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128d640` | Engine::AccountSetAchievementListDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::AccountSetAchievementListDisplayAction::vftable into *this with EH state 0; when the int argument is… |
| `0128d6d0` | Engine::AccountSetAchievementListDisplayAction::GetSourceTag | confirmed | Returns the address of the constant string kSetAchievementListDisplayAction : 0 : (accounthandlers.cc line 80); 6 bytes, mov eax,imm32 then ret. |
| `0128d6e0` | Engine::AccountSetAchievementListDisplayAction::ScalarDelet… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::AccountUsernameRegistrationResultsDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e9a0` | Engine::AccountUsernameRegistrationResultsDisplayAction::Ge… | confirmed | Returns the address of the constant string kUsernameRegistrationResultsDisplayAction : 0 : (accounthandlers.cc line 888); 6 bytes, mov eax,imm32 then ret. |
| `0128e9b0` | Engine::AccountUsernameRegistrationResultsDisplayAction::Sc… | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::CollectionReloadCollectionDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012769b0` | Engine::CollectionReloadCollectionDisplayAction::ctor | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionReloadCollectionDisplayAction::vftable into the object; when the int arg… |
| `01276a40` | Engine::CollectionReloadCollectionDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadCollectionDisplayAction : 0 : ..\common\...\collectionhandlers.cc:119. |
| `01276a50` | Engine::CollectionReloadCollectionDisplayAction::ScalarDele… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::CollectionSynchronizationReceiveInstancesDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276320` | Engine::CollectionSynchronizationReceiveInstancesDisplayAct… | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationReceiveInstancesDisplayAction::vftable into the object; w… |
| `012763b0` | Engine::CollectionSynchronizationReceiveInstancesDisplayAct… | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationReceiveInstancesDisplayAction : 0 : ..\common\...\collectionhandlers.… |
| `012763c0` | Engine::CollectionSynchronizationReceiveInstancesDisplayAct… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::CollectionSynchronizationRemoveInstancesDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276860` | Engine::CollectionSynchronizationRemoveInstancesDisplayActi… | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationRemoveInstancesDisplayAction::vftable into the object; wh… |
| `012768f0` | Engine::CollectionSynchronizationRemoveInstancesDisplayActi… | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationRemoveInstancesDisplayAction : 0 : ..\common\...\collectionhandlers.c… |
| `01276900` | Engine::CollectionSynchronizationRemoveInstancesDisplayActi… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::CollectionSynchronizationStartDeliveryDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276710` | Engine::CollectionSynchronizationStartDeliveryDisplayAction… | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::CollectionSynchronizationStartDeliveryDisplayAction::vftable into the object; when… |
| `012767a0` | Engine::CollectionSynchronizationStartDeliveryDisplayAction… | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationStartDeliveryDisplayAction : 0 : ..\common\...\collectionhandlers.cc:… |
| `012767b0` | Engine::CollectionSynchronizationStartDeliveryDisplayAction… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::DeckCollectionReadyDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012742f0` | Engine::DeckCollectionReadyDisplayAction::ctor | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckCollectionReadyDisplayAction::vftable into the object; when the int argument i… |
| `01274380` | Engine::DeckCollectionReadyDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kCollectionReady : 0 : ..\common\...\deckhandlers.cc:24. |
| `01274390` | Engine::DeckCollectionReadyDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::DeckOnlineDecksReceivedDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01272910` | Engine::DeckOnlineDecksReceivedDisplayAction::DebugName | confirmed | Slot [4] of Engine::DeckOnlineDecksReceivedDisplayAction::vftable. Bytes b8 imm32 c3: returns the constant C string kOnlineDecksReceivedDisplayAction : 0 : ..\… |
| `01272920` | Engine::DeckOnlineDecksReceivedDisplayAction::ScalarDeletin… | confirmed | Slot [0] of Engine::DeckOnlineDecksReceivedDisplayAction::vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argumen… |

**Engine::DeckReloadCollectionDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012729e0` | Engine::DeckReloadCollectionDisplayAction::ctor | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckReloadCollectionDisplayAction::vftable into the object; when the int argument … |
| `01272a70` | Engine::DeckReloadCollectionDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadCollectionDisplayAction : 0 : ..\common\...\deckhandlers.cc:39. |
| `01272a80` | Engine::DeckReloadCollectionDisplayAction::ScalarDeletingDt… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::DeckReloadDeckDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01272b30` | Engine::DeckReloadDeckDisplayAction::ctor | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::DeckReloadDeckDisplayAction::vftable into the object; when the int argument is non… |
| `01272bc0` | Engine::DeckReloadDeckDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kReloadDeckDisplayAction : 0 : ..\common\...\deckhandlers.cc:50. |
| `01272bd0` | Engine::DeckReloadDeckDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::EngineCollectionSynchronizationCompleteInstancesDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012765c0` | Engine::EngineCollectionSynchronizationCompleteInstancesDis… | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::EngineCollectionSynchronizationCompleteInstancesDisplayAction::vftable into the ob… |
| `01276650` | Engine::EngineCollectionSynchronizationCompleteInstancesDis… | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationCompleteInstancesDisplayAction : 0 : ..\common\...\collectionhandlers… |
| `01276660` | Engine::EngineCollectionSynchronizationCompleteInstancesDis… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::EngineUpdateDataVersionDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124e430` | Engine::EngineUpdateDataVersionDisplayAction::DebugName | confirmed | Virtual method in slot [4] of Engine::EngineUpdateDataVersionDisplayAction::vftable. Returns the constant C string 'kUpdateDataVersionDisplayAction : 0 : ..\co… |
| `0124e440` | Engine::EngineUpdateDataVersionDisplayAction::ScalarDeletin… | confirmed | Slot [0] of Engine::EngineUpdateDataVersionDisplayAction::vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argumen… |
| `0124e3a0` | Engine::EngineUpdateDataVersionDisplayAction::ctor | likely | Returns this. Stores the base Engine::IDisplayActionHandler::vftable and then Engine::EngineUpdateDataVersionDisplayAction::vftable into the object; when the i… |

**Engine::FRTWAccountPostLogin** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01285110` | Engine::FRTWAccountPostLogin::FRTWAccountPostLogin | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWAccountPostLogin::vftable into *param_1, returns this; when the second argumen… |
| `01285180` | Engine::FRTWAccountPostLogin::GetName | confirmed | Returns a pointer to the literal PostLogin (mov eax, imm32; ret). No arguments read. |
| `012851a0` | Engine::FRTWAccountPostLogin::GetLogTag | confirmed | Returns a pointer to the literal PostLogin : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `012851b0` | Engine::FRTWAccountPostLogin::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is… |

**Engine::FRTWAccountQueryPlayerInfo** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283880` | Engine::FRTWAccountQueryPlayerInfo::FRTWAccountQueryPlayerI… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWAccountQueryPlayerInfo::vftable into *param_1, returns this; when the second a… |
| `012838f0` | Engine::FRTWAccountQueryPlayerInfo::GetName | confirmed | Returns a pointer to the literal QueryPlayerInfo (mov eax, imm32; ret). No arguments read. |
| `01283910` | Engine::FRTWAccountQueryPlayerInfo::GetLogTag | confirmed | Returns a pointer to the literal QueryPlayerInfo : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `01283920` | Engine::FRTWAccountQueryPlayerInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is… |

**Engine::FRTWAccountRespondDBAvatars** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01285010` | Engine::FRTWAccountRespondDBAvatars::FRTWAccountRespondDBAv… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondDBAvatars::vftable into *param_1, returns this; when the second argum… |
| `012850a0` | Engine::FRTWAccountRespondDBAvatars::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondDBAvatarsDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `012850b0` | Engine::FRTWAccountRespondDBAvatars::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondDBAwards** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283040` | Engine::FRTWAccountRespondDBAwards::FRTWAccountRespondDBAwa… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondDBAwards::vftable into *param_1, returns this; when the second argume… |
| `012830d0` | Engine::FRTWAccountRespondDBAwards::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondDBAwardsDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `012830e0` | Engine::FRTWAccountRespondDBAwards::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondDBHonors** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283190` | Engine::FRTWAccountRespondDBHonors::FRTWAccountRespondDBHon… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondDBHonors::vftable into *param_1, returns this; when the second argume… |
| `01283220` | Engine::FRTWAccountRespondDBHonors::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondDBHonorsDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `01283230` | Engine::FRTWAccountRespondDBHonors::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondPlayerAvatars** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283430` | Engine::FRTWAccountRespondPlayerAvatars::FRTWAccountRespond… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondPlayerAvatars::vftable into *param_1, returns this; when the second a… |
| `012834c0` | Engine::FRTWAccountRespondPlayerAvatars::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondPlayerAvatarsDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `012834d0` | Engine::FRTWAccountRespondPlayerAvatars::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondPlayerHonors** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283580` | Engine::FRTWAccountRespondPlayerHonors::FRTWAccountRespondP… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondPlayerHonors::vftable into *param_1, returns this; when the second ar… |
| `01283610` | Engine::FRTWAccountRespondPlayerHonors::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondPlayerHonorsDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `01283620` | Engine::FRTWAccountRespondPlayerHonors::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondPlayerInfo** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012832e0` | Engine::FRTWAccountRespondPlayerInfo::FRTWAccountRespondPla… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondPlayerInfo::vftable into *param_1, returns this; when the second argu… |
| `01283370` | Engine::FRTWAccountRespondPlayerInfo::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondPlayerInfoDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `01283380` | Engine::FRTWAccountRespondPlayerInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAccountRespondPlayerMatches** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012836d0` | Engine::FRTWAccountRespondPlayerMatches::FRTWAccountRespond… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::FRTWAccountRespondPlayerMatches::vftable into *param_1, returns this; when the second a… |
| `01283760` | Engine::FRTWAccountRespondPlayerMatches::GetLogTag | confirmed | Returns a pointer to the literal kUserDataCommandRespondPlayerMatchesDisplayAction : 0 : ... (mov eax, imm32; ret). No arguments read. |
| `01283770` | Engine::FRTWAccountRespondPlayerMatches::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Engine::IDisplayActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is set. |

**Engine::FRTWAddTerrain** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01251d30` | Engine::FRTWAddTerrain::DebugName | confirmed | Slot [4] of Engine::FRTWAddTerrain::vftable. Bytes b8 imm32 c3: returns the constant C string 'kAddTerrainDisplayAction : 0 : ..\engine\game\frtwgamehandlers.c… |
| `01251d40` | Engine::FRTWAddTerrain::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01251ca0` | Engine::FRTWAddTerrain::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWAddTerrain::vftable into the object; when the int argument is nonzero it calls FUN_012ea30… |

**Engine::FRTWCampaignCampaignCompletionUpdate** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01250f10` | Engine::FRTWCampaignCampaignCompletionUpdate::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kCampaignCompletionUpdate : 0 : ..\engine\campaign\frtwcampaignhandlers.cc:64'. |
| `01250f20` | Engine::FRTWCampaignCampaignCompletionUpdate::ScalarDeletin… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01250e80` | Engine::FRTWCampaignCampaignCompletionUpdate::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWCampaignCampaignCompletionUpdate::vftable into the object; when the int argument is nonzer… |

**Engine::FRTWClearWaitingDialog** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01251be0` | Engine::FRTWClearWaitingDialog::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kClearWaitingDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:352'. |
| `01251bf0` | Engine::FRTWClearWaitingDialog::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01251b50` | Engine::FRTWClearWaitingDialog::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWClearWaitingDialog::vftable into the object; when the int argument is nonzero it calls FUN… |

**Engine::FRTWEngineUpdateElementDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252790` | Engine::FRTWEngineUpdateElementDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateElementDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2004'. |
| `012527a0` | Engine::FRTWEngineUpdateElementDisplayAction::ScalarDeletin… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01252700` | Engine::FRTWEngineUpdateElementDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWEngineUpdateElementDisplayAction::vftable into the object; when the int argument is nonzer… |

**Engine::FRTWGameGetAIInputDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253930` | Engine::FRTWGameGetAIInputDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameModifyPlayerAttributeDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253670` | Engine::FRTWGameModifyPlayerAttributeDisplayAction::ScalarD… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameMoveCardDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01251fb0` | Engine::FRTWGameMoveCardDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kMoveCardDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:1860'. |
| `01251fc0` | Engine::FRTWGameMoveCardDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01251f20` | Engine::FRTWGameMoveCardDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameMoveCardDisplayAction::vftable into the object; when the int argument is nonzero it ca… |

**Engine::FRTWGamePhaseTurnChangeDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253240` | Engine::FRTWGamePhaseTurnChangeDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kPhaseTurnChangeDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2405'. |
| `01253250` | Engine::FRTWGamePhaseTurnChangeDisplayAction::ScalarDeletin… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameRebuildInitiativeListDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253bd0` | Engine::FRTWGameRebuildInitiativeListDisplayAction::ScalarD… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameResetUIStateDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252fa0` | Engine::FRTWGameResetUIStateDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kResetUIStateDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2331'. |
| `01252fb0` | Engine::FRTWGameResetUIStateDisplayAction::ScalarDeletingDt… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameSetActivePlayerDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012533b0` | Engine::FRTWGameSetActivePlayerDisplayAction::ScalarDeletin… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01253310` | Engine::FRTWGameSetActivePlayerDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetActivePlayerDisplayAction::vftable into the object; when the int argument is nonzer… |

**Engine::FRTWGameSetPhaseDialogDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253e70` | Engine::FRTWGameSetPhaseDialogDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetPhaseDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2480'. |
| `01253e80` | Engine::FRTWGameSetPhaseDialogDisplayAction::ScalarDeleting… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01253de0` | Engine::FRTWGameSetPhaseDialogDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetPhaseDialogDisplayAction::vftable into the object; when the int argument is nonzero… |

**Engine::FRTWGameSetPlaymatModeDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252a40` | Engine::FRTWGameSetPlaymatModeDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetPlaymatModeDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2233'. |
| `01252a50` | Engine::FRTWGameSetPlaymatModeDisplayAction::ScalarDeleting… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `012529b0` | Engine::FRTWGameSetPlaymatModeDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameSetPlaymatModeDisplayAction::vftable into the object; when the int argument is nonzero… |

**Engine::FRTWGameSetupDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253d20` | Engine::FRTWGameSetupDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSetupDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2466'. |
| `01253d30` | Engine::FRTWGameSetupDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameShowEndOfGameDialogDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012530f0` | Engine::FRTWGameShowEndOfGameDialogDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kShowEndOfGameDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2349'. |
| `01253100` | Engine::FRTWGameShowEndOfGameDialogDisplayAction::ScalarDel… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameUpdateBillboardModifiersDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252ba0` | Engine::FRTWGameUpdateBillboardModifiersDisplayAction::Scal… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameUpdateCreatureActionsDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253a80` | Engine::FRTWGameUpdateCreatureActionsDisplayAction::ScalarD… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `012539e0` | Engine::FRTWGameUpdateCreatureActionsDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWGameUpdateCreatureActionsDisplayAction::vftable into the object; when the int argument is … |

**Engine::FRTWGameUpdatePlayerDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012537d0` | Engine::FRTWGameUpdatePlayerDisplayAction::ScalarDeletingDt… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWGameUpdateWaitingHintsDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01253500` | Engine::FRTWGameUpdateWaitingHintsDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateWaitingHintsDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2418'. |
| `01253510` | Engine::FRTWGameUpdateWaitingHintsDisplayAction::ScalarDele… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWManaGeneratedDisplayAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252e40` | Engine::FRTWManaGeneratedDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kManaGeneratedDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:2299'. |
| `01252e50` | Engine::FRTWManaGeneratedDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWRequestPlayerClick** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01251940` | Engine::FRTWRequestPlayerClick::DebugName | confirmed | Slot [4] of Engine::FRTWRequestPlayerClick::vftable. Bytes b8 imm32 c3: returns the constant C string 'kRequestPlayerClickDisplayAction : 0 : ..\engine\game\fr… |
| `01251950` | Engine::FRTWRequestPlayerClick::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `012518b0` | Engine::FRTWRequestPlayerClick::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWRequestPlayerClick::vftable into the object; when the int argument is nonzero it calls FUN… |

**Engine::FRTWSpawnWaitingDialog** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01251a90` | Engine::FRTWSpawnWaitingDialog::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kSpawnWaitingDialogDisplayAction : 0 : ..\engine\game\frtwgamehandlers.cc:343'. |
| `01251aa0` | Engine::FRTWSpawnWaitingDialog::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01251a00` | Engine::FRTWSpawnWaitingDialog::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWSpawnWaitingDialog::vftable into the object; when the int argument is nonzero it calls FUN… |

**Engine::FRTWStopTimerCriticalAction** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252cf0` | Engine::FRTWStopTimerCriticalAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kStopTimerCriticalAction : 0 : ..\engine\game\frtwgamehandlers.cc:2292'. |
| `01252d00` | Engine::FRTWStopTimerCriticalAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |

**Engine::FRTWUIUpdateElementDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012528e0` | Engine::FRTWUIUpdateElementDisplayAction::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'kUpdateElementDisplayAction : 1 : ..\engine\game\frtwgamehandlers.cc:2136'. |
| `012528f0` | Engine::FRTWUIUpdateElementDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `01252850` | Engine::FRTWUIUpdateElementDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::FRTWUIUpdateElementDisplayAction::vftable into the object; when the int argument is nonzero it… |

**Engine::GatewayServerErrorDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124ec60` | Engine::GatewayServerErrorDisplayAction::DebugName | confirmed | Slot [4] of Engine::GatewayServerErrorDisplayAction::vftable. Returns the constant C string 'kGatewayServerErrorDisplayAction : 0 : ..\common\coreclient\engine… |
| `0124ec70` | Engine::GatewayServerErrorDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `0124ebd0` | Engine::GatewayServerErrorDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::GatewayServerErrorDisplayAction::vftable into the object; when the int argument is nonzero it … |

**Engine::GatewayServerSuccessDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124eb10` | Engine::GatewayServerSuccessDisplayAction::DebugName | confirmed | Slot [4] of Engine::GatewayServerSuccessDisplayAction::vftable. Returns the constant C string 'kGatewayServerSuccessDisplayAction : 0 : ..\common\coreclient\en… |
| `0124eb20` | Engine::GatewayServerSuccessDisplayAction::ScalarDeletingDt… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `0124ea80` | Engine::GatewayServerSuccessDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::GatewayServerSuccessDisplayAction::vftable into the object; when the int argument is nonzero i… |

**Engine::IDisplayActionHandler** (113)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123c480` | Engine::IDisplayActionHandler::BaseDtor | confirmed | Bytes c7 01 3c 88 8c 01 c3: stores Engine::IDisplayActionHandler::vftable into the first word of the object and returns. |
| `0123c490` | Engine::IDisplayActionHandler::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object and frees it through _free when bit 0 of the flag argument is set; returns the object pointer. |
| `0123f1c0` | Engine::IDisplayActionHandler::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object at ECX and returns; same 67-byte body as 0123f070. |
| `0123f310` | Engine::IDisplayActionHandler::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable into the object at ECX and returns; same 67-byte body as 0123f070. |
| `01248560` | Engine::IDisplayActionHandler::Ctor | confirmed | Stores the Engine::IDisplayActionHandler vtable into the object at param_1 and returns. Base-class constructor body with no other effect. |
| `012485b0` | Engine::IDisplayActionHandler::Ctor | confirmed | Byte-identical to 01248560: stores the Engine::IDisplayActionHandler vtable into the object at param_1 and returns. |
| `01248700` | Engine::IDisplayActionHandler::Ctor | confirmed | Byte-identical to 01248560: stores the Engine::IDisplayActionHandler vtable into the object at param_1 and returns. |
| `0127c120` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c170` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c2c0` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c410` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c560` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c6b0` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127cab0` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127d160` | Engine::IDisplayActionHandler::SetBaseVftable | confirmed | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01281690` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor: stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `012816e0` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor: stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. Same body as 01281690. No calls. |
| `01281880` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor: stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. Same body as 01281690. No calls. |
| `01282fa0` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body: stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls, no stack frame beyond the store. |
| `01282ff0` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `01283140` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `01283290` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `012833e0` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `01283530` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `01283680` | Engine::IDisplayActionHandler::IDisplayActionHandler | confirmed | Base constructor body (67 bytes): stores Engine::IDisplayActionHandler::vftable into *param_1 and returns. No calls. |
| `0123c4b0` | Engine::IDisplayActionHandler::Ctor | likely | Installs an EH frame, stores Engine::IDisplayActionHandler::vftable into the object, clears the frame state and restores ExceptionList before returning. |
| `0123c590` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; byte-identical copy of the base install at 0123c4b0. |
| `0123c6e0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; byte-identical copy of the base install at 0123c4b0. |
| `0123c830` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; byte-identical copy of the base install at 0123c4b0. |
| `0123c980` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; byte-identical copy of the base install at 0123c4b0. |
| `0123cad0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123cc20` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123cd70` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123cec0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d060` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e4a0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e5f0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e740` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e890` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e9e0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123eb30` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123ec80` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123edd0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123ef20` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0124e300` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns void. Nothing else. |
| `0124e350` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Byte-identical twin of 0124e300: stores Engine::IDisplayActionHandler::vftable into *this and returns. |
| `0124e4a0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124e5f0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124e8e0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124ea30` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124eb80` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124ee20` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `0124ef80` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01250d20` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01251810` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01251860` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `012519b0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01251b00` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01251c50` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01251ed0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `012526b0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252800` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252960` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252ab0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252c10` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252d60` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01252ec0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253010` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253160` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `012532c0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253420` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253580` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `012536e0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253840` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253990` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253ae0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253c40` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253d90` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `01253ee0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body identical to 0124e300. |
| `012727e0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. 67-byte body with no calls. |
| `01272830` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; byte-identical to 012727e0 with no calls. |
| `01272990` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01272ae0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276280` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012762d0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276420` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276570` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012766c0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276810` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276960` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012772f0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `0127c800` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127c950` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127cc00` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127cd50` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127ceb0` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0127d010` | Engine::IDisplayActionHandler::SetBaseVftable | likely | Stores Engine::IDisplayActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `0128d420` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128d5f0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128d750` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128d8a0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128d9f0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128db40` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128dc90` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128dde0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128df30` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e080` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e1d0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e320` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e470` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e5c0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e720` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128e8c0` | Engine::IDisplayActionHandler::Ctor | likely | Stores Engine::IDisplayActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |

**Engine::IntroduceAccountDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f630` | Engine::IntroduceAccountDisplayAction::Ctor | confirmed | Stores Engine::IDisplayActionHandler::vftable then Engine::IntroduceAccountDisplayAction::vftable into *this with EH state 0; when the int argument is nonzero … |
| `0128f6c0` | Engine::IntroduceAccountDisplayAction::GetSourceTag | confirmed | Returns the address of the constant string kIntroduceAccountDisplayAction : 0 : (accounthandlers.cc line 44); 6 bytes, mov eax,imm32 then ret. |
| `0128f6d0` | Engine::IntroduceAccountDisplayAction::ScalarDeletingDtor | confirmed | Stores Engine::IDisplayActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns this. |

**Engine::LobbyAddLobbyGroupDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127f580` | Engine::LobbyAddLobbyGroupDisplayAction::Vslot2 | likely | Virtual slot [2]. Calls FUN_012e0ba0 and returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entries 0, 1 and 2 as integers (type check, 0 on fail… |

**Engine::LobbyChangeLobbyStatusDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127fb10` | Engine::LobbyChangeLobbyStatusDisplayAction::Vslot2 | likely | Virtual slot [2]. Returns 1 when FUN_00f9c010() is zero. Otherwise reads ValueData entry 0 as an integer, entry 1 as an integer and entry 2 as a boolean (type … |

**Engine::LobbyChangeScreenByGroupID** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e5b0` | Engine::LobbyChangeScreenByGroupID::LobbyChangeScreenByGrou… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyChangeScreenByGroupID::vftable into *param_1, returns this; when the second a… |
| `0127e620` | Engine::LobbyChangeScreenByGroupID::GetName | confirmed | Returns the literal string ChangeScreenByGroupID (mov eax, imm32; ret). |
| `0127e640` | Engine::LobbyChangeScreenByGroupID::GetLogTag | confirmed | Returns a pointer to the literal ChangeScreenByGroupID : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:685 (mov eax, imm32; ret). |
| `0127e650` | Engine::LobbyChangeScreenByGroupID::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01281090` | Engine::LobbyChangeScreenByGroupID::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull: reads GroupID through FUN_013b1560 and FUN_012eb290. Chooses the screen name by the group value: 1 gives UI::… |

**Engine::LobbyDisplayMOTDDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127fcf0` | Engine::LobbyDisplayMOTDDisplayAction::Vslot2 | likely | Virtual slot [2]. Returns when FUN_0127a580() is null. Otherwise reads entry 1 as an integer (assert and 0 on failure), and entry 2 as a C string (type 3; asse… |

**Engine::LobbyErrorCode** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d1b0` | Engine::LobbyErrorCode::LobbyErrorCode | confirmed | Constructor: stores the base Engine::IDisplayActionHandler vftable into *this, then the Engine::LobbyErrorCode vftable, and returns this. When the second argum… |
| `0127d240` | Engine::LobbyErrorCode::GetLogTag | confirmed | Returns a pointer to the literal string kErrorCodeDisplayAction : 1 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:362 (mov eax, imm32; ret). No argument… |
| `0127d250` | Engine::LobbyErrorCode::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the Engine::IDisplayActionHandler vftable into *this, returns this, and calls _free(this) when bit 0 of the argument is set. |
| `012800b0` | Engine::LobbyErrorCode::Vslot2 | likely | Virtual slot [2]. Reads ValueData entry 0 as an integer (assert on a non-integer type, then returns 2). When the value is 0x9e builds the tag matchMakingTimeou… |

**Engine::LobbyExitMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e1c0` | Engine::LobbyExitMatch::LobbyExitMatch | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyExitMatch::vftable into *param_1, returns this; when the second argument is n… |
| `0127e230` | Engine::LobbyExitMatch::GetName | confirmed | Returns the literal string ExitMatch (mov eax, imm32; ret). |
| `0127e250` | Engine::LobbyExitMatch::GetLogTag | confirmed | Returns a pointer to the literal ExitMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:620 (mov eax, imm32; ret). |
| `0127e260` | Engine::LobbyExitMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127ec80` | Engine::LobbyExitMatch::Vslot2 | likely | Virtual slot [2]: calls FUN_012e0ba0(), gets a manager pointer from FUN_0127a580(); if non-null, calls the manager's virtual slot at offset 0x1c. Returns 1. |

**Engine::LobbyGoTo** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127ea80` | Engine::LobbyGoTo::LobbyGoTo | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyGoTo::vftable into *param_1, returns this; when the second argument is nonzer… |
| `0127eaf0` | Engine::LobbyGoTo::GetName | confirmed | Returns the address of static data DAT_018caa50 (mov eax, imm32; ret). The bytes at that address begin with the NUL-terminated text GoTo, so this returns a con… |
| `0127eb10` | Engine::LobbyGoTo::GetLogTag | confirmed | Returns a pointer to the literal GoTo : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:392 (mov eax, imm32; ret). |
| `0127eb20` | Engine::LobbyGoTo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |

**Engine::LobbyInitializeLobbyPlayers** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e700` | Engine::LobbyInitializeLobbyPlayers::LobbyInitializeLobbyPl… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyInitializeLobbyPlayers::vftable into *param_1, returns this; when the second … |
| `0127e770` | Engine::LobbyInitializeLobbyPlayers::GetName | confirmed | Returns the literal string InitializeLobbyPlayers (mov eax, imm32; ret). |
| `0127e790` | Engine::LobbyInitializeLobbyPlayers::GetLogTag | confirmed | Returns a pointer to the literal InitializeLobbyPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:717 (mov eax, imm32; ret). |
| `0127e7a0` | Engine::LobbyInitializeLobbyPlayers::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |

**Engine::LobbyJoinLobby** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d890` | Engine::LobbyJoinLobby::LobbyJoinLobby | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyJoinLobby::vftable into *param_1, returns this; when the second argument is n… |
| `0127d900` | Engine::LobbyJoinLobby::GetName | confirmed | Returns the literal string JoinLobby (mov eax, imm32; ret). |
| `0127d920` | Engine::LobbyJoinLobby::GetLogTag | confirmed | Returns a pointer to the literal JoinLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:489 (mov eax, imm32; ret). |
| `0127d930` | Engine::LobbyJoinLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |

**Engine::LobbyJoinMatch** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d9e0` | Engine::LobbyJoinMatch::LobbyJoinMatch | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyJoinMatch::vftable into *param_1, returns this; when the second argument is n… |
| `0127da50` | Engine::LobbyJoinMatch::GetName | confirmed | Returns the literal string JoinMatch (mov eax, imm32; ret). |
| `0127da70` | Engine::LobbyJoinMatch::GetLogTag | confirmed | Returns a pointer to the literal JoinMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:505 (mov eax, imm32; ret). |
| `0127da80` | Engine::LobbyJoinMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |

**Engine::LobbyLaunchGameDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127ffc0` | Engine::LobbyLaunchGameDisplayAction::Vslot2 | likely | Virtual slot [2]. When FUN_00f9c130() is nonzero reads ValueData entry 0 as an integer (assert and 0 on failure), then calls FUN_01239bc0 with the tag UI::Game… |

**Engine::LobbyLeaveLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127db30` | Engine::LobbyLeaveLobby::LobbyLeaveLobby | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyLeaveLobby::vftable into *param_1, returns this; when the second argument is … |
| `0127dba0` | Engine::LobbyLeaveLobby::GetName | confirmed | Returns the literal string LeaveLobby (mov eax, imm32; ret). |
| `0127dbc0` | Engine::LobbyLeaveLobby::GetLogTag | confirmed | Returns a pointer to the literal LeaveLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:522 (mov eax, imm32; ret). |
| `0127dbd0` | Engine::LobbyLeaveLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127ec40` | Engine::LobbyLeaveLobby::Vslot2 | likely | Virtual slot [2]: calls FUN_012e0ba0(), gets a manager pointer from FUN_0127a580(); if non-null, builds the string GroupID via FUN_012eb290("GroupID",0) and ca… |

**Engine::LobbyOnJoinedGroup** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e850` | Engine::LobbyOnJoinedGroup::LobbyOnJoinedGroup | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyOnJoinedGroup::vftable into *param_1, returns this; when the second argument … |
| `0127e8c0` | Engine::LobbyOnJoinedGroup::GetName | confirmed | Returns the literal string OnJoinedGroup (mov eax, imm32; ret). |
| `0127e8e0` | Engine::LobbyOnJoinedGroup::GetLogTag | confirmed | Returns a pointer to the literal OnJoinedGroup : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:736 (mov eax, imm32; ret). |
| `0127e8f0` | Engine::LobbyOnJoinedGroup::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01281410` | Engine::LobbyOnJoinedGroup::Vslot2 | likely | Virtual slot [2]. When FUN_00f9c010() is nonzero: calls FUN_012ebc00, sets GroupID (FUN_012eb290 lookup) and passes the tag OnJoinedGroup to FUN_01230b70. Retu… |

**Engine::LobbyQueryFriendsList** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127df20` | Engine::LobbyQueryFriendsList::LobbyQueryFriendsList | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryFriendsList::vftable into *param_1, returns this; when the second argume… |
| `0127df90` | Engine::LobbyQueryFriendsList::GetName | confirmed | Returns the literal string QueryFriendsList (mov eax, imm32; ret). |
| `0127dfb0` | Engine::LobbyQueryFriendsList::GetLogTag | confirmed | Returns a pointer to the literal QueryFriendsList : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:582 (mov eax, imm32; ret). |
| `0127dfc0` | Engine::LobbyQueryFriendsList::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280b00` | Engine::LobbyQueryFriendsList::Vslot2 | likely | Virtual slot [2]. When FUN_0127ebb0() (LookupFieldIfOwnerThread) is nonzero: gets AccountID (FUN_012eb290), builds the tag OpenDialog with FUN_012ec220, sets A… |

**Engine::LobbyQueryJoinMatch** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e310` | Engine::LobbyQueryJoinMatch::LobbyQueryJoinMatch | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryJoinMatch::vftable into *param_1, returns this; when the second argument… |
| `0127e380` | Engine::LobbyQueryJoinMatch::GetName | confirmed | Returns the literal string QueryJoinMatch (mov eax, imm32; ret). |
| `0127e3a0` | Engine::LobbyQueryJoinMatch::GetLogTag | confirmed | Returns a pointer to the literal QueryJoinMatch : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:631 (mov eax, imm32; ret). |
| `0127e3b0` | Engine::LobbyQueryJoinMatch::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280d60` | Engine::LobbyQueryJoinMatch::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull and FUN_013b1630 of GroupID is nonzero: calls FUN_0126dad0() and, when it is nonnull, copies the string return… |

**Engine::LobbyQueryLobby** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127dc80` | Engine::LobbyQueryLobby::LobbyQueryLobby | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobby::vftable into *param_1, returns this; when the second argument is … |
| `0127dcf0` | Engine::LobbyQueryLobby::GetName | confirmed | Returns the literal string QueryLobby (mov eax, imm32; ret). |
| `0127dd10` | Engine::LobbyQueryLobby::GetLogTag | confirmed | Returns a pointer to the literal QueryLobby : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:536 (mov eax, imm32; ret). |
| `0127dd20` | Engine::LobbyQueryLobby::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280820` | Engine::LobbyQueryLobby::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull and the GroupID value passes FUN_013b15f0: builds the tag InitializeLobby with FUN_012ec220, sets GroupID, pas… |

**Engine::LobbyQueryLobbyStats** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d350` | Engine::LobbyQueryLobbyStats::LobbyQueryLobbyStats | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobbyStats::vftable into *param_1, returns this; when the second argumen… |
| `0127d3c0` | Engine::LobbyQueryLobbyStats::GetName | confirmed | Returns the literal string QueryLobbyStats (mov eax, imm32; ret). No arguments read. |
| `0127d3e0` | Engine::LobbyQueryLobbyStats::GetLogTag | confirmed | Returns a pointer to the literal QueryLobbyStats : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:417 (mov eax, imm32; ret). |
| `0127d3f0` | Engine::LobbyQueryLobbyStats::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is… |
| `012803a0` | Engine::LobbyQueryLobbyStats::Vslot2 | likely | Virtual slot [2]. Reads GroupID with FUN_012eb290("GroupID",1), takes a count from FUN_012d2200, builds the tag QueryLobbyStats with FUN_012ec220, sets GroupID… |

**Engine::LobbyQueryLobbyStatus** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e460` | Engine::LobbyQueryLobbyStatus::LobbyQueryLobbyStatus | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryLobbyStatus::vftable into *param_1, returns this; when the second argume… |
| `0127e4d0` | Engine::LobbyQueryLobbyStatus::GetName | confirmed | Returns the literal string QueryLobbyStatus (mov eax, imm32; ret). |
| `0127e4f0` | Engine::LobbyQueryLobbyStatus::GetLogTag | confirmed | Returns a pointer to the literal QueryLobbyStatus : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:662 (mov eax, imm32; ret). |
| `0127e500` | Engine::LobbyQueryLobbyStatus::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280f60` | Engine::LobbyQueryLobbyStatus::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull: reads GroupID (FUN_012eb290); if FUN_013b15f0 of it is nonzero, sets a flag from FUN_0130ae30(FUN_012d08f0())… |

**Engine::LobbyQueryMatchInfo** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127ddd0` | Engine::LobbyQueryMatchInfo::LobbyQueryMatchInfo | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryMatchInfo::vftable into *param_1, returns this; when the second argument… |
| `0127de40` | Engine::LobbyQueryMatchInfo::GetName | confirmed | Returns the literal string QueryMatchInfo (mov eax, imm32; ret). |
| `0127de60` | Engine::LobbyQueryMatchInfo::GetLogTag | confirmed | Returns a pointer to the literal QueryMatchInfo : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:558 (mov eax, imm32; ret). |
| `0127de70` | Engine::LobbyQueryMatchInfo::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280980` | Engine::LobbyQueryMatchInfo::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull and FUN_013b1630 of the GroupID value is nonzero: builds the tag OpenDialog with FUN_012ec220, sets GroupID, p… |

**Engine::LobbyQueryMotd** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e070` | Engine::LobbyQueryMotd::LobbyQueryMotd | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyQueryMotd::vftable into *param_1, returns this; when the second argument is n… |
| `0127e0e0` | Engine::LobbyQueryMotd::GetName | confirmed | Returns the literal string QueryMotd (mov eax, imm32; ret). |
| `0127e100` | Engine::LobbyQueryMotd::GetLogTag | confirmed | Returns a pointer to the literal QueryMotd : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:603 (mov eax, imm32; ret). |
| `0127e110` | Engine::LobbyQueryMotd::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `01280c20` | Engine::LobbyQueryMotd::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull: builds the tag QueryMotd with FUN_012ec220, sets GroupID, then writes FUN_013b1800 of GroupID through FUN_012… |

**Engine::LobbyRemoveLobbyGroupDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127f340` | Engine::LobbyRemoveLobbyGroupDisplayAction::Vslot2 | likely | Virtual slot [2]. Calls FUN_012e0ba0(), returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entry 0 as an integer (type check against kIntegerType… |

**Engine::LobbyRemoveLobbyParticipantDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127f3d0` | Engine::LobbyRemoveLobbyParticipantDisplayAction::Vslot2 | likely | Virtual slot [2]. Calls FUN_012e0ba0 and returns 1 when FUN_0127a580() is null. Otherwise reads ValueData entries 0 and 1 as integers (type check, 0 on failure… |

**Engine::LobbyRequestMatches** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d740` | Engine::LobbyRequestMatches::LobbyRequestMatches | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestMatches::vftable into *param_1, returns this; when the second argument… |
| `0127d7b0` | Engine::LobbyRequestMatches::GetName | confirmed | Returns the literal string RequestMatches (mov eax, imm32; ret). |
| `0127d7d0` | Engine::LobbyRequestMatches::GetLogTag | confirmed | Returns a pointer to the literal RequestMatches : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:474 (mov eax, imm32; ret). |
| `0127d7e0` | Engine::LobbyRequestMatches::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame with state -1, stores Scaleform::IScaleformActionHandler::vftable into *param_1, and calls _free(this) when bit 0 of the a… |
| `01280500` | Engine::LobbyRequestMatches::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull: gets a value from FUN_013b1560, looks up the GroupID key with FUN_012eb290, and when FUN_013b15f0 of that is … |

**Engine::LobbyRequestPlayers** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d5f0` | Engine::LobbyRequestPlayers::LobbyRequestPlayers | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestPlayers::vftable into *param_1, returns this; when the second argument… |
| `0127d660` | Engine::LobbyRequestPlayers::GetName | confirmed | Returns the literal string RequestPlayers (mov eax, imm32; ret). |
| `0127d680` | Engine::LobbyRequestPlayers::GetLogTag | confirmed | Returns a pointer to the literal RequestPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:459 (mov eax, imm32; ret). |
| `0127d690` | Engine::LobbyRequestPlayers::ScalarDeletingDtor | confirmed | Scalar deleting destructor: stores Scaleform::IScaleformActionHandler::vftable into *param_1, returns this, and calls _free(this) when bit 0 of the argument is… |
| `01280490` | Engine::LobbyRequestPlayers::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull: gets a value from FUN_013b1560, looks up the GroupID key with FUN_012eb290, and when FUN_013b15f0 of that is … |

**Engine::LobbyRequestSubLobbyPlayers** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127d4a0` | Engine::LobbyRequestSubLobbyPlayers::LobbyRequestSubLobbyPl… | confirmed | Constructor: stores Scaleform::IScaleformActionHandler::vftable then Engine::LobbyRequestSubLobbyPlayers::vftable into *param_1, returns this; when the second … |
| `0127d510` | Engine::LobbyRequestSubLobbyPlayers::GetName | confirmed | Returns the literal string RequestSubLobbyPlayers (mov eax, imm32; ret). |
| `0127d530` | Engine::LobbyRequestSubLobbyPlayers::GetLogTag | confirmed | Returns a pointer to the literal RequestSubLobbyPlayers : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:430 (mov eax, imm32; ret). |
| `0127d540` | Engine::LobbyRequestSubLobbyPlayers::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame with state -1, stores Scaleform::IScaleformActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argum… |
| `01281500` | Engine::LobbyRequestSubLobbyPlayers::Vslot2 | likely | Virtual slot [2]. When FUN_0127a580() is nonnull and the GroupID value passes FUN_013b15f0: loops over five fixed codes 3, 1, 6, 10, 11, collecting entries wit… |

**Engine::LobbySetMatchStateDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127e980` | Engine::LobbySetMatchStateDisplayAction::LobbySetMatchState… | confirmed | Constructor: stores Engine::IDisplayActionHandler::vftable then Engine::LobbySetMatchStateDisplayAction::vftable into *param_1, returns this; when the second a… |
| `0127ea10` | Engine::LobbySetMatchStateDisplayAction::GetLogTag | confirmed | Returns a pointer to the literal kSetMatchStateDisplayAction : 0 : ..\common\coreclient\engine\lobby\lobbyhandlers.cc:123 (mov eax, imm32; ret). |
| `0127ea20` | Engine::LobbySetMatchStateDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: EH frame, stores Engine::IDisplayActionHandler::vftable into *param_1, calls _free(this) when bit 0 of the argument is set. |
| `0127f1d0` | Engine::LobbySetMatchStateDisplayAction::Vslot2 | likely | Virtual slot [2]. Returns 1 when FUN_00f9c010() is zero. Otherwise reads ValueData entries 0 and 1 (each must be integer type 2, else the assert text mTypeID =… |

**Engine::LobbyUpdateLobbyStatsDisplayAction** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0127fee0` | Engine::LobbyUpdateLobbyStatsDisplayAction::Vslot2 | likely | Virtual slot [2]. Calls FUN_012d46a0(1) and FUN_012d2200(1) for a user count, builds the tag QueryLobbyStats with FUN_012ec220, sets keys GroupID (constant 1) … |

**Engine::LoginResultsDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124e9c0` | Engine::LoginResultsDisplayAction::DebugName | confirmed | Slot [4] of Engine::LoginResultsDisplayAction::vftable. Returns the constant C string 'kLoginResultsDisplayAction : 0 : ..\common\coreclient\engine\network\net… |
| `0124e9d0` | Engine::LoginResultsDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of Engine::LoginResultsDisplayAction::vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _fr… |
| `0124e930` | Engine::LoginResultsDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::LoginResultsDisplayAction::vftable into the object; when the int argument is nonzero it calls … |

**Engine::NetworkConnectDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124ef00` | Engine::NetworkConnectDisplayAction::DebugName | confirmed | Slot [4] of Engine::NetworkConnectDisplayAction::vftable. Returns the constant C string 'kNetworkConnectDisplayAction : 0 : ..\common\coreclient\engine\network… |
| `0124ef10` | Engine::NetworkConnectDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `0124ee70` | Engine::NetworkConnectDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::NetworkConnectDisplayAction::vftable into the object; when the int argument is nonzero it call… |

**Engine::NetworkDisconnectDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124f060` | Engine::NetworkDisconnectDisplayAction::DebugName | confirmed | Slot [4] of Engine::NetworkDisconnectDisplayAction::vftable. Returns the constant C string 'kNetworkDisconnectDisplayAction : 0 : ..\common\coreclient\engine\n… |
| `0124f070` | Engine::NetworkDisconnectDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `0124efd0` | Engine::NetworkDisconnectDisplayAction::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::NetworkDisconnectDisplayAction::vftable into the object; when the int argument is nonzero it c… |

**Engine::NetworkSynchronizationCompleteInstancesDisplayAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124e580` | Engine::NetworkSynchronizationCompleteInstancesDisplayActio… | confirmed | Slot [4] of its vftable. Returns the constant C string 'kSynchronizationCompleteInstancesDisplayAction : 1 : ..\common\coreclient\engine\network\networkhandler… |
| `0124e590` | Engine::NetworkSynchronizationCompleteInstancesDisplayActio… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this; when bit 0 of the flag is set, calls _free(this) via the 00d82463 thunk; returns thi… |
| `0124e4f0` | Engine::NetworkSynchronizationCompleteInstancesDisplayActio… | likely | Returns this. Stores the base IDisplayActionHandler vftable then Engine::NetworkSynchronizationCompleteInstancesDisplayAction::vftable; when the int argument i… |
| `0124f370` | Engine::NetworkSynchronizationCompleteInstancesDisplayActio… | likely | Slot [2] of its vftable. Calls FUN_012e0ba0, then FUN_0124f2e0 (thread-owned object lookup). When that returns a non-null object it calls FUN_012adee0, uses it… |

**Engine::SystemMessageDisplayActionHandler** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124f170` | Engine::SystemMessageDisplayActionHandler::DebugName | confirmed | Slot [4] of Engine::SystemMessageDisplayActionHandler::vftable. Returns the constant C string 'kSystemMessageDisplayAction : 0 : ..\common\coreclient\engine\ne… |
| `0124f180` | Engine::SystemMessageDisplayActionHandler::ScalarDeletingDt… | confirmed | Slot [0] of its vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00d82463 t… |
| `0124f0e0` | Engine::SystemMessageDisplayActionHandler::ctor | likely | Returns this. Stores IDisplayActionHandler::vftable then Engine::SystemMessageDisplayActionHandler::vftable into the object; when the int argument is nonzero i… |

**Engine::UICollectionSynchronizationCompleteInstancesDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01276470` | Engine::UICollectionSynchronizationCompleteInstancesDisplay… | confirmed | Returns this. Stores Engine::IDisplayActionHandler::vftable and then Engine::UICollectionSynchronizationCompleteInstancesDisplayAction::vftable into the object… |
| `01276500` | Engine::UICollectionSynchronizationCompleteInstancesDisplay… | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string kSynchronizationCompleteInstancesDisplayAction : 0 : ..\common\...\collectionhandlers… |
| `01276510` | Engine::UICollectionSynchronizationCompleteInstancesDisplay… | confirmed | Slot [0] of its vftable. Stores Engine::IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through the 00… |

**Engine::UIUpdateDataVersionDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124e6d0` | Engine::UIUpdateDataVersionDisplayAction::DebugName | confirmed | Slot [4] of Engine::UIUpdateDataVersionDisplayAction::vftable. Returns the constant C string 'kUpdateDataVersionDisplayAction : 0 : ..\common\coreclient\engine… |
| `0124e6e0` | Engine::UIUpdateDataVersionDisplayAction::ScalarDeletingDtor | confirmed | Slot [0] of Engine::UIUpdateDataVersionDisplayAction::vftable. Stores IDisplayActionHandler::vftable into *this and, when bit 0 of the byte argument is set, ca… |
| `0124e640` | Engine::UIUpdateDataVersionDisplayAction::ctor | likely | Returns this. Stores the base Engine::IDisplayActionHandler::vftable and then Engine::UIUpdateDataVersionDisplayAction::vftable into the object; when the int a… |

**Messaging::LoopBase** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ea910` | Messaging::LoopBase::Dtor | confirmed | Destructor body for Messaging::LoopBase: stores the LoopBase vftable (018d5fd4) into this, destroys the two map members through 012ea8a0 and 012ea830, runs a m… |
| `012eaa90` | Messaging::LoopBase::Ctor | confirmed | Base constructor: stores the LoopBase vftable (018d5fd4) at this, initialises a std::string at this+4 (assign of the name param_2 through 00420ff1, then the st… |
| `012eab40` | Messaging::LoopBase::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the LoopBase destructor body 012ea910, then frees this with _free only when bit 0 of the argument is set. Returns this. |
| `012eab60` | Messaging::LoopBase::CtorWithFlag | confirmed | Constructor variant of Messaging::LoopBase::Ctor that takes a name string (param_2) and an extra dword (param_3) which it stores at this+0x74 instead of the co… |

**Scaleform::IScaleformActionHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012837d0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor body (67 bytes): stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. No calls. |
| `01283830` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor body (67 bytes): stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. No calls. |

**Utility::DisplayActionBuilder** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013275c0` | Utility::DisplayActionBuilder::ctor | confirmed | Fastcall constructor (returns this): stores the Utility::DisplayActionBuilder vtable into the object and zeroes the payload pointer at +4. |
| `01327680` | Utility::DisplayActionBuilder::DtorBody | confirmed | Thiscall destructor body: stores the Utility::DisplayActionBuilder vtable, then when the payload at +4 is non-null destroys it through 012f9810, frees it with … |
| `01327750` | Utility::DisplayActionBuilder::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): stores the builder vtable, destroys the payload at +4 through 012f9810 and _free when present, then frees this when… |
| `013276b0` | Utility::DisplayActionBuilder::SetPayloadDword | likely | Thiscall (builder, dword value): frees the existing payload at +4 (012f9810 and _free), allocates a 0x14-byte record through 00d833a6, zeroes its fields throug… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ea4f0` | ScaleformActionOwner_Register | likely | Thiscall on an owner object (ECX) with one argument, a non-null IScaleformActionHandler object (param_2). It reads the action name from vtable+4 (a C string), … |

### `engine.effect`

**ActorCameraEffectEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dd120` | ActorCameraEffectEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorCameraEffectEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +8 i… |
| `007dd1c0` | ActorCameraEffectEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dd120 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dd1e0` | ActorCameraEffectEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dd1c0. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorEffectEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dbab0` | ActorEffectEvent::DtorBody | confirmed | Fastcall (ECX=event). Stores the ActorEffectEvent vtable at +0 and at +0x4c. If the pointer at +0x50 is non-null it calls that object's virtual slot [0] with f… |
| `007dbc20` | ActorEffectEvent::ScalarDeletingDtor | confirmed | Virtual slot [3] (ECX=event, flag argument). Calls the base body 007dbab0, then calls _free(this) only when bit 0 of the flag is set; returns this. |
| `007dc7b0` | ActorEffectEvent::ActorEffectEvent | confirmed | thiscall, one stack arg stored at this+0x54. Calls 007dc290 first. Stores EffectEventInterface::vftable at this+0x4c, then ActorEffectEvent::vftable at this+0 … |
| `007dc890` | ActorEffectEvent::VSlot1_ProcessFlags | likely | thiscall, args: an ignored dword and a flags word; returns bool. Bit 1 with a valid child at this+0x50 (child vtable +0xc returns true): calls child vtable +8 … |

**ActorEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007db930` | ActorEvent::DtorBody | confirmed | Fastcall (ECX=event). Stores ActorEvent::vftable at +0, then calls 007e7980 (base cleanup; the receiver and the frame cookie are not visible in the pseudocode). |
| `007dba00` | ActorEvent::ScalarDeletingDtor | confirmed | Virtual slot [3] (ECX=event, flag argument). Stores ActorEvent::vftable, calls 007e7980, and calls _free(this) only when bit 0 of the flag is set; returns this. |
| `007dc290` | ActorEvent::Ctor | confirmed | Thiscall (ECX=event, three stack arguments). Calls 007e78e0 (base constructor), stores the ActorEvent vtable at +0, and stores its two stack arguments at +0x44… |

**ActorEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c57b0` | ActorEventHandler::ActorEventHandler | confirmed | Installs an SEH frame, calls FUN_008c49a0 with incoming this in ECX, stores ActorEventHandler::vftable at this+0 and param_2 at this+0xc, restores the prior ex… |
| `008c5860` | ActorEventHandler::ScalarDeletingDtor | confirmed | Stores TargetProcessorEventHandler::vftable at this+0. If flag bit 0 is set, calls _free(this); otherwise returns this. Returns this with ret 4. |

**ActorLightEmitterEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dcef0` | ActorLightEmitterEvent::ActorLightEmitterEvent | confirmed | thiscall, one stack arg stored at this+0xa0 (owner record). Calls 007dc7b0 (ActorEffectEvent constructor), stores ActorLightEmitterEvent::vftable at this+0 and… |
| `007dcfa0` | ActorLightEmitterEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorLightEmitterEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +8 i… |
| `007dd040` | ActorLightEmitterEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dcfa0 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dd060` | ActorLightEmitterEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dd040. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorLightsaberTrailEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dd420` | ActorLightsaberTrailEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorLightsaberTrailEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +… |
| `007dd4c0` | ActorLightsaberTrailEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dd420 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dd4e0` | ActorLightsaberTrailEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dd4c0. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorMeshEmitterEvent** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dd1f0` | ActorMeshEmitterEvent::ActorMeshEmitterEvent | confirmed | thiscall, one stack arg stored at this+0xa0 (owner record). Calls 007dc7b0 (ActorEffectEvent constructor), stores ActorMeshEmitterEvent::vftable at this+0 and … |
| `007dd2a0` | ActorMeshEmitterEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorMeshEmitterEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +8 it… |
| `007dd340` | ActorMeshEmitterEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dd2a0 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dd360` | ActorMeshEmitterEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dd340. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorModelMaterialEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dce20` | ActorModelMaterialEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorModelMaterialEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +8 … |
| `007dcec0` | ActorModelMaterialEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dce20 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dcee0` | ActorModelMaterialEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dcec0. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorParticleEmitterEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dcca0` | ActorParticleEmitterEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorParticleEmitterEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +… |
| `007dcd40` | ActorParticleEmitterEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dcca0 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dcd60` | ActorParticleEmitterEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, 8 bytes: sub ecx,0x4c then jmp 007dcd40. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting destructor. |

**ActorRumbleEffectEvent** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c5540` | ActorRumbleEffectEvent::ActorRumbleEffectEvent | confirmed | Calls FUN_007dc7b0, stores ActorRumbleEffectEvent::vftable at this+0 and this+0x4c, stores the stack argument at this+0xa0, increments the refcount at argument… |
| `008c55f0` | ActorRumbleEffectEvent::~ActorRumbleEffectEvent | confirmed | Stores ActorRumbleEffectEvent::vftable at this+0 and this+0x4c. If the owner at this+0xa0 has a positive refcount at owner+8, decrements it. Calls FUN_007dbab0… |
| `008c5680` | ActorRumbleEffectEvent::VSlot1_GetField54 | confirmed | Bytes: mov eax,[ecx+0x54]; ret. Returns the dword at this+0x54 (secondary-base field). |
| `008c5690` | ActorRumbleEffectEvent::ScalarDeletingDtor | confirmed | Calls destructor 008c55f0 on this, then frees with _free when bit 0 of the stack flag byte is set. Returns this (ret 4). |
| `008c56b0` | ActorRumbleEffectEvent::ScalarDeletingDtorThunk | confirmed | Bytes: sub ecx,0x4c; jmp 008c5690. Adjusts this by -0x4c and jumps to ScalarDeletingDtor 008c5690. |
| `008d1c20` | ActorRumbleEffectEvent::Slot10_CreateFromActor | likely | Virtual slot [10] of the actor rumble effect event: resolves an actor through FUN_007e8270 and, only when that object exists and the byte at +0x22d is set, all… |

**ActorSoundEmitterEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dcb10` | ActorSoundEmitterEvent::DtorBody | confirmed | fastcall (this=ECX), no stack args. Stores ActorSoundEmitterEvent::vftable at this+0 and this+0x4c, then if the owner at this+0xa0 has a positive dword at +8 i… |
| `007dcbb0` | ActorSoundEmitterEvent::VSlot3_ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dcb10 (DtorBody) with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Re… |
| `007dcbd0` | ActorSoundEmitterEvent::SecondaryVSlot0_DeletingDtorThunk | likely | void, no args, bytes 83 e9 4c e9 d8 ff ff ff: sub ecx,0x4c then jmp 007dcbb0. Adjusts this by -0x4c to the primary object and tail-jumps to the scalar deleting… |

**AppCompositeEffectEntity** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dc320` | AppCompositeEffectEntity::DtorBody | confirmed | Fastcall (ECX=entity). Stores AppCompositeEffectEntity::vftable at +0 and +0x14. When the pointer at +0x30 is non-null it decrements the count at its +4 under … |
| `007dd4f0` | AppCompositeEffectEntity::ScalarDeletingDtor | confirmed | thiscall, one stack arg (flag byte), ret 4. Calls 007dc320 with this, then if flag bit 0 is set calls _free on this (through the 00d82463 thunk). Returns this. |

**AppLightsaberTrailEntity** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007d8ab0` | AppLightsaberTrailEntity::Ctor | confirmed | Thiscall (ECX=entity, six stack arguments, one is a float[4] pointer; SEH frame). Stores the RefCounted vtable at +0 with the count at +4 (set to 1, then incre… |
| `007d8cf0` | AppLightsaberTrailEntity::DtorBody | confirmed | Fastcall (ECX=entity). Stores the entity vtable at +0 and +8, frees the buffers at +0x9c, +0x98, +0xa4 and +0xa0 with _free, stores the record-list vtable at +… |
| `007d9580` | AppLightsaberTrailEntity::ScalarDeletingDtor | confirmed | Virtual slot [0] (ECX=entity, flag argument). Calls the destructor body 007d8cf0, then calls _free(this) only when bit 0 of the flag is set; returns this. |
| `007d8cb0` | AppLightsaberTrailEntity::CopyRecord16Dwords | likely | Virtual slot [7] of the secondary vtable (ECX = object+8, stack: pointer to 16 dwords). Copies 16 dwords (64 bytes) from the argument into this+0x38 (secondary… |
| `007d9360` | AppLightsaberTrailEntity::DtorWrapper | likely | Void wrapper (ECX passed through unchanged) that installs an SEH frame and calls the entity destructor body 007d8cf0. No caller is recorded in the database. |
| `007db310` | AppLightsaberTrailEntity::UpdateTrail | likely | Virtual slot [3] of the second AppLightsaberTrailEntity vtable (object+8, ECX adjusted; 1456 bytes). When the entity's flag at +0x80 bit 0 is clear and the own… |

**EffectEventInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007dba70` | EffectEventInterface::DtorBody | confirmed | Fastcall (ECX=object). Stores EffectEventInterface::vftable at +0 and nothing else (decoded: mov dword ptr [ecx], 0x17f0a0c; ret). |
| `007dba90` | EffectEventInterface::ScalarDeletingDtor | confirmed | Virtual slot [0] (ECX=object, flag argument). Stores EffectEventInterface::vftable, calls _free(this) only when bit 0 of the flag is set; returns this. |

**FishingProcessor** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b641c0` | SpawnEffectAtScaledPosition | likely | When the global object returned by FUN_00769710 has a non-null +0x38 field it builds a size vector from param_3 * DAT_01814c14 (with DAT_017ea5a4 - that value … |

**SensoryEffectDefinitionManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ee20` | SensoryEffectDefinitionManagerInterface::~SensoryEffectDefi… | confirmed | Seven-byte destructor body: writes SensoryEffectDefinitionManagerInterface::vftable into the object and returns; no field is released. |
| `0089ee30` | SensoryEffectDefinitionManagerInterface::ScalarDeletingDest… | confirmed | MSVC scalar deleting destructor: writes SensoryEffectDefinitionManagerInterface::vftable into the object and calls _free(this) when bit 0 of param_2 is set. |

**SoeUtil::HashListMap** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a9e80` | HashListMap_ActorCompositeEffectDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorCompositeEffectDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (… |
| `008a9f40` | HashListMap_ActorModelMaterialDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorModelMaterialDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x… |
| `008a9fa0` | HashListMap_ActorCameraEffectDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorCameraEffectDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x4… |
| `008aa000` | HashListMap_ActorRumbleEffectDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorRumbleEffectDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x4… |
| `008aa060` | HashListMap_ActorMeshEmitterDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorMeshEmitterDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x40… |
| `008aa0c0` | HashListMap_ActorLightsaberTrailDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorLightsaberTrailDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (… |
| `008aa730` | HashListMap_ActorSoundEmitterDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorSoundEmitterDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x4… |
| `008aa790` | HashListMap_ActorProjectileDefinition_Construct | confirmed | Constructor: installs the SoeUtil::HashListMap<int,ActorProjectileDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x400… |
| `008a9dc0` | HashListMap_ActorParticleEmitterDefinition_Construct | likely | Constructor: installs the SoeUtil::HashListMap<int,ActorParticleEmitterDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (… |
| `008a9e20` | HashListMap_ActorLightEmitterDefinition_Construct | likely | Constructor: installs the SoeUtil::HashListMap<int,ActorLightEmitterDefinition,256,-1> vtable, zeroes +4/+8/+0xc/+0x10 and memsets the 256 bucket pointers (0x4… |

**SoeUtil::HashMap** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a9e50` | HashMap_NameEntry_ActorLightEmitterDefinition_Construct | confirmed | Constructor of the name-lookup map for actor light emitters: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorLightEmitterDefinition,256>:… |
| `008a9eb0` | HashMap_NameEntry_ActorCompositeEffectDefinition_Construct | confirmed | Constructor of the name-lookup map for composite actor effects: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorCompositeEffectDefinition… |
| `008a9f70` | HashMap_NameEntry_ActorModelMaterialDefinition_Construct | confirmed | Constructor of the name-lookup map for actor model materials: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorModelMaterialDefinition,256… |
| `008a9fd0` | HashMap_NameEntry_ActorCameraEffectDefinition_Construct | confirmed | Constructor of the name-lookup map for actor camera effects: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorCameraEffectDefinition,256>:… |
| `008aa090` | HashMap_NameEntry_ActorMeshEmitterDefinition_Construct | confirmed | Constructor of the name-lookup map for actor mesh emitters: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorMeshEmitterDefinition,256>::N… |
| `008aa0f0` | HashMap_NameEntry_ActorLightsaberTrailDefinition_Construct | confirmed | Constructor of the name-lookup map for lightsaber trails: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorLightsaberTrailDefinition,256>:… |
| `008aa760` | HashMap_NameEntry_ActorSoundEmitterDefinition_Construct | confirmed | Constructor of the name-lookup map for actor sound emitters: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorSoundEmitterDefinition,256>:… |
| `008aa7c0` | HashMap_NameEntry_ActorProjectileDefinition_Construct | confirmed | Constructor of the name-lookup map for actor projectiles: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorProjectileDefinition,256>::Name… |
| `008a9df0` | HashMap_NameEntry_ActorParticleEmitterDefinition_Construct | likely | Constructor of the name-lookup map for actor particle emitters: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorParticleEmitterDefinition… |
| `008aa030` | HashMap_NameEntry_ActorRumbleEffectDefinition_Construct | likely | Constructor of the name-lookup map for actor rumble effects: installs the SoeUtil::HashMap<int,SensoryEffectDefinitionManager<ActorRumbleEffectDefinition,256>:… |

**SoundControlDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008d1100` | SoundControlDefinition::~SoundControlDefinition | confirmed | Non-deleting destructor of the effect definition: rewrites its own vtable to SoundControlDefinition::vftable, then calls the base-destructor body FUN_007e0b10 … |
| `008d11a0` | SoundControlDefinition::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor (vtable slot [0]): rewrites the object vtable to SoundControlDefinition::vftable, inlines the destructor body by calling the ba… |
| `008d1190` | SoundControlDefinition::Slot4_Is9cZero | likely | Twelve-byte predicate decoded from the raw bytes (33 c0 / 39 81 9c 00 00 00 / 0f 94 c0 / c3): xors eax with itself, compares the dword at this+0x9c against zer… |

**TargetProcessorEventHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c5810` | TargetProcessorEventHandler::TargetProcessorEventHandler | confirmed | Sets up an exception frame, stores TargetProcessorEventHandler::vftable at this+0 (mov [ecx],0x01802274) and tears the frame down. No other fields are written … |
| `008c5750` | TargetProcessorEventHandler::ScalarDeletingDtor | likely | Stores TargetProcessorEventHandler::vftable at this+0, then calls _free on the block when bit 0 of the stack flag byte is set. Returns this (ret 4). Exception … |

**TileEventHandler** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c58c0` | TileEventHandler::ScalarDeletingDtor | confirmed | Stores TargetProcessorEventHandler::vftable at this+0. If flag bit 0 is set, calls _free(this); otherwise returns this. Returns this with ret 4. |

**WorldCameraEffectEvent** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b61d0` | WorldCameraEffectEvent::WorldCameraEffectEvent | confirmed | Calls the WorldEffectEvent constructor 008b6130, installs WorldCameraEffectEvent::vftable at *this and at +0x44, zeroes +0x94, stores param_4 into +0x90 and in… |
| `008b6410` | WorldCameraEffectEvent::~WorldCameraEffectEvent | confirmed | Stores WorldCameraEffectEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007d1b00 and 007d21d0(0), clears +0x94 … |
| `008b6280` | WorldCameraEffectEvent::GetField4c | likely | Returns the dword at this+0x4c; a single-load getter with no other side effect. |
| `008b6290` | WorldCameraEffectEvent::TailCallOwnerVtableSlot4 | likely | Bytes 8b 89 90 00 00 00 / 8b 01 / 8b 50 10 / ff e2: loads the owner pointer at this+0x90, takes its vtable and tail-jumps to byte offset 0x10 (slot 4) of it wi… |
| `008b63c0` | WorldCameraEffectEvent::StopCameraEffect | likely | Bytes show: when the handle at +0x94 is non-null, 007d1b00 is called only if param_3 is non-zero, while 007d21d0(param_2) is always called and the handle at +0… |
| `008b6400` | WorldCameraEffectEvent::AdjustingScalarDeletingDestructorTh… | likely | Subtracts 0x44 from ECX, then tail-jumps to 008b64b0; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b64b0` | WorldCameraEffectEvent::ScalarDeletingDestructor | likely | Calls the camera effect event destructor 008b6410 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
| `008d1220` | WorldCameraEffectEvent::Slot8_ComposeMatrix | likely | Virtual slot [8] of the camera effect event, taking a pointer to sixteen floats: reads the 4x4 matrix at (*(this+0x90))+0x50..+0x8c, computes the row-major pro… |

**WorldCompositeEffectEvent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c5380` | WorldCompositeEffectEvent::~WorldCompositeEffectEvent | confirmed | Stores WorldCompositeEffectEvent::vftable at this+0 and this+0x44 (param_1[0x11]). If this+0x94 is nonzero, calls FUN_00769710 and, when its +0x16c field is no… |
| `008c5410` | WorldCompositeEffectEvent::VSlot12 | confirmed | If this+0x94 is nonzero, calls FUN_00769710 and, when its +0x16c field is nonzero, calls FUN_007dc1e0 on that sub-object with a float 0.0 and int 2 pushed as s… |
| `008c5500` | WorldCompositeEffectEvent::ScalarDeletingDtor | confirmed | Calls the destructor 008c5380 on this, then frees the block with _free when bit 0 of the stack flag byte is set. Returns this (ret 4). |
| `008c5450` | WorldCompositeEffectEvent::VSlot8_CopyBlock64AndNotify | likely | Copies 16 dwords (64 bytes) from param_2 into this+0x50. If this+0x94 is nonzero and FUN_00769710()+0x16c is nonzero, runs NodeTable_FindKey on key this+0x94 (… |
| `008d1200` | WorldCompositeEffectEvent::Slot1_ManagerLookup | likely | Virtual slot [1] override for the world composite effect event: calls the client-singleton accessor FUN_00769710, loads the key at this+0x4c, fetches the manag… |

**WorldEffectEvent** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a2660` | WorldEffectEvent::~WorldEffectEvent | confirmed | Stores WorldEffectEvent::vftable at +0 and EffectEventInterface::vftable at +0x44, calls FUN_007e7980 with this in ECX, then restores the SEH chain. |
| `008b6130` | WorldEffectEvent::WorldEffectEvent | confirmed | Runs a base initialiser through 007e78e0, stores EffectEventInterface::vftable at +0x44, then WorldEffectEvent::vftable at *this and at +0x44, zeroes the byte … |
| `008a27f0` | WorldEffectEvent::ScalarDeletingDestructor | likely | Thin deleting destructor: calls FUN_008a2660 on this and then calls _free(this) when param_2 bit 0 is set. |

**WorldLightEmitterEvent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b7a20` | WorldLightEmitterEvent::Ctor | confirmed | Thiscall, ECX=new object, two stack arguments plus param_4 (an object with a refcount dword at +8). Calls the base WorldEffectEvent constructor 008b6130, store… |
| `010b7ac0` | AdjustorThunk_sub0x44_to_010b7c60 | confirmed | Adjustor thunk (bytes 83 e9 44 e9 98 01 00 00): subtracts 0x44 from ECX and jumps to 010b7c60. It is slot [0] of the secondary table at +0x44 of WorldLightEmit… |
| `010b7ad0` | WorldLightEmitterEvent::Dtor | confirmed | Fastcall, ECX=this. Stores WorldLightEmitterEvent::vftable at +0 and +0x44. If the flag byte at +0x48 is set and the child at +0x94 is non-null, calls 007d01f0… |
| `010b7c60` | WorldLightEmitterEvent::ScalarDeletingDtor | confirmed | Virtual [3] of WorldLightEmitterEvent, thiscall with one byte flag. Calls the body 010b7ad0, then calls _free on this when flag bit 0 is set, and returns this. |
| `010b7c80` | WorldLightEmitterEvent::Slot8_ComposeMatrix | likely | Virtual [8] of WorldLightEmitterEvent, thiscall with ECX=this and one float* stack argument. Copies 16 floats from the argument into this+0x50..+0x8c. If the c… |

**WorldMeshEmitterEvent** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b6960` | WorldMeshEmitterEvent::~WorldMeshEmitterEvent | confirmed | Stores WorldMeshEmitterEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007ab880 and 007ac200(0), clears +0x94 a… |
| `008b6810` | WorldMeshEmitterEvent::WorldMeshEmitterEvent | likely | Calls the WorldEffectEvent constructor 008b6130, installs WorldMeshEmitterEvent::vftable at *this and +0x44, stores param_4 into +0x90, zeroes +0x94, increment… |
| `008b68b0` | WorldMeshEmitterEvent::GetField4c | likely | Returns the dword at this+0x4c; the third copy of this getter in the shard (camera 008b6280, rumble 008b6580). |
| `008b68c0` | WorldMeshEmitterEvent::TailCallOwnerVtableSlot4 | likely | Loads the pointer at this+0x90, takes its vtable and tail-calls slot 0x10 with no arguments; the third copy of this thunk. |
| `008b68e0` | WorldMeshEmitterEvent::StopAndDeleteIfHandleMissingOrOwnerB… | likely | When the pointer at +0x94 is null, or the byte at owner+0x10 has bit 0 clear, calls virtual slot 0x30 with (1,1) and then slot 0xc with 1; slot 0xc holds 008b6… |
| `008b6910` | WorldMeshEmitterEvent::StopMeshEmitterEffect | likely | When the handle at +0x94 is non-null it calls 007ab880 only if param_3 is non-zero, always calls 007ac200(param_2), then clears +0x94 and the byte at +0x48. |
| `008b6950` | WorldMeshEmitterEvent::AdjustingScalarDeletingDestructorThu… | likely | Subtracts 0x44 from ECX, then tail-jumps to 008b6a00; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b6a00` | WorldMeshEmitterEvent::ScalarDeletingDestructor | likely | Calls the mesh-emitter event destructor 008b6960 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |
| `008d1370` | WorldMeshEmitterEvent::Slot8_ComposeMatrix | likely | Virtual slot [8] of the mesh emitter event, taking a pointer to sixteen floats: reads the 4x4 matrix at (*(this+0x90))+0x50..+0x8c, computes the row-major prod… |

**WorldParticleEmitterEvent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b71a0` | WorldParticleEmitterEvent::Ctor | confirmed | Thiscall, ECX=new object, two stack arguments plus the last argument param_4 (an object with a refcount dword at +8). Calls the base WorldEffectEvent construct… |
| `010b7240` | AdjustorThunk_sub0x44_to_010b7300 | confirmed | Adjustor thunk (bytes 83 e9 44 e9 b8 00 00 00): subtracts 0x44 from ECX and jumps to 010b7300. It is slot [0] of the secondary table at +0x44 of WorldParticleE… |
| `010b7250` | WorldParticleEmitterEvent::Dtor | confirmed | Fastcall, ECX=this. Stores WorldParticleEmitterEvent::vftable at +0 and +0x44. If the flag byte at +0x48 is set, and the pointer at +0x94 is non-null, calls 00… |
| `010b7300` | WorldParticleEmitterEvent::ScalarDeletingDtor | confirmed | Virtual [3] of WorldParticleEmitterEvent, thiscall with one byte flag (ret 4). Calls the body 010b7250, then calls _free on this when flag bit 0 is set, and re… |
| `010b7320` | WorldParticleEmitterEvent::Slot8_ComposeMatrix | likely | Virtual [8] of WorldParticleEmitterEvent, thiscall with ECX=this and one float* stack argument. Copies 16 floats from the argument into this+0x50..+0x8c. If th… |

**WorldRumbleEffectEvent** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008b6750` | WorldRumbleEffectEvent::~WorldRumbleEffectEvent | confirmed | Stores WorldRumbleEffectEvent::vftable at *this and +0x44; when the byte at +0x48 is set and +0x94 is non-null it calls 007d30a0 and 007d30b0(0), clears +0x94 … |
| `008b64d0` | WorldRumbleEffectEvent::WorldRumbleEffectEvent | likely | Calls the WorldEffectEvent constructor 008b6130, installs WorldRumbleEffectEvent::vftable at *this and +0x44, stores param_4 into +0x90, increments param_4's r… |
| `008b6580` | WorldRumbleEffectEvent::GetField4c | likely | Returns the dword at this+0x4c; identical to 008b6280 in the camera event class. |
| `008b6590` | WorldRumbleEffectEvent::TailCallOwnerVtableSlot4 | likely | Loads the pointer at this+0x90, takes its vtable and tail-calls slot 0x10 with no arguments; byte-identical to 008b6290 in the camera event class. |
| `008b66c0` | WorldRumbleEffectEvent::StopRumbleEffect | likely | Bytes show: when the handle at +0x94 is non-null, 007d30a0 is called only if param_3 is non-zero, while 007d30b0(param_2) is always called and the handle at +0… |
| `008b6740` | WorldRumbleEffectEvent::AdjustingScalarDeletingDestructorTh… | likely | Subtracts 0x44 from ECX, then tail-jumps to 008b67f0; the existing stack deleting-flag argument reaches that target unchanged. |
| `008b67f0` | WorldRumbleEffectEvent::ScalarDeletingDestructor | likely | Calls the rumble effect event destructor 008b6750 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**WorldSoundEmitterEvent** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b7540` | WorldSoundEmitterEvent::Ctor | confirmed | Thiscall, ECX=new object, two stack arguments plus param_4 (an object with a refcount dword at +8). Calls the base WorldEffectEvent constructor 008b6130, store… |
| `010b75e0` | AdjustorThunk_sub0x44_to_010b76d0 | confirmed | Adjustor thunk (bytes 83 e9 44 e9 e8 00 00 00): subtracts 0x44 from ECX and jumps to 010b76d0. It is slot [0] of the secondary table at +0x44 of WorldSoundEmit… |
| `010b75f0` | WorldSoundEmitterEvent::Dtor | confirmed | Fastcall, ECX=this. Stores WorldSoundEmitterEvent::vftable at +0 and +0x44. If the flag byte at +0x48 is set and the child pointer at +0x94 is non-null, sets t… |
| `010b76d0` | WorldSoundEmitterEvent::ScalarDeletingDtor | confirmed | Virtual [3] of WorldSoundEmitterEvent, thiscall with one byte flag. Calls the body 010b75f0, then calls _free on this when flag bit 0 is set, and returns this. |
| `010b7820` | WorldSoundEmitterEvent::Slot8_ComposeMatrix | likely | Virtual [8] of WorldSoundEmitterEvent, thiscall with ECX=this and one float* stack argument. Copies 16 floats from the argument into this+0x50..+0x8c. If the c… |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008c9560` | DtorRecord_TriggerArrayIString | confirmed | Sets count this+0x20 to 0, stores Array<ActorCompositeEffectTrigger,0,1> vftable at this+0x18, frees buffer +0x1c with alignment 4, and clears +0x1c. Then stor… |
| `008ee850` | ProcessCompositeEffectAtLocation | confirmed | Thiscall with an output pointer (param_2), an optional sequence value (param_3), an effect definition pointer (param_4), a float duration (param_7, replaced by… |
| `008eed20` | ProcessCompositeEffectAtLocationB | confirmed | Thiscall with the same first seven arguments as ProcessCompositeEffectAtLocation (plus two extra dwords param_8 and param_9 and a flag param_10). Same structur… |
| `010bf310` | EffectTypeName | confirmed | Stack-argument function with no callees. Switch on the effect type code: 0 PARTICLE, 1 SOUND, 2 MODELMATERIAL, 3 LIGHT, 4 CAMERA, 5 PROJECTILE, 6 MESH, 7 LIGHT… |
| `007dc1e0` | EffectDispatch_CallPair | likely | Thiscall with four stack arguments. Calls 007e7df0(param_2..param_5) once, and calls it again with the same arguments when the dword at this+8 is non-zero. |
| `007dc240` | EffectDispatch_CallPairFixed | likely | Thiscall with one stack argument. Calls 007e7df0(0, 4, argument, 0), and calls it again with the same values when the dword at this+8 is non-zero. |
| `007dc400` | EffectDispatch_CallPairFrom2c | likely | Fastcall (ECX=emitter). Reads the sub-object at +0x2c and the value at +0x24, calls 007e7df0(0, 4, value, 0), and calls it again with the same arguments when t… |
| `007dc450` | EffectRef_ReleaseWithStop | likely | Thiscall with one stack flag (ret 4 by convention of the caller; not decoded). When the flag is non-zero it calls 007dbc50 (Event_StopAndRelease). Then it decr… |
| `007de010` | CreateActorModelMaterialEvent | likely | thiscall, one stack out-pointer. Same shape as 007dde10: lookup 007dd650 (256-bucket by id), store at *out, predicate vtable +8, operator new 00d833a6 null-che… |
| `007df2a0` | AppCompositeEffectEntityPoolCreate | likely | thiscall with five stack args, returns a dword. Acquires an element with 007debd0 (pool pop, SEH-framed); when that returns zero it returns 0, otherwise it cal… |
| `008adf00` | Wrapper_010b5460 | likely | Sets up an SEH frame and calls FUN_010b5460 with incoming ECX unchanged; restores ExceptionList and returns. The callee tears down ActorMeshEmitterDefinition a… |
| `008adf50` | Wrapper_010b6220 | likely | Sets up an SEH frame and calls FUN_010b6220 with incoming ECX unchanged; restores ExceptionList and returns. The callee tears down ActorProjectileDefinition an… |
| `0091b6d0` | CleanupCall_ActorCompositeEffectDef | likely | SEH-framed cleanup leaf that calls FUN_00914a10, the destructor of an ActorCompositeEffectDefinition-shaped object (which restores ActorCompositeEffectDefiniti… |
| `00a2a540` | QueueCompositeEffectAtOwnLocation | likely | Calls the vtable slot 4 of the object at this+0x54, bails out unless FUN_0072a110 and FUN_0072f4d0 are both non-zero, then takes the record at *(*(this+0x808)+… |
| `00cc72c0` | ApplyParametersToLinkedObjects | likely | Requires an input and available context, transforms the supplied point, walks objects from FUN_008c52a0, applies a parameter block through vtable slot +0x20, a… |
| `00cdd040` | SyncTwoEffectTransforms | likely | Reads four transform dwords from the object returned through this+0xc0, then for each non-null handle at +0x144 and +0x14c calls its update slot and advances i… |
| `010bf460` | EffectTypeParse | likely | Stack-argument function taking a text pointer. Parses the text with _strtol (base 10); when it is numeric and at most 8, returns the number. Otherwise hashes t… |

### `engine.overhead`

**OverHeadBitmapElement** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4b0a0` | OverHeadBitmapElement::~OverHeadBitmapElement | confirmed | Stores the OverHeadBitmapElement vtable at ECX, runs the OverHeadBitmap array teardown 00a49c10 on the object, then stores the OverHeadElement base vtable; it … |
| `00a4b100` | OverHeadBitmapElement::ScalarDeletingDtor | confirmed | Virtual slot [0] of OverHeadBitmapElement: calls the destructor body 00a4b0a0 and calls _free on the object when bit 0 of the flag is set; returns the object. |

**OverHeadChatBubbleElement** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bf0350` | OverHeadChatBubbleElement::Ctor | confirmed | Constructs the overhead base element and string array, initializes the chat bubble vtable, copies three constructor values, and sets timing defaults. |
| `00bf0450` | OverHeadChatBubbleElement::Dtor | confirmed | Restores the chat bubble vtable, destroys its fixed-string array and string member, then restores the OverHeadElement base vtable. |
| `00bf04d0` | OverHeadChatBubbleElement::ScalarDeletingDtor | confirmed | Calls the chat bubble destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |

**OverHeadElement** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00957900` | OverHeadElement::GetOffset18 | likely | Returns this pointer plus 0x18. |
| `00957910` | OverHeadElement::GetOffset40 | likely | Returns this pointer plus 0x40. |

**OverHeadNameIconElement** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4c330` | OverHeadNameIconElement::OverHeadNameIconElement | confirmed | Constructor: copies a 4-dword parameter block from the static data at 01cb53d0 and calls FUN_00a4c260 (OverHeadBitmapElement constructor chain) forwarding the … |
| `00a4c430` | OverHeadNameIconElement::ScalarDeletingDtor | confirmed | MSVC scalar deleting destructor occupying vtable slot 0: restores the OverHeadNameIconElement vtable, runs the base-subobject cleanup FUN_00a4b0a0, and frees t… |
| `00a4c3e0` | OverHeadNameIconElement::~OverHeadNameIconElement | likely | Non-deleting destructor body: stores the OverHeadNameIconElement vtable back into the object and calls FUN_00a4b0a0, which resets the vtable to OverHeadBitmapE… |

**OverHeadTextElement** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0096f540` | OverHeadTextElementCtor | confirmed | Builds base overhead-element state from its arguments, installs the OverHeadTextElement vtable, stores two supplied values, sets fields to 0 and 0x14, and call… |
| `0096f620` | OverHeadTextElementDtor | confirmed | Installs the OverHeadTextElement vtable, calls the text-element cleanup routine, then restores the OverHeadElement base vtable. |
| `0096f680` | OverHeadTextElementScalarDeletingDtor | confirmed | Calls the OverHeadTextElement destructor and frees object storage when the deleting flag's low bit is set. |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a4c490` | OverHead_ChatBubbleResourcesInit | confirmed | Initialises a chat-bubble/overhead resource object: stores the owner at +8, default-constructs its element-set arrays (FUN_00a4b2e0, FUN_00a4b340, FUN_00a4b3a0… |
| `00962ef0` | SetDisplayTransformAndRefresh | likely | Copies four supplied words to this+0x12c..+0x138, sets bit 0 at +0x140, and calls FUN_009622f0 to refresh the display element. |
| `00a4b120` | OverHeadElementSet_DtorBody | likely | Calls 00a47cc0 for each of the eight group indices 0 to 7 (deleting the pointers in each group), then runs the eight-inline 0x24-element array teardown 00a49d6… |
| `00a4cdf0` | OverHeadElementSet_Init | likely | Initialiser of an overhead element-set object (0x160 bytes): stores the two caller dwords at +0/+4, copies a 4-dword vector (four floats) from param_4 into +0x… |
| `00a4cea0` | OverHeadManager_CreateElementSet | likely | Factory: allocates a 0x160-byte object through FUN_00d833a6, constructs it with FUN_00a4cdf0 (passing *(this+8) and the caller argument), then appends the new … |
| `00a4d2c0` | OverHead_AppendRow9 | likely | Appends a 9-value row to the row table at +0x538 (count +0x53c): ensures the next element slot through FUN_00a4c020, forces the element's backing storage to 9 … |
| `00a4d3e0` | OverHead_AppendRow3_A | likely | Appends a 3-value row to the row table at +0x5f4 (count +0x5f8): ensures the next element slot through FUN_00a4c140, forces a 3-entry backing store (FUN_00738f… |
| `00a4d480` | OverHead_AppendRow3_B | likely | Appends a 3-value row to the row table at +0x628 (count +0x62c): same as the +0x5f4 builder but writing to the second 3-int table; ensures the slot through FUN… |
| `00b6a6e0` | BuildOverheadTextEntry | likely | Looks up a keyed entry, checks its status, builds an IString and an overhead-text element when needed, and updates the associated entry state. |
| `00b6a8c0` | RefreshOverheadTextEntries | likely | Walks the linked entries, resolves their associated keys, builds or updates IString values and overhead-text elements, and releases temporary strings. |

### `engine.render`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a6460` | BaseClient::InitGraphicsDLL | confirmed | One-shot graphics bootstrap guarded by the byte DAT_01be47e8: when clear, it reads three client config strings (Libraries/GraphicsDll -> GraphicsDriver.dll, Li… |

**Deep::Camera** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0074bbb0` | CameraScalarDeletingDtor | confirmed | Thiscall with one byte flag. Calls 0074bb10 (Deep::Camera destructor body), calls _free(this) when bit 0 of the flag is set, and returns this. Virtual slot [0]… |
| `0074ba50` | CameraCtor | likely | Thiscall with three stack args (param_2, param_3, param_4). Calls 0073b3c0 and the RefCounted base init 0071b950, stores Deep::Camera::vftable at +0, zeroes th… |
| `0074bb10` | CameraDtorBody | likely | Fastcall with an SEH frame. Stores Deep::Camera::vftable at +0, releases the object at +0x16c through its vtable slot 0 with flag 1 (scalar deleting destructor… |

**Deep::Engine::RenderThread** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007485b0` | Deep::Engine::RenderThread::RenderThread | confirmed | Constructor: calls 00fc4cc0 with stack size 0x20000, second argument 4 and the thread name string Deep Engine Renderer, stores the RenderThread vftable, stores… |
| `00748700` | Deep::Engine::RenderThread::~RenderThread | confirmed | Scalar deleting destructor (vtable slot [0] of RenderThread): calls the body 00748670 on this, then calls _free on this when bit 0 of the flag argument is set;… |
| `00748670` | Deep::Engine::RenderThread::~RenderThread | likely | Destructor body: stores the RenderThread vftable, then calls 00fc45a0 (guard destroy), 00fc4da0 (close the thread handle), 00fc5140 (counted gate destructor bo… |
| `0074ea70` | Deep::Engine::RenderThread::ServiceLoop | likely | Virtual slot [2] of Deep::Engine::RenderThread::vftable. Loops until the thread-state byte (00fc4ac0) is set: waits on the object's semaphore (00fc4dd0, infini… |

**Deep::Matrix44Parameter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0074b870` | Matrix44ParameterCtorChain | confirmed | Fastcall (ECX = this). Stores three vftables into *this in order: Deep::ParameterValue::vftable, Deep::UndefinedParameter::vftable, Deep::Matrix44Parameter::vf… |
| `0074b8f0` | Matrix44ParameterGetValuePtr | likely | Virtual slot [11] of Deep::Matrix44Parameter::vftable, thiscall with no stack args. Returns this+0x10 (decoded: lea eax,[ecx+0x10]; ret). It does not read the … |
| `0074d5e0` | Matrix44ParameterAssignChanged | likely | Virtual slot [7] of Deep::Matrix44Parameter::vftable, thiscall with one stack arg (a value object). Calls the argument's vtable slot +0x2c to get a float* (old… |
| `0074d620` | Matrix44ParameterAssignChangedRaw | likely | Virtual slot [3] of Deep::Matrix44Parameter::vftable, thiscall with one stack arg (pointer to 16 floats). Compares this+0x10 with the argument through 0074b9e0… |

**Deep::ParameterValue** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0074b900` | ParameterValueSetVtable | likely | Fastcall (ECX = this) with an SEH frame. Stores Deep::ParameterValue::vftable at +0 and returns. The pseudocode shows no other field work; it is called from on… |
| `0074b980` | ParameterValueDeletingDtor | likely | Thiscall with one byte flag, SEH frame. Stores Deep::ParameterValue::vftable at +0, calls _free(this) when bit 0 of the flag is set, and returns this. |

**GDrawDeviceD3D9** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `004622fe` | LockVertexBufferRegion | likely | Locks a vertex-buffer region. If cursor + minimum count (param_3) exceeds capacity, resets cursor to 0. Requested count (param_4) defaults to capacity when zer… |
| `004623b5` | LockIndexBufferRegion | likely | Same ring-buffer lock as 004622fe but for the 16-bit index buffer: wraps the cursor at +0x44, clamps the count against capacity +0x40, calls index-buffer slot … |
| `004624ac` | ApplyInitialRenderState | likely | Programs the draw device standard 2-D state block while byte +0x74 is set: about twenty SetRenderState calls through device slot +0xe4 (states 0x16=1, 0x10=0, … |
| `00466d6e` | DrawDeviceCtor | likely | Constructor of the Direct3D 9 draw device: stores the GInterface vtable, calls FUN_004c9e10 for base state, installs the interface vtables PTR_FUN_017649e8 / P… |
| `00466e97` | ScalarDeletingDtorDrawDevice | likely | Scalar deleting destructor of the draw device: runs the destructor body FUN_00462140 and, when bit 0 of the flag argument is set, frees the object through the … |

**(no class)** (57)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00463942` | RegisterGDrawDeviceD3D9Classes | confirmed | Registers the Direct3D 9 draw backend with the GFC class factory twice: first the class GDrawDeviceD3D9 (description Direct3D 9 DrawDevice) implementing interf… |
| `0074fd60` | InputLayoutLoadXml | confirmed | Thiscall walking the XML elements named InputLayout. For each layout it reads the Name attribute (DAT_017ea850), hashes it with the inlined Jenkins one-at-a-ti… |
| `007502c0` | CreateIndexBufferChecked | confirmed | Thiscall (device wrapper, flag, name, byte size, usage code, out). Validates size 1..999999 and usage 1 or 2, then calls device vtable slot +0x7c to create the… |
| `00750670` | CreateTextureChecked | confirmed | Thiscall. Calls device vtable slot +0x8c to create the texture, with additional device queries through slots +0x2c/+0x30 and +0x14c/+0x148; on failure it probe… |
| `008ddbc0` | SetRenderFloraEnabled | confirmed | Writes the exact config pair Terrain/RenderFlora through 006fe530 with the value pointer selected by the bool argument (DAT_01802bc0 when true, PTR_DAT_0177051… |
| `008ef350` | VideoCaptureSetOutputFile | confirmed | Thiscall with an output-file string argument. Copies the settings record at this+0x8f8 into a local (008ef220). Calls 010f3360 and, when it returns zero, appli… |
| `008ef510` | VideoCaptureSetResolution | confirmed | Thiscall with an int argument. Maps 1 to 1, 2 to 3, and any other value to 5 as the resolution code. Copies the settings record at this+0x8f8 into a local (008… |
| `008ef740` | VideoCaptureSetQuality | confirmed | Thiscall with an argument param_2. Copies the settings record at this+0x8f8 into a local (008ef220), queries and applies through 010f3360, 010f37e0 and vtable … |
| `009173d0` | EnsureRenderQualityKeysSet | confirmed | When this+0x6d8 is non-null and FUN_006fc440() is nonzero, reads GraphicsQuality, TextureQuality and ShadowQuality under Rendering. For each value equal to -1,… |
| `0091dc20` | UpdateCameraFromConfigOrPlayer | confirmed | Computes camera position/orientation and applies it: when the byte at this+0x8f is set it reads Camera.X/Camera.Y/Camera.Z/Camera.Heading/Camera.Pitch/Camera.R… |
| `0091dde0` | LoadGraphicsQualitySettings | confirmed | Reads the Rendering/GraphicsQuality, Rendering/TextureQuality and Rendering/ShadowQuality settings through FUN_009109c0 (defaults -1) and then applies them via… |
| `0091e5a0` | ApplyShadowQualitySettings | confirmed | Configures shadow rendering: asks the renderer for default/current values (FUN_007929d0(99), FUN_00792310, FUN_00791300) and writes/reads the Rendering setting… |
| `0091e670` | ApplyRenderDistance | confirmed | Resolves the Rendering/RenderDistance setting (FUN_00919990 with a default); when the value is not positive it derives one from the current renderer configurat… |
| `004627cc` | InitStageStateDefaults | likely | Writes the default texture-stage state record of one stage inside the draw-state cache: the record starts at this + stage*0x28 and receives 1, 2, 0, 1, 2, 0 at… |
| `00462910` | DtorVtable1764A18 | likely | Destructor body of the small record class whose vtable is 0x1764a18: rewrites the vtable pointer, calls the base teardown FUN_0045b04d, and then stores the pur… |
| `0046295d` | ScalarDeletingDtor1764A18 | likely | Scalar deleting destructor: runs the object destructor body FUN_0045b132, and when the flag argument has bit 0 set frees the object with _free; returns the obj… |
| `00462979` | RecordCtor1764A48 | likely | Constructs a draw record: stores vtable 0x1764a48, owner param_2 at +0x4, zeroes the list links +0x8/+0xc, sets state +0x10 = 2 and +0x14/+0x18 = 0, then copie… |
| `004629d3` | DtorVtable1764A48 | likely | Destructor body of the record class whose vtable is 0x1764a48: rewrites the vtable pointer, calls the base teardown FUN_0045b21a, and stores the purecall base … |
| `00462c36` | RecordCtor1764670 | likely | Constructs the 0x28-byte record whose vtable is 0x1764670: zeroes +0x8/+0xc/+0x14/+0x18, stores the vtable, owner param_2 at +0x4 and state 2 at +0x10, then co… |
| `00462c72` | ScalarDeletingDtor1764670 | likely | Scalar deleting destructor: runs the destructor body FUN_0045b2cf and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `00462d00` | RecordCtor1764A68 | likely | Constructs the 0x4c-byte record whose vtables are 0x1764a68 and 0x1764a60 (base purecall 0x1764610): zeroes +0x8/+0xc/+0x14/+0x24/+0x28, stores owner param_2 a… |
| `00462d6a` | DtorVtable1764A68 | likely | Destructor of the 0x1764a68 record: restores the two vtable pointers (0x1764a68 at +0, 0x1764a60 at +0x28), calls the base teardown FUN_0045b2fa, unlinks the r… |
| `00462de3` | RecordCtor1764A88 | likely | Constructs the 0x38-byte record whose vtables are 0x1764a88 and 0x1764a80: zeroes +0x8/+0xc, stores owner param_2 at +0x4, state 0 at +0x10, zeroes +0x18, call… |
| `00462e52` | DtorVtable1764A88 | likely | Destructor of the 0x1764a88 record: restores the vtables 0x1764a88/0x1764a80, calls the base teardown FUN_0045b316, unlinks the record from its list, and leave… |
| `00462e95` | RecordCtor1764688 | likely | Constructs a 0x20-byte list record: stores vtable 0x1764688, owner param_2 at +0x4, state 1 at +0x10, links +0x8 to the list head at owner+0x54 and +0xc to own… |
| `00462ed2` | ScalarDeletingDtor1764688 | likely | Scalar deleting destructor: runs the destructor body FUN_0045b395 and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `0046333c` | EnsureGridRowRecords | likely | Creates the per-row draw records of a grid lazily: when the table state at +0x1c is 0 it allocates a 0x2c-byte record through FUN_00d833a6 and initialises it w… |
| `0046340f` | ProbeDeviceCaps | likely | Fills a caller-provided device information block: writes 7 at +0, zeroes +4..+0x10, calls FUN_0045bb76 on the four middle fields, sets +0x14..+0x20 to 0x7fff, … |
| `004634d7` | GetDisplayExtent | likely | Computes the display rectangle: when the global function pointer DAT_01bc45f4 is null it fills the caller rectangle from GetSystemMetrics values with a -1 adju… |
| `0046358f` | GetDeviceDirect3D | likely | Returns the IDirect3D9 object behind the draw device: when this+0x14 is set it takes the device pointer at *(this+0x14)+0x10, calls device virtual slot +0x18 w… |
| `004636e0` | PresentDirtyRegion | likely | Presents the frame with an optional dirty rectangle: returns 0 unless byte +0x49 is set and the display object at +0x40 exists, and returns 0 while the held in… |
| `00463dc9` | HitTestTypedRecords | likely | When the object field +0x1c is non-null, scans its 0x20-byte records using count +0x28. For tag 5, resolves the page/index through 0045fc2b, prepares the decod… |
| `00463ef4` | CollectRegistryPointers | likely | Requires a non-null interface argument, a nonzero receiver field +0x40, and a nonzero result from interface slot +0x170. Gets the registry count from 0045c30d.… |
| `00465260` | ScalarDeletingDtorRecordA | likely | Scalar deleting destructor of a small record: releases the interface pointer stored at +0xc through its virtual slot +0x10 with 0, then frees the record with _… |
| `00465285` | ScalarDeletingDtorRecordB | likely | Scalar deleting destructor of a small record: releases the interface pointer stored at +0x10 through its virtual slot +0x10 with 0, then frees the record with … |
| `004652aa` | AppendEncodedDrawRecord60 | likely | Reserves 0x3c bytes through 00461ffa, passes the reserved address and a local copy of the state pointer to 004600c2, appends the number of encoded bytes return… |
| `00465395` | EncodeDrawRecord14 | likely | Appends one encoded 20-byte draw record: allocates 0x14 bytes through the arena allocator FUN_00461ffa, encodes the state structure with FUN_004606d0, appends … |
| `004653d8` | EncodeDrawRecords14Batch | likely | Bulk variant of 00465395: while the input cursor still holds records it allocates 0x14-byte records (single-record fallback on failure), decodes each source re… |
| `00465480` | EncodeDrawRecord14B | likely | Second emission of the 20-byte record encoder for a different source structure: allocates 0x14 bytes through FUN_00461ffa, encodes with FUN_0045a0d4, appends t… |
| `0046556b` | EncodeDrawRecord28 | likely | Appends one encoded 28-byte draw record: allocates 0x1c bytes through FUN_00461ffa, encodes with FUN_004607c5, appends through FUN_0045a7ad and increments the … |
| `004655ae` | EncodeDrawRecords28Batch | likely | Bulk variant of 0046556b: allocates 0x1c-byte records while the input cursor holds entries, decodes each source record through FUN_00460885, applies the offset… |
| `00465656` | EncodeDrawRecord36 | likely | Appends one encoded 36-byte draw record: allocates 0x24 bytes through FUN_00461ffa, encodes with FUN_0046093b, appends through FUN_0045a7ad and increments the … |
| `00465699` | EncodeDrawRecords36Batch | likely | Bulk variant of 00465656: allocates 0x24-byte records while the input cursor holds entries, decodes each source record through FUN_0045a147, applies the offset… |
| `00466172` | DrawSurfaceCtorB | likely | Second constructor variant of the draw-surface class: runs the base constructor FUN_00486164, stores the vtables 0x1764d40/0x1764d3c, zeroes the same field set… |
| `00466213` | BindSurface | likely | If receiver field +0x44 is nonzero, calls receiver virtual slot +0x128. If param_2 is null, returns false; otherwise calls 0045f753(param_2,0) and returns whet… |
| `00466279` | DrawSurfaceDetach | likely | Detaches the surface object: returns 1 when nothing is attached at +0x44; otherwise it calls the callback at +0x50 when present (returning 0 if that callback r… |
| `00466ce6` | DrawDevicePopNode32 | likely | Pops a node from the draw device chain at +0x1c: when empty it allocates 0x20 bytes and constructs it through FUN_004c9ebe (with the FUN_00466b28 retry path), … |
| `00466f8b` | ConfigureStageStatesDefault | likely | Resets the draw state cache to the untextured default: writes the cache bases 1, 3 and 0, calls FUN_0045af64(0) to clear the textured stage mode, and re-initia… |
| `00467190` | ScalarDeletingDtor1764A18B | likely | Scalar deleting destructor: runs the destructor body 00462910 and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `004671ac` | ScalarDeletingDtor1764A48B | likely | Scalar deleting destructor: runs the destructor body 004629d3 and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `00467330` | ScalarDeletingDtor1764A68B | likely | Scalar deleting destructor: runs the destructor body 00462d6a and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `0046734c` | ScalarDeletingDtor1764A88B | likely | Scalar deleting destructor: runs the destructor body 00462e52 and, when bit 0 of the flag argument is set, frees the object with _free; returns the pointer. |
| `00746140` | RenderThreadShutdownAndJoin | likely | Thiscall with no arguments: sets the thread state flag through 00fc4a80, releases one count on the semaphore through 00fc4db0, and waits infinitely on the thre… |
| `00748900` | VertexDeclTypeFromName | likely | Plain function (one stack arg: char* type name). Takes the string length, then matches the name against the XML vertex-element type names with StrnCmpNoCase (0… |
| `00748b90` | ArrayFindNextNameIndex | likely | Thiscall (ECX = this, args: start index, C string). Scans the pointer array at this+0x8b8 from index start+1 up to the count at this+0x8bc; for each non-null e… |
| `008ef220` | CopyConstructRecord40 | likely | Thiscall copy constructor (ECX = destination, one stack argument = source; returns the destination, ret 4). Copies dwords 0..6 from the source; initialises an … |
| `008ef2e0` | CopyAssignRecord40 | likely | Thiscall copy assignment (ECX = destination, one stack argument = source; returns the destination, ret 4). Copies dwords 0..6, copy-assigns the IString at +0x1… |

### `engine.screenshot`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a80d0` | BaseClient::StartImageCapture | confirmed | Screenshot start: returns early when this+0x8c0 is already set or this+0x8e4 is clear (logging 'BaseClient::StartImageCapture - aborting image capture. Started… |

**CaptureCommon::JobQueueCallbackHandlerT<ImageCapture::JobRec>** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d0b3a0` | ImageCaptureJobQueueCallbackHandler_Ctor | confirmed | Installs JobQueueHandler then capture callback-handler vtables, stores callback value, and configures handler with bounds 0 and 0x7fffffff. |
| `00d0b410` | JobQueueCallbackHandler_DtorBody | confirmed | Restores callback-handler vtable, calls cleanup helper, then restores base JobQueueHandler vtable. |
| `00d0b470` | AdjustorThunk_00fc4db0 | confirmed | The 16-byte x86 body writes constant 1 as stack argument, adjusts ECX by 8, and jumps to 00fc4db0. |
| `00d0b490` | JobQueueCallbackHandler_ScalarDeletingDtor | confirmed | Calls callback-handler destructor, frees object when requested, and otherwise returns original pointer. |
| `00d0be00` | JobQueueCallbackHandler_OnJob | confirmed | Forwards callback arguments to routine that appends the job pointer to the queue record. |

**CaptureCommon::JobT<ImageCapture::JobRec>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d0be50` | ImageCaptureJob_Construct | confirmed | Initializes base job with argument 0, installs ImageCapture JobT vtable, and move-constructs its record reference. |
| `00d0bee0` | ImageCaptureJob_DtorBody | confirmed | Installs derived job vtable, releases referenced object through IString cleanup, and restores Job and RefCounted base vtables. |
| `00d0bf50` | ImageCaptureJob_ScalarDeletingDtor | confirmed | Calls JobT destructor, frees object when requested, and otherwise returns its original pointer. |

**ControllerPhotographer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af40c0` | ControllerPhotographer::CapturePhoto | confirmed | Builds a numbered path with the format %s/photo%05d.png (00402fd0), using the IString buffer at this+0x388 and the counter at this+0x394 (incremented after use… |

**FotomatPortraitData** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd3dc0` | FotomatPortraitData::ReleaseImageArrays | likely | Thiscall (ECX = record): for the three Array<unsigned_char,0,1> members at +0x2100, +0x20f0 and +0x20e0 it sets the array vftable, frees the buffer through 004… |
| `00bd3f10` | FotomatPortraitData::FotomatPortraitData | likely | Thiscall ctor with two stack dwords: stores arg1 at +0 and arg2 at +4, constructs an embedded object at +0x10 via 00ad5f60, and sets three Array<unsigned_char,… |
| `00bd4250` | FotomatPortraitData::ScalarDeletingDtor | likely | Thiscall scalar deleting destructor: calls 00bd3dc0 (release of the three image arrays) on this, then calls _free(this) (thunk 00d82463) when bit 0 of the flag… |

**FotomatPortraitDataManager** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd43f0` | FotomatPortraitDataManager::GetHeadshotBuffer | confirmed | Virtual slot [0] of FotomatPortraitDataManager::vftable (thiscall, two id dwords and two out-pointers, ret 0x10): looks up the id through 00bd2730; when there … |
| `00bd4f90` | FotomatPortraitDataManager::AdjustorThunk_ScalarDtor | confirmed | Adjustor thunk (8 bytes: sub ecx,4 then a jump) at slot [0] of the secondary FotomatPortraitDataManager vtable 01826874: it moves this back by 4 bytes and jump… |
| `00bd5190` | FotomatPortraitDataManager::FotomatPortraitDataManager | confirmed | Thiscall ctor (ECX = manager, one stack arg = owner): installs CharacterImageProvider and AssetDeliveryHandlerInterface vftables, copies the dword at singleton… |
| `00bd3ff0` | FotomatPortraitDataManager::RequestPortrait 🌐 | likely | Thiscall (ECX = manager) with id halves (arg1, arg2) and a kind string (arg3): does nothing when the id equals the sentinel pair DAT_01beaa20/DAT_01beaa24. Oth… |
| `00bd47c0` | FotomatPortraitDataManager::InsertEntry | likely | Thiscall (ECX = manager) with two dword args (id halves), ret 8: key = arg1 xor arg2 in a local; inserts a node through 00bd4620 into the map at this+0x1bc usi… |
| `00bd4930` | FotomatPortraitDataManager::StoreHeadshotBytes | likely | Thiscall (ECX = manager), four stack args (id lo, id hi, data pointer, size), ret 0x10: when size > 0, finds the entry through 00bd2730 or inserts one through … |
| `00bd5260` | FotomatPortraitDataManager::ScalarDeletingDtor | likely | Thiscall scalar deleting destructor of the manager: calls the dtor body 00bd4ed0 on this, then calls _free(this) (thunk 00d82463) when bit 0 of the flag byte i… |

**ScreenshotManager** (23)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00b4b520` | ScreenshotManager::SetupCapture | confirmed | Thiscall (ECX = manager; param_2 = character object; param_3 = capture id). Fails with a log when param_2 is null or its vtable +0x24 call returns 0, when the … |
| `00b4bb50` | ScreenshotManager::StoreHeadShotBufferToFile | confirmed | Thiscall (ECX = manager; guid pair, path IString, ownership flag). Finds the entry whose key is the guid pair combined with 0x2095e230 (HEADSHOT hash) by walki… |
| `00b4bd40` | ScreenshotManager::StorePortraitBufferToFile | confirmed | Thiscall (ECX = manager; guid pair, path IString, ownership flag). Same structure as 00b4bb50 with the key constant 0xd7a6acc7 (PORTRAIT hash): finds the entry… |
| `00b4c350` | ScreenshotManager::GetPortraitBuffer | confirmed | Stdcall-style with four stack arguments (key1, key2, out pointer, out size; ret 0x10), ECX = manager passed through unchanged. Calls 00b4b840(key1, key2, 'PORT… |
| `00b4c440` | ScreenshotManager::InitPortraitScene | confirmed | Thiscall (ECX = manager). Calls 00d30eb0(0), allocates objects of sizes 0x93b8, 0x10, 0x60, 0x8a0 and 0x2830 with 00d833a6 (each null-checked), creates rendere… |
| `00b4cf00` | ScreenshotManager::OnPostResult | confirmed | Virtual method in ScreenshotManager::vftable slot 3. Increments the dword at this+0x280, logs 'ScreenshotManager::OnPostResult RequestId=%d HttpResult=%d Url=%… |
| `00b4d2e0` | ScreenshotManager::StorePhotoToFile | confirmed | Thiscall (ECX = manager; path IString, ownership flag). The key is the two global dwords DAT_01be9770 and DAT_01be9774 (not parameters) combined with 0xd599dc1… |
| `00b4e220` | ScreenshotManager::LoadDefaultHeadShots | confirmed | Thiscall (ECX = manager). Assigns 'icon_UI_empty_portrait_64.png' to a local IString, loads the resource through 00769710 and a vtable call chain (+0x18 then +… |
| `00b4e810` | ScreenshotManager::ScreenshotManager | confirmed | Thiscall constructor with one stack argument (the engine pointer). Stores TinyHttp::HttpManagerHandler vftable, then ScreenshotManager vftable at [0], stores t… |
| `00b4e970` | ScreenshotManager::CreatePhoto | confirmed | Thiscall (ECX = manager) with nine stack arguments. Logs 'ScreenshotManager::CreatePhoto - request photo render' through 00704dd0, calls 00b4e650(DAT_01be9770,… |
| `00b4e9f0` | ScreenshotManager::CaptureTCGPortraitToBuffer | confirmed | Thiscall (ECX = manager; param_2 = character object). Calls 00b4b520(character, 0x7a) (SetupCapture) and, on success, logs 'ScreenshotManager::CaptureTCGPortra… |
| `00b4eba0` | ScreenshotManager::CapturePortraitToBuffer | confirmed | Thiscall (ECX = manager; param_2 = character object). Calls 00b4b520(character, 9) (SetupCapture); on success logs 'ScreenshotManager::CapturePortraitToBuffer … |
| `00b4ed20` | ScreenshotManager::CaptureHeadShotToBuffer | confirmed | Thiscall (ECX = manager; param_2 = character object). Returns 0 when 00b4b520(character, 9) fails. Otherwise logs 'ScreenshotManager::CaptureHeadShotToBuffer -… |
| `00b4faa0` | ScreenshotManager::GetHeadShotBuffer | confirmed | Stack arguments: guid pair and two out pointers. First looks up the HEADSHOT node with 00b4b840 (ECX = manager); when found it writes node+0xc to the first out… |
| `00b4fd60` | ScreenshotManager::PostPortraitToHttp | confirmed | Thiscall (ECX = manager; param_2 = character object). Reads the guid through the character interface (vtable +0x10 at param_2+0x54) and looks up the PORTRAIT n… |
| `00b4afa0` | ScreenshotManager::SetupCapture | likely | Sets up a capture: requires the headshot proxy from FUN_00cc5a30 and a device at [this+0x264]; fails with distinct log messages for an invalid HeadshotProxy, a… |
| `00b4b240` | ScreenshotManager::GetDefaultHeadShot | likely | Returns the cached default headshot: when the buffer pointer at [this+0x248] is non-null it writes that pointer to *param_4, the length at [this+0x24c] to *par… |
| `00b4b840` | ScreenshotManager::FindStorageNode | likely | Thiscall (ECX = manager), three stack arguments (key1, key2, kind string), ret 0xc. Hashes the kind string with 00705bc0, forms the key key2 ^ key1 ^ hash, ind… |
| `00b4e950` | ScreenshotManager::ScalarDeletingDestructor | likely | Scalar deleting destructor in ScreenshotManager::vftable slot 0: calls 00b4e4f0 on the object (ECX), then calls _free(this) when bit 0 of its stack byte is set… |
| `00b4f190` | ScreenshotManager::CaptureHeadShotToBuffer | likely | Thiscall (ECX = manager; param_2 = guid-bearing object). Gates on 00b4afa0(param_2, 9); logs 'ScreenshotManager::CaptureHeadShotToBuffer - request headshot ren… |
| `00b4f870` | ScreenshotManager::LoadHeadShot | likely | Thiscall (ECX = manager; param_2 = guid pair object; param_3 = dword). If the ready flag this+0x274 is clear it calls 00b4c440. Formats 'Resources\images\heads… |
| `00b50190` | ScreenshotManager::ProcessPortraitRequest | likely | Thiscall (ECX = manager; param_2 = guid object; param_3 = second dword; param_4 = flag byte). Calls 00b4c440 when the ready flag +0x274 is clear, then looks th… |
| `00b50330` | ScreenshotManager::Update | likely | Per-frame pump for queued image jobs. If the dword at +0x284 is set it calls 00d31890 first. Headshot queue: while the count at +0x240 is positive it takes the… |

**SoeUtil::HashList<FotomatPortraitData,8,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd5170` | HashList8_ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor in the HashList<FotomatPortraitData,8,-1> vtable (slot [0]): calls the map body 00bd4e60 on this, then calls _free(this) (t… |

**SoeUtil::HashListMap<int,FotomatPortraitData,8,-1>** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00bd4e20` | HashListMap8_ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor: stores the map vftable, drains the list through 00bd46f0 while the head (+8) is non-null, then calls _free(this) (thunk 00… |
| `00bd50f0` | HashListMap8_Ctor | confirmed | Thiscall ctor (ECX = map): stores the HashListMap<int,FotomatPortraitData,8,-1> vftable at +0, zeroes dwords +4 through +0x30 (head, tail, count and the eight … |
| `00bd46f0` | HashListMap8_RemoveNode | likely | Thiscall (ECX = map, arg = node): finds the node in its bucket (key at node+0x2118 & 7) and unlinks it; relinks the list neighbours (next +0x2110, prev +0x2114… |
| `00bd4d10` | HashListMap8_ClearHead | likely | Thiscall (ECX = map): loops while the list head (+8) is non-null, calling 00bd46f0 with the head node each time; returns nothing (plain ret). |
| `00bd4df0` | HashListMap8_DtorBody | likely | Thiscall (ECX = map): stores the map vftable at +0, then loops while the list head (+8) is non-null, calling 00bd46f0 to remove the head node each time. |
| `00bd4e60` | HashListMap8_DtorBodyEH | likely | Thiscall (ECX = map): stores the map vftable at +0, then loops while the list head (+8) is non-null, calling 00bd46f0 on the head node; the same body as the dr… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a8750` | EnsureRootStore | confirmed | First calls virtual slot +0x28 with this in ECX and no explicit stack arguments. If that returns nonzero and this+0x908 is null, it loads ./Resources/Security/… |
| `00b4d210` | FormatHeadShotPathAndCall | confirmed | Stack arguments: two dwords (guid). Formats 'Resources\images\heads\%llu.png' into a local SoeUtil IString with 00402fd0, passes that string and 0 to 00704840,… |
| `008a8500` | BufferEncoderStep | likely | Returns false when this+0x8e8 is null. Otherwise calls 010f3360 with ECX loaded from this+0x8f8 to choose the capture path: one branch compares stored dimensio… |
| `008a86b0` | GuardedWrapper_010003e0 | likely | Byte-verified conditional tail jump: mov ecx,[ecx+0x8e8] (8b 89 e8 08 00 00), test ecx,ecx (85 c9), jz +5 (74 05); when this+0x8e8 is non-zero it jumps (e9) to… |
| `008a86c0` | GuardedWrapper_010003f0 | likely | Byte-verified conditional tail jump: mov ecx,[ecx+0x8e8] (8b 89 e8 08 00 00), test ecx,ecx (85 c9), jz +5 (74 05); when this+0x8e8 is non-zero it jumps (e9) to… |
| `008a86d0` | Wrapper_010f4f20 | likely | Byte-verified __stdcall forwarder (8b 44 24 04 / 50 / e8 / 83 c4 04 / c2 04 00): pushes its single dword argument and calls FUN_010f4f20, then cleans the stack… |
| `008a86f0` | Wrapper_010f5350 | likely | Byte-verified thiscall thunk: mov ecx,[ecx+0x900] (8b 89 00 09 00 00) then jmp FUN_010f5350 (e9). FUN_010f5350 formats a local-time stamp with the pattern _%04… |
| `008a8990` | GuardedWrapper_0108c680 | likely | Byte-verified guarded tail jump: mov ecx,[ecx+0x904] (8b 89 04 09 00 00), test ecx,ecx, jz +5; when this+0x904 is non-zero it jumps (e9) to FUN_0108c680, other… |
| `008a8de0` | SetStoredVectorFromLookup | likely | Byte-verified: looks param_1 up through FUN_0102fb50 called on the sub-object at this+0x158 (8b 8e 58 01 00 00); when found, loads the three floats at node+0xc… |
| `008a8e50` | RestoreStoredVector | likely | Byte-verified (8b 89 ec 01 00 00 / e8 / b0 01 / c3): loads ecx from this+0x1ec (the same sub-object 008a8de0 writes) and calls FUN_009cef10 on it - that helper… |
| `00bd43a0` | HeadshotLookupOrRequest | likely | Thiscall with two dword args (id halves): calls 00bd2730 (lookup); when an entry exists it returns entry+0x2100 (the Headshot array); otherwise it calls 00bd3f… |
| `00bd4490` | PortraitKindWriteToPath | likely | Stack args (id lo, id hi, kind string, one more value): compares the kind string with Portrait (then uses 00bd4350) or Headshot (then uses 00bd43a0); when the … |
| `00bd4810` | HeadshotUpdateNotify | likely | Thiscall (ECX = owner object) with two id dwords: when the sub-object at this+0xc+0x3e0 exists and the id equals its guid at +0x3c8/+0x3cc, calls 00b4b820 with… |
| `00bd48a0` | PortraitRequestTcgOrPortrait | likely | Three stack args (id lo, id hi, flag byte): when the id is not the sentinel pair DAT_01beaa20/DAT_01beaa24, calls 00bd4300 (TCG portrait) if the flag is nonzer… |
| `00d2e930` | CopyParamsQualityTo10000 | likely | Thiscall copy of a parameter block: copies the words of param_1 into param_2 (word 0, then 2 from 1, 3 from 2, 10 from 3, 11 from 12), sets word 13 to (param_1… |
| `00d2ed20` | BuildTimestampedJpgName | likely | Fills six date/time fields through 00702670 and 007026b0 (their meaning is inferred from use), calls 00894b10 with the value at +0x14 and (+0x18 - 4), then app… |
| `010f4fa0` | ScreenshotFolderPath | likely | Gets the current directory (GetCurrentDirectoryA, 0x104 bytes) and checks it with 010f3de0. If that returns nonzero it formats the path through 00402fd0 with t… |
| `010f5350` | AppendTimestampSuffix | likely | Thiscall(this=param_1, output string=param_2). Gets the current time through 00702670 and 007026b0 (field split), calls 006f9e60 on the IString at this+0x10 be… |

### `lib.dpvs.camera`

**DPVS::Camera** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc3730` | DPVS::Camera::Camera | confirmed | Thiscall with one dword argument p2 (the owner). Sets +8 = 1, +0xc = 0, +0x10 = p2, stores the DPVS::Camera vtable at +0, then under the enter call (slot 0x14 … |
| `00dc37b0` | DPVS::Camera::Create | confirmed | No arguments. Under the enter call (slot 0x14 of PTR_DAT_01b85ac0): allocates 0x300 bytes with FUN_00debb30 and, when that succeeds, calls FUN_00dedd40() (resu… |

### `lib.dpvs.cell`

**DPVS::Cell** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc3960` | DPVS::Cell::DestroyAndRelease | confirmed | Fastcall with ECX = p. Under the enter call (slot 0x14), calls vtable slot 0 of the object at p+0x10 with 0, calls vtable slot 0 of p with 0, calls FUN_00debb6… |

### `lib.dpvs.model`

**DPVS::Model** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc3a10` | DPVS::Model::DestroyAndRelease | confirmed | Fastcall with ECX = p. Under the enter call (slot 0x14), calls vtable slot 0 of the object at p+0x10 with 0, calls vtable slot 0 of p with 0, calls FUN_00debb6… |

**DPVS::SphereModel** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc3f90` | DPVS::SphereModel::CreateViaTemp | confirmed | Two stack dword arguments. Under the enter call (slot 0x14 of PTR_DAT_01b85ac0): builds a local block with FUN_00df3f40(&local, p1, p2), allocates 0x70 bytes w… |

### `lib.dpvs.object`

**DPVS::Object** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc4170` | DPVS::Object::DestroyAndRelease | confirmed | Fastcall with ECX = p. Under the enter call (slot 0x14): calls vtable slot 0 of the object at p+0x10 with 0, calls vtable slot 0 of p with 0, calls FUN_00debb6… |

**DPVS::PhysicalPortal** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc4320` | DPVS::PhysicalPortal::Create | confirmed | Two stack arguments. Under the enter call (slot 0x14 of PTR_DAT_01b85ac0): returns 0 for a null first argument; otherwise allocates 0xec bytes with FUN_00debb3… |

**DPVS::RegionOfInfluence** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc4220` | DPVS::RegionOfInfluence::Create | confirmed | One int argument. Under the enter call (slot 0x14 of PTR_DAT_01b85ac0): returns 0 for a null argument; otherwise allocates 0xd8 bytes with FUN_00debb30, calls … |

### `lib.dpvs.refcount`

**DPVS::ReferenceCount** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc34b0` | DPVS::ReferenceCount::DestroyAndRelease | confirmed | Fastcall with ECX = p. When p is non-null: enter call on PTR_DAT_01b85ac0 (slot 0x14), calls vtable slot 0 of p with argument 0, calls FUN_00debb60(p), then le… |
| `00dc3720` | DPVS::ReferenceCount::~ReferenceCount | confirmed | Fastcall with ECX = p. Stores the DPVS::ReferenceCount vtable at +0 and calls FUN_00dc35b0(0) (the locked wrapper). Destructor body, not a deleting destructor. |

### `lib.umbra`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc2d50` | SetServicesInterface | likely | Cdecl with one pointer argument. Stores it in PTR_DAT_01b854b0; when the argument is null stores &DAT_01bf3e20 instead, then returns. |

### `lib.umbra.camera`

**Umbra::Camera** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc800` | Umbra::Camera::~Camera | confirmed | Stores the Umbra::Camera vtable, then under the enter/leave pair calls vtable slot 0 with 0 on the object at +0x10 and FUN_00dc4950 on it when non-null, clears… |
| `00dbca60` | Umbra::Camera::~Camera | confirmed | Calls FUN_00dbc800(this) (the Camera destructor body), then calls _free(this) when bit 0 of the flag argument is set, and returns this. Scalar deleting destruc… |

### `lib.umbra.commander`

**Umbra::Commander** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc1580` | Umbra::Commander::Commander | confirmed | Fastcall with ECX = p1 (the object). Stores the Umbra::Commander vftable at +0 and zero at +4. With DAT_01bf3db4 non-zero it runs under the enter/leave pair: a… |
| `00dc2020` | Umbra::Commander::~Commander | confirmed | Fastcall with ECX = this. Stores the Umbra::Commander vftable at +0. Non-profiled path calls slot 0 with argument 0 on the object at this+4, then frees that ob… |
| `00dc27b0` | Umbra::Commander::~Commander | confirmed | Scalar deleting destructor shape (30 bytes): calls 00dc2020 (the Commander destructor body) on this, then calls _free(this) when bit 0 of the flag argument is … |

**Umbra::FallbackCommander** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbd9c0` | Umbra::FallbackCommander::FallbackCommander | confirmed | Thiscall constructor: calls base constructor 00ddda60 on this (that routine stores DPVS::Commander::vftable), stores its stack argument at +8 and then stores t… |
| `00dbd9f0` | Umbra::FallbackCommander::ForwardSlot1 | confirmed | Loads the object at this+8, reads its vtable and tail-jumps through vtable slot 1 with that object as ECX (no arguments added or changed). |
| `00dc0f00` | Umbra::FallbackCommander::~FallbackCommander | confirmed | Scalar deleting destructor shape: calls 00dde020 on this (base body that stores DPVS::Commander::vftable) and calls _free(this) when bit 0 of the flag argument… |

### `lib.umbra.info`

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbec40` | Umbra::GetInfoString | confirmed | One dword index. Non-profiled path returns FUN_00ddaee0(index). Otherwise, under the enter/leave pair, returns by index: 0 formats Umbra 1.0.3 with _sprintf in… |

### `lib.umbra.model`

**Umbra::OBBModel** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbbe70` | Umbra::OBBModel::Create | confirmed | Copies two 3-dword vectors (param_1, param_2) into locals, builds a local block with FUN_00dcde40/FUN_00dcd210, allocates 0x70 bytes with FUN_00dc4920, runs FU… |

**Umbra::SphereModel** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc0f0` | Umbra::SphereModel::CreateViaTemp | confirmed | Builds a local 4-dword (16-byte) block with FUN_00dce1d0(block, param_1, param_2), allocates 0x70 bytes with FUN_00dc4920, calls FUN_00dcc0e0 with the local bl… |

### `lib.umbra.object`

**Umbra::Object** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc220` | Umbra::Object::Create | confirmed | Returns 0 when param_1 is 0 (after the DAT_01bf3db4 gated enter call). Otherwise allocates 0x20 bytes with FUN_00dc4920, sets +4=-1, +8=1, +0xc=0 and the Umbra… |
| `00dbc320` | Umbra::Object::DestroyAndRelease | confirmed | Calls vtable slot 0 with argument 0 on the object pointed to by +0x10, then vtable slot 0 with argument 0 on this, then calls FUN_00dc4950(this), bracketed by … |

**Umbra::PhysicalPortal** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc500` | Umbra::PhysicalPortal::Create | confirmed | Returns FUN_00dc4320(param_1, param_2) when DAT_01bf3db4 is zero. Otherwise returns 0 for a null param_1, else allocates 0x34 bytes with FUN_00dc4920, sets +4=… |

**Umbra::RegionOfInfluence** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc3d0` | Umbra::RegionOfInfluence::Create | confirmed | Returns FUN_00dc4220(param_1) when DAT_01bf3db4 is zero. Otherwise returns 0 for a null param_1, else allocates 0x20 bytes with FUN_00dc4920, sets +4=-1, +8=1,… |

**Umbra::VirtualPortal** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc630` | Umbra::VirtualPortal::Create | confirmed | Returns FUN_00dc4420(param_1, param_2) when DAT_01bf3db4 is zero. Otherwise returns 0 for a null param_1, else allocates 0x3c bytes with FUN_00dc4920, sets +4=… |

### `lib.umbra.refcount`

**Umbra::ReferenceCount** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dbc7d0` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0) (base body), calls _free(this) when bit 0 of the flag argument is set, and returns this. S… |
| `00dbc8b0` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc8e0` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc910` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc940` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc970` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc9a0` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbc9d0` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbca00` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |
| `00dbca30` | Umbra::ReferenceCount::~ReferenceCount | confirmed | Stores the Umbra::ReferenceCount vtable at +0, calls FUN_00dbb590(0), calls _free(this) when bit 0 of the flag argument is set, returns this. Scalar deleting d… |

### `lib.umbra.services`

**Umbra::LibraryDefs::Services** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00dc2b30` | Umbra::LibraryDefs::Services::Services | confirmed | Fastcall constructor (ECX = this): stores the Umbra::LibraryDefs::Services vtable (0x185212c) at +0 and returns. |
| `00dc2b40` | Umbra::LibraryDefs::Services::~Services | confirmed | Fastcall (ECX = this): stores the Umbra::LibraryDefs::Services vtable at +0 and returns. Vtable store only, the same store as the constructor 00dc2b30 (no call… |
| `00dc2b50` | Umbra::LibraryDefs::Services::Print | confirmed | Stdcall with one argument (ret 4). Pushes the argument and the format at DAT_017ed2ec and calls 00d83239 (Ghidra FID label _wprintf, unverified) with the forma… |
| `00dc2b70` | Umbra::LibraryDefs::Services::Alloc | confirmed | Stdcall with one argument (ret 4). Pushes the argument and calls _malloc (00d8a681); EAX is not changed afterwards, so the malloc result is the return value. |
| `00dc2b80` | Umbra::LibraryDefs::Services::Free | confirmed | Stdcall with one argument (ret 4). Pushes the argument and calls _free (00d8a9d8). |
| `00dc2bd0` | Umbra::LibraryDefs::Services::ElapsedSeconds | confirmed | Returns an x87 float: on the first call stores the counter (DAT_01bf3e08 low, DAT_01bf3e0c high) and sets DAT_01bf3e10; then returns (now - base) divided by Qu… |
| `00dc2c70` | Umbra::LibraryDefs::Services::~Services | confirmed | Thiscall with one byte flag. Stores the Umbra::LibraryDefs::Services vtable at +0, calls _free(this) when bit 0 of the flag is set, returns this. Scalar deleti… |

### `render`

**Deep::SpherePrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00759180` | Deep::SpherePrimitive::Ctor | confirmed | Stores Deep::SpherePrimitive::vftable at +0, copies the four dwords at the pointer argument param_2 into this+0x10..+0x1c, then stores the next dword arguments… |

**GraphicsDllWrapper** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007954c0` | GraphicsDllWrapper::Ctor | confirmed | Fastcall (ECX = this). Stores the GraphicsDllWrapper vtable, stores 0 at this+4, constructs the Array<unsigned char,32,1> at this+0x208 through 00fc5460, zeroe… |
| `00795700` | GraphicsDllWrapper::DtorBody | confirmed | Fastcall (ECX = this). Stores the GraphicsDllWrapper vtable, frees the buffer at this+0x44c through 00d023c0 and _free when set, sets this+0x450 to -1, calls 0… |

**(no class)** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c165a0` | SetFloraEnabled | confirmed | Lua C function bound as SetFloraEnabled: reads argument 1 as an integer, calls FUN_008ddbc0(value != 0) to toggle the flora subsystem, and when the client opti… |
| `00791330` | GetQualityTierName | likely | Fastcall (ret 0). Reads the device object at this+0x25c+0x1d0; when it is nonnull, calls its vtable slot +0x124 and returns the string "High" for 0, "Medium" f… |
| `00791910` | ReleaseDeviceSubobjects | likely | Fastcall teardown of member objects: releases the object at this+0x2e4 (vtable slot 0 with 1), frees the object at +0x2e0 through 00813230 and _free, releases … |
| `00792310` | SetClampedRenderOptions | likely | Thiscall with two int arguments and one float argument (ret 0xc behaviour not verified). Stores argument 1 clamped to 1..8 at this+0x344, argument 2 clamped to… |
| `007929d0` | ShadowQualitySetLevel | likely | Thiscall (ECX = this, arg = level). If the level is 99 or 0..11 it stores it at this+0x340; otherwise it stores max(0, 2 - v) where v is a virtual query result… |
| `007941d0` | AutoDetectGraphicsQualityDefaults | likely | Thiscall (ECX = this, param_2 = graphics quality out, param_3 = second out, param_4 = unused by the logic shown). Queries the D3D object at this+0x25c through … |
| `007947d0` | GetFormattedShadowLevelString | likely | Fastcall (ECX = this). The first call lazily constructs the global string object PTR_DAT_01770518 with 00793d30 and registers its exit destructor FUN_01746b70 … |
| `00795830` | SetupRenderPipeline | likely | Thiscall (ECX = graphics object, with 11 stack args). Stores the size arguments at this+0x244/0x248 and flag and extra values at this+0x254 and this+0x258, cal… |
| `00b564d0` | DrawPetPathIndicators | likely | When its enable flag is set, resolves aoe_ring.dds, reads the R_WRIST transform, evaluates an interpolated path, generates 100 samples and submits the ring/arr… |
| `00c164f0` | GetRenderDistanceInPercent | likely | Lua C function bound as GetRenderDistanceInPercent: reads the current render distance through FUN_00791440, defaults the result to 100, and when the value is b… |
| `00c16560` | SetShadowQuality | likely | Lua C function bound as SetShadowQuality: when exactly one argument is present it reads it as a number (FUN_00e3a770 + FUN_00d83c90) and calls FUN_009178a0(lev… |

### `render.batch`

**Deep::CommandListBatcher** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007446b0` | CommandListBatcher::InitBatchArrays | likely | Initialises three embedded SoeUtil::Array members in place: Batch array at this+4, MaterialBatch array at this+0x14 and OrderedBatch array at this+0x24, each w… |
| `00744740` | CommandListBatcher::ReleaseBatchArrays | likely | Releases the three batch arrays embedded in the object: frees the OrderedBatch, MaterialBatch and Batch buffers through FreeMemory (00401c00, alignment 4) and … |
| `00745670` | CommandListBatcher::ReleaseMembers | likely | Releases the members of a command list batcher: frees the Material pointer array buffer and the unsigned char array buffer through FreeMemory (00401c00), clear… |
| `00745720` | CommandListBatcher::~CommandListBatcher | likely | Scalar deleting destructor: calls 00745670 (ReleaseMembers) on this, then calls _free on this when bit 0 of the flag argument is set; returns this with ret 4. |
| `00745740` | CommandListBatcher::Destroy | likely | Free-style helper taking a batcher pointer: when the pointer is nonzero it calls 00745670 (ReleaseMembers) and then _free on the pointer; returns nothing. |
| `00746280` | CommandListBatcher::Create | likely | Allocates 0x58 bytes through 00d833a6; when the allocation succeeds it runs the CommandListBatcher constructor 007455e0 on the new block with the dword at this… |

**Deep::CommandListBatcher::BatchSlot** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00744ae0` | BatchSlot::~BatchSlot | likely | Scalar-style destructor: calls 00744740 (ReleaseBatchArrays) on this, then calls _free on this only when bit 0 of the flag argument is set; returns this with r… |

**(no class)** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00744a80` | BatchSlotInitIfNonNull | likely | Stack-argument routine (ret 4): if the argument is nonzero it calls 007446b0 (BatchSlot array initialiser) with that argument in ECX; returns nothing. |
| `00744b00` | DequeInitSlotRange | likely | Thiscall over a range of slot indices in a 64-slot chunked deque: for each index from the start argument through the end argument it computes the slot address … |
| `00744c80` | DequeReleaseSlotRange | likely | Thiscall over a range of slot indices from the first argument to the second: for each index it computes the 64-byte slot from the chunk table at this+8 and the… |
| `00744cc0` | DequeAppendSlots | likely | Thiscall with a count argument: reserves chunks through 00744600 for count more slots, initialises the new slots through 00744b00, adds count to the slot count… |
| `00744d20` | DequeShrinkBack | likely | Thiscall with a count argument: releases the last count slots through 00744c80, lowers the slot count at this+0x28, and when the deque becomes empty resets the… |
| `00744e70` | DequeClear | likely | Thiscall: calls 00744d20 with the current slot count to remove every slot, then when the two signed fields at this+0x3c and this+0x40 are both negative it call… |
| `010a97d0` | MergeSectionVerticesIndices | likely | Thiscall (ECX = section list holder; stack args: destination vertex holder, destination index holder, key, flag): calls 010a8500 to total the section vertex an… |

### `render.command`

**Deep::Model** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a7bc0` | DeepModelCtor | confirmed | Constructor (ECX = model; stack args: source definition pointer and an optional material palette): stores this+0x10 = definition, the Deep::Model vftable, allo… |
| `010a7da0` | DeepModelDtorBody | confirmed | Destructor body (ECX = model): stores the Deep::Model vftable at +0, releases the refcounted object at +0x84 (LOCK decrement, vtable +4 at zero), destroys the … |
| `010a7ea0` | DeepModelScalarDeletingDtor | confirmed | Virtual slot 0 of the Deep::Model vftable (thiscall, one stack byte flag, ret 4): calls the destructor body 010a7da0 on this, calls _free on this when flag bit… |

**GraphicsDriver::CommandCall** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fabff0` | GraphicsDriver::CommandCall::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): walks the nested record buffer at the object referenced by this+4 (same loop as ExecuteCommandBuffer) and returns… |

**GraphicsDriver::CommandClear** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab010` | GraphicsDriver::CommandClear::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): calls the device vtable +0xac with (device, 0, 0, this+0x10, this+4, this+8, this+0xc) and returns this+0x14. |

**GraphicsDriver::CommandContext** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fad8c0` | GraphicsDriver::CommandContext::DestructorBody | confirmed | fastcall(this): stores the CommandContext vtable and destroys the embedded Array<Texture*,16,1> through 00fad6e0. |
| `00fada40` | GraphicsDriver::CommandContext::CommandContext | confirmed | thiscall(this, arg2): sets the CommandContext vtable, stores arg2 at +0xc, zeroes the fields at +4, +8, +0x10, +0x68..., and initialises the embedded pointer a… |

**GraphicsDriver::CommandDraw** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab1a0` | GraphicsDriver::CommandDraw::CommandDraw | confirmed | thiscall(this, type 0..4 mapped to 4,5,6,2,3, arg3 field, arg4 vertex count): sets the vtable, the mapped type and the arg3 field, and the primitive count from… |

**GraphicsDriver::CommandDrawIndexed** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab100` | GraphicsDriver::CommandDrawIndexed::CommandDrawIndexed | confirmed | thiscall(this, type 0..4 mapped to values 4,5,6,2,3, arg3..arg5 fields, arg6 vertex count, arg7): sets the vtable, stores the mapped type, arguments and the pr… |

**GraphicsDriver::CommandGammaRamp** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faadc0` | GraphicsDriver::CommandGammaRamp::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): calls device vtable +0x54 with (device, 0, 0, this+4) and returns this+0x604. |

**GraphicsDriver::CommandIndexBuffer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faacf0` | GraphicsDriver::CommandIndexBuffer::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): ECX = (context+0xc)+0x5e0, pushes the dword at (this+4)+0x1c, calls 00fb7840; returns this+8. |

**GraphicsDriver::CommandInputLayout** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab250` | GraphicsDriver::CommandInputLayout::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): ECX = (context+0xc)+0x5e0, pushes the dword at (this+4)+0x1c, calls 00fb7860; returns this+8. |

**GraphicsDriver::CommandProjectionMatrix** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fad020` | GraphicsDriver::CommandProjectionMatrix::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): calls device vtable +0xb0 with (device, 3, this+0x10) and returns this+0x50. |
| `00fad630` | EmitProjectionMatrix | confirmed | thiscall(this=command buffer, arg2 matrix): appends an 0x50-byte record (ProjectionMatrix vtable and the 16-dword matrix copied through 00facc50). |

**GraphicsDriver::CommandRenderTarget** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fadbd0` | GraphicsDriver::CommandRenderTarget::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): resolves the target surface (record +4 nonzero: calls its vtable +0x30 and the operand's +0x48 path; zero: the co… |

**GraphicsDriver::CommandStretchRect** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab2a0` | GraphicsDriver::CommandStretchRect::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): resolves two surface operands from this+4 and this+8 (0 selects the context default at context+0x1774, otherwise … |

**GraphicsDriver::CommandVertexBuffer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faac70` | GraphicsDriver::CommandVertexBuffer::Execute | confirmed | Virtual slot 0, method on the record: reads the dword at (this+4)+0x1c, calls the object's virtual slot 0x14 for a second value, calls 00fb78c0(dword at this+8… |

**GraphicsDriver::CommandViewport** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faad70` | GraphicsDriver::CommandViewport::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): loads the device pointer at context+0xc+0x5d8 and calls its vtable +0xbc with (device, this+4); returns this+0x1c. |

**(no class)** (24)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fab4b0` | EmitCommandEffect | confirmed | thiscall(this=command buffer, arg2): bumps the used length by 8 and writes the CommandEffect vtable and arg2 into the new record at base+used-8. |
| `00fab530` | EmitEffectParameterGroupSelect | confirmed | thiscall(this=command buffer, arg2): appends an 8-byte record with the GroupSelect vtable and arg2. |
| `00fab5b0` | EmitEffectParameterElementSelect | confirmed | thiscall(this=command buffer, arg2): appends an 8-byte record with the ElementSelect vtable and arg2. |
| `00fab630` | EmitVertexBuffer | confirmed | thiscall(this=command buffer, arg2, arg3): appends a 12-byte record (VertexBuffer vtable, arg2, arg3) and returns the record's field pointer. |
| `00fab6c0` | EmitIndexBuffer | confirmed | thiscall(this=command buffer, arg2): appends an 8-byte record (IndexBuffer vtable, arg2) and returns the record's field pointer. |
| `00fab750` | EmitViewport | confirmed | thiscall(this=command buffer, arg2 six-dword block): appends a 0x1c-byte record (Viewport vtable and six dwords copied from arg2). |
| `00fab7f0` | EmitGammaRamp | confirmed | thiscall(this=command buffer, arg2 block): appends a 0x604-byte record (GammaRamp vtable and 0x180 dwords copied from arg2). |
| `00fab920` | EmitDepthStencil | confirmed | thiscall(this=command buffer, arg2, arg3 byte): appends a 0xc-byte record (DepthStencil vtable, arg2 at +4, arg3 byte at +8). |
| `00fab9b0` | EmitClear | confirmed | thiscall(this=command buffer, arg2, arg3, arg4, three byte flags): appends a 0x14-byte record (Clear vtable, arg2 at +4, arg3 at +8, arg4 at +0xc) and the flag… |
| `00fabba0` | EmitInputLayout | confirmed | thiscall(this=command buffer, arg2): appends an 8-byte record (InputLayout vtable, arg2). |
| `00fabc20` | EmitStretchRect | confirmed | thiscall(this=command buffer, arg2, arg3): appends a 0xc-byte record (StretchRect vtable, arg2, arg3). |
| `00fac260` | EmitCall | confirmed | thiscall(this=command buffer, arg2): appends an 8-byte record (CommandCall vtable and arg2 as the nested buffer pointer). |
| `0073b460` | EmitStageTextureCommands | likely | For each of [this+0x1c8] texture records (base index [this+0x1cc], records reached through [this+0x1a8]): hashes the record's name at +8/+0x3c with the inline … |
| `00faadf0` | ClampToEnum0To5 | likely | Switch on arg1: returns 0..5 unchanged for values 0 to 5, and returns 0x7fffffff for any other value. |
| `00fab0a0` | PrimitiveCountForTopology | likely | Switch on arg1 (topology index 0 to 4): returns arg2/3 for 0, arg2-2 for 1 and 2, arg2/2 for 3, arg2-1 for 4; returns -1 for any other index. |
| `00fab3c0` | ExecuteCommandBuffer | likely | thiscall(this, arg2 context): walks the record buffer from the base at this+0xc for this+0x10 bytes; each record's first dword is a vtable whose slot 0 is call… |
| `00fab420` | CopyBufferBytesOut | likely | thiscall(this, arg2 destination): memcpy of the used bytes (base at this+0xc, length at this+0x10) into arg2. |
| `00fab440` | AppendBytesToBuffer | likely | thiscall(this, arg2 source, arg3 count): adds arg3 to the used length at this+0x10 and memcpy's arg3 bytes from arg2 to the old end of the used region. |
| `00fab470` | AppendBufferContents | likely | thiscall(this, arg2 other buffer): adds the other buffer's used length to this used length and memcpy's the other buffer's bytes to this buffer's old end. |
| `00faba70` | EmitDrawIndexed | likely | thiscall(this=command buffer, arg2..arg7): bumps the used length by 0x1c and constructs a CommandDrawIndexed record at the new slot through 00fab100 with arg2.… |
| `00fabb10` | EmitDraw | likely | thiscall(this=command buffer, arg2..arg4): bumps the used length by 0x10 and constructs a CommandDraw record at the new slot through 00fab1a0. |
| `010a6320` | MultiModelSubmitLoop | likely | Thiscall (ECX = this; ret 0x14, five stack args): clears the counter at this+0x88, then for each entry i of the count at (this+0x10)+0x74 +0x10: calls 00758cb0… |
| `010a63c0` | MultiModelDrawLoop | likely | Thiscall (ECX = this; six stack arguments): clears the counter at this+0x8c, then for each entry i of the same count as 010a6320: calls 00757e30 with element i… |
| `010a7b40` | ModelRecordDtorBody | likely | Element destructor for the 0xb0-byte per-model records (ECX = record, fastcall): releases the refcounted object at +0x98 (LOCK decrement of its +4 count, its v… |

### `render.effect`

**ActorCompositeEffectReference** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c4520` | ActorCompositeEffectReference::ReleaseInstanceAndResetTarget | likely | Virtual slot [4] (24 bytes): calls this->vtable[+8] with (0,1) (instance release, see 007c4620) and then tail-calls this->vtable[+0x24] (target clear, see 007c… |
| `007c45b0` | ActorCompositeEffectReference::RecreateInstance | likely | Virtual slot [1] (112 bytes), thiscall with one stack argument. When this+0x34 is non-null it calls 007dc450(1) first. If this+0x30 is null it resolves it thro… |
| `007c4620` | ActorCompositeEffectReference::ReleaseInstance | likely | Thiscall; stack: unused dword and a char flag. When this+0x34 is non-null: when the flag is nonzero calls 007dc400(), then calls 007dc450 with the first stack … |
| `007c4660` | ActorCompositeEffectReference::ReleaseInstanceAndClearTarget | likely | Virtual slot [9] (49 bytes): calls this->vtable[+8] with (1,1) (instance release, flag 1), then when the pointer at this+0x30 is non-null decrements the count … |
| `007c4760` | ActorCompositeEffectReference::ResolveTarget | likely | Fastcall, ECX = this (77 bytes). Returns this+0x30 when set. Otherwise calls 00769ab0 as a test; when it is nonzero calls 00769ab0 with the key at this+0x28 an… |

**AppParticleEntity** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0079f400` | AppParticleEntityDtorBody | confirmed | Destructor body (ECX=this, no stack arguments): stores the AppParticleEntity vtable at this+0x0 and this+0x8; calls _free on the pointer at this+0xfe4; if the … |
| `0079f4b0` | AppParticleEntitySecondaryDtorThunk | confirmed | Adjustor thunk (8 bytes): sub ecx,8 then jmp 0079f830 (the scalar deleting destructor). It does nothing else. |
| `0079f830` | AppParticleEntity::~AppParticleEntity | confirmed | Scalar deleting destructor: calls 0079f400 (dtor body) on this, then if bit 0 of the byte stack argument is set calls _free(this); returns this (ret 4). |
| `0079fb20` | AppParticleEntity::Slot12 | confirmed | Virtual slot [12] (secondary vtable at object+0x8 of AppParticleEntity). Reads the dword at this-0x4, calls 0086a960, then if that dword is not 1 and bit 2 of … |
| `0079fec0` | AppParticleEntity::Slot7 | confirmed | Virtual slot [7] (secondary vtable at object+0x8; also the RefObjectPool Node vtable). If the flag byte at this+0xfe4 has 0x20 set and this+0xfcc and this+0xfd… |

**GraphicsDriver::CommandEffect** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faabb0` | GraphicsDriver::CommandEffect::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 context): loads ECX from arg2 (the context object), pushes the dword at this+4 and calls 00faab10 (SetEffectObject); return… |

**GraphicsDriver::CommandEffectFloatParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac4a0` | ApplyFloatParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): same group and element selection as ApplyIntParameter, then calls the selected object's vtable +4 with the dword a… |
| `00fad0f0` | EmitFloatParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the float parameter payload through 00fac430, sets the Command… |

**GraphicsDriver::CommandEffectFloatRefParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac870` | ApplyFloatRefParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0xc), then calls the selected object's vtable +4 with the float pointed … |
| `00fad3e0` | EmitFloatRefParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the float-ref payload through 00fac800, sets the CommandEffect… |

**GraphicsDriver::CommandEffectIntParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac3e0` | ApplyIntParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): selects the group (this+4 below 2 uses this+0xc, otherwise the group from target+0x6c), then the element through v… |
| `00fad050` | EmitIntParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the int parameter payload through 00fac370, sets the CommandEf… |

**GraphicsDriver::CommandEffectIntRefParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac7b0` | ApplyIntRefParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0xc), then calls the selected object's vtable +0 with the dword pointed … |
| `00fad350` | EmitIntRefParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the int-ref payload through 00fac740, sets the CommandEffectIn… |

**GraphicsDriver::CommandEffectMatrix44Parameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac630` | ApplyMatrix44Parameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0x50), then calls the selected object's vtable +0xc with (this+0x10); re… |
| `00fad210` | EmitMatrix44Parameter | confirmed | thiscall(this=command buffer, arg2, arg3 matrix, arg4 count): reserves 4*count+0x50 bytes, copies the matrix through 00facc50, constructs the payload through 0… |

**GraphicsDriver::CommandEffectMatrix44RefParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac9f0` | ApplyMatrix44RefParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0xc), then calls the selected object's vtable +0xc with the pointer at t… |
| `00fad510` | EmitMatrix44RefParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the matrix-ref payload through 00fac980, sets the CommandEffec… |

**GraphicsDriver::CommandEffectParameter<GraphicsDriver::TextureInterface**,GraphicsDriver::CommandEffectTextureRefParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faca40` | InitTextureRefParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameter<GraphicsDriver::TextureInterface*,GraphicsDriver::CommandEffectTextureParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac680` | InitTextureParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to texture id, arg4 count): sets the template vtable, count at +4, the id from *arg3 at +8, and copies count dwords fr… |

**GraphicsDriver::CommandEffectParameter<SoeUtil::Math::Matrix44_t<float>,GraphicsDriver::CommandEffectMatrix44Parameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac5c0` | InitMatrix44ParameterRecord | confirmed | thiscall(this, arg2 values, arg3 matrix, arg4 count): sets the template vtable, count at +4, copies the 16-dword matrix from arg3 to +0x10..+0x4c, then copies … |

**GraphicsDriver::CommandEffectParameter<SoeUtil::Math::Vector<float,4>,GraphicsDriver::CommandEffectVectorParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac4f0` | InitVectorParameterRecord | confirmed | thiscall(this, arg2 values, arg3 four-float vector, arg4 count): sets the template vtable, count at +4, copies the four vector dwords to +0x10..+0x1c and count… |

**GraphicsDriver::CommandEffectParameter<float,GraphicsDriver::CommandEffectFloatParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac430` | InitFloatParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameter<float_const*,GraphicsDriver::CommandEffectFloatRefParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac800` | InitFloatRefParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameter<float_const*,GraphicsDriver::CommandEffectMatrix44RefParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac980` | InitMatrix44RefParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameter<float_const*,GraphicsDriver::CommandEffectVectorRefParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac8c0` | InitVectorRefParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameter<int,GraphicsDriver::CommandEffectIntParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac370` | InitIntParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, the id dword from *arg3 at +8, and copies count dwords from… |

**GraphicsDriver::CommandEffectParameter<int_const*,GraphicsDriver::CommandEffectIntRefParameter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac740` | InitIntRefParameterRecord | confirmed | thiscall(this, arg2 values, arg3 pointer to id, arg4 count): sets the template vtable, count at +4, id from *arg3 at +8, and copies count dwords from arg2 to +… |

**GraphicsDriver::CommandEffectParameterElementSelect** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faac20` | GraphicsDriver::CommandEffectParameterElementSelect::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 target): stores the dword at this+4 into target+0x70 and returns this+8. |

**GraphicsDriver::CommandEffectParameterGroupSelect** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faabf0` | GraphicsDriver::CommandEffectParameterGroupSelect::Execute | confirmed | Virtual slot 0, thiscall(this, arg2 target): stores the dword at this+4 into target+0x6c and returns this+8. |

**GraphicsDriver::CommandEffectTextureParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac6f0` | ApplyTextureParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0xc), then calls the selected object's vtable +0x10 with the dword at th… |
| `00fad2b0` | EmitTextureParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the texture parameter payload through 00fac680, sets the Comma… |

**GraphicsDriver::CommandEffectTextureRefParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00facab0` | ApplyTextureRefParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0xc), then calls the selected object's vtable +0x10 with the dword point… |
| `00fad5a0` | EmitTextureRefParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4 count): reserves 4*count+0xc bytes, constructs the texture-ref payload through 00faca40, sets the CommandEffe… |

**GraphicsDriver::CommandEffectVectorParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac570` | ApplyVectorParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection as ApplyIntParameter (base at this+0x30), then calls the selected object's vtable +8 w… |

**GraphicsDriver::CommandEffectVectorRefParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fac930` | ApplyVectorRefParameter | confirmed | Virtual slot 0, thiscall(this, arg2 target): group and element selection (base at this+0x10), then calls the selected object's vtable +8 with (the dword at thi… |
| `00fad470` | EmitVectorRefParameter | confirmed | thiscall(this=command buffer, arg2, arg3 id, arg4, arg5 count): reserves 4*count+0x10 bytes, constructs the vector-ref payload through 00fac8c0, sets the Comma… |

**LineEntity** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c12a0` | LineEntity::Ctor | confirmed | Fastcall constructor body (ECX = this). Runs the Deep::Entity constructor 007648d0, then zeroes +0x14, +0x18 and +0x1c, sets +0xa0 and +0xa4 to -1, +0xa8 to th… |
| `010c1340` | LineEntity::DtorBody | confirmed | Fastcall destructor body (ECX = this). Stores LineEntity::vftable into [this] and calls the Deep::Entity destructor body 00764e10 on this. Does not free the ob… |
| `010c1390` | LineEntity::ScalarDeletingDtor | confirmed | Thiscall with one flag byte (ret 4). Stores LineEntity::vftable into [this], calls the Deep::Entity destructor body 00764e10, frees this through the _free thun… |

**PhotoPostEntity** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00af11a0` | PhotoPostEntity::Ctor | confirmed | Constructor (ECX = this, two stack args forwarded). Calls 010c74b0 with the two forwarded arguments and stores PhotoPostEntity::vftable at [this], returning th… |
| `00af1270` | PhotoPostEntity::ScalarDeletingDtor | confirmed | Scalar deleting destructor (ECX = this, flag argument): stores PhotoPostEntity::vftable at [this], calls 010c7390 (base destructor body), then calls _free(this… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c6cc0` | CreateActorCompositeEffectReference | likely | Same shape as 007c6c00 (90 bytes): calls the constructor 007bb4f0 with the dwords at *param_2 and *param_3 and returns the first argument. |
| `00a2f7f0` | DrawAoeRingIndicator | likely | When the active flag at +0x61 is set, takes the centre at +0x50..+0x5c and radius at +0x20 (or DAT_017649d4 for the ring-relative case), builds a min/max bound… |
| `00cc75e0` | ClearTrackedEffects | likely | Walks nodes at this+0x144, unregisters effect handles through FUN_007e7df0 when context exists, frees the tracked-node container, and clears this+0x1d0. |
| `00cc7670` | UpdateNearbyGesturePointEffect | likely | Looks up the point keyed by this+0x1d0, compares squared distance with this+0x24, removes its old effect if present, creates a replacement, and increments the … |
| `00facc50` | CopyMatrix44_t | likely | thiscall(this=dest, arg2=src): copies 16 dwords from src to dest (4x4 matrix of floats). |
| `00fad190` | EmitVectorParameter | likely | thiscall(this=command buffer, arg2, arg3, arg4, arg5 count): reserves 4*count+0x30 bytes and constructs the vector parameter through 00face00 when the reservat… |

### `render.emitter`

**ActorCameraEffectEmitter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007764d0` | ActorCameraEffectEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorCameraEffectEmitter::vftable at +0, zeroes th… |
| `00776590` | ActorCameraEffectEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c10b0 on the object, frees it through _free when flag bit 0 is set,… |
| `007c1040` | ActorCameraEffectEmitter::CompareFloat0xa8 | confirmed | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xa8 with DAT_01762a30 and returns a value whose l… |
| `007c1070` | ActorCameraEffectEmitter::ReleaseA8 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d1b00 first if the byte argument is non-zero, then 007d21d0 with the d… |
| `007c13c0` | ActorCameraEffectEmitter::BuildObject | confirmed | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero, stores the factory result uVar3 at this+0xa8, sets a temporary string from the C string a… |

**ActorCompositeEffectEmitter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776900` | ActorCompositeEffectEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c2b60 on the object, frees it through _free when flag bit 0 is set,… |
| `007c2b20` | ActorCompositeEffectEmitter::ReleaseA4 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa4 is non-null, calls 007dc400 first if the byte argument is non-zero, then 007dc450 with the d… |
| `007c2b60` | ActorCompositeEffectEmitter::~ActorCompositeEffectEmitter | confirmed | Fastcall (ECX = object): sets the vtable to ActorCompositeEffectEmitter::vftable; computes a flag from the sub-object at +0xa0 (its vtable slot +0x10 result, o… |
| `00776820` | ActorCompositeEffectEmitterCtor | likely | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorCompositeEffectEmitter::vftable at +0, zeroes… |

**ActorCompositeEffectReference** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007772b0` | ActorCompositeEffectReferenceDtor | confirmed | Fastcall destructor body (ECX = object). Stores ActorCompositeEffectReference::vftable at +0, calls 007c4520 on the object (unlabelled cleanup), then calls the… |
| `00777320` | ActorCompositeEffectReference::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007772b0 on the object, frees it through _free when flag bit 0 is set,… |

**ActorEffectEmitter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00775f70` | ActorEffectEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args stored at +0x10, +0x14 and +0x90). Stores ActorEffectEmitter::vftable at +0, the first argument at +0x10 a… |
| `00776050` | ActorEffectEmitterGetByte98 | confirmed | Thiscall virtual slot [8] (ECX = object, no stack args). Bytes 8a 81 98 00 00 00 c3: loads the byte at +0x98 into AL and returns it. |
| `00776060` | ActorEffectEmitterDtor | confirmed | Fastcall destructor body (ECX = object). Stores ActorEffectEmitter::vftable at +0, then releases three embedded SoeUtil::IString<char> members in reverse order… |
| `00776140` | ActorEffectEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag, ret 4). Calls the base destructor 00776060 on the object, frees it through _free when flag bit 0 … |
| `0076a9d0` | ActorEmitterSetByte98 | likely | thiscall: stores the byte argument (stack [esp+4]) into this+0x98 and returns (ret 4). 13 bytes. |

**ActorEffectReference** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00777080` | ActorEffectReferenceDtor | confirmed | Fastcall destructor body (ECX = object). Stores ActorEffectReference::vftable at +0; releases the IString<char> member at +0x14 (buffer at +0x18, heap flag at … |
| `00777150` | ActorEffectReference::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 00777080 on the object, frees it through _free when flag bit 0 is set,… |

**ActorLightEmitter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007763f0` | ActorLightEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorLightEmitter::vftable at +0, zeroes the dword… |
| `007764b0` | ActorLightEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c09d0 on the object, frees it through _free when flag bit 0 is set,… |
| `007c0910` | ActorLightEmitter::CompareFloat0xbc | confirmed | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xbc with DAT_01762a30; returns a value whose low … |
| `007c0940` | ActorLightEmitter::ReleaseA8 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d01f0 first if the byte argument is non-zero, then calls 007d06b0 with… |
| `007c0ce0` | ActorLightEmitter::BuildLightObject | likely | Virtual slot [1] (fastcall, ECX = object, no stack args). When 007cfe50 is non-zero it computes four floats from the sub-object at +0xa0 (+0xa0..0xac scaled by… |

**ActorLightsaberTrailEmitter** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776740` | ActorLightsaberTrailEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorLightsaberTrailEmitter::vftable at +0, zeroes… |
| `00776800` | ActorLightsaberTrailEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c24f0 on the object, frees it through _free when flag bit 0 is set,… |
| `007c2470` | ActorLightsaberTrailEmitter::ReleaseA8 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d3da0 first if the byte argument is non-zero, then 007d4580 with the d… |
| `007c24f0` | ActorLightsaberTrailEmitter::~ActorLightsaberTrailEmitter | confirmed | Fastcall (ECX = object): sets the vtable to ActorLightsaberTrailEmitter::vftable; when object+0xa8 is non-null calls 007d3da0 and then 007d4580(0) and clears +… |

**ActorMeshEmitter** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776690` | ActorMeshEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorMeshEmitter::vftable at +0, zeroes the dwords… |
| `00776720` | ActorMeshEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c1de0 on the object, frees it through _free when flag bit 0 is set,… |
| `007c1d40` | ActorMeshEmitter::CompareFloat0xac | confirmed | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xac with DAT_01762a30 and returns a value whose l… |
| `007c1d70` | ActorMeshEmitter::ReleaseA8 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007ab880 first if the byte argument is non-zero, then 007ac200 with the d… |
| `007c1dc0` | ActorMeshEmitter::CallSlot3IsZero | confirmed | Virtual slot [4] (fastcall, ECX = object, no stack args): calls the object's vtable slot [3] (+0xc) and returns 1 when that call's byte result is zero, else 0. |
| `007c2160` | ActorMeshEmitter::BuildObjectIfAbsent | confirmed | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero and this+0xa8 is null, stores uVar5 at this+0xa8, sets a temporary string from (+0xa0)+0x2… |

**ActorMeshEmitterDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b5460` | ActorMeshEmitterDefinition::~ActorMeshEmitterDefinitionBody | confirmed | Destructor body of ActorMeshEmitterDefinition: stores ActorMeshEmitterDefinition::vftable at this+0 and at the secondary subobject this+0x90, releases the IStr… |

**ActorModelMaterialEmitter** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776310` | ActorModelMaterialEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorModelMaterialEmitter::vftable at +0, zeroes t… |
| `00776390` | ActorModelMaterialEmitter::GetByteA4 | confirmed | Thiscall virtual slot [6] (ECX = object, no stack args). Bytes 8a 81 a4 00 00 00 c3: loads the byte at +0xa4 into AL and returns it. |
| `007763d0` | ActorModelMaterialEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor 007c06f0 on the object, frees it through _free when flag bit 0 is set, and … |
| `007c0690` | ActorModelMaterialEmitter::CallWith0xa0_0x40 | confirmed | Virtual slot [1] (fastcall, ECX = object, no stack args): calls 007cfe50; when it returns non-zero, reads the pointer at object+0xa0 and calls 0076f2d0 with th… |
| `007c06c0` | ActorModelMaterialEmitter::CallWith0xa0_0x58 | confirmed | Virtual slot [2] (fastcall, ECX = object): calls 007cfe50 and, when it returns non-zero, calls 0076f390 with the dword at the sub-object (object+0xa0) +0x58. |
| `007c06f0` | ActorModelMaterialEmitter::~ActorModelMaterialEmitter | confirmed | Fastcall (ECX = object, no stack args): sets the vtable to ActorModelMaterialEmitter::vftable, calls 007cfe50 and, when non-zero, 0076f390 with the dword at (o… |

**ActorParticleEmitter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007c2dc0` | ActorParticleEmitter::ReleaseA4 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa4 is non-null, calls 0086a8f0 first if the byte argument is non-zero, then calls the vtable sl… |
| `007c2fe0` | ActorParticleEmitter::BuildObject | likely | Virtual slot [1] (fastcall, ECX = object, 602 bytes): calls 007cfe50 and, on the build path, uses 00705bc0, 00729610, 007660b0, 00769710, 00769cb0, 0076f5b0, 0… |

**ActorParticleEmitterDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e90b0` | ActorParticleEmitterDefinition::DtorBody | confirmed | Fastcall (ECX = object). Stores ActorParticleEmitterDefinition::vftable at +0 and at +0x90. Releases the reference-counted object at +0xbc (owner vtable +0x3c … |
| `007e9fe0` | ActorParticleEmitterDefinition::ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the ActorParticleEmitterDefinition destructor body 007e90b0, then calls _free on this when bit 0 of the flag is set. Retur… |
| `007ea3a0` | ActorParticleEmitterDefinition::ReadAttributes | confirmed | Thiscall virtual at vtable slot [7] of ActorParticleEmitterDefinition (017f1218), with one stack arg (input node) and one more arg. It reads the named values t… |

**ActorProjectileEmitter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c0f60` | ActorProjectileEmitter::DtorBody | confirmed | Fastcall destructor body (ECX = this). Stores ActorProjectileEmitter::vftable into [this], decrements the reference count of the object at +0xa0 when positive,… |
| `010c0e80` | ActorProjectileEmitter::UpdateNodeTransform | likely | Fastcall virtual (slot [1]) with this in ECX. Gets the context from 007cfe50 and copies the 16-float matrix at DAT_01bc2660. Resolves the named node at this+0x… |

**ActorRumbleEffectEmitter** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007765b0` | ActorRumbleEffectEmitterCtor | confirmed | Thiscall constructor (ECX = object, three stack args forwarded). Calls the base constructor 00775f70, stores ActorRumbleEffectEmitter::vftable at +0, zeroes th… |
| `00776670` | ActorRumbleEffectEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c1740 on the object, frees it through _free when flag bit 0 is set,… |
| `007c16d0` | ActorRumbleEffectEmitter::CompareFloat0xac | confirmed | Virtual slot [6] (fastcall, ECX = object): when object+0xa0 is non-null, compares the float at (object+0xa0)+0xac with DAT_01762a30 and returns a value whose l… |
| `007c1700` | ActorRumbleEffectEmitter::ReleaseA8 | confirmed | Virtual slot [2] (thiscall, two stack args, ret 8): when this+0xa8 is non-null, calls 007d30a0 first if the byte argument is non-zero, then 007d30b0 with the d… |
| `007c1a50` | ActorRumbleEffectEmitter::BuildObject | confirmed | Virtual slot [1] (fastcall, ECX = object): when 007cfe50 is non-zero, stores the factory result uVar3 at this+0xa8, sets a temporary string from (+0xa0)+0x2c (… |

**SoeUtil::HashListMap<int,ActorMeshEmitterDefinition*,40,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010c17f0` | HashListMap_ActorMeshEmitterDefinition_Ctor | confirmed | Fastcall constructor body (ECX = this). Stores the map vtable into [this], zeroes +0x4, +0x8, +0xc and +0x10, clears the 0xa0-byte bucket array at +0x14 (40 dw… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ab880` | ClearHandleSetState2 | likely | If the ushort state at this+0x182 is below 2, clears the dword at this+0x184 and sets the state to 2; otherwise it does nothing. |
| `010c1060` | ProjectileEmitterInit | likely | Thiscall with the emitter in ECX and a parameter object (stack argument). Stores the object at this+0xa0 and increments its reference count at +8, calling its … |

### `render.fog`

**Deep::FogEntity** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b2af0` | FogEntity::~FogEntityBody | confirmed | Destructor body of Deep::FogEntity: stores Deep::FogEntity::vftable into this, then calls 010b26e0 (the LightEntity dtor body) on this. |
| `010b2d70` | FogEntity::~FogEntity | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::FogEntity: stores Deep::FogEntity::vftable, calls 010b26e0 (LightEntity dtor body), then if bit 0 of the f… |
| `010b3270` | FogEntity::FogEntity | confirmed | Constructor of Deep::FogEntity (same shape as the sky-light constructor 010b3080): calls 010b2df0 with key 0xf36060ed and constructs the LightEntity base on th… |

### `render.layout`

**Deep::InputLayoutEntry** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00748390` | Deep::InputLayoutEntry::InputLayoutEntry | confirmed | Constructor: stores the InputLayoutEntry vftable, zeroes the fields at this+4..+0xc, then copies a NUL-terminated semantic name argument into a block allocated… |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007487a0` | VertexElementSemanticParse | likely | Parses a vertex element semantic name against the literals Position, PositionT, Color, Texcoord, Normal, Tangent, Binormal and a blend-weight literal, returnin… |

### `render.light`

**Deep::DirectionalLightEntity** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b3b00` | DirectionalLightEntity::DirectionalLightEntity | confirmed | Constructor of Deep::DirectionalLightEntity (fastcall, this in ECX): calls 010b2df0 with key 0x874be757 and constructs the LightEntity base on this from the re… |
| `010b3d20` | DirectionalLightEntity::~DirectionalLightEntity | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::DirectionalLightEntity: calls 010b26e0 (the LightEntity dtor body) on this, then if bit 0 of the flag argu… |

**Deep::LightEntity** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b26e0` | LightEntity::~LightEntityBody | confirmed | Destructor body of Deep::LightEntity: stores Deep::LightEntity::vftable into this, releases the refcounted object at this+0x1c and then the one at this+0x18 (L… |
| `010b2790` | LightEntity::LightEntity | confirmed | Constructor of Deep::LightEntity: calls 007648d0 at entry, stores Deep::LightEntity::vftable into this, stores its argument at this+0x14, sets this+0x18 to 007… |
| `010b2850` | LightEntity::~LightEntity | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::LightEntity: calls the dtor body 010b26e0 on this (ECX), then if bit 0 of the flag argument is set frees t… |

**Deep::PointLightEntity** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b3aa0` | PointLightEntity::~PointLightEntity | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::PointLightEntity: calls 010b26e0 (the LightEntity dtor body) on this, then if bit 0 of the flag argument i… |

**LightSystem** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b13f0` | LightSystem::Load | likely | Initialises the object at ECX from its definition argument (stored at this+0x198) through 010b0b00 and 007575b0, then reads the Definition children: the Lights… |

**SoeUtil::Deque<Deep::LightDefinition,16>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b0f20` | SoeUtil_DequeLightDef16_EmplaceBackInit | likely | Reserves one more element slot at the back of the LightDefinition deque in ECX (through 010b0a00), increments the size at +0x28, and when the new slot is non-n… |

### `render.material`

**Deep::FloatParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00745d60` | Deep::FloatParameter::FloatParameter | confirmed | Constructor: stores the ParameterValue, UndefinedParameter and FloatParameter vftables into this, returns this. |

**Deep::IntParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00745cf0` | Deep::IntParameter::IntParameter | confirmed | Constructor: stores the ParameterValue, UndefinedParameter and then IntParameter vftables into this (the last store wins), and returns (no value argument). |

**Deep::Material** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00758870` | Deep::Material::DtorBody | confirmed | Stores Deep::Material::vftable at +0. Walks the Array<ParameterEntry> at +0x18 (storage +0x1c, count +0x20, 12-byte entries): calls 00753fa0 with ECX=[this+0xc… |
| `00758c90` | Deep::Material::ScalarDeletingDtor | confirmed | Thiscall with a flag byte. Calls the Material body destructor 00758870, then calls _free(this) only when bit 0 of the flag is set. Returns this. |

**Deep::MaterialDefinition** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007641f0` | Deep::MaterialDefinition::~MaterialDefinition | confirmed | Destructor body: stores Deep::MaterialDefinition::vftable, frees the buffers at +0x18 and +0x20 (sets +0x1c to -1), calls virtual slot [0] with flag 1 on each … |
| `00764490` | Deep::MaterialDefinition::ctor | confirmed | Constructor for Deep::MaterialDefinition (__thiscall, 6 args): stores Deep::MaterialDefinition::vftable, zeroes +4/+8/+0xc/+0x10, stores param_2 at +0x14, init… |
| `007647e0` | Deep::MaterialDefinition::ScalarDeletingDtor | confirmed | Scalar deleting destructor (30 bytes): calls 007641f0 (the MaterialDefinition destructor body) on this, and if the low bit of the flag argument is set frees th… |

**Deep::MaterialPalette** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010a4bb0` | MaterialPaletteDtorBody | confirmed | Destructor body (ECX = this, fastcall): stores the Deep::MaterialPalette vftable at +0, frees the heap block at +0x7c through 00d83ac6 (thunk to _free), runs t… |
| `010a4d50` | MaterialPaletteCtorFromDmat | confirmed | Constructor from a DMAT block (ECX = this; stack args: owner pointer param_2 and source pointer param_3). Calls 0071b950 for the RefCounted base, stores the De… |
| `010a4fb0` | MaterialPaletteScalarDeletingDtor | confirmed | Virtual slot 0 of the Deep::MaterialPalette vftable (thiscall, one stack byte flag, ret 4): calls the destructor body 010a4bb0 on this, calls _free on this whe… |
| `010a4fd0` | MaterialPaletteCtorSized | confirmed | Constructor with a count (ECX = this, one stack arg): calls 0071b950 for the RefCounted base, stores the Deep::MaterialPalette vftable, constructs the Array<En… |
| `010a5220` | MaterialPaletteCopyCtor | confirmed | Copy constructor (ECX = this, one stack arg = source Deep::MaterialPalette): calls 0071b950 for the RefCounted base, stores the Deep::MaterialPalette vftable, … |

**Deep::Matrix44ParameterRef** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00758ec0` | Deep::Matrix44ParameterRef::Ctor | confirmed | Thiscall with one dword argument. Stores the vtables in order Deep::ParameterValue::vftable, Deep::UndefinedParameter::vftable, then Deep::Matrix44ParameterRef… |

**Deep::ParameterValue** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00758f50` | Deep::ParameterValue::DtorBody | confirmed | Stores Deep::ParameterValue::vftable at +0 and returns. Frees nothing. |

**Deep::TextureParameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00745fa0` | Deep::TextureParameter::TextureParameter | confirmed | Constructor: stores the ParameterValue, UndefinedParameter and TextureParameter vftables into this, zeroes the dword at this+4 (texture reference) and returns. |
| `007481d0` | Deep::TextureParameter::~TextureParameter | confirmed | Scalar deleting destructor (vtable slot [0] of TextureParameter): calls the body 00748150 on this, then calls _free on this when bit 0 of the flag argument is … |

**Deep::Vector4Parameter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00745dd0` | Deep::Vector4Parameter::Vector4Parameter | confirmed | Constructor without a value argument: stores the ParameterValue, UndefinedParameter and Vector4Parameter vftables into this and returns. |
| `00745e40` | Deep::Vector4Parameter::ConstructFilled | likely | Constructor with one value argument: stores the ParameterValue, UndefinedParameter and Vector4Parameter vftables, then stores the argument into all four dwords… |

**Deep::Vector4ParameterRef** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007480f0` | Deep::Vector4ParameterRef::~Vector4ParameterRef | confirmed | Scalar deleting destructor (vtable slot [0] of Vector4ParameterRef): stores the ParameterValue vftable into this and calls _free on this when bit 0 of the flag… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0074f790` | ParameterGroupLoadXml | confirmed | Fastcall with one argument (the owner). Walks the XML children named ParameterGroup (0073ef30 / 0073b7b0 selectors). For each group it reads the Name attribute… |
| `00763140` | TextureSlotBindFromAttributes | likely | For each id in param_2's array (count at +0x1c, ids at +0x18) it gets an object through param_2+0xc vtable +0x1c, queries its 'TextureFile' property (vtable +0… |

### `render.occlusion`

**Deep::MeshVisibilityDefinition** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00892d90` | MeshVisibilityDefinitionScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityDefinition dtor body 00892280 on this, then frees the block with _free when the low bit of the flag argument is… |
| `00892df0` | MeshVisibilityDefinitionCtorU16 | confirmed | Constructor for a mesh visibility definition from a vertex base and 16-bit index records: calls 00892500, stores the MeshVisibilityDefinition vftable, stores t… |
| `00892f20` | MeshVisibilityDefinitionFromArrays | confirmed | Constructor that first builds a temporary Vector<float,3> array and an int array with 010a97d0 from its two arguments, then (when the vertex count is positive)… |
| `00892bc0` | MeshVisibilityDefinitionCtor | likely | Constructor for a mesh visibility definition from vertex base param_2 with stride param_3 and index base param_5 with stride param_6: calls 00892500, stores th… |

**Deep::MyLibraryServices** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891d10` | MyLibraryServicesCtor | confirmed | Constructor: calls 00dc2b30 on this, then stores the Deep::MyLibraryServices vftable at [this] and returns this. |
| `00891dd0` | MyLibraryServicesScalarDeletingDtor | confirmed | Scalar deleting destructor: calls 00dc2b40 on the object, then frees the block with _free when the low bit of the flag argument is set; returns the object. |

**Deep::OBBVisibilityDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008927f0` | OBBVisibilityDefinitionScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityDefinition dtor body 00892280 on this, then frees the block with _free when the low bit of the flag argument is… |

**Deep::SphereVisibilityDefinition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00892640` | SphereVisibilityDefinitionScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityDefinition dtor body 00892280 on this, then frees the block with _free when the low bit of the flag argument is… |

**Deep::VisibilityBoxObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008919d0` | VisibilityBoxObjectScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityObject dtor body 00891010 on the object, then frees the block with _free when the low bit of the flag argument … |

**Deep::VisibilityDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008922e0` | VisibilityDefinitionScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityDefinition dtor body 00892280 on this, then frees the block with _free when the low bit of the flag argument is… |
| `00892500` | VisibilityDefinitionCtor | confirmed | Constructor: calls 0071b950 (RefCounted-implicit base constructor) on this, then stores the Deep::VisibilityDefinition vftable at [this]. |

**Deep::VisibilityObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891730` | VisibilityObjectScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityObject dtor body 00891010 on ECX=this, then if the low bit of the flag argument is set calls _free(this); retur… |

**Deep::VisibilityPortal** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891a90` | VisibilityPortalScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityObjectInterface dtor body 008913c0 on the object, then frees the block with _free when the low bit of the flag … |

**Deep::VisibilityRoom** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891c80` | VisibilityRoomScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the room dtor body 00891b40, then frees the block with _free when the low bit of the flag argument is set; returns the object. |

**Deep::VisibilitySphereObject** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891a30` | VisibilitySphereObjectScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the VisibilityObject dtor body 00891010 on the object, then frees the block with _free when the low bit of the flag argument … |

**Deep::VisibilitySystem** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891ee0` | VisibilitySystemCreateRoom | confirmed | Virtual slot [3]: allocates 0x160 bytes with operator new 00d833a6, and if the allocation is non-null constructs a VisibilityRoom in it with 00891ab0 and retur… |
| `00892010` | VisibilitySystemScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the system dtor body 00891e30 on the object, then frees the block with _free when the low bit of the flag argument is set; re… |
| `00892030` | VisibilitySystemCtor | confirmed | Constructor: stores the VisibilitySystemInterface then VisibilitySystem vftables at [this], increments the global counter DAT_01be4710, zeroes the eleven dword… |
| `00891f50` | VisibilitySystemGetParams11 | likely | Virtual slot [4]: for the eleven keys 0x6e,0x6f,0x72,0x73,0x74,0x75,0x77,0x76,0x78,0x79,0x7a it calls 00dbefa0(key) then 00d83c90() and stores the result in th… |

**Deep::VisibilitySystemInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00891cd0` | VisibilitySystemInterfaceDtorBody | confirmed | Seven-byte destructor body: stores the Deep::VisibilitySystemInterface vftable at [ecx] and returns. |
| `00891ce0` | VisibilitySystemInterfaceScalarDeletingDtor | confirmed | Scalar deleting destructor: stores the VisibilitySystemInterface vftable at [this] and frees the block with _free when the low bit of the flag argument is set;… |

**OcclusionManager** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b6ac0` | OcclusionManager::Dtor | confirmed | Fastcall, ECX=this. Stores OcclusionManager::vftable at this+0, calls 010b6a40 (clear the keyed map) and 010b6880 (free the node list), then calls the AssetDel… |
| `010b6b30` | OcclusionManager::LoadZonesAndOverrides | confirmed | Thiscall, ECX=OcclusionManager, one stack argument (document or stream). Clears the zone map at this+8 and the override list at this+0x58 (010b6a40, 010b6880),… |
| `008b60b0` | OcclusionManager::ScalarDeletingDestructor | likely | Calls the destructor at 010b6ac0 and then calls operator delete on this when bit 0 of the flag argument is set; returns this. |

**(no class)** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b6990` | ZoneRecordCtor | likely | Fastcall, ECX=new record. Stores SoeUtil::IString<char>::vftable at +0, the static empty buffer at +4, and zeroes dwords +0x8, +0xc, +0x14, +0x1c, +0x20, +0x24… |

### `render.particle`

**ActorParticleEmitter** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00776f40` | ActorParticleEmitterCtor | confirmed | Thiscall constructor (ECX = object, four stack args; the first is passed to the base constructor 00775f70 with ECX = object). Stores ActorParticleEmitter::vfta… |
| `00777060` | ActorParticleEmitter::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 007c2ee0 on the object, frees it through _free when flag bit 0 is set,… |

**ActorParticleReference** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00777210` | ActorParticleReferenceDtor | confirmed | Fastcall destructor body (ECX = object). Stores ActorParticleReference::vftable at +0, calls 007c3e70 on the object (unlabelled cleanup), then calls the base d… |
| `00777290` | ActorParticleReference::ScalarDeletingDtor | confirmed | Thiscall virtual slot [0] (ECX = object, one byte stack flag). Calls the destructor body 00777210 on the object, frees it through _free when flag bit 0 is set,… |
| `007c3e70` | ActorParticleReference::ReleaseInstanceAndResetTarget | likely | Virtual slot [4] (24 bytes): calls this->vtable[+8] with (0,1), then tail-calls this->vtable[+0x24]. Byte-identical call sequence to 007c3890 (sound family). |
| `007c3f20` | ActorParticleReference::ReleaseInstance | likely | Thiscall; stack: unused dword and a char flag. When this+0x34 (particle instance) is non-null: clears this+0x34 first, calls 0086a8f0 when the flag is nonzero,… |
| `007c3fa0` | ActorParticleReference::ReleaseInstanceAndClearTarget | likely | Virtual slot [9] (49 bytes): calls this->vtable[+8] with (1,1) (the instance release, 007c3f20, with flag 1), then when the pointer at this+0x30 is non-null de… |
| `007c40b0` | ActorParticleReference::ResolveTarget | likely | Fastcall, ECX = this (129 bytes). Returns this+0x30 when set. Otherwise asks the provider at this+0x24 through its vtable[+0x64] whether it can resolve (test c… |
| `007c4140` | ActorParticleReference::UpdateCreateInstance | likely | Virtual slot [1] (989 bytes), thiscall with one object argument P (may be null). When this+0x34 (instance) is null and the target at this+0x30 is set (resolved… |

**DeepParticle::ApplyVelocityController** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00887110` | ApplyVelocityController::EvalRange | confirmed | Thiscall virtual, slot [6]; stack args: owner (param_2), begin (param_3), end (param_4); ret 0xc. Calls FUN_00878e20(0x7214b4a8); when the result is nonzero, f… |

**DeepParticle::ColorCurveController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00886420` | ColorCurveController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. EH frame. Calls FUN_0087aa60 (base constructor; receiver not decoded from the bytes), stores DeepParticle::Col… |
| `008866c0` | ColorCurveController::EvalColorRange | confirmed | Thiscall virtual, slot [6]. Stack args: owner (param_2), begin (param_3), end (param_4). Calls FUN_00886320 on the owner's colour cache to get a 16-float cubic… |

**DeepParticle::ColorInitializer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00886100` | ColorInitializer::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00885c80 on this (GenericParticleController<ParticleColor,ParticleInitializer> constructor),… |

**DeepParticle::ColorRangeController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00885d50` | ColorRangeController::FillRandomRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4). Reads two 16-byte entries from owner.+0xc.+0x148.+0x34 at indices shor… |
| `008861c0` | ColorRangeController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_00886100 on this (ColorInitializer constructor), stores DeepParticle::ColorRangeController::vftable,… |

**DeepParticle::ConstantColorController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00885a30` | ConstantColorController::FillRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4); ret 0xc. Reads a 16-byte entry (four dwords) from owner.+0xc.+0x148.+0… |
| `008868e0` | ConstantColorController::ctorAppend | confirmed | Thiscall constructor, args: four-float value (param_2) and owner object (param_3); ret 8. Calls FUN_00886100 on this (ColorInitializer constructor), stores Dee… |
| `008869f0` | ConstantColorController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_00886100 on this (ColorInitializer constructor), stores DeepParticle::ConstantColorController::vftab… |

**DeepParticle::ConstantFrameController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884130` | ConstantFrameController::ctor | confirmed | Thiscall constructor with three stack args (ret 0xc). EH frame. Calls FUN_008839d0 on this (FrameInitializer constructor), stores DeepParticle::ConstantFrameCo… |
| `00884980` | ConstantFrameController::ctorAppend | confirmed | Thiscall constructor, args: value (param_2) and owner object (param_3); ret 8. EH frame. Calls FUN_008839d0 on this, stores DeepParticle::ConstantFrameControll… |

**DeepParticle::ConstantSizeController** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884a60` | ConstantSizeController::FillRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner object (param_2), begin (param_3), end (param_4); ret 0xc. Reads one dword from the owner table at owner.+0xc.+0x… |
| `00885510` | ConstantSizeController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_008850a0 on this (SizeInitializer constructor), stores DeepParticle::ConstantSizeController::vftable… |
| `008858a0` | ConstantSizeController::ctorAppend | confirmed | Thiscall constructor, args: value (param_2) and owner object (param_3); ret 8. Calls FUN_008850a0 on this (SizeInitializer constructor), stores DeepParticle::C… |

**DeepParticle::FrameFilter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00883450` | FrameFilter::ctor | confirmed | Fastcall constructor (ECX=this, returns this). Sets up an MSVC EH frame, calls FUN_00882f90 on this, then stores DeepParticle::FrameFilter::vftable at [this]. |

**DeepParticle::FrameInitializer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008839d0` | FrameInitializer::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00883380 on this (the GenericParticleController<ParticleFrame,ParticleInitializer> construct… |

**DeepParticle::FrameSequenceController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00883a90` | FrameSequenceController::EvalCurveRange | confirmed | Thiscall virtual, slot [6]. Stack args: owner object (param_2), begin index (param_3), end index (param_4). Calls FUN_008830b0 on this+0x10 to get four cubic c… |
| `008845e0` | FrameSequenceController::ctor | confirmed | Thiscall constructor with three stack args (ret 0xc). EH frame. Calls FUN_00883450 on this (FrameFilter constructor), stores DeepParticle::FrameSequenceControl… |

**DeepParticle::GenericParticleController<DeepParticle::ParticleColor,DeepParticle::ParticleInitializer>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00885c80` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_008819d0 on this (the ParticleInitializer constructor, per its vtable store in shard S0213),… |

**DeepParticle::GenericParticleController<DeepParticle::ParticleFrame,DeepParticle::ParticleInitializer>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00883380` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). Sets up an MSVC EH frame, calls FUN_008819d0 on this (a ParticleInitializer constructor per its vtable store in … |

**DeepParticle::GenericParticleController<DeepParticle::ParticlePosition,DeepParticle::ParticleFilter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00886d90` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00878c90 on this (base constructor, labelled only in other shards), then stores GenericParti… |

**DeepParticle::GenericParticleController<DeepParticle::ParticlePosition,DeepParticle::ParticleInitializer>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00887040` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_008819d0 on this (the ParticleInitializer constructor, per its vtable store in shard S0213),… |

**DeepParticle::GenericParticleController<DeepParticle::ParticleSize,DeepParticle::ParticleFilter>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884aa0` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00878c90 on this (base constructor, labelled only in other shards), then stores GenericParti… |

**DeepParticle::GenericParticleController<DeepParticle::ParticleSize,DeepParticle::ParticleInitializer>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884bc0` | GenericParticleController::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_008819d0 on this (the ParticleInitializer constructor, per its vtable store in shard S0213),… |

**DeepParticle::HSVToRGBColorController** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00885e40` | HSVToRGBColorController::EvalRange | confirmed | Virtual slot [6] (ECX unused). Stack args: owner object (param_1; input base array at +0x14.+0x18, output at +0x18), begin (param_2), end (param_3). For each i… |

**DeepParticle::OffsetWaveController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008872d0` | OffsetWaveController::FillRandomRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4). Bounds are the owner table entries at indices short(this+0x72) and sho… |
| `00887af0` | OffsetWaveController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. EH frame. Calls FUN_008871c0 on this (PositionFilter constructor), stores DeepParticle::OffsetWaveController::… |

**DeepParticle::OffsetWaveControllerInstance** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00887d60` | OffsetWaveControllerInstance::ctor | confirmed | Thiscall constructor, args: owner (param_2) and a third value (param_3); ret 8. Calls FUN_00878590(param_2, param_3) on this, stores DeepParticle::OffsetWaveCo… |
| `00887e10` | OffsetWaveControllerInstance::DeleteViaSlot1 | confirmed | Fastcall (ECX = object pointer). If the pointer is non-null, calls the object's virtual slot [1] with the argument 1 and returns. Does not store a vtable or fr… |

**DeepParticle::OrbitController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00886ea0` | OrbitController::EvalOrbitRotate | confirmed | Virtual slot [6] (ECX unused). Stack args: owner object (param_1; buffers read at +0x14/+0x18/+0x24/+0x34/+0x44 and a float at owner.+0xc.+0x20), begin (param_… |
| `00887490` | OrbitController::EvalOrbitRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4). Per index: calls FUN_00886bf0 (RandomDirectionVec4) twice with the own… |

**DeepParticle::ParticleController** (30)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008833f0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with a flag byte [esp+4]. Sets up an MSVC EH frame, stores DeepParticle::ParticleController::vftable at [this], and if the flag bit 0 is set calls _fr… |
| `008834b0` | ParticleController::~ParticleController | confirmed | Fastcall with ECX=this and no arguments. Stores DeepParticle::ParticleController::vftable at [this] and returns. Has an MSVC EH frame but no other work. |
| `00883500` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with a flag byte [esp+4]. Sets up an MSVC EH frame, stores DeepParticle::ParticleController::vftable at [this], and if flag bit 0 is set calls _free(t… |
| `00883a30` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this) (thunk to 00d8a9d8). R… |
| `008841d0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00884820` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this) (thunk to 00d8a9d8). R… |
| `00884b00` | ParticleController::~ParticleController | confirmed | Fastcall with ECX=this and no arguments. Stores DeepParticle::ParticleController::vftable at [this] and returns. Same bytes as 008834b0 and 008847d0. |
| `00884b50` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00884c30` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00884eb0` | ParticleController::~ParticleController | confirmed | Fastcall with ECX=this and no arguments. Stores DeepParticle::ParticleController::vftable at [this] and returns. Same bytes as 008834b0, 008847d0 and 00884b00. |
| `00884f00` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00885100` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `008855c0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00885620` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00885840` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00885cf0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00886160` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `008863c0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00886660` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00886990` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00886df0` | ParticleController::~ParticleController | confirmed | Fastcall with ECX=this and no arguments. Stores DeepParticle::ParticleController::vftable at [this] and returns. Same bytes as 008834b0, 00884b00 and 00884eb0. |
| `00886e40` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `008870b0` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00887220` | ParticleController::~ParticleController | confirmed | Fastcall with ECX=this and no arguments. Stores DeepParticle::ParticleController::vftable at [this] and returns. Same bytes as 008834b0. |
| `00887270` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00887a90` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `00887d00` | ParticleController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+4]. Stores DeepParticle::ParticleController::vftable at [this] and, if flag bit 0 is set, calls _free(this). Returns this. Same bo… |
| `008857f0` | ParticleController::SetVtable | likely | Fastcall (ECX=this), no arguments, no EH frame and no base call. Stores DeepParticle::ParticleController::vftable at [this] and returns void. Bytes match 00885… |
| `008862d0` | ParticleController::SetVtable | likely | Fastcall (ECX=this), no arguments, no EH frame and no base call. Stores DeepParticle::ParticleController::vftable at [this] and returns void. |
| `00886610` | ParticleController::SetVtable | likely | Fastcall (ECX=this), no arguments, no EH frame and no base call. Stores DeepParticle::ParticleController::vftable at [this] and returns void. |

**DeepParticle::ParticleEntity** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0086a580` | ParticleEntity::Slot3 | confirmed | Virtual slot [3] (secondary vtable at object+8, thiscall, one stack argument param_2). If *(*param_2 + 0x170) is non-zero: sets byte this+0x17 = 1; if the byte… |
| `0086a960` | ParticleEntity::Slot12 | confirmed | Virtual slot [12] (secondary vtable at object+8, thiscall). If the dword at this-4 equals 1, tail-calls 00868be0 with ECX=this-8. Otherwise calls 0075c060 on t… |
| `0086a990` | ParticleEntity::Slot9 | confirmed | Virtual slot [9] (secondary vtable at object+8, thiscall, one stack argument float-vector pointer param_2). Returns the Euclidean distance between the four flo… |
| `0086b3e0` | ParticleEntity::Slot4UpdateBounds | confirmed | Virtual slot [4] (secondary vtable at object+8, thiscall, one stack pointer param_2). When the argument's vtable entry 0x170 is non-null and this+0x9a4 is non-… |
| `0086c010` | ParticleEntity::Slot7 | confirmed | Virtual slot [7] (secondary vtable at object+8, thiscall, one stack pointer param_2 to a 4x4 matrix). If 0074b9e0 reports the matrix differs from the stored on… |

**DeepParticle::ParticleEntity::InstanceUpdateEvent** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0086bd50` | InstanceUpdateEvent::AdvanceByTime | confirmed | Virtual slot [0] (thiscall, one float stack argument, the time to advance). Queries the duration of the current segment from the sub-object at owner+0xe4 (vtab… |

**DeepParticle::PositionFilter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008871c0` | PositionFilter::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00886d90 on this (GenericParticleController<ParticlePosition,ParticleFilter> constructor), t… |

**DeepParticle::PositionFromShapeController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00886d10` | PositionFromShapeController::EvalOffsetRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner (param_2), begin (param_3), end (param_4). Reads the scale from the owner table at owner+0xc.+0x148.+0x24 indexed… |
| `00887ef0` | PositionFromShapeController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_00887a30 on this (PositionInitializer constructor), stores DeepParticle::PositionFromShapeController… |

**DeepParticle::PositionInitializer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00887a30` | PositionInitializer::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00887040 on this (GenericParticleController<ParticlePosition,ParticleInitializer> constructo… |

**DeepParticle::RandomFrameController** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008845c0` | RandomFrameController::ScalarDeletingDtor | confirmed | Thiscall with flag byte [esp+8]. Calls FUN_00884560 on this (its destructor body, analysed separately), then if flag bit 0 is set calls _free(this). Returns th… |

**DeepParticle::ScaleCurveController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00885270` | ScaleCurveController::EvalScaleRange | confirmed | Thiscall virtual, slot [6]. Stack args: owner object (param_2), begin (param_3), end (param_4); ret 0xc. Calls FUN_008830b0 on the owner's cubic cache to get f… |
| `00885680` | ScaleCurveController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_00884e50 on this (SizeFilter constructor), stores DeepParticle::ScaleCurveController::vftable, sets … |

**DeepParticle::SizeFilter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884e50` | SizeFilter::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00884aa0 on this (GenericParticleController<ParticleSize,ParticleFilter> constructor), then … |

**DeepParticle::SizeInitializer** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008850a0` | SizeInitializer::ctor | confirmed | Fastcall constructor (ECX=this, returns this). EH frame. Calls FUN_00884bc0 on this (GenericParticleController<ParticleSize,ParticleInitializer> constructor), … |

**DeepParticle::SizeRangeController** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00884c90` | SizeRangeController::FillRandomRange | confirmed | Thiscall virtual, slot [5]. Stack args: owner object (param_2), begin (param_3), end (param_4). Reads two float table entries from owner.+0xc.+0x148.+0x24 at t… |
| `00885160` | SizeRangeController::ctor | confirmed | Thiscall constructor, three stack args; ret 0xc. Calls FUN_008850a0 on this (SizeInitializer constructor), stores DeepParticle::SizeRangeController::vftable, s… |

**DeepParticle::Timeline::Event** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e7690` | DeepParticle::Timeline::Event::DtorBody | confirmed | Fastcall (ECX = object, no args). Stores DeepParticle::Timeline::Event::vftable into the object's first dword and returns. Releases nothing. |

**DeepParticle::VariableBank::Shared** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ea240` | DeepParticle::VariableBank::Shared::ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the Shared destructor body 007ea1e0, then calls _free on this when bit 0 of the flag is set. Returns nothing (returns this… |
| `007ea1e0` | VariableBankSharedDtorBody | likely | Fastcall (ECX = object). Calls 007e9400 (the 32,32 HashListMap destructor body) on the object's variable map, then stores SoeUtil::RefCounted::vftable at the o… |

**SoeUtil::HashList<DeepParticle::VariableKey<DeepParticle::Untyped>,32,32>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007ea000` | HashListCtor32x32 | confirmed | Fastcall (ECX = object). Calls 007e9ed0 (the 32,32 HashListMap constructor) on the object, then stores the SoeUtil::HashList<VariableKey,32,32> vtable and retu… |
| `007ea0d0` | HashList32x32ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the 32,32 destructor body 007e9400 with a cookie value, then calls _free on this when bit 0 of the flag is set. Returns th… |

**SoeUtil::HashListMap<int,DeepParticle::VariableKey<DeepParticle::Untyped>,32,-1>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e8350` | HashListMapCtor32 | confirmed | Fastcall (ECX = object). Stores the HashListMap vtable at the object, zeroes the dwords at +4, +8, +0xc and +0x10, and clears the 0x80-byte bucket array at +0x… |
| `007e8dc0` | HashListMapDtorBody | confirmed | Fastcall (ECX = object). Saves the dword at +8, stores the HashListMap vtable, then while the saved value is non-zero calls 007e87b0 on the node at +8 (when it… |
| `007e8e30` | HashListMapScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Stores the HashListMap vtable, then drains the node list at +8 by calling 007e87b0 on each node (body inlined, same loop as 007e… |

**SoeUtil::HashListMap<int,DeepParticle::VariableKey<DeepParticle::Untyped>,32,32>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007e9400` | HashListMapDtorBody32x32 | confirmed | Fastcall (ECX = object). Stores the 32,32 HashListMap vtable, drains the node list at +8 through 007e87b0, frees the tree at +0x94 through 007e8970 (children a… |
| `007e9eb0` | HashListMap32x32ScalarDeletingDtor | confirmed | Thiscall with a byte flag arg. Calls the 32,32 destructor body 007e9400, then calls _free on this when bit 0 of the flag is set. Returns this. |
| `007e9ed0` | HashListMapCtor32x32 | confirmed | Fastcall (ECX = object). Stores the 32,-1 HashListMap vtable, zeroes +4, +8, +0xc and +0x10, clears the 0x80-byte bucket array at +0x14, then stores the 32,32 … |

**SoeUtil::List<DeepParticle::ParticleEntity::Instance,-1>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0086e530` | ListInstanceDtorBody | confirmed | Destructor body (fastcall): installs the SoeUtil::List<DeepParticle::ParticleEntity::Instance,-1> vtable, then removes every node from the list through 0086c53… |
| `0086e560` | ListInstanceScalarDtor | confirmed | Scalar deleting destructor (thiscall, one byte flag): installs the Instance list vtable, releases every node through 0086c530 (ListUnlinkRelease), then calls _… |

**(no class)** (18)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008835b0` | FreeBuffer4 | confirmed | Virtual, slot [4] of four shared vtables. ECX unused. Calls FUN_00401c00(ptr, 4) on the stack pointer at [esp+4]. Ret 4. |
| `00883620` | FreeBuffer4 | confirmed | Virtual, slot [4] of three shared vtables. ECX unused. Calls FUN_00401c00(ptr, 4) on the stack pointer at [esp+4]. Ret 4. |
| `00884fb0` | FreeBuffer4 | confirmed | Virtual, slot [4] of four shared vtables. ECX unused. Calls FUN_00401c00(ptr, 4) on the stack pointer at [esp+4]. Ret 4. |
| `00885020` | FreeBuffer4 | confirmed | Virtual, slot [4] of three shared vtables. ECX unused. Calls FUN_00401c00(ptr, 4) on the stack pointer at [esp+4]. Ret 4. |
| `00885ff0` | FreeBuffer16 | confirmed | Virtual, slot [4] of four shared vtables. ECX unused. Calls FUN_00401c00(ptr, 0x10) on the stack pointer at [esp+4]. Ret 4. |
| `00887820` | FreeBuffer16 | confirmed | Virtual, slot [4] of three shared vtables. ECX unused. Calls FUN_00401c00(ptr, 0x10) on the stack pointer at [esp+4]. Ret 4. |
| `00887890` | FreeBuffer16 | confirmed | Virtual, slot [4] of five shared vtables. ECX unused. Calls FUN_00401c00(ptr, 0x10) on the stack pointer at [esp+4]. Ret 4. |
| `007c6c60` | CreateActorParticleReference | likely | Same shape as 007c6c00 (90 bytes): calls the constructor 007bb300 with the dwords at *param_2 and *param_3 and returns the first argument. |
| `0086a8f0` | ParticleInstanceListRelease | likely | If the byte at this+0x1e is zero: for each node of the list at this+0x17c (next +0xf0) clears bit 30 and sets bit 31 of the dword at (node+0xe4)+0x98, then cal… |
| `0086c1f0` | ParticleInstanceListAppendNew | likely | fastcall (list object in ECX: head +4, tail +8, count +0xc): EH-framed. Allocates one block through the list's allocator (vtable slot [2]); if non-null constru… |
| `0086c5f0` | StructCopyAssignRefCounted | likely | thiscall (param_2 = source struct): zeroes this+8, copies the source's byte buffer (+4/+8) into this buffer through 00869290 growth and memcpy, calls 0073c370 … |
| `00883560` | AllocateCountTimes4 | likely | Virtual, shared by four vtables (slot [3]). Calls FUN_006fb9c0(), then FUN_006fb860 with ECX set to FUN_006fb9c0's result. Allocates count*4 bytes with alignme… |
| `008835d0` | AllocateCountTimes4 | likely | Virtual, slot [3] of three shared vtables. Same body as 00883560 (bytes differ only in call displacements): calls FUN_006fb9c0(), then FUN_006fb860 with ECX se… |
| `00884f60` | AllocateCountTimes4 | likely | Virtual, slot [3] of four shared vtables. Calls FUN_006fb9c0(), then FUN_006fb860 with ECX set to its result, and allocates count*4 bytes with alignment 4 thro… |
| `00884fd0` | AllocateCountTimes4 | likely | Virtual, slot [3] of three shared vtables. Calls FUN_006fb9c0(), then FUN_006fb860 with ECX set to its result, and allocates count*4 bytes with alignment 4 thr… |
| `00885fa0` | AllocateCountTimes16 | likely | Virtual, slot [3] of four shared vtables. Calls FUN_006fb9c0(), then FUN_006fb860 with ECX set to its result, and allocates count*16 bytes with alignment 16 th… |
| `008877d0` | AllocateCountTimes16 | likely | Virtual, slot [3] of three shared vtables. Calls FUN_006fb9c0(), then FUN_006fb860 with ECX set to its result, and allocates count<<4 bytes with alignment 16 t… |
| `00887840` | AllocateCountTimes16 | likely | Virtual, slot [3] of five shared vtables (Position filter, offset and orbit family). Same body as 008877d0: calls FUN_006fb9c0(), then FUN_006fb860 with ECX se… |

### `render.path_debug`

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009cb730` | DrawTwoVectorChains | likely | Walks chains at object offsets +0x8bc and +0x54, offsets each Vector4, then draws each point and each segment with distinct color constants. |
| `009cba90` | RenderNearbyVectorPointsAndSegments | likely | Copies two Vector4 lists, highlights points within a squared-distance threshold of the supplied vector, draws nearby connecting segments, then destroys both te… |

### `render.pipeline`

**Deep::FloatParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739980` | Deep::FloatParameter::~FloatParameter | confirmed | Scalar deleting destructor: stores Deep::ParameterValue::vftable at [this] and frees the object with _free (00d82463) when bit 0 of the flag argument is set; r… |

**Deep::IntParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739920` | Deep::IntParameter::~IntParameter | confirmed | Scalar deleting destructor: stores Deep::ParameterValue::vftable at [this] and frees the object with _free (00d82463) when bit 0 of the flag argument is set; r… |

**Deep::ParameterValue** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739850` | GetStaticMatrixCopy | likely | Returns the address of the static 16-float block at 0x01bdfb80; on the first call (guarded by bit 0 of DAT_01bdfbc0) it copies the 16 dwords at 0x01bc2660 into… |

**Deep::Pipeline** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0073d990` | Deep::Pipeline::Dtor | confirmed | Destructor of Deep::Pipeline: stores Deep::Pipeline::vftable, releases the reference-counted object at [this+8] (LOCK-decrement of [obj+4], virtual destroy thr… |

**Deep::Pipeline::Definition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00742830` | Deep::Pipeline::Definition::Definition | confirmed | Constructor for Deep::Pipeline::Definition: stores its owner argument at this+8, sets the vftable 017ea824, initialises two SoeUtil::IString<char> members to t… |
| `007422c0` | Deep::Pipeline::Definition::BuildStage | likely | Thiscall on Pipeline::Definition (ECX=this) with one child node pushed as the stack argument. Reads the node's Scene/Callback type, PixelCoordinates, DrawStyle… |

**Deep::Stage** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00742810` | Deep::Stage::~Stage | confirmed | Scalar deleting destructor: calls the Stage body destructor 007421d0 on this, then frees this via _free (00d82463) only when bit 0 of the argument byte is set;… |

**Deep::UndefinedParameter** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007398c0` | Deep::UndefinedParameter::~UndefinedParameter | confirmed | Scalar deleting destructor: stores Deep::ParameterValue::vftable at [this] and, when bit 0 of the flag argument is set, frees the block with _free (00d82463); … |

**Deep::Vector4Parameter** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `007399e0` | Deep::Vector4Parameter::Slot7_AssignFromSource | confirmed | Calls param_2's vtable slot 10 (+0x28) to fetch a float4, copies its four floats into this+0x10..+0x1c and returns 1 when at least one lane differs from the pr… |
| `00739a50` | Deep::Vector4Parameter::Slot4_Assign | confirmed | Copies param_2[0..3] into this+0x10..+0x1c and returns 1 when at least one of the four floats differs from the old value, else 0 (SSE cmpeqps/movmskps on the o… |
| `00739a90` | Deep::Vector4Parameter::~Vector4Parameter | confirmed | Scalar deleting destructor: stores Deep::ParameterValue::vftable at [this] and frees the object with _free (00d82463) when bit 0 of the flag argument is set; r… |

**(no class)** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739bc0` | GetPtrIfFlagSet | likely | Returns the dword at [this+0x28] when the dword at [this+0x2c] is non-zero, otherwise 0. Single conditional field read, no other side effects. |
| `00739c40` | FindEntryByString | likely | Linear search over the array at [this+0x8c] with count [this+0x90]: for each element takes the char* at [element+4] and compares it with the NUL-terminated arg… |
| `00739cc0` | BuildStageParameterBuffer | likely | Builds the 4-row parameter buffer of one stage. Takes the render device from [this+4]+0x1d0, asks param_2's vtable slot 7 (+0x1c) for a value and calls the dev… |
| `0073a200` | LookupByNameInList | likely | Walks the singly linked list headed at [this+0x20] (next pointer at [node+0x18]) comparing each node's string ([node], byte length [node+8], or the empty-strin… |
| `0073b7b0` | LookupByNameInListB | likely | Second instantiation of the name lookup: walks the singly linked list headed at [this+0x2c] (next pointer at [node+0x2c]) comparing each node's string ([node],… |
| `0073c7e0` | XmlDecodeEntityValueA | likely | In-place XML value decoder: skips leading bytes while the skip table DAT_017ea060 is set, then copies bytes down over the skipped prefix and expands entity ref… |
| `0073ca70` | XmlDecodeEntityValueB | likely | Second copy of the in-place XML value decoder, identical to 0073c7e0 but using the skip table DAT_017ea160 and the value table DAT_017e9f38: expands '&#xNN;'/'… |
| `0073cd00` | XmlDecodeEntityValueInPlaceA | likely | In-place XML value decoder variant: skips leading bytes while the skip table DAT_017ea360 is set, then rewrites the bytes in place (destination and source in t… |
| `0073cfa0` | XmlDecodeEntityValueInPlaceB | likely | Second in-place decoder variant: skips leading bytes while the table DAT_017ea460 is set, rewrites the buffer in place expanding '&#xNN;'/'&#NN;' through 00739… |
| `00742ab0` | LoadPipelineDefinitionFile | likely | Thiscall. Reads the text file named by its argument into an IString via ReadTextFileToIString (00fbf4a0), then allocates 0x5c0 bytes (00d833a6), constructs a P… |

### `render.primitive`

**Deep::BoxPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075b610` | Deep::BoxPrimitive::Ctor | confirmed | Thiscall with five stack arguments. Stores Deep::BoxPrimitive::vftable at +0, copies sixteen dwords from the pointer arg5 to +0x10..+0x4c, copies eight dwords … |

**Deep::CapsulePrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075d6e0` | Deep::CapsulePrimitive::Slot0BuildMesh | likely | Same algorithm as 0075d1f0 (SpherePrimitive::Slot0BuildMesh) with different field offsets: n = this+0x3c, the flag is at +0x40, and the radius-like fields are … |

**Deep::Circle2DPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a300` | Deep::Circle2DPrimitive::Slot0BuildCircleBuffers | confirmed | Thiscall with one stack argument (factory object), ret 4. N = this+0x14. Calls the factory's vtable slot [0] with (0x481718ce, 3, N, &vertexOut, 2N, &indexOut)… |

**Deep::DebugText2DPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a8b0` | Deep::DebugText2DPrimitive::Ctor | confirmed | Thiscall with four stack arguments (dword, two floats, dword). Stores Deep::DebugText2DPrimitive::vftable at +0. Its loop writes DAT_01b83900 into +8 and +0xc … |

**Deep::FatLinePrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a230` | Deep::FatLinePrimitive::Ctor | confirmed | Thiscall with five stack arguments (two pointers and three dwords). Stores Deep::FatLinePrimitive::vftable at +0, copies four dwords from arg2 to +0x10..+0x1c … |

**Deep::GridPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075dbd0` | Deep::GridPrimitive::Slot0BuildLines | likely | Thiscall with the factory object argument. Derives a scale from the reciprocal of this+0x60, computes the grid bounds from this+0x50, +0x54, +0x58 and +0x5c th… |

**Deep::Line2DPrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a0d0` | Deep::Line2DPrimitive::Ctor | confirmed | Thiscall with six stack arguments. Stores Deep::Line2DPrimitive::vftable at +0. Its loop sets dwords +4..+0x10 to DAT_01b83900, and the stores that follow over… |

**Deep::LinePrimitive** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a000` | Deep::LinePrimitive::Ctor | confirmed | Thiscall with four stack arguments. Stores Deep::LinePrimitive::vftable at +0, copies the four dwords at pointer arg2 to +0x10..+0x1c and the four dwords at po… |

**Deep::PrimitiveEntity** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075bef0` | Deep::PrimitiveEntity::Ctor | confirmed | Thiscall with (manager pointer arg2, dword arg3). Stores Deep::PrimitiveBatcherInterface::vftable at +0x14, then Deep::PrimitiveEntity::vftable at +0 and at +0… |
| `00759c60` | Deep::PrimitiveEntity::CreateBatch | likely | Thiscall with six stack arguments (ret 0x18). If this+0x10 > 0, calls vtable slot [3] of the objects at [[this+0xc]+8] and [[this+0xc]+0xc] when non-null. Allo… |

**Deep::Quad2DPrimitive** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a480` | Deep::Quad2DPrimitive::Ctor | confirmed | Thiscall with ten stack arguments (two floats, eight dwords). Stores Deep::Quad2DPrimitive::vftable at +0. Stores param_2 minus the float constant DAT_017649d4… |
| `0075a570` | Deep::Quad2DPrimitive::Slot0BuildQuadBuffer | likely | Thiscall with one stack argument (factory object), ret 4. Calls the factory's vtable slot [0] with (0x7c670e85, 0, 6, &out, 0, 0). Keeps the factory's return v… |

**Deep::TrianglePrimitive** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0075a920` | Deep::TrianglePrimitive::Ctor | confirmed | Thiscall with five stack arguments (three pointers, a dword, a byte). Stores Deep::TrianglePrimitive::vftable at +0, copies four dwords from arg2 to +0x10..+0x… |
| `0075cac0` | Deep::TrianglePrimitive::Slot0BuildPrimitive3D | confirmed | One stack argument (factory object), ret 4. Calls 0075a990 (BuildTriangleOutlineKeyed) on this with the factory argument and the string "Primitive3D" as its ke… |

### `render.sky`

**Deep::SkyLightEntity** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b2cf0` | SkyLightEntity::~SkyLightEntityBody | confirmed | Destructor body of Deep::SkyLightEntity: stores Deep::SkyLightEntity::vftable into this, calls 00804590 on this with two zero stack arguments, then calls 010b2… |
| `010b3080` | SkyLightEntity::SkyLightEntity | confirmed | Constructor of Deep::SkyLightEntity: takes an owner object as its first stack argument; calls 010b2df0 on that owner with key 0x39ce7021 and constructs the Lig… |
| `010b3250` | SkyLightEntity::~SkyLightEntity | confirmed | Scalar deleting destructor in vtable slot 0 of Deep::SkyLightEntity: calls 010b2cf0 (the SkyLight dtor body) on this, then if bit 0 of the flag argument is set… |

**Sky::LinearBlendController<SkyDefinition::Blend<SkyDefinition::Horizon>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0080a700` | LinearBlendController_HorizonCtor | confirmed | Fastcall SEH-framed ctor (ECX = this): stores the Sky::LinearBlendController<SkyDefinition::Blend<SkyDefinition::Horizon>_> vtable at this+0, calls 00808b00 (C… |

**Sky::LinearBlendController<SkyDefinition::Blend<SkyDefinition::TintedTexture>_>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00807f70` | LinearBlendController_TintedCtor | confirmed | Fastcall SEH-framed ctor: stores the Sky::LinearBlendController<SkyDefinition::Blend<SkyDefinition::TintedTexture>_> vtable at this+0, default-constructs three… |

**Sky::LinearBlendController<SkyDefinition::Satellite>** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00808340` | LinearBlendController_SatelliteCtor | confirmed | Fastcall ctor (ECX = this): stores the Sky::LinearBlendController<SkyDefinition::Satellite> vtable at this+0, default-constructs three sub-objects by calling 0… |
| `00808900` | Sky::LinearBlendController_SatelliteScalarDeletingDtor | confirmed | Thiscall with one byte flag: calls 008083d0 (the Satellite blend-controller dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns… |
| `008083d0` | LinearBlendController_SatelliteDtor | likely | Fastcall SEH-framed teardown: stores the Sky::LinearBlendController<SkyDefinition::Satellite> vtable at this+0, calls 00805910 (base teardown), then calls 0080… |

**Sky::LinearBlendController<SkyDefinition::TintedTexture>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008088e0` | Sky::LinearBlendController_TintedScalarDeletingDtor | confirmed | Thiscall with one byte flag: calls 00808040 (the TintedTexture blend-controller dtor body) with this, then calls _free(this) when bit 0 of the flag is set; ret… |
| `00808040` | LinearBlendController_TintedDtor | likely | Fastcall SEH-framed teardown: stores the Sky::LinearBlendController<SkyDefinition::TintedTexture> vtable at this+0, calls 00805890 (base teardown; receiver not… |

**SkyDefinition** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00815260` | SkyDefinition::LoadXml | confirmed | Thiscall with ECX = SkyDefinition, a stack pointer argument stored at this+8, and a float argument. Installs SkyDefinition::vftable, builds TintedTexture tempo… |
| `00818050` | SkyDefinition::ScalarDeletingDtor | confirmed | Thiscall scalar deleting destructor, vtable slot 0 of SkyDefinition. Calls the dtor body 00814f20 on this, then if the low bit of the flag argument is set call… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00794550` | LoadSkyXml | likely | Fastcall (ECX = this). When this+0x2e0 is set and 006fc5b0 finds the literal name sky.xml, and this+0x2e0 and this+0x2e4 are set, it allocates 0x2830 bytes, cr… |
| `00794690` | LoadSkyIndoorXml | likely | Fastcall (ECX = this). Same control flow as 00794550 with the literal name sky_indoor.xml: existence check, 0x2830-byte object via 00815260, 008034a0, 006fa730… |

### `render.target`

**Deep::RenderTarget** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00748280` | Deep::RenderTarget::~RenderTarget | confirmed | Destructor body: stores the RenderTarget vftable, frees the name buffer at this+0x14 with _free, releases the reference-counted object at this+0x18 (LOCK decre… |
| `007482c0` | Deep::RenderTarget::~RenderTarget | confirmed | Scalar deleting destructor (vtable slot [0] of RenderTarget): stores the RenderTarget vftable, frees the name buffer through _free, releases the reference-coun… |

**Deep::RenderTargetSet** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00739b20` | Deep::RenderTargetSet::~RenderTargetSet | confirmed | Destructor body: stores Deep::RenderTargetSet::vftable at [this], frees the pointer at [this+0x14] through _free (00d83ac6) and zeroes the dwords at +0x18, +0x… |
| `00739b50` | Deep::RenderTargetSet::~RenderTargetSet_Delete | confirmed | Scalar deleting destructor (vtable slot 0): stores Deep::RenderTargetSet::vftable, frees the pointer at +0x14 with _free, zeroes +0x18..+0x28, then frees the o… |
| `00748310` | Deep::RenderTargetSet::RenderTargetSet | confirmed | Constructor: stores the RenderTargetSet vftable, zeroes the fields at this+4..+0x8 and this+0x28 and the byte at this+0x2c, then copies a NUL-terminated name a… |

## Source shards

[[S0020 004621ca-0046734c|S0020]] (80), [[S0152|S0152]] (1), [[S0154|S0154]] (22), [[S0156|S0156]] (57), [[S0157|S0157]] (20), [[S0158|S0158]] (12), [[S0160|S0160]] (43), [[S0162|S0162]] (27), [[S0163|S0163]] (2), [[S0164|S0164]] (3), [[S0165|S0165]] (48), [[S0168|S0168]] (10), [[S0170|S0170]] (20), [[S0171|S0171]] (9), [[S0173|S0173]] (10), [[S0175|S0175]] (2), [[S0176|S0176]] (1), [[S0178 007bd5b0-007c2fe0|S0178]] (29), [[S0179|S0179]] (20), [[S0183|S0183]] (33), [[S0184|S0184]] (65), [[S0186|S0186]] (41), [[S0192 00805a50-0080a790|S0192]] (9), [[S0195|S0195]] (5), [[S0209|S0209]] (27), [[S0214 008830b0-00887ef0|S0214]] (92), [[S0217|S0217]] (42), [[S0219 0089d990-008a3c80|S0219]] (6), [[S0220 008a3ce0-008a8500|S0220]] (3), [[S0221 008a86b0-008b1d80|S0221]] (37), [[S0222 008b1e00-008b7730|S0222]] (31), [[S0223 008b7790-008bcf40|S0223]] (3), [[S0225 008c52a0-008ca980|S0225]] (30), [[S0227 008d0920-008d5c60|S0227]] (7), [[S0229 008d92f0-008dea46|S0229]] (1), [[S0233 008ec2a0-008f0cc0|S0233]] (8), [[S0237 008ff3d0-00904e70|S0237]] (3), [[S0241 009162f0-0091aa10|S0241]] (3), [[S0242 0091af50-009200d0|S0242]] (5), [[S0254 00954ec0-0095a710|S0254]] (22), [[S0256 009606e0-00965340|S0256]] (3), [[S0259 0096e490-00972d30|S0259]] (3), [[S0275 009c8e90-009cdee0|S0275]] (2), [[S0292 00a29620-00a2fe20|S0292]] (2), [[S0297|S0297]] (21), [[S0298 00a4b480-00a53380|S0298]] (12), [[S0302 00a62280-00a67470|S0302]] (1), [[S0323 00ad3270-00ad83b0|S0323]] (1), [[S0328|S0328]] (5), [[S0329|S0329]] (1), [[S0344 00b44fa0-00b4b360|S0344]] (3), [[S0345 00b4b3e0-00b50190|S0345]] (28), [[S0346 00b50330-00b55fc0|S0346]] (1), [[S0347 00b56090-00b5b410|S0347]] (1), [[S0349 00b5ffe0-00b66a80|S0349]] (1), [[S0350 00b66b80-00b6b400|S0350]] (2), [[S0372 00bd37e0-00bd5d40|S0372]] (28), [[S0378 00becaa0-00bf1530|S0378]] (3), [[S0384 00c0dcd0-00c171d0|S0384]] (3), [[S0385 00c17390-00c1f0a0|S0385]] (1), [[S0399 00c68550-00c6de90|S0399]] (2), [[S0416 00cc6c40-00ccbaa0|S0416]] (4), [[S0420 00cdb370-00ce0140|S0420]] (1), [[S0428 00d07500-00d0bf70|S0428]] (9), [[S0437 00d297c0-00d2f870|S0437]] (4), [[S0440|S0440]] (2), [[S0466|S0466]] (149), [[S0540|S0540]] (20), [[S0544|S0544]] (78), [[S0558|S0558]] (50), [[S0559|S0559]] (161), [[S0587|S0587]] (15), [[S0589|S0589]] (1), [[S0590|S0590]] (28), [[S0591|S0591]] (38), [[S0592|S0592]] (10), [[S0594|S0594]] (1), [[S0602|S0602]] (25), [[S0654 012393a0-0123f010|S0654]] (21), [[S0655|S0655]] (3), [[S0656|S0656]] (3), [[S0658 0124dfe0-01253ee0|S0658]] (138), [[S0665 01271690-01277340|S0665]] (43), [[S0666 012773d0-0127d160|S0666]] (14), [[S0667 0127d1b0-01281de0|S0667]] (127), [[S0668|S0668]] (38), [[S0670 0128c350-01290f70|S0670]] (65), [[S0687|S0687]] (8), [[S0690|S0690]] (12), [[S0698|S0698]] (7), [[S0705|S0705]] (1), [[S0804 0174b4a0-0174f4f0|S0804]] (74)
