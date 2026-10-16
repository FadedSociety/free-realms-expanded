---
type: build-guide
section: Client Engine
updated: 2026-10-12
---

# Scaleform and Flash UI

Part of [[Build Guides]] › Client Engine. Related: [[UI Framework (GFC and XML)]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

596 accepted functions: 142 confirmed, 286 likely, 138 speculative, 30 unknown. Network-linked (`wire`): 1.

Subsystems: `ui.scaleform` (284), `lib.scaleform` (276), `ui.browser` (36)

### `lib.scaleform`

**GTabCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00661037` | GTabCtrl::AccumulateTotalSlot65 | likely | fastcall, this in ECX. Starts from 0040eb1d(). When this+0xfc is non-null and its vtbl+0x3c() returns this: adds the held child's value (004c6dcb(&this,0), 004… |
| `00660f32` | — | unknown | thiscall(this = tab control state, param_2 = property sink). Returns 0 when 00418ce1(sink) returns false. Otherwise: TabFlags (value string GFC_CF_TAB_LEFT, va… |

**GTableAccess** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `006634e4` | GTableAccess::InitCellRecordByType | likely | thiscall(this, param_2, param_3 type code, param_4, param_5 out record). Returns 0 when param_5 is null or param_3 is neither 0 nor 2. For type 0 sets *param_5… |
| `00663588` | GTableAccess::ForwardKeyedToSlot110 | speculative | Reads a held object from this+0x24 through its vtbl+0x18 with DAT_01bd9f14 (a 16-bit key). When that object is non-null and its vtbl+0x110(param_2, this, param… |
| `006635d3` | GTableAccess::KeyedForwardToSlot10C | speculative | Zeroes the two dwords at param_1 (the output), reads a held object from this+0x24 through its vtbl+0x18 with DAT_01bd9f14, and when non-null calls its vtbl+0x1… |
| `0066347d` | — | unknown | Stack argument code (param_2) mapped to a dword: 4 gives 0x14 via a keyed read (DAT_01bdeaac through vtbl+0x18 of the sub-object at +0x24, written into the loc… |

**GTableCtrl** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00663b50` | GTableCtrl::WriteTableProperties | confirmed | thiscall(this, param_2 property sink). Returns 0 when 00418ce1(sink) is false. Otherwise writes TableFlags (value string GFC_CF_TABLE_NOBORDER, this+0xe8 and +… |
| `00663ed9` | GTableCtrl::AccumulateTotalSlot65 | likely | fastcall, this in ECX. total = 0040eb1d(); when this+0x1c4 is non-null adds its vtbl+0x3c(0,0). Loop 1: for each index below 0055244a() calls 00552460(index) a… |

**Scaleform::IScaleformActionHandler** (171)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0123bc70` | Scaleform::IScaleformActionHandler::BaseDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first word of the object and returns. |
| `0123bc80` | Scaleform::IScaleformActionHandler::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object and frees it through _free when bit 0 of the flag argument is set; returns the object pointe… |
| `0123f5b0` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123f700` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123f9a0` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123faf0` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123fc40` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0127d2b0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns (void, 67 bytes with trailing padding). No calls. |
| `0127d300` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Identical body to 0127d2b0: stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. No calls. |
| `0127d450` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. Same body as 0127d2b0 and 0127d300. No calls. |
| `0127d5a0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. Same body as 0127d2b0. No calls. |
| `0127d6f0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. Same body as 0127d2b0. No calls. |
| `0127d990` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127dae0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127dc30` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127dd80` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127ded0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e170` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e2c0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e410` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `01281830` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `012819d0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | confirmed | Base constructor, 67 bytes: stores Scaleform::IScaleformActionHandler::vftable into *param_1 and returns. No calls. |
| `0123bca0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first word of the object; the body is only this store. |
| `0123d1b0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d200` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d350` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d4a0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d5f0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d740` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d890` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123d9e0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123db40` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123dc90` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123ddf0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123df50` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e0a0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e200` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0123e350` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into the object inside an EH frame and returns; inlined base-constructor copy. |
| `0127d840` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | likely | Base constructor, EH-framed (push -1, handler, fs:[0] link): stores Scaleform::IScaleformActionHandler::vftable into [ecx], restores the frame and returns. No … |
| `0127e020` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | likely | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e560` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | likely | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e6b0` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | likely | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0127e800` | Scaleform::IScaleformActionHandler::IScaleformActionHandler | likely | Base constructor, EH-framed: stores Scaleform::IScaleformActionHandler::vftable into [ecx] and returns. No calls. |
| `0174b4a0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd07c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd07c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b4f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd0e4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd0e4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b540` | Scaleform::IScaleformActionHandler::StaticInit_01cbd09c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd09c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b590` | Scaleform::IScaleformActionHandler::StaticInit_01cbd034 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd034, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b5e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd060 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd060, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b630` | Scaleform::IScaleformActionHandler::StaticInit_01cbd03c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd03c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b680` | Scaleform::IScaleformActionHandler::StaticInit_01cbd05c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd05c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b6d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd06c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd06c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b720` | Scaleform::IScaleformActionHandler::StaticInit_01cbd038 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd038, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b770` | Scaleform::IScaleformActionHandler::StaticInit_01cbd030 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd030, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b7c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd050 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd050, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b810` | Scaleform::IScaleformActionHandler::StaticInit_01cbd0c4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd0c4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b8b0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd098 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd098, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b900` | Scaleform::IScaleformActionHandler::StaticInit_01cbd0cc | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd0cc, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b950` | Scaleform::IScaleformActionHandler::StaticInit_01cbcff8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbcff8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174b9f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd0c0 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd0c0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ba40` | Scaleform::IScaleformActionHandler::StaticInit_01cbd04c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd04c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c070` | Scaleform::IScaleformActionHandler::StaticInit_01cbd1dc | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd1dc, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c0c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd150 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd150, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c110` | Scaleform::IScaleformActionHandler::StaticInit_01cbd1c8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd1c8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c160` | Scaleform::IScaleformActionHandler::StaticInit_01cbd140 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd140, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c1b0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd168 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd168, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c200` | Scaleform::IScaleformActionHandler::StaticInit_01cbd19c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd19c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c250` | Scaleform::IScaleformActionHandler::StaticInit_01cbd194 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd194, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c2a0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd14c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd14c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c2f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd190 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd190, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c340` | Scaleform::IScaleformActionHandler::StaticInit_01cbd154 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd154, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c390` | Scaleform::IScaleformActionHandler::StaticInit_01cbd138 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd138, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c3e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd1a4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd1a4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c430` | Scaleform::IScaleformActionHandler::StaticInit_01cbd130 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd130, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c480` | Scaleform::IScaleformActionHandler::StaticInit_01cbd164 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd164, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c4d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd178 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd178, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c520` | Scaleform::IScaleformActionHandler::StaticInit_01cbd1e0 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd1e0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c570` | Scaleform::IScaleformActionHandler::StaticInit_01cbd230 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd230, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c5c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd204 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd204, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c610` | Scaleform::IScaleformActionHandler::StaticInit_01cbd210 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd210, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c660` | Scaleform::IScaleformActionHandler::StaticInit_01cbd234 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd234, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c6b0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd200 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd200, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c700` | Scaleform::IScaleformActionHandler::StaticInit_01cbd208 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd208, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c7a0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd254 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd254, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c7f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd250 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd250, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c840` | Scaleform::IScaleformActionHandler::StaticInit_01cbd238 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd238, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174c890` | Scaleform::IScaleformActionHandler::StaticInit_01cbd20c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd20c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ca20` | Scaleform::IScaleformActionHandler::StaticInit_01cbd288 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd288, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ca70` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2ac | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2ac, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cac0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd29c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd29c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cb10` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2a4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2a4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cb60` | Scaleform::IScaleformActionHandler::StaticInit_01cbd298 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd298, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cbb0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd280 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd280, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cc00` | Scaleform::IScaleformActionHandler::StaticInit_01cbd270 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd270, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cc50` | Scaleform::IScaleformActionHandler::StaticInit_01cbd300 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd300, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cca0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2a8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2a8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ccf0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2ec | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2ec, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cd40` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2c0 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2c0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cd90` | Scaleform::IScaleformActionHandler::StaticInit_01cbd294 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd294, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cde0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd284 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd284, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ce30` | Scaleform::IScaleformActionHandler::StaticInit_01cbd274 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd274, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ce80` | Scaleform::IScaleformActionHandler::StaticInit_01cbd290 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd290, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ced0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2a0 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2a0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cf20` | Scaleform::IScaleformActionHandler::StaticInit_01cbd278 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd278, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174cf70` | Scaleform::IScaleformActionHandler::StaticInit_01cbd2e4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd2e4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d1f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd390 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd390, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d240` | Scaleform::IScaleformActionHandler::StaticInit_01cbd308 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd308, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d290` | Scaleform::IScaleformActionHandler::StaticInit_01cbd318 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd318, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d2e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd340 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd340, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d330` | Scaleform::IScaleformActionHandler::StaticInit_01cbd368 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd368, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d380` | Scaleform::IScaleformActionHandler::StaticInit_01cbd314 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd314, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d3d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd394 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd394, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d600` | Scaleform::IScaleformActionHandler::StaticInit_01cbd35c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd35c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d880` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3c4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3c4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d8d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3a8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3a8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d920` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3ac | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3ac, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174dec0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd4a8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd4a8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174df10` | Scaleform::IScaleformActionHandler::StaticInit_01cbd45c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd45c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174df60` | Scaleform::IScaleformActionHandler::StaticInit_01cbd418 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd418, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174dfb0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd478 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd478, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e050` | Scaleform::IScaleformActionHandler::StaticInit_01cbd494 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd494, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e3c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd410 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd410, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e4b0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd4e8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd4e8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e550` | Scaleform::IScaleformActionHandler::StaticInit_01cbd50c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd50c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e7d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd520 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd520, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e820` | Scaleform::IScaleformActionHandler::StaticInit_01cbd51c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd51c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ed20` | Scaleform::IScaleformActionHandler::StaticInit_01cbd59c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd59c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174edc0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5b0 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5b0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ee10` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5d8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5d8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ee60` | Scaleform::IScaleformActionHandler::StaticInit_01cbd58c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd58c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174eeb0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd580 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd580, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ef00` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5dc | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5dc, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174ef50` | Scaleform::IScaleformActionHandler::StaticInit_01cbd590 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd590, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174efa0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5ac | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5ac, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174eff0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5e8 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5e8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f040` | Scaleform::IScaleformActionHandler::StaticInit_01cbd5a4 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd5a4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f0e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd658 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd658, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f130` | Scaleform::IScaleformActionHandler::StaticInit_01cbd668 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd668, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f180` | Scaleform::IScaleformActionHandler::StaticInit_01cbd65c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd65c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f1d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd66c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd66c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f220` | Scaleform::IScaleformActionHandler::StaticInit_01cbd63c | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd63c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174f270` | Scaleform::IScaleformActionHandler::StaticInit_01cbd660 | likely | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd660, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d470` | Scaleform::IScaleformActionHandler::StaticInit_01cbd330 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd330, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d4c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd388 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd388, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d510` | Scaleform::IScaleformActionHandler::StaticInit_01cbd32c | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd32c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d560` | Scaleform::IScaleformActionHandler::StaticInit_01cbd358 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd358, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d5b0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd344 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd344, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d650` | Scaleform::IScaleformActionHandler::StaticInit_01cbd31c | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd31c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d6a0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd30c | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd30c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d6f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3ec | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3ec, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d740` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3e8 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3e8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d790` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3a4 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3a4, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d7e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3c0 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3c0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d830` | Scaleform::IScaleformActionHandler::StaticInit_01cbd400 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd400, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d970` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3a0 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3a0, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174d9c0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd3c8 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd3c8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174de70` | Scaleform::IScaleformActionHandler::StaticInit_01cbd484 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd484, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e000` | Scaleform::IScaleformActionHandler::StaticInit_01cbd450 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd450, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e0a0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd464 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd464, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e0f0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd438 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd438, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e140` | Scaleform::IScaleformActionHandler::StaticInit_01cbd424 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd424, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e190` | Scaleform::IScaleformActionHandler::StaticInit_01cbd454 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd454, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e1e0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd4b8 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd4b8, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e230` | Scaleform::IScaleformActionHandler::StaticInit_01cbd458 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd458, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e280` | Scaleform::IScaleformActionHandler::StaticInit_01cbd41c | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd41c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e2d0` | Scaleform::IScaleformActionHandler::StaticInit_01cbd47c | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd47c, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e320` | Scaleform::IScaleformActionHandler::StaticInit_01cbd444 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd444, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0174e370` | Scaleform::IScaleformActionHandler::StaticInit_01cbd420 | speculative | Stores the Scaleform::IScaleformActionHandler vftable into the dword at 01cbd420, the vptr of a file-scope object; MSVC SEH frame with security cookie, no call… |
| `0123f460` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123f850` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123fd90` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |
| `0123fee0` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable into the object at ECX, its only field write, and returns this (67-byte base-part constructor). |

**(no class)** (97)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0066073c` | SplitPanelWriteProperties | confirmed | thiscall(this = split-panel state, param_2 = property sink). Returns 0 when 00418ce1(param_2) returns false. Otherwise writes named properties through the sink… |
| `0066250b` | TabItemWriteProperties | confirmed | thiscall(this = tab item state, param_2 = property sink). Returns 0 when 00653cea(sink) is false. Writes ItemFlags (value string GFC_TAB_ITEM_DISABLED, this+0x… |
| `00662727` | TabItemReadProperties | confirmed | thiscall(this = tab item state, param_2 = property source). Returns 0 when 00653c22(source) is false. Reads ItemFlags (value string GFC_TAB_ITEM_DISABLED) thro… |
| `00662b38` | TabItemListWriteProperties | confirmed | thiscall(this, param_2 property sink). Returns 0 when 004c6e70(sink) is false or when (this+0x14)->vtbl+0x30() object's vtbl+0x28(sink) is false. Otherwise obt… |
| `006637b2` | HeaderItemReadProperties | confirmed | thiscall(this, param_2 item key, param_3 out header record). Obtains an object for type GHeaderItem through this vtbl+0x68; returns 0 when null. Reads HeaderCe… |
| `00663980` | HeaderItemWriteProperties | confirmed | thiscall(this = sink-owner, param_2 item key, param_3 header record of 16-bit and dword fields). Obtains an object for type GHeaderItem from this vtbl+0x6c(par… |
| `0065f8e5` | FormatScaledValue | likely | thiscall with a stack argument param_2 (output object) and param_3 (input value). Returns 0 when param_2 is null. Otherwise: if *(this+4) is null, computes sca… |
| `0065f9b1` | CopyCtorIfaceHolder | likely | Copy constructor: calls base ctor 0065f68b(source), stores vtable pointers PTR_FUN_0177ef70 at +0, PTR_LAB_0177ef6c at +4 and PTR_LAB_0177ef38 at +0x2c, copies… |
| `0065f9fb` | StringAccessIfAccepted | likely | Returns 0 when 006523c1(param_2) is zero. Otherwise, when this->+0x30 is non-null, calls 004e889c(L"StringAccess", -1), then 004d86f8(param_2, key, this) and 0… |
| `0065faa7` | DtorBodyIface30 | likely | Destructor body: stores vptrs PTR_FUN_0177ef70, PTR_LAB_0177ef6c and PTR_LAB_0177ef38; when the held pointer at +0x30 is non-null calls its vtbl+0x5c(&record) … |
| `0065fb42` | CopyCtorIface34 | likely | Copy constructor: calls base ctor 0065f68b(source), stores vptrs PTR_FUN_017b9d38, PTR_LAB_017b9d34, PTR_LAB_017b9d00, copies the 16-bit field source+0x30, cop… |
| `0065fb98` | ImageAccessIfAccepted | likely | Returns 0 when 006523c1(param_2) is zero. Otherwise, when this->+0x34 is non-null, calls 004e889c(L"ImageAccess", -1), then 004d86f8(param_2, key, this) and 00… |
| `0065fc44` | DtorBodyIface34 | likely | Destructor body: stores vptrs PTR_FUN_017b9d38, PTR_LAB_017b9d34 and PTR_LAB_017b9d00; when the held pointer at +0x34 is non-null calls its vtbl+0x5c(&record) … |
| `0065fce4` | SetHeldIfaceNotify | likely | thiscall(this, param_2 interface). When this->+0x34 differs from param_2: builds a temporary record (vptr PTR_FUN_0175d184) via 004c9ebe; calls vtbl+0x5c on th… |
| `0065fec4` | CopyCtorIface2C | likely | Copy constructor: calls base ctor 0065f68b(source), stores vptrs PTR_FUN_017b9e88 and PTR_LAB_017b9e84, calls vtbl+0xc on the held pointer at source+0x2c when … |
| `0065fef8` | DtorBodyIface2C | likely | Destructor body: stores vptrs PTR_FUN_017b9e88 and PTR_LAB_017b9e84; when the held pointer at +0x2c is non-null calls its vtbl+0x10(0); then tail-jumps to base… |
| `0065ff4d` | CopyCtorDerivedVptrs | likely | Copy constructor of a derived object: calls base copy ctor 0065fec4(source) with this as receiver, then stores the vptr sub-object pairs at +0x30/+0x34 (PTR_LA… |
| `00660027` | DefaultCtorDerivedVptrs | likely | Default constructor: calls 0065d5b4() for the base, zeroes +0x2c, stores sub-vptr pairs at +0x30/+0x34 and the primary vptr PTR_FUN_017ba0e8 with +4 PTR_LAB_01… |
| `00660060` | CtorDerivedWithIfaceArg | likely | thiscall constructor with arguments (param_2, param_3, param_4) and param_5 (interface pointer). Calls 00560709(param_2,param_3,param_4) first (role not verifi… |
| `0066015e` | DtorDerivedVptrs | likely | Destructor body: stores vptrs PTR_FUN_017ba0e8, PTR_LAB_017ba0e4, PTR_LAB_017ba0b0 and PTR_LAB_017ba07c; when the held pointer at +0x2c is non-null calls 00609… |
| `00660222` | SetHeldIfaceKeyed | likely | thiscall(this, param_2 interface). When this->+0x2c differs from param_2: calls param_2->vtbl+8(004cd10a()) as a keyed support check and proceeds only when it … |
| `006603b0` | LifecycleOpDispatchA | likely | Stack-argument operation dispatcher on op = param_1 with param_2 (object) and param_3 (value). op 0: when param_2 is non-null calls 00550aa8(); op 1: when para… |
| `00660449` | LifecycleOpDispatchB | likely | Same operation dispatcher as 006603b0 over a second helper set: op 0 calls 00560a20() when param_2 non-null; op 1 calls 00560a7d(param_3) when param_2 non-null… |
| `006604ef` | LifecycleOpDispatchC | likely | Same operation dispatcher as 006603b0 over a third helper set: op 0 calls 00550bd0() when param_2 non-null; op 1 calls 00550c25(param_3) when param_2 non-null;… |
| `00660588` | LifecycleOpDispatchF | likely | Operation dispatcher on op = param_1 with param_2 and param_3, same structure as LifecycleOpDispatchA (006603b0) with a 0x14-byte allocation via 00653120: op 0… |
| `0066063e` | DtorBodyPrimaryVptr017ba358 | likely | fastcall destructor body: stores vptrs PTR_FUN_017ba358, PTR_LAB_017ba354 and PTR_LAB_017ba240 (at +0x24); when the held pointer at +0x118 is non-null calls it… |
| `00660dac` | SetHeldChild118 | likely | thiscall(this, param_2 new object). When this+0x118 differs from param_2: calls 005378cb(this+0x110, this+0x108, 0), notifies the old held object via vtbl+0x5c… |
| `00661107` | SetHeldChild0xfc | likely | thiscall(this, param_2 new object). When this+0xfc differs from param_2: notifies the old held object via vtbl+0x5c, calls param_2->vtbl+0xc when non-null, rel… |
| `00662003` | LifecycleOpDispatchSize10 | likely | Stack-argument operation dispatcher on op = param_1 with param_2 (object) and param_3. op 0: when param_2 is non-null sets its flag byte at +4 to 0 and its vpt… |
| `00662096` | LifecycleOpDispatchSize18 | likely | Stack-argument operation dispatcher like LifecycleOpDispatchSize10 but for a 0x18-byte object: op 0 sets flag byte +4 = 0, vptr PTR_LAB_017b8b40, dwords +0x10 … |
| `00662a62` | LifecycleOpDispatchD | likely | Operation dispatcher on op = param_1 with param_2 and param_3, same structure as LifecycleOpDispatchA (006603b0): op 0 calls 005510e2() for non-null param_2; o… |
| `006633f2` | LifecycleOpDispatchE | likely | Operation dispatcher on op = param_1 with param_2 and param_3, same structure as LifecycleOpDispatchA (006603b0): op 0 calls 00551561() for non-null param_2; o… |
| `0066361d` | LifecycleOpDispatchSizeF0 | likely | Operation dispatcher on op = param_1 with param_2 and param_3. op 0 calls 0054dca8(0,0) for non-null param_2; op 1 calls 0064e636(param_3) for non-null param_2… |
| `006636b4` | LifecycleOpDispatchSize138 | likely | Operation dispatcher on op = param_1 with param_2 and param_3, same structure as LifecycleOpDispatchSizeF0 but allocating 0x138 bytes via 0065462f: op 0 calls … |
| `0065fa41` | HeldChildTotalAt30 | speculative | fastcall, this in ECX. Adds 004c6e81() (a body that returns 0) to a total. When this->+0x30 is non-null and its vtbl+0x3c() returns this: calls 004c6dcb(&this,… |
| `0065fbde` | HeldChildTotalAt34 | speculative | fastcall, this in ECX. Adds 004c6e81() (a body that returns 0) to a total. When this->+0x34 is non-null and its vtbl+0x3c() returns this: calls 004c6dcb(&this,… |
| `0065fdd5` | SetShortFieldNotify | speculative | thiscall with this in ESI (ECX) and stack argument param_2 (16-bit). If this->+4 differs: stores the short at +4; if this->+8 is null, builds a temporary recor… |
| `006604d4` | ForwardEcxArg3ReturnFirst | speculative | Forwarder: pushes param_4, sets ECX = param_3, pushes param_2 and param_1, calls 0065f711, then returns param_1 (ret 0x14). Decoded from bytes 55 8b ec ff 75 1… |
| `0066097d` | HeldChildTotalAt118 | speculative | fastcall, this in ECX. total = 0040eb1d(this). When this->+0x118 is non-null and its vtbl+0x3c() returns this: calls 004c6dcb(&this, 0), takes the result of 00… |
| `00660bef` | ReadIntKeyedDefault10 | speculative | fastcall, this in ECX. Sets a local int to 10, selects a 16-bit property id (DAT_01bde714 when this->+0xf0 bit 0 is set, else DAT_01bdc07c; both are undefined2… |
| `00660c30` | OverflowCountAfterReadInt | speculative | fastcall, this in ECX. Chooses a pair of ints (+0x9c/+0x94, or +0xa0/+0x98 when this+0xf0 bit 0 is set). Range count uVar1 = 0 when the first is below the seco… |
| `00660cc8` | ReadPairKeyed | speculative | thiscall(this, param_2 out pair). Zeroes param_2[0] and param_2[1], selects a 16-bit property id (DAT_01bda754 when this+0xf0 bit 0 is set, else DAT_01bda860),… |
| `00660d0e` | ReadIntMinusPair | speculative | No stack arguments; this is passed in ECX. Calls 00660bef() (ECX = this) to get an int, then 00660cc8 with a local two-dword buffer, and returns the int minus … |
| `00660d37` | GetRangePairOrHeldChild | speculative | thiscall(this, param_2 out pair). When this->+0x118 is non-null: calls its vtbl+0x7c(param_2) and returns 1. Otherwise copies this+0x108 and this+0x10c into pa… |
| `00660d6b` | RangePairEqualsOrHeldCompare | speculative | thiscall(this, param_2 pair pointer). Copies the pair to locals; when this+0x108 and this+0x10c equal the two dwords, returns 1 (bool in AL). Otherwise returns… |
| `00660eca` | HeldVtbl70Accepts | speculative | thiscall(this, one stack argument). Returns 1 when this+0x118 is non-null and the held object's vtbl+0x70(arg) returns nonzero; otherwise returns 0 (decoded fr… |
| `00660eea` | ForwardArgToModeSet0 | speculative | thiscall-style wrapper with one stack argument: pushes 0 and the argument and calls 00660ac2 with the incoming ECX unchanged, so 00660ac2 gets (this, arg, 0); … |
| `00661201` | GetHeldChild0xfc | speculative | fastcall, this in ECX. Returns the dword at this+0xfc (7 bytes: mov eax,[ecx+0xfc]; ret). No reference count change. |
| `00661208` | ReadSevenKeyedIntsIntoRecord | speculative | thiscall(param_1 = output record of nine dwords, param_2 = array of seven pointers to 16-bit ids, param_3 = property sink). Initialises the record (dword0 = 0x… |
| `006613e0` | ReadStateKeyedRecord | speculative | thiscall(param_1 = output record, param_2 = owner object). Derives an index iVar5 (0..3) and a flag uVar4 from owner+0xf0 (bits 0x30 and 0x3). Then calls vtbl+… |
| `0066151f` | CopyFifteenDwordRecord | speculative | thiscall(param_1 = destination, param_2 = source). Copies fifteen dwords (60 bytes, indices 0..0xe) from source to destination in a fixed order and returns not… |
| `00661586` | ComputeOutRectFromModeKeys | speculative | thiscall(this, param_2 out four-dword record). Computes iVar5 = (+0xa0 - +0x98) + 1 (0 when +0xa0 < +0x98) and iVar4 = (+0x9c - +0x94) + 1 (0 when smaller), ea… |
| `0066174a` | SpanRecordWithFlag | speculative | thiscall(this, param_2 out four-dword record, param_3 char). Local value = this+0x118 when param_3 is nonzero, else 0 (Ghidra's name _param_3; used as an int i… |
| `00661810` | ForwardHeldChildSlot68 | speculative | fastcall, this in ECX. When this+0xfc is non-null, tail-jumps (jmp [eax+0x68]) into the held child's vtbl+0x68 with ECX = child; otherwise returns 0. Decoded f… |
| `00661822` | GetHeldChildRef | speculative | thiscall(this, param_2 out pointer, param_3 int pointer). When this+0xfc is null: passes null in param_3 and uses it as the result. Otherwise calls the held ch… |
| `0066192d` | ForwardHeldSlot7C4Args | speculative | thiscall with four stack arguments (ret 0x10). Returns 0 when this+0xfc is null; otherwise calls the held child's vtbl+0x7c(p2,p3,p4,p5) and returns its result… |
| `00661956` | ForwardHeldSlot78_5Args | speculative | thiscall with five stack arguments (ret 0x14). Returns 0 when this+0xfc is null; otherwise calls the held child's vtbl+0x78(p2..p6) and returns its result as a… |
| `00661982` | ForwardHeldSlot74_5Args | speculative | thiscall with five stack arguments (ret 0x14). Returns 0 when this+0xfc is null; otherwise calls the held child's vtbl+0x74(p2..p6) and returns its result as a… |
| `006619ae` | ForwardHeldSlot90 | speculative | thiscall with one stack argument (ret 4). Returns 0 when this+0xfc is null; otherwise calls the held child's vtbl+0x90(p2) and returns its result as a byte (de… |
| `00661a07` | ChildKeyedInvoke6C | speculative | Takes one stack argument (param_2) and a this in ECX. Obtains a retained child through 00661822 into a local, calls 00653acc on a local, gets key = 004cd10a(),… |
| `00661b00` | ChildInvokeSlot88 | speculative | Stack argument param_1 is the query input and the result slot: calls 00661822(&param_1, param_1) to obtain a retained object (this+0xfc query, ECX unchanged). … |
| `00661b38` | ChildInvokeSlot8C | speculative | Same shape as 00661b00: obtains a retained object through 00661822(&param_1, param_1); when non-null calls its vtbl+0x8c(param_2), releases it via vtbl+0x10(0)… |
| `00661b70` | ChildSlot68Result | speculative | Takes one stack argument. Obtains a retained object through 00661822(&param_1, param_1); when null returns 0, otherwise returns the result of its vtbl+0x68() a… |
| `00661baa` | ChildFlagsSetMasked | speculative | Obtains a retained object through 00661822(&param_1, param_1). When non-null: reads its value through vtbl+0x68(), writes (value & ~param_3) \| param_2 through… |
| `00661bee` | ChildInvokeSlot80 | speculative | Same shape as 00661b00: obtains a retained object through 00661822(&param_1, param_1); when non-null calls its vtbl+0x80(param_2), releases it via vtbl+0x10(0)… |
| `00661c26` | ChildInvokeSlot84 | speculative | Same shape as 00661b00: obtains a retained object through 00661822(&param_1, param_1); when non-null calls its vtbl+0x84(param_2), releases it via vtbl+0x10(0)… |
| `00661c5e` | ChildFlagBit1Clear | speculative | Takes one stack argument and passes it through ECX-inherited 00661b70 (ChildSlot68Result). Returns (~(v >> 1)) & 1 for the returned value v: 1 when bit 1 of v … |
| `00661c71` | ChildFlagBit2Store | speculative | Obtains a retained object through 00661822(&param_1, param_1) (param_1 stack argument, param_2 char). Returns 0 when null. Otherwise reads v = vtbl+0x68(); set… |
| `00661cd1` | ChildFlagBit0Store | speculative | Obtains a retained object through 00661822(&param_1, param_1) (param_1 stack argument, param_2 char). Returns 0 when null. Otherwise reads v = vtbl+0x68(); whe… |
| `00661d1c` | ChildFlagsDecode | speculative | thiscall with one stack argument. v = 00661b70(arg) (the child flags dword, 0 when no child). Returns v & 0xffffff00 when bit 1 of v is set. Otherwise returns … |
| `00661d4d` | FindChildIndexByKey | speculative | Takes a pointer to a pointer (param_1). Count = 00661810() (held child's vtbl+0x68 result). For i from 0 to count-1: sets a local slot to &DAT_01bc7e20, calls … |
| `00662136` | ForwardIfSlotNonZero8C | speculative | thiscall(this = pointer p, stack param_2 object, stack param_3). Returns 0 when *p is zero; otherwise returns (byte) the result of param_2->vtbl+0x8c(param_3, … |
| `00662165` | ProbeThenKeyedRead | speculative | thiscall(this = param_1 object, param_2 owner). Calls this->vtbl+0x1c(); when that returns 0 calls this->vtbl+0x24(&local) and, when the low 16 bits of local a… |
| `006621aa` | SetSubobjectRef0cNotify | speculative | thiscall(this, param_2 object). When this+0xc differs from param_2: calls param_2->vtbl+0xc when non-null, releases the old object at +0xc via vtbl+0x10(0), st… |
| `00662206` | SetSubobjectRef18Notify | speculative | thiscall(this, param_2 object). When this+0x18 differs from param_2: calls param_2->vtbl+0xc when non-null, releases the old object at +0x18 via vtbl+0x10(0), … |
| `0066228b` | AllocChild24ByteOutput | speculative | thiscall(this = state, param_2 out pointer). When state+0x40 is zero and (signed state+0x38 < 0 or byte state+0x3c is zero) writes 0 to *param_2 and returns. O… |
| `00662368` | BuildPropertyInterfaceAndInvoke114 | speculative | thiscall(this, param_2 result, param_3 property sink, plus four more stack arguments). Reads a 16-bit id DAT_01bde870 from the sink's sub-object at +0x24 into … |
| `006629dd` | TotalWithHeldChildAt48 | speculative | fastcall, this in ECX. total = 00653dee(this) + 004ca721(). When this+0x48 is non-null and its vtbl+8(004cd10a()) returns an object, adds that object's vtbl+0x… |
| `00662c3b` | ListElementTotal | speculative | fastcall, this in ECX. total = (this+0x14)->vtbl+0x30()->vtbl+0x30() plus 004c6e81() (returns 0). For each index below 00655e4a() adds (00655e5d(index))->vtbl+… |
| `0066374d` | SetFiveFieldsThenNotify27C | speculative | thiscall(this, five stack arguments and a pair). Stores param_2 at this+0xe8, param_7 at this+0xec, param_8 at this+0xf0, param_3 at this+0x12c, param_4 at thi… |
| `00663f7a` | ReadWordKeyedDefault | speculative | thiscall, one stack argument (out pointer). Writes DAT_01bd3070 as a 16-bit value to *out, then calls the sub-object at this+0x24 through vtbl+0x18 with (DAT_0… |
| `00663fa0` | GetPair13C_138 | speculative | thiscall, one stack argument (out pair). out[0] = dword this+0x13c and out[1] = dword this+0x138 (decoded from bytes, ret 4). |
| `00663fb8` | GetPair114_110 | speculative | thiscall, one stack argument (out pair). out[0] = dword this+0x114 and out[1] = dword this+0x110 (decoded from bytes, ret 4). |
| `00663fd0` | GetPair164 | speculative | thiscall, one stack argument (out pair). out[0] = dword this+0x164 and out[1] = dword this+0x168 (decoded from bytes, ret 4). |
| `00664170` | GetHeldAt1C4 | speculative | fastcall, this in ECX. Returns the dword at this+0x1c4 (7 bytes: mov eax,[ecx+0x1c4]; ret). |
| `00664219` | SelectPairSlotOrCount | speculative | thiscall(this, param_2 selector). When param_2 is zero and this+0xf4 bit 0x1000000 is clear and this+0x1c4 is non-null, returns this+0x118. When param_2 is non… |
| `00664271` | GetPair118 | speculative | thiscall, one stack argument (out pair). out[0] = dword this+0x118 and out[1] = dword this+0x11c (decoded from bytes, ret 4). |
| `00664323` | ResolveTwoIndexedSlots | speculative | thiscall(this, param_2 out1, param_3 out2, param_4 index1, param_5 index2). For non-null param_2: when this+0xf4 bit 0x1000000 is clear, requires index1 < this… |
| `006643d0` | IndexPairInBounds | speculative | thiscall(this, param_2 index x, param_3 index y). When this+0xf0 has neither 0x300000 bit: returns 0 when either index is negative, x >= this+0x118, y >= this+… |
| `00664515` | CreateKeyedChildObject | speculative | thiscall(this, param_2 value, and stack arguments 3..7). Stores param_2 at this+0x184; reads a keyed value into param_2 through this+0x24 vtbl+0x18 with DAT_01… |
| `01539d8a` | DelayLoad_TCSetExternalInterface | speculative | Delay-load import stub (10 bytes): mov eax to import slot 0x1bc23c8 and jumps to delay-load helper 010c7056. |
| `01539d9a` | DelayLoad_TCMovieSetExternalInterfaceStringRetVal | speculative | Delay-load import stub (10 bytes): mov eax to import slot 0x1bc23c4 and jumps to delay-load helper 010c7056. |
| `01539daa` | DelayLoad_TCGetGuiGfxParam | speculative | Delay-load import stub (10 bytes): mov eax to import slot 0x1bc2494 and jumps to delay-load helper 010c7056. |
| `00661a96` | — | unknown | Takes one stack argument (param_2) and a this in ECX. Obtains a retained child through 00661822, calls 00653acc on a local, gets key = 004cd10a(), and calls ch… |
| `00662c93` | — | unknown | thiscall(param_1 = output record of about 0x56 dwords, param_2 = property sink, param_3 and param_4 = arrays of pointers to 16-bit ids). Writes default record … |
| `00663fe7` | — | unknown | fastcall, this in ECX. local = held child (this+0x1c4) vtbl+0x11c(this,0,0,0,0) or 0 when null. Clears the bit mask 0xf03fffff of this+0xf4 and rewrites this+0… |
| `00664289` | — | unknown | thiscall(this, param_2 out pair, param_3 wide string object). Returns (-1, -1) unless the first callee check (00406665) matches this and the string at param_3 … |

### `ui.browser`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009947d0` | BaseClient::InitializeBrowserConfiguration | likely | When state +0x14 is zero, queries the online-help browser and window controls, dispatches initialization calls when both exist, stores param_2 at +0x10, assign… |

**EmbeddedWebBrowserLibrary** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00993cf0` | EmbeddedWebBrowserLibrary::OnAssetsLoaded | confirmed | On the success condition, logs successful completion and stores state 2; otherwise clears the success byte, logs the error code, and stores state 2. |
| `00993e30` | EmbeddedWebBrowserLibrary::OnAssetLoaded | confirmed | Logs asset success when result equals 1; otherwise clears the library success byte and logs the result code, asset name, and disk-write flag. |
| `00993e90` | EmbeddedWebBrowserLibraryDtor | confirmed | Installs the browser-library vtable, invokes cleanup helper 00703350, then performs embedded-browser cleanup through 00ae8460. |
| `00993fb0` | EmbeddedWebBrowserLibrary::Ctor | likely | Installs the browser-library vtable, clears the state and callback fields, initializes a large member, and initializes the base logging object. |
| `009940b0` | EmbeddedWebBrowserLibraryScalarDeletingDtor | likely | Calls the embedded-browser library destructor, frees the object when the deleting flag is set, and otherwise returns the object pointer. |

**EmbeddedWebBrowserLibraryHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00994030` | EmbeddedWebBrowserLibraryHandlerDtor | confirmed | Replaces the object's vtable pointer with EmbeddedWebBrowserLibraryHandler::vftable during destruction. |
| `00994040` | EmbeddedWebBrowserLibraryHandlerScalarDeletingDtor | confirmed | Sets the class vtable and frees the object when the low flag bit is set, otherwise returns the object pointer. |

**GuiWebBrowserCtrl::UiHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00994060` | GuiWebBrowserCtrl::UiHandlerDtor | confirmed | Writes GuiWebBrowserCtrl::UiHandler::vftable to the object during teardown. |
| `00994070` | GuiWebBrowserCtrl::UiHandlerScalarDeletingDtor | confirmed | Writes the UI handler vtable and conditionally frees the object according to the deleting flag. |

**OnlineHelpWindow** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00994470` | OnlineHelpWindowDtor | confirmed | Resets OnlineHelpWindow subobject vtables, frees the browser library member, and releases embedded strings and fixed-string members. |
| `009946d0` | OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus4 | confirmed | Subtracts 4 from ECX, then jumps to the scalar deleting destructor wrapper at 009947b0. |
| `009946e0` | OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus8 | confirmed | Subtracts 8 from ECX, then jumps to the scalar deleting destructor wrapper at 009947b0. |
| `009947b0` | OnlineHelpWindow::ScalarDeletingDtor | confirmed | Calls the full OnlineHelpWindow destructor and conditionally frees the object based on the deleting flag. |
| `00995060` | OnlineHelpWindow::HandleUrlCommand 🌐 | confirmed | Parses the current browser command string, branches on frgp://, visitHouse/, and teleportTo/ forms, then dispatches help handling, house entry, or a parsed pos… |
| `00995620` | OnlineHelpWindow::OnNavigationResult | confirmed | For a successful callback, runs the browser-action predicate and stores state 3 before reporting status 1; otherwise reports status 3. |
| `00994980` | — | unknown | Installs browser-handler, singleton, and online-help vtables; initializes embedded strings and state; creates the EmbeddedWebBrowserLibrary member and register… |

**Singleton<OnlineHelpWindow>** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009942e0` | OnlineHelpSingletonDtor | confirmed | Stores Singleton<OnlineHelpWindow>::vftable into the object during teardown. |
| `00994310` | OnlineHelpSingletonScalarDeletingDtor | confirmed | Writes the singleton vtable and frees the object when the deleting flag bit is set. |

**(no class)** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009051a0` | WebBrowserIme_SetText | confirmed | Cdecl, param_1 is the owning object. Requests parameter slot 0 through 0070b470 when the Lua parameter array is empty, writes the string at param_1+4 into it v… |
| `00905380` | WebBrowserIme_IsBrowserShown | confirmed | Calls Lua WebBrowserHandler:getIsBrowserShown with no arguments via 009797a0, then sets bl to the result payload byte at result+8 when the result tag is 5, oth… |
| `009057c0` | WebBrowserIme_IsPanelShown | confirmed | Calls Lua WebBrowserHandler:getIsIMEPanelShown with no arguments via 009797a0. Sets bl to the result payload byte at result+8 when the result tag is 5, otherwi… |
| `009940f0` | FindOnlineHelpBrowserControl | confirmed | Looks up Main.wndOnlineHelp.webBrowser, constructs a local query object, calls virtual slot +8 on the resolved browser control, and returns its result or zero … |
| `00994150` | FindOnlineHelpWindow | confirmed | Looks up Main.wndOnlineHelp by name, initializes a local query structure, and invokes virtual slot +8 on the resolved object; returns zero when lookup fails. |
| `009954d0` | HandlerOnlineHelpWindowOnStatus | confirmed | Builds a status callback string when HandlerOnlineHelpWindow exists, appends :OnStatus, converts callback arguments, and invokes the script handler. |
| `00993ce0` | SetField2BA70 | likely | Stores the supplied dword into the object field at offset 0x2ba70 and returns. |
| `00993de0` | ProcessBrowserLoadState | likely | For state 1, invokes helper 00ae6b90; for state 2, snapshots the success byte, resets state to zero, conditionally sets byte +9, and calls the nested object's … |
| `00993f00` | LoadBrowserAssets | likely | Rejects a duplicate load while state is 1, assigns and logs the file list, obtains the GUI manager, starts asynchronous asset loading, and sets state 1 if the … |
| `009943c0` | QueryBrowserControlAndDispatchResult | likely | Calls 009940f0, moves its returned browser-control pointer into ECX, then jumps to 008457d0. |
| `009955f0` | CanDispatchOrReportStatus | likely | If the nested state byte at +9 is set, calls 009943d0 and returns 1 when that helper succeeds; otherwise calls 009954d0 with status 2 and returns 0. |
| `00993dc0` | SetStoppedAndCancelNestedObject | speculative | Clears byte +9, checks the nested pointer at +0x2ba70, and if present calls its virtual slot +8 with argument 1. |
| `00904fe0` | — | unknown | Calls Lua WebBrowserHandler:getIMEText through ClientLuaCallByName (009797a0). If it returns a value, passes *(0070ba10()+4) and param_1 to 00701290, then free… |
| `00905270` | — | unknown | Cdecl, param_1 is a char. Requests parameter slot 0 through 0070b470 when the array is empty, releases the previous tagged value (tags 3/6/7/8 via vtable slot … |
| `00994290` | — | unknown | Queries Main.wndOnlineHelp.webBrowser and, when present, passes the control in ECX to a helper that invokes its virtual slot +0x8c. |
| `009943d0` | — | unknown | Gets the browser control, checks its active state and callback, invokes a short handler for nonzero input, and otherwise dispatches a three-field action struct… |
| `00995660` | — | unknown | Calls 009940f0 with the original this pointer to obtain a browser-control pointer, copies the NUL-terminated string at this+0x8d8 into a local std::string, cal… |

### `ui.scaleform`

**Engine::FRTWDeployFiguresFromFile** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252250` | Engine::FRTWDeployFiguresFromFile::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'DeployFiguresFromFile : 0 : ..\engine\game\frtwgamehandlers.cc:1970'. |
| `01252260` | Engine::FRTWDeployFiguresFromFile::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012521c0` | Engine::FRTWDeployFiguresFromFile::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWDeployFiguresFromFile::vftable into the object; when the int argument is nonz… |
| `01252230` | Engine::FRTWDeployFiguresFromFile::TypeName | likely | Slot [1] of Engine::FRTWDeployFiguresFromFile::vftable. Bytes b8 imm32 c3: returns the constant C string 'DeployFiguresFromFile'. |
| `0125e010` | — | unknown | Reads the string argument named FileName (012eb250), assigns it to a local std::string, and passes that string to 013210e0; returns no value. It does not creat… |

**Engine::FRTWReadyApplyModifier** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012520e0` | Engine::FRTWReadyApplyModifier::TypeName | confirmed | Slot [1] of Engine::FRTWReadyApplyModifier::vftable. Bytes b8 imm32 c3: returns the constant C string 'ReadyApplyModifier'. |
| `01252100` | Engine::FRTWReadyApplyModifier::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'ReadyApplyModifier : 0 : ..\engine\game\frtwgamehandlers.cc:1962'. |
| `01252110` | Engine::FRTWReadyApplyModifier::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01252070` | Engine::FRTWReadyApplyModifier::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWReadyApplyModifier::vftable into the object; when the int argument is nonzero… |

**Engine::FRTWSetAutoPassAI** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012524f0` | Engine::FRTWSetAutoPassAI::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'SetAutoPassAI : 0 : ..\engine\game\frtwgamehandlers.cc:1988'. |
| `01252500` | Engine::FRTWSetAutoPassAI::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01252460` | Engine::FRTWSetAutoPassAI::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWSetAutoPassAI::vftable into the object; when the int argument is nonzero it c… |
| `012524d0` | Engine::FRTWSetAutoPassAI::TypeName | likely | Slot [1] of Engine::FRTWSetAutoPassAI::vftable. Bytes b8 imm32 c3: returns the constant C string 'SetAutoPassAI'. |

**Engine::FRTWUndeployFigures** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012523a0` | Engine::FRTWUndeployFigures::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'UndeployFigures : 0 : ..\engine\game\frtwgamehandlers.cc:1980'. |
| `012523b0` | Engine::FRTWUndeployFigures::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01252310` | Engine::FRTWUndeployFigures::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUndeployFigures::vftable into the object; when the int argument is nonzero it… |
| `01252380` | Engine::FRTWUndeployFigures::TypeName | likely | Slot [1] of Engine::FRTWUndeployFigures::vftable. Bytes b8 imm32 c3: returns the constant C string 'UndeployFigures'. |

**Engine::FRTWUnsetAutoPassAI** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01252640` | Engine::FRTWUnsetAutoPassAI::DebugName | confirmed | Slot [4] of its vftable. Bytes b8 imm32 c3: returns the constant C string 'UnsetAutoPassAI : 0 : ..\engine\game\frtwgamehandlers.cc:1996'. |
| `01252650` | Engine::FRTWUnsetAutoPassAI::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `012525b0` | Engine::FRTWUnsetAutoPassAI::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::FRTWUnsetAutoPassAI::vftable into the object; when the int argument is nonzero it… |
| `01252620` | Engine::FRTWUnsetAutoPassAI::TypeName | likely | Slot [1] of Engine::FRTWUnsetAutoPassAI::vftable. Bytes b8 imm32 c3: returns the constant C string 'UnsetAutoPassAI'. |

**FlashEntity** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56520` | FlashEntity::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FlashEntity::Dtor (00d56270) on this, then calls _free(this) when bit 0 of the flags argument is set; returns this and pops o… |
| `00d55730` | FlashEntity::DestroyViaVtable | speculative | Calls FUN_0075c060 first, then if the argument pointer is non-null invokes its vtable slot 0 with flag 1 (scalar deleting destructor, i.e. delete-by-vtable). |
| `00d55c40` | — | unknown | Resolves the current frame counter: from the field +0x38 of the object at this+4 when set, else from FUN_00729610 when this+8 is set, else zero; reads its +0x4… |
| `00d561c0` | — | unknown | Installs FlashEntity::vftable, stores the argument pointer at this+0x14, registers this in a list through 00d55e00, and when bit 1 of the flags word at this+4 … |
| `00d56270` | — | unknown | Restores FlashEntity::vftable in *param_1, calls ListRemoveValue (00d56140) on this entity's child list, then calls the base teardown FUN_00764e10 with the sav… |

**FlashManager** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56540` | FlashManager::Dtor | confirmed | Sets FlashManager::vftable and the map vtable, walks the tree from its leftmost node deleting each value through vtable slot 0 with flag 1, then destroys tree … |
| `00d56830` | FlashManager::RemoveByKey | likely | Finds the tree node with key param_2 under the root at this+8; when found, deletes the value pointer's object through its vtable slot 0 with flag 1 if non-null… |
| `00d56b60` | FlashManager::GetOrCreateMovie | likely | Computes a key from its name argument with FUN_00705b00, searches the manager tree at this+8, and on a miss allocates and constructs a FlashMovie (FUN_00d56890… |
| `00d55790` | FlashManager::GetInt64At10 | speculative | Returns the 64-bit value stored at this+0x10 (eax low dword, edx high dword) as an undefined8 return. |
| `00d557a0` | FlashManager::GetInt64At18 | speculative | Returns the 64-bit value stored at this+0x18 (eax low dword, edx high dword). |
| `00d557b0` | FlashManager::ZeroPairs10To1C | speculative | Writes zero to the four dwords at this+0x10, +0x14, +0x18 and +0x1c, clearing two 64-bit fields. |
| `00d55cb0` | FlashManager::LookupByKey | speculative | Searches a binary search tree rooted at this+0x8 for the unsigned key argument (node key at +0x4; the +0xc child holds keys below the node key (taken when the … |
| `00d55cf0` | FlashManager::GetField0C | speculative | Returns the 32-bit value at this+0xc (mov eax,[ecx+0xc]; ret). |

**FlashManagerInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `008a5b90` | FlashManagerInterface::ScalarDeletingDestructor | confirmed | Scalar deleting destructor: calls FUN_010b48f0 with this in ECX, then calls _free(this) when param_2 bit 0 is set and returns this. |
| `010b48f0` | FlashManagerInterface::ResetVtable | confirmed | Stores FlashManagerInterface::vftable into the first dword of the object in ECX and returns (no arguments). |

**FlashMovie** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00d56810` | AdjustorThunk_sub14_00d56810 | confirmed | Adjustor thunk: subtracts 0x14 from ecx (this) (bytes 83 e9 14) and jumps (e9) to 00d56b40, the FlashMovie scalar deleting destructor. |
| `00d56b40` | FlashMovie::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls FlashMovie::Dtor (00d56720) on this, then calls _free(this) when bit 0 of the flags argument is set. |
| `00d56720` | FlashMovie::Dtor | likely | Restores FlashMovie::vftable in *param_1 and its secondary subobject, drops three LOCK-decremented shared references at +0x30, +0x2c and +0x28 (releasing each … |
| `00d55700` | FlashMovie::IncrementSharedRefcount | speculative | Loads the pointer at this+0x1c, then atomically increments the 32-bit counter at (pointer+4) with LOCK XADD; returns the pointer in eax and the previous counte… |
| `00d55720` | FlashMovie::SetFloat4C | speculative | Stores the float argument (from [esp+4]) into this+0x4c with movss and returns with ret 4. |
| `00d56890` | — | unknown | Installs FlashMovieInterface::vftable and FlashMovie::vftable, clears the movie fields, creates a Scaleform movie for the file name argument via GFxWrap::Creat… |

**FlashMovieInterface** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `010b48e0` | FlashMovieInterface::ResetVtable | confirmed | Stores FlashMovieInterface::vftable into the first dword of the object in ECX and returns (no arguments). |
| `00d55690` | FlashMovieInterface::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the base teardown FUN_010b48e0 on this, then calls _free(this) when bit 0 of the flags argument is set; returns this in eax a… |

**GuiFlashCtrl** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0083d800` | GuiFlashCtrl::HitTestForward | confirmed | Thiscall with two stack arguments, ret 0x10. Pushes 2, the second argument, [this+0x118] (the GFxWrap movie) and the first argument, then calls the GFxWrap::Hi… |
| `0083e470` | GuiFlashCtrl::RouteMouseInput | confirmed | Thiscall with five stack arguments, ret 0x14. Calls 00976630 with the string ShowUI and a local flag (receiver [01be1090]); returns 0 if the flag is 0. If the … |
| `008404b0` | GuiFlashCtrlLoadMovie | confirmed | __fastcall, this=ECX. Zeroes this+0x120. If this+0x11c is null: logs 'Loading movie %s' with this+0x130 through 00704db0 and creates a movie with GFxWrap::Crea… |
| `00840d80` | GuiFlashCtrl::ShowOrMouseLeave | confirmed | Thiscall with one char argument, ret 4. If the argument is 0 and global 01be15ca is nonzero and the movie at [this+0x118] is non-null, it calls GFxWrap::MouseL… |
| `00842a10` | GuiFlashCtrlScalarDeletingDtor | confirmed | thiscall scalar deleting destructor (30 bytes): calls 00840ba0 (GuiFlashCtrlDtorBody), then calls 004c6a61 (pooled free) on this when bit 0 of arg1 is set. Byt… |
| `0083da60` | GuiFlashCtrl::ForwardToQuerySizeConstraint | likely | Pushes its second stack argument then its first, calls 004115b9 with ECX unchanged (this), and returns the first argument in EAX with ret 8. |
| `00840200` | GuiFlashCtrlCtor | likely | __fastcall constructor (313 bytes, SEH frame). Calls 00429076 as the base constructor with flags 0x4000000, constructs the embedded GuiProperties<GuiFlashCtrl>… |
| `008403a0` | ThunkSub4_To00842a10 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,4 then jumps to 00842a10. Not a body; the jump target carries the work. |
| `008403b0` | ThunkSub24_To00842a10 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,0x24 then jumps to 00842a10. Not a body; the jump target carries the work. |
| `008403c0` | ThunkSubE8_To00842a10 | likely | Secondary-base adjustor thunk (11 bytes): sub ecx,0xe8 then jumps to 00842a10. Not a body; the jump target carries the work. |
| `008403d0` | ThunkSub24_To00840370 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,0x24 then jumps to 00840370. Not a body; the jump target carries the work. |
| `008403e0` | ThunkSub24_To00840340 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,0x24 then jumps to 00840340. Not a body; the jump target carries the work. |
| `008403f0` | ThunkSub24_To0083d720 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,0x24 then jumps to 0083d720. Not a body; the jump target carries the work. |
| `00840400` | ThunkSub24_To00840390 | likely | Secondary-base adjustor thunk (12 bytes incl. padding): sub ecx,0x24 then jumps to 00840390, which is itself a jump thunk to 004c6dfd. Not a body. |
| `00840410` | ThunkSub24_To00840350 | likely | Secondary-base adjustor thunk (8 bytes): sub ecx,0x24 then jumps to 00840350. Not a body; the jump target carries the work. |
| `008405e0` | GuiFlashCtrlReleaseThenLoad | likely | __thiscall (16 bytes): push esi; esi=this; call 00840420 (ReleaseMovieHandle) with this; ecx=this; pop esi; then tail-jumps to 008404b0 (GuiFlashCtrlLoadMovie)… |
| `00840ba0` | GuiFlashCtrlDtorBody | likely | __fastcall destructor body (376 bytes, no free). Stores the GuiFlashCtrl vtables at +0, +4, +0x24 and +0xe8. Walks the global list at DAT_01b3672c (next at +4,… |
| `00841580` | ReplaceFlashLoaderJob | likely | __fastcall, this=ECX (178 bytes). Releases the refcounted object at this+0x11c (refcount word at +4 minus one; its vftable slot 1 when it reaches zero) and cle… |
| `00841660` | GuiFlashCtrlAdvanceFrame | likely | Virtual slot 105 of GuiFlashCtrl (524 bytes, thiscall). If the sub-object at this+0x24 reports ready through its vftable slot 42 (+0xa8), sets bit 1 (0x2) of t… |
| `00841870` | GuiFlashCtrlSetProperty | likely | Virtual slot 5 of GuiFlashCtrl at its GuiProperties base (+0xe8), thiscall with key (param_2) and value (param_3). First calls vftable slot 12 (+0x30) with the… |
| `008419a0` | GuiFlashCtrlPropertyGet | likely | Virtual slot 8 of GuiFlashCtrl at its GuiProperties base (+0xe8), 625 bytes, thiscall with key and out arguments. First calls vftable slot 26 (+0x68) with the … |
| `00841c20` | GuiFlashCtrlInvoke | likely | __thiscall(this, name char*, args Array<GFxWrap::GuiGfxParam,0,1>*). If the movie at +0x118 is null: clears bit 2 of the flags at +0x120 and calls 008404b0 (Gu… |
| `00842830` | GuiFlashCtrlCopyCtor | likely | __thiscall copy constructor (474 bytes, SEH frame). Calls 00429325 (control base constructor) with the source pointer, calls 00841c70 with source+0xe8 as its a… |
| `00840340` | ForwardSub0cSlot0 | speculative | Virtual slot 3 of GuiFlashCtrl (13 bytes: lea eax,[ecx+4]; mov ecx,[eax+8]; mov edx,[ecx]; push eax; call edx; pop ecx; ret). Calls vftable slot 0 of the objec… |
| `00840350` | ForwardSub0cSlot1Arg | speculative | Virtual slot 4 of GuiFlashCtrl, one stack argument (23 bytes, ret 4). Calls vftable slot 1 of the object at this+0xc with (this+4, arg1). |
| `00840370` | ForwardThreeArgsTo005017f8 | speculative | Virtual slot 2 of GuiFlashCtrl, two stack arguments (22 bytes, ret 8). Pushes arg2, arg1 and this, calls 005017f8 cdecl-style, pops 12 bytes and returns with r… |

**GuiFlashCtrlCallbackHandler** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0083f800` | LookupStringOrNotFound | likely | Virtual slot 0 of GuiFlashCtrlCallbackHandler (283 bytes, SEH frame). Returns 0 when arg1 is null or the lookup object at FUN_00769710 has no table (+0x10 zero… |
| `0083f2f0` | CbHandlerSlot3ChunkedLoad | speculative | Virtual slot 3 of GuiFlashCtrlCallbackHandler (366 bytes, SEH frame, 4 KB stack buffer). Calls 0083e960 on a stack-held argument; if it returns a non-null obje… |
| `00840df0` | CbHandlerSlot4GameSceneRect | speculative | Virtual slot 4 of GuiFlashCtrlCallbackHandler (961 bytes, five float arguments and a count). Looks up the GuiFlashCtrl entry with 0083e960 and returns if it is… |
| `008411c0` | — | unknown | Virtual slot 13 of GuiFlashCtrlCallbackHandler, one C-string argument (522 bytes). Takes the larger of two lengths from 0083cfd0 and skips past it, then compar… |

**GuiProperties<GuiFlashCtrl>** (17)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0083ff10` | GuiPropertiesScalarDeletingDtor | confirmed | thiscall scalar deleting destructor (30 bytes): calls the GuiProperties dtor body 0083fcc0, then calls _free(this) when bit 0 of arg1 is set. Bytes after the _… |
| `0083f920` | GuiPropertiesCtor | likely | __fastcall constructor: sets this[1] and this[2] to the same DAT_017efbd4 value, zeroes this[3] and this[4], stores the GuiProperties<GuiFlashCtrl> vtable at +… |
| `0083fbb0` | GetEntryByIndex | likely | Virtual slot 2 of GuiProperties<GuiFlashCtrl> (31 bytes). Pushes its four arguments in reverse order and calls 0083fb50 (StaticListEntryFields), then pops 16 b… |
| `0083fbd0` | GetEntryFieldByIndex | likely | Virtual slot 3 of GuiProperties<GuiFlashCtrl> (59 bytes). Returns 0 if the static list head DAT_01be1974 is null, steps param_1 times through the +0x18 link, r… |
| `0083fc10` | GetFirstEntry | likely | Virtual slot 0 of GuiProperties<GuiFlashCtrl> (73 bytes). Sets the global cursor DAT_01be1960 to the static list head DAT_01be1974. If the head is non-null, wr… |
| `0083fc60` | GetNextEntry | likely | Virtual slot 1 of GuiProperties<GuiFlashCtrl> (80 bytes). If the global cursor DAT_01be1960 is non-null, advances it to cursor+0x18; if the new cursor is non-n… |
| `0083fcb0` | GetStaticCount | likely | Virtual slot 4 of GuiProperties<GuiFlashCtrl> (6 bytes). Returns the global DAT_01be197c. |
| `0083fcc0` | GuiPropertiesDtorBody | likely | __fastcall destructor body (no free): stores the GuiProperties vtable, stores the SoeUtil::Map<int,int,-1,0> vtable at this+0x14, then if this[6] is non-null c… |
| `00840750` | PropertySetOrErase | likely | Virtual slot 5 of GuiProperties<GuiFlashCtrl> (135 bytes, thiscall, args (value, index-or-id)). Calls 008210a0 on the third argument; looks up the key built fr… |
| `00841c70` | GuiPropertiesCopyCtor | likely | __thiscall copy-style constructor (144 bytes): sets this[1] and this[2] to DAT_017efbd4, zeroes this[3], this[4], this[6], this[7], stores the GuiProperties<Gu… |
| `00841e40` | GetPropertyStringNarrow | likely | Virtual slot 29 of GuiProperties<GuiFlashCtrl> (459 bytes, thiscall, args key and out IString<char>). Calls vftable slot 9 (+0x24) with the key to get a wide S… |
| `00842010` | GetPropertyStringNarrowAlt | likely | Virtual slot 30 of GuiProperties<GuiFlashCtrl> (459 bytes, same body as 00841e40 apart from the getter). Calls vftable slot 8 (+0x20) with the key to get a wid… |
| `008421e0` | GetPropertyWideFormat | likely | Virtual slot 9 of GuiProperties<GuiFlashCtrl> (77 bytes, thiscall, args key and out). Looks up the name-table entry (&DAT_01b34f8c)[key*4] with 008260d0. If fo… |
| `0083f9a0` | ContainsValue | speculative | __thiscall, arg1 = value. Calls this->vftable[4] (+0x10) for the element count, then for each index calls this->vftable[2] (+8) with (index, &local_c, &value_o… |
| `0083fa00` | RemoveFirstByValue | speculative | __thiscall, arg1 = value. Loops over the indices from vftable[4] (+0x10) count, reading each entry through vftable[2] (+8) until the value output equals arg1. … |
| `0083fd50` | BuildPropertyStringFlashanimBaked | speculative | Virtual slot 11 of GuiProperties<GuiFlashCtrl> (306 bytes). Starts the slot-0 cursor walk, then loops through the vtable at offsets +0x74 and +4 to read each e… |
| `008405f0` | — | unknown | Virtual slot 6 of GuiProperties<GuiFlashCtrl> (349 bytes, thiscall, arg1 = key). Calls vftable[2] (GetEntryByIndex) with the key to fetch an entry; returns if … |

**Scaleform::IScaleformActionHandler** (62)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248850` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. Base-class constructor body with no other effect. |
| `012488a0` | Scaleform::IScaleformActionHandler::Ctor | confirmed | Byte-identical to 01248850: stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. |
| `012779a0` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01277b00` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `012794c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279510` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279660` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `012797b0` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279900` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279a50` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279ba0` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279cf0` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279e40` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01279f90` | Scaleform::IScaleformActionHandler::SetBaseVftable | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01248a00` | Scaleform::IScaleformActionHandler::Ctor | likely | Byte-identical to 01248850: stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. |
| `01248b60` | Scaleform::IScaleformActionHandler::Ctor | likely | Byte-identical to 01248850: stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. |
| `01248cc0` | Scaleform::IScaleformActionHandler::Ctor | likely | Byte-identical to 01248850: stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. |
| `01248e10` | Scaleform::IScaleformActionHandler::Ctor | likely | Byte-identical to 01248850: stores the Scaleform::IScaleformActionHandler vtable into the object at param_1 and returns. |
| `0124e740` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns void. 67-byte body. |
| `0124e790` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Byte-identical twin of 0124e740: stores Scaleform::IScaleformActionHandler::vftable into *this and returns. |
| `0124ecd0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `012507a0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `012507f0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01250b80` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01250bd0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `012517b0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01252020` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01252170` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `012522c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01252410` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01252560` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body identical to 0124e740. |
| `01272c30` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. 67-byte body with no calls. |
| `01272dd0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01272f20` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273070` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012731c0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273310` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273460` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012735b0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273700` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273850` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `012739a0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273b00` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273c50` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273da0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01273ef0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01274040` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01274190` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276ab0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276b00` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276c50` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276da0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01276ef0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01277040` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01277190` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with no calls. |
| `01277440` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01277590` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `012776e0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01277840` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01277c60` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01277dc0` | Scaleform::IScaleformActionHandler::SetBaseVftable | likely | Stores Scaleform::IScaleformActionHandler::vftable into the first dword of *this and returns. No other effect. |
| `01272c80` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; byte-identical copy of 01272c30. |

**Scaleform::ScaleformAction** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012ec070` | Scaleform::ScaleformAction::DtorBody | confirmed | Destructor body: stores the ScaleformAction vftable (018d6014), frees the name buffer at +0x28 when its capacity at +0x3c is above 15, resets the name to empty… |
| `012ec0f0` | Scaleform::ScaleformAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor: calls the ScaleformAction destructor body 012ec070, then frees this with _free only when bit 0 of the flag argument is set. Returns… |
| `012ec110` | Scaleform::ScaleformAction::Ctor | confirmed | Fastcall constructor (returns this): runs the ScaleformArguments base constructor 012ebc00, stores the ScaleformAction vftable (018d6014) at this, zeroes the n… |
| `012ec190` | Scaleform::ScaleformAction::CtorWithArgs | confirmed | Constructor with a name string (param_2) and two more stack arguments (param_3, param_4): runs the ScaleformArguments base constructor 012ebc00, stores the Sca… |

**Scaleform::ScaleformArguments** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012eb980` | Scaleform::ScaleformArguments::DtorBody | confirmed | Destructor body: stores the ScaleformArguments vftable, frees the element buffer at +0x18 when it is non-null, zeroes +0x18..+0x20, stores the array vftable at… |
| `012ebc00` | Scaleform::ScaleformArguments::Ctor | confirmed | Fastcall constructor: stores the ScaleformArguments vftable (018d5ffc) at this and the SoeUtil::Array<GFxWrap::GuiGfxParam,0,1> vftable (017fa6d0) at +4, then … |
| `012ebc70` | Scaleform::ScaleformArguments::ScalarDeletingDtor | likely | Scalar deleting destructor: calls the ScaleformArguments destructor body 012eb980, then frees this with _free only when bit 0 of the flag argument is set. Retu… |

**Scaleform::ScaleformDelegate** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012f7630` | Scaleform::ScaleformDelegate::ExternalInterfaceDispatch | confirmed | Virtual at ScaleformDelegate vftable[0] (the body is not a destructor): first tries the localisation callback 012f73e0; when that reports the name unhandled (r… |
| `012f7330` | Scaleform::ScaleformDelegate::InitVtable | likely | Stores the Scaleform::ScaleformDelegate vtable into *this (9 bytes, no calls). Used as a constructor or destructor body; the two cannot be told apart from this… |

**Scaleform::ScaleformInvoke** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012508d0` | Scaleform::ScaleformInvoke::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'Invoke : 0 : ..\common\coreclient\scaleform\scaleformhandlers.cc:43'. |
| `012508e0` | Scaleform::ScaleformInvoke::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01250840` | Scaleform::ScaleformInvoke::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Scaleform::ScaleformInvoke::vftable into the object; when the int argument is nonzero it … |
| `012508b0` | Scaleform::ScaleformInvoke::TypeName | likely | Slot [1] of Scaleform::ScaleformInvoke::vftable. Returns the constant C string 'Invoke'. |
| `01250a90` | Scaleform::ScaleformInvoke::InvokeMethodByName | likely | Slot [2] of Scaleform::ScaleformInvoke::vftable, 226 bytes. Calls FUN_012e0ba0 and FUN_00f9c010; if that returns an object it reads the 'MethodName' value thro… |

**Scaleform::ScaleformService** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f9ab30` | ScaleformService::ScalarDeletingDtor | confirmed | Virtual slot 0 of Scaleform::ScaleformService: stores the vtable at +0, calls 01230340 (base destructor body), then frees this through _free when bit 0 of the … |
| `012303e0` | Scaleform::ScaleformService::ReleaseExternalInterface | likely | If the pointer at this+8 is set, calls GFxWrap::TCSetExternalInterface(0), frees the pointer with _free and zeroes +8; then calls 01230d40 (empty stub). |
| `012304d0` | Scaleform::ScaleformService::ApplySurfaceSize | likely | Calls 0083da40 with 1, then calls the GFx object at this+0xc through its vtable +0x1dc with the stored width (+0x10) and height (+0x14). |
| `01230630` | Scaleform::ScaleformService::BuildGenericInfoDialogArgs | likely | Builds a ScaleformArguments list (012ebc00) with the string argument as TitleText, a second string as DisplayText, the 4-byte third argument as IsModal (012ebe… |
| `01230b90` | Scaleform::ScaleformService::ChangeScreen | likely | Builds an argument list with ScreenName and GroupID entries (00c587d0, 01230860, 00703350) with the second argument converted to double through 01230720, then … |
| `01230500` | Scaleform::ScaleformService::GetFlagAt18 | speculative | Returns the byte at this+0x18. |
| `01230510` | Scaleform::ScaleformService::SetFlagAt18 | speculative | Stores the byte argument at this+0x18. |
| `01230520` | Scaleform::ScaleformService::Wrapper_01230520 | speculative | Loads ECX from this+0xc (the GFx object) and jumps to 0083db40 (tail jump). |
| `00f9aa80` | — | unknown | Fastcall constructor: calls 012302d0 (base constructor) with an EH cookie, then stores Scaleform::ScaleformService::vftable at +0 and returns this. |
| `00f9aae0` | — | unknown | Stores Scaleform::ScaleformService::vftable at +0, then calls 01230340 (base destructor body) with an EH cookie; does not free this. |
| `012304b0` | — | unknown | Stores param_2 at this+0x10 (width) and param_3 at this+0x14 (height), then calls the virtual at vtable +0x18 of this. |
| `01230530` | — | unknown | Calls 0083db60 with no arguments and returns. |

**Scaleform::ScaleformServiceBase** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012302d0` | Scaleform::ScaleformServiceBase::ScaleformServiceBase | confirmed | Calls the ServiceBase ctor 01230d10, stores the Scaleform::ScaleformServiceBase vftable at +0, zeroes +8 and +0xc, sets +0x10 to 800 and +0x14 to 600, and clea… |
| `01230580` | Scaleform::ScaleformServiceBase::~ScaleformServiceBase | confirmed | Scalar deleting destructor: stores the Scaleform::ScaleformServiceBase vftable at +0, calls 01230d30, and calls _free on this when bit 0 of the argument is set. |
| `01230340` | Scaleform::ScaleformServiceBase::DestructBody | likely | Stores the Scaleform::ScaleformServiceBase vftable at +0 and calls 01230d30 (ServiceBase dtor body). It does not free the object. |

**Scaleform::ScaleformSetFocus** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012509c0` | Scaleform::ScaleformSetFocus::TypeName | confirmed | Slot [1] of Scaleform::ScaleformSetFocus::vftable. Returns the constant C string 'SetFocus'. |
| `012509e0` | Scaleform::ScaleformSetFocus::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'SetFocus : 0 : ..\common\coreclient\scaleform\scaleformhandlers.cc:29'. |
| `012509f0` | Scaleform::ScaleformSetFocus::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `01250950` | Scaleform::ScaleformSetFocus::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Scaleform::ScaleformSetFocus::vftable into the object; when the int argument is nonzero i… |
| `01250a50` | Scaleform::ScaleformSetFocus::ApplyFocus | likely | Slot [2] of Scaleform::ScaleformSetFocus::vftable, 62 bytes. Calls FUN_012e0ba0, then FUN_00f9c010; if it returns an object it takes FUN_012eb310('Focus', 0) a… |

**UI::UIEndRotate3DView** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248e60` | UI::UIEndRotate3DView::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UIEndRotate3DView vtable (018ca33c); when the flag argument is non-zero it calls 012ea4f0 on… |
| `01248ed0` | UI::UIEndRotate3DView::ActionName | confirmed | Virtual slot [1] of UI::UIEndRotate3DView (vftable 018ca33c). Returns the constant string EndRotate3DView (6-byte body). |
| `01248ef0` | UI::UIEndRotate3DView::DisplayName | confirmed | Virtual slot [4] of UI::UIEndRotate3DView (vftable 018ca33c). Returns the constant trace string EndRotate3DView : 0 : frtwuihandlers.cc:295 (6-byte body). |
| `01248f00` | UI::UIEndRotate3DView::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |
| `012493b0` | UI::UIEndRotate3DView::CallSlot14ThenSlot18 | confirmed | Virtual slot [2] of UI::UIEndRotate3DView (vftable 018ca33c). Gets the owner-thread object (012e0ba0, then 01249290); if non-null calls its virtual +0x14 to ge… |

**UI::UIGetAdvertisement** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248a50` | UI::UIGetAdvertisement::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UIGetAdvertisement vtable (018ca238); when the flag argument is non-zero it calls 012ea4f0 o… |
| `01248ac0` | UI::UIGetAdvertisement::ActionName | confirmed | Virtual slot [1] of UI::UIGetAdvertisement (vftable 018ca238). Returns the constant string GetAdvertisement (6-byte body). |
| `01248ae0` | UI::UIGetAdvertisement::DisplayName | confirmed | Virtual slot [4] of UI::UIGetAdvertisement (vftable 018ca238). Returns the constant trace string GetAdvertisement : 0 : frtwuihandlers.cc:257 (6-byte body). |
| `01248af0` | UI::UIGetAdvertisement::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |

**UI::UIGetNews** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248bb0` | UI::UIGetNews::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UIGetNews vtable (018ca298); when the flag argument is non-zero it calls 012ea4f0 on the obj… |
| `01248c20` | UI::UIGetNews::ActionName | confirmed | Virtual slot [1] of UI::UIGetNews (vftable 018ca298). Returns the constant string GetNews (6-byte body). |
| `01248c40` | UI::UIGetNews::DisplayName | confirmed | Virtual slot [4] of UI::UIGetNews (vftable 018ca298). Returns the constant trace string GetNews : 0 : frtwuihandlers.cc:267 (6-byte body). |
| `01248c50` | UI::UIGetNews::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |

**UI::UIGetRotator** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012488f0` | UI::UIGetRotator::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UIGetRotator vtable (018ca1e8); when the flag argument is non-zero it calls 012ea4f0 on the … |
| `01248960` | UI::UIGetRotator::ActionName | confirmed | Virtual slot [1] of UI::UIGetRotator (vftable 018ca1e8). Returns the constant string GetRotator (6-byte body). |
| `01248980` | UI::UIGetRotator::DisplayName | confirmed | Virtual slot [4] of UI::UIGetRotator (vftable 018ca1e8). Returns the constant trace string GetRotator : 0 : frtwuihandlers.cc:247 (6-byte body). |
| `01248990` | UI::UIGetRotator::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |

**UI::UIRespondAdvertisementDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248600` | UI::UIRespondAdvertisementDisplayAction::Ctor | confirmed | Stores the Engine::IDisplayActionHandler vtable, then the UI::UIRespondAdvertisementDisplayAction vtable (018ca10c) into the object; when the flag argument is … |
| `01248690` | UI::UIRespondAdvertisementDisplayAction::DisplayName | confirmed | Virtual slot [4] of UI::UIRespondAdvertisementDisplayAction (vftable 018ca10c). Returns the constant trace string kUserDataCommandRespondAdvertisementDisplayAc… |
| `012486a0` | UI::UIRespondAdvertisementDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Engine::IDisplayActionHandler vtable into the object and frees it with _free when bit 0 of the… |

**UI::UIRespondFeatureRotatorDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01249000` | UI::UIRespondFeatureRotatorDisplayAction::Ctor | confirmed | Stores the Engine::IDisplayActionHandler vtable, then the UI::UIRespondFeatureRotatorDisplayAction vtable (018ca398); when the flag argument is non-zero it cal… |
| `01249090` | UI::UIRespondFeatureRotatorDisplayAction::DisplayName | confirmed | Virtual slot [4] of UI::UIRespondFeatureRotatorDisplayAction (vftable 018ca398). Returns the constant trace string kUserDataCommandRespondFeatureRotatorDisplay… |
| `012490a0` | UI::UIRespondFeatureRotatorDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Engine::IDisplayActionHandler vtable and frees the object with _free when bit 0 of the flag ar… |

**UI::UIRespondNewsMessageDisplayAction** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248750` | UI::UIRespondNewsMessageDisplayAction::Ctor | confirmed | Stores the Engine::IDisplayActionHandler vtable, then the UI::UIRespondNewsMessageDisplayAction vtable (018ca178) into the object; when the flag argument is no… |
| `012487e0` | UI::UIRespondNewsMessageDisplayAction::DisplayName | confirmed | Virtual slot [4] of UI::UIRespondNewsMessageDisplayAction (vftable 018ca178). Returns the constant trace string kUserDataCommandRespondNewsMessageDisplayAction… |
| `012487f0` | UI::UIRespondNewsMessageDisplayAction::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Engine::IDisplayActionHandler vtable and frees the object with _free when bit 0 of the flag ar… |

**UI::UIRotate3DView** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01248d10` | UI::UIRotate3DView::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UIRotate3DView vtable (018ca2e4); when the flag argument is non-zero it calls 012ea4f0 on th… |
| `01248d80` | UI::UIRotate3DView::ActionName | confirmed | Virtual slot [1] of UI::UIRotate3DView (vftable 018ca2e4). Returns the constant string Rotate3DView (6-byte body). |
| `01248da0` | UI::UIRotate3DView::DisplayName | confirmed | Virtual slot [4] of UI::UIRotate3DView (vftable 018ca2e4). Returns the constant trace string Rotate3DView : 0 : frtwuihandlers.cc:277 (6-byte body). |
| `01248db0` | UI::UIRotate3DView::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |
| `01249360` | UI::UIRotate3DView::ApplyDirection | confirmed | Virtual slot [2] of UI::UIRotate3DView (vftable 018ca2e4). Gets the owner-thread object (012e0ba0, then 01249290); if non-null reads the integer under the key … |

**UI::UISetOverrideCursor** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01249100` | UI::UISetOverrideCursor::Ctor | confirmed | Stores the Scaleform::IScaleformActionHandler vtable, then the UI::UISetOverrideCursor vtable (018ca408); when the flag argument is non-zero it calls 012ea4f0 … |
| `01249170` | UI::UISetOverrideCursor::ActionName | confirmed | Virtual slot [1] of UI::UISetOverrideCursor (vftable 018ca408). Returns the constant string SetOverrideCursor (6-byte body). |
| `01249190` | UI::UISetOverrideCursor::DisplayName | confirmed | Virtual slot [4] of UI::UISetOverrideCursor (vftable 018ca408). Returns the constant trace string SetOverrideCursor : 0 : frtwuihandlers.cc:234 (6-byte body). |
| `012491a0` | UI::UISetOverrideCursor::ScalarDeletingDtor | confirmed | Scalar deleting destructor shape at vftable slot [0]: stores the Scaleform::IScaleformActionHandler vtable and frees the object with _free when bit 0 of the fl… |
| `01249320` | UI::UISetOverrideCursor::ForwardCursorValue | confirmed | Virtual slot [2] of UI::UISetOverrideCursor (vftable 018ca408). ECX is loaded from the first stack argument (the argument list). Reads the integer under the ke… |

**(no class)** (61)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009d2170` | UpdateSceneTextTimer | confirmed | Finds Main.wndSceneText.swfSceneText, casts it to GuiFlashCtrl, obtains a client timer, sets a parameter value and mode, and invokes the GiveTime callback. |
| `00a89b90` | FlashTimerResetTimer | confirmed | If flag 8 at this+0xc is set, calls 009797a0 with FlashTimer:ResetTimer and clears flag 8; returns otherwise. |
| `012f73e0` | ScaleformExternalCallback | confirmed | Scaleform ExternalInterface handler (movie view, callback name, GFxValue arguments, argument count): allocates a block of count 0x48-byte GuiGfxParam elements … |
| `008424f0` | LogSwfFrameInfo | likely | No arguments (816-byte body with 4 KB locals). Writes a framed log block: '*** Begin Swf Frame Info ***', then for each entry of the global GuiFlashCtrl list (… |
| `00843e90` | GfxInitMoviePlayer | likely | Three stack arguments (D3D device, present parameters, window handle). If DAT_01be15dc is null, allocates 0x114 bytes through 00d833a6 and stores the result of… |
| `00844280` | GuiFlashCtrlLifecycle | likely | Three stack arguments (op, object, source/arg). Switch on op (jump table): 0 calls 00840200 (GuiFlashCtrlCtor) on a non-null object; 1 calls 00842830 (GuiFlash… |
| `00844420` | GfxShutdownMoviePlayer | likely | One argument (teardown flag). Clears DAT_01be15ca. When the argument is non-zero: loops calling 00fc61c0 and, while it returns non-zero, 00fc6740, 006ffcb0(0x1… |
| `00844c00` | GuiFlashCtrlBindImage | likely | Three stack arguments (id, number, pointer to a struct). When the id is non-zero and 0083e960 finds a non-null registry entry with a non-zero word: formats 'im… |
| `009d22d0` | UpdateSceneTextTimerIfEnabled | likely | Calls FUN_009d2170 only when its argument is nonzero. |
| `00c26aa0` | FindGuiImageCtrl | likely | Hashes the supplied name, resolves an object through 00819650, and returns it only when RTTI casts from GObject to GuiImageCtrl. |
| `00ca8090` | GuiMovieShowOnce | likely | Shows the cached GFx movie at this+0xc4 once: when the movie object at +0xc is non-null and the byte at +0xf5 is zero, it calls GFxWrap::SetLimitAdvance, GFxWr… |
| `00ca8110` | FlashMoviePlayerInitOnce | likely | If the byte flag at this+0x28 is not 1, fetches a device object through 00790c60, probes it through device vtable slot 0x38, calls GFxWrap::InitFlashMoviePlaye… |
| `00ca8180` | GuiMovieRelease | likely | Releases the held state object at this+0x8 when set (calling 009f7b40 if its +0xf4 flag is set and the object at this+4 has +0x24c non-zero, then 00fc4a80 and … |
| `00d55a70` | GfxMovieAdvanceTimed | likely | Takes a timestamp with FUN_007029b0, calls GFxWrap::AdvanceMovie on the movie at this+0x20, takes a second timestamp, and adds (difference/1000) as a 64-bit va… |
| `00d55ad0` | GfxMovieRenderTimed | likely | Takes a timestamp with FUN_007029b0, fetches the object at [this+0x24]+0x5d8 and the object at this+0x28 and calls methods through their vtables, calls GFxWrap… |
| `00f9d120` | GetOrCreateScaleformService | likely | Fastcall registry accessor identical in structure to 00f9ce10 but keyed by DAT_01cbd6b0: creates Scaleform::ScaleformService through 00f9aa80 on first use, reg… |
| `01230420` | InitScaleformExternalInterface | likely | Once only (guarded by the byte at this+0x19): sets that byte to 1, calls 0083d9d0 with the address &LAB_012302c0, allocates 4 bytes via 00d833a6, and if the al… |
| `012308f0` | Scaleform::DispatchMainWindow | likely | If the dword at this+0x14 is nonzero, builds a GuiGfxParam argument list: one entry from the string at this (heap pointer at +4 when the capacity at +0x18 is 1… |
| `012eb130` | NamedArgs_FindKeyIndex | likely | Thiscall with a C-string key and ret 4: scans the named-argument entries (0x48 bytes from table+8, count at +0xc), considers entries whose tag at +0x40 is 0, c… |
| `012eb2d0` | NamedArgs_GetTag2DoubleOrDefault | likely | Thiscall with a C-string key, a float default and ret: looks up the key through 012eb130, takes the entry after it, and if that entry's tag at +0x40 is 2 retur… |
| `012eb310` | NamedArgs_GetTag1BoolOrDefault | likely | Thiscall with a C-string key and a bool default byte: looks up the key through 012eb130, takes the entry after it, and if its tag at +0x40 is 1 returns (dword … |
| `012ebd00` | ScaleformArgs_AddCString | likely | Thiscall on an argument list with one stack argument (ret 4), a C string: measures it, copies it into a new heap buffer (00d833a6 then _strcpy_s), wraps the co… |
| `012ebe00` | ScaleformArgs_AddNamedString | likely | Thiscall with a C-string name and a std::string argument (ret 8): appends the name through 012ebd00, then appends the string data, which is the heap pointer at… |
| `012ebe40` | ScaleformArgs_AddNamedInt | likely | Thiscall with a C-string name and an int (ret 8): appends the name through 012ebd00 and the value through 012eba10. |
| `012ebe60` | ScaleformArgs_AddNamedFloat | likely | Thiscall with a C-string name and a float (ret 8): appends the name through 012ebd00 and the value through 012eba80. |
| `012ebe80` | ScaleformArgs_AddNamedBool | likely | Thiscall with a C-string name and a bool byte (ret 8): appends the name through 012ebd00 and the value through 012ebaf0. |
| `012ebfb0` | ScaleformArgs_AppendNamedEntries | likely | Two stack arguments (table base, count): for each of count entries of 0x48 bytes starting at base+0x38, reads the entry's tag at +8 and appends the entry: tag … |
| `012ec220` | ScaleformAction_NewInvokeMethodName | likely | Allocates 0x40 bytes with operator new (00d833a6); if that succeeds it assigns the literal action name 'Invoke' into a local std::string, constructs a Scalefor… |
| `0083fb50` | StaticListEntryFields | speculative | Stdcall-style, four stack arguments (index, out name pointer, out id, out field). Starts at the global head DAT_01be1974 and steps index times through the link… |
| `0083ff30` | NameTableResolveThenSlot5 | speculative | thiscall, arg1 = value (param_2). Calls this->vftable[8] (+0x20) with the value and an IString<unsigned short> temporary to get a name, then 0083d1e0 on it. Se… |
| `00840420` | ReleaseMovieHandle | speculative | __fastcall, this=ECX (fields at +0x118 and +0x11c). Looks up the entry keyed by this+0x118 with 0083e960; if found, calls 0083f560 on it and frees it through t… |
| `008407f0` | GuiFlashCtrlConstructAtArg | speculative | One stack argument (copied into ECX). If the argument is non-zero, calls 00840200 (GuiFlashCtrlCtor) with ECX set to the argument, so the argument is the stora… |
| `00840d20` | GuiFlashCtrlFlagSetOrRelease | speculative | __fastcall, this=ECX, flags word at +0x120. If bit 1 (0x2) is set, sets bit 2 (0x4). Otherwise, if the movie at +0x118 is non-null, clears bit 2 and transfers … |
| `00840d50` | GuiFlashCtrlClearFlagAndLoad | speculative | __fastcall, this=ECX. Clears bit 2 (0x4) of the flags word at +0x120 when set. If the movie at +0x118 is null, jumps to 008404b0 (GuiFlashCtrlLoadMovie); other… |
| `008413d0` | GuiFlashCtrlListPendingUpdate | speculative | No arguments. Calls 00fc6820(-1), then 009763c0 with 'PrimsGFX' and DAT_01be15d8, and clears DAT_01be15d8. Walks the global GuiFlashCtrl list from DAT_01b3672c… |
| `00841d00` | KeyRecordCreateAndRegister | speculative | Three stack arguments (id, two values). Allocates 0x1c bytes through 00d833a6; on success stores id at +0, value 1 at +4, value 2 at +8, and zeroes +0xc, +0x10… |
| `00841d90` | RegisterKeyRecords0to6 | speculative | No arguments. Calls 00841d00 seven times, in the order ids 0, 1, 4, 5, 2, 3, 6. The second argument is 0 in every call; the third is 1 only for id 1 and 0 othe… |
| `00842a30` | InsertKeyValuePairs121 | speculative | No arguments, 5106 bytes of straight-line code. Calls 008422d0 (HashListInsertNode) 121 times with a 3-word record (key, value, 0) built on the stack. Keys are… |
| `00843ff0` | RegisterKeyRecordsExtra | speculative | No arguments. Calls 00841d90 (RegisterKeyRecords0to6) and then 00841d00 eleven more times with ids 0xe, 0xf, 0x10, 0x11, 0x12, 0x13, 0x1c, 0x1b, 0x41, 0x3d, 0x… |
| `00844080` | GuiFlashCtrlCopyIfNonNull | speculative | Two stack arguments. If the first argument is non-zero, calls 00842830 (GuiFlashCtrlCopyCtor) with the second argument; otherwise does nothing. Saves and resto… |
| `00ca86d0` | CreateMovieWhenEnabled | speculative | Reads the singleton from 00769710; when its +0x1c field is non-zero returns GFxWrap::CreateMovie for the path at this+0x20, otherwise returns null. |
| `01230410` | Wrapper_01230410 | speculative | Loads ECX from this+0xc and jumps to 00840d20 (tail jump, no return of its own). |
| `012305e0` | Wrapper_012305e0 | speculative | Saves ECX, calls 01230420 on it, then loads ECX from this+0xc and jumps to 00840d50 (tail jump). |
| `01230720` | CtorDoubleValue | speculative | Calls 00703540 on this, then stores the double argument at +0x38, zero at +0x34 and 2 at +0x40, and returns this. |
| `01230b70` | Wrapper_01230b70 | speculative | Passes its second stack argument in ECX to 012eac40 (returns that pointer plus 4), then calls 012308f0 on this with the first stack argument and that pointer; … |
| `012eac40` | ReturnThisPlus4 | speculative | Fastcall accessor: returns this+4 (lea eax, [ecx+4]); nothing else. |
| `012eb070` | NamedArgs_GetTag0ByIndex | speculative | Thiscall with an index and a default (ret 8): when the index is below the entry count at +0xc and the entry's tag at +0x40 (entries 0x48 bytes at table+8) is 0… |
| `012eb0a0` | NamedArgs_GetTag1ByIndex | speculative | Thiscall with an index and a default (ret 8): when the index is below the entry count at +0xc and the entry's tag at +0x40 is 1, returns the dword at entry+0x3… |
| `012eb250` | NamedArgs_GetTag0OrDefault | speculative | Fastcall on a named-argument table (entries 0x48 bytes at table+8, count at +0xc) with two stack arguments: looks up the named key through 012eb130, then takes… |
| `012eb290` | NamedArgs_GetTag1OrDefault | speculative | Fastcall on the same named-argument table as NamedArgs_GetTag0OrDefault: resolves the key through 012eb130, and if the next entry's tag at +0x40 is 1 it return… |
| `012eb350` | NamedArgs_HasKey | speculative | Thiscall with a C-string key: returns whether the lookup 012eb130 finds an index below the entry count at +0xc (found = true), 27 bytes, no other work. |
| `012eba10` | ScaleformArgs_PushInt | speculative | Thiscall with one int stack argument (ret 4): wraps the value in a fixed-string holder through 00c58830, appends the holder to the argument list through 012308… |
| `012eba80` | ScaleformArgs_PushFloat | speculative | Thiscall with one float argument: converts it to double through 01230720 and appends it to the argument list through 01230860; the fixed-string destructor 0070… |
| `012ebaf0` | ScaleformArgs_PushBool | speculative | Thiscall with one byte (bool) argument: wraps it in a fixed-string holder through 00c58830, appends the holder through 01230860, and runs 00703350 as cleanup. |
| `012ebdb0` | ScaleformArgs_AddStdStringData | speculative | Thiscall with one stack argument (a std::string, ret 4) that appends its character data through 012ebd00: uses the heap pointer when the capacity at +0x18 is a… |
| `012ebde0` | ScaleformArgs_AddTwoCStrings | speculative | Thiscall with two stack arguments (ret 8): appends the first C string and then the second through 012ebd00 with the same list as the receiver. |
| `012ebf20` | ScaleformArgs_AddStringNameInt | speculative | Thiscall with a std::string name (read from the inline buffer at +4 when capacity at +0x18 is below 16, else the heap pointer) and an int stack argument (ret):… |
| `012ec050` | ReturnThisPlus24 | speculative | Fastcall accessor: returns this+0x24 (lea eax, [ecx+0x24]); nothing else. |
| `012ec320` | ReturnThisPlus4B | speculative | Fastcall accessor: returns this+4 (return param_1 + 4); same body as ReturnThisPlus4 (012eac40). |
| `00843e30` | — | unknown | __fastcall, receiver ECX returned unchanged. Calls 00842230 (HashListMapCtor) on a stack-local argument (Ghidra shows a stack-cookie expression; the receiver i… |
| `00c254c0` | — | unknown | Reads a scene object, a name, and four numeric values, allocates/configures a GUI object, attaches it to GameScene, and pushes success or failure. |

## Source shards

[[S0116 0065f895-00664515|S0116]] (102), [[S0202 0083ea10-00844c00|S0202]] (61), [[S0220 008a3ce0-008a8500|S0220]] (1), [[S0238|S0238]] (5), [[S0265 0098ec00-00995660|S0265]] (31), [[S0276 009cdf80-009d3fe0|S0276]] (2), [[S0309 00a87ba0-00a8cdc0|S0309]] (1), [[S0386 00c1f190-00c26aa0|S0386]] (2), [[S0410 00ca6401-00cab4e0|S0410]] (4), [[S0445 00d54b90-00d59cb0|S0445]] (22), [[S0542|S0542]] (4), [[S0590|S0590]] (2), [[S0652|S0652]] (18), [[S0654 012393a0-0123f010|S0654]] (18), [[S0655|S0655]] (9), [[S0656|S0656]] (42), [[S0658 0124dfe0-01253ee0|S0658]] (43), [[S0661 0125d190-012619f0|S0661]] (1), [[S0665 01271690-01277340|S0665]] (25), [[S0666 012773d0-0127d160|S0666]] (18), [[S0667 0127d1b0-01281de0|S0667]] (20), [[S0687|S0687]] (31), [[S0689|S0689]] (3), [[S0797|S0797]] (3), [[S0804 0174b4a0-0174f4f0|S0804]] (124), [[S0809 0066ca4d-0147b010|S0809]] (4)
