---
type: build-guide
section: Network
updated: 2026-10-12
---

# Login and Account

Part of [[Build Guides]] › Network. Related: [[Gateway and Transport]], [[Command System]].

<!-- NARRATIVE START -->
## How it works

_Not written yet. An agent fills this from the shard findings, labels and video evidence: what the feature does, its data formats and packets, what the server must do, and open questions._
<!-- NARRATIVE END -->

## Mapped functions (generated)

651 accepted functions: 327 confirmed, 129 likely, 129 speculative, 66 unknown. Network-linked (`wire`): 68.

Subsystems: `net.login` (349), `net.account_command` (147), `net.clientupdate` (54), `ui.welcome` (42), `ui.account` (41), `net.namechange` (18)

### `net.account_command`

**AccountCommand_RequestActivateStadiumPiece** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e8d00` | AccountCommand_RequestActivateStadiumPiece::~AccountCommand… | confirmed | Stores the AccountCommand_RequestActivateStadiumPiece vftable into this, frees the list storage at param_1[3] when non-null, zeroes param_1[3], [4] and [5], th… |
| `014e8d90` | AccountCommand_RequestActivateStadiumPiece::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RequestActivateStadiumPiece. Returns the constant string 'AccountCommand_RequestActivateStadiumPiece'. |
| `014e8e10` | AccountCommand_RequestActivateStadiumPiece::ScalarDeletingD… | confirmed | Virtual slot [0] of AccountCommand_RequestActivateStadiumPiece. Calls the body destructor 014e8d00, then frees the object through _free when bit 0 of the flag … |
| `014e8e30` | AccountCommand_RequestActivateStadiumPiece::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RequestActivateStadiumPiece. Writes 'Request Activate Stadium Piece Account Command' through 01241650, then calls base summ… |
| `014e8e60` | AccountCommand_RequestActivateStadiumPiece::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RequestActivateStadiumPiece. Writes the 46-byte string 'Request Activate Stadium Piece Account Command' through 01241650 an… |
| `014e8e80` | AccountCommand_RequestActivateStadiumPiece::AccountCommand_… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable and zeroes the fields at +0xc, +0x10 and +0x14. Returns the object. |
| `014e9060` | AccountCommand_RequestActivateStadiumPiece::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RequestActivateStadiumPiece (NetworkCommand source RTTI); on null it asserts 'clone' at Acc… |
| `014e8dd0` | AccountCommand_RequestActivateStadiumPiece::Write | likely | Virtual slot [10] of AccountCommand_RequestActivateStadiumPiece. Calls the slot-2 virtual, calls base Write 01407d30, calls 012fbec0 on this+8, then calls the … |
| `014e9190` | AccountCommand_RequestActivateStadiumPiece::Read | likely | Virtual slot [9] of AccountCommand_RequestActivateStadiumPiece. Calls the slot-4 virtual on the archive with a pointer to a local holding param_1, calls base R… |
| `014e8da0` | AccountCommand_RequestActivateStadiumPiece::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RequestActivateStadiumPiece. Returns global DAT_01cbe8fc. |
| `014e8db0` | AccountCommand_RequestActivateStadiumPiece::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RequestActivateStadiumPiece. Stores its argument into global DAT_01cbe8fc. |
| `014e8ff0` | — | unknown | Virtual slot [17] of AccountCommand_RequestActivateStadiumPiece. Allocates 0x18 bytes via 00d833a6; if non-null it constructs the object with 014e8e80, then ca… |

**AccountCommand_RequestStadiumInfo** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e88c0` | AccountCommand_RequestStadiumInfo::AccountCommand_RequestSt… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable to AccountCommand_RequestStadiumInfo and zeroes the field at +8. Returns the object. |
| `014e8940` | AccountCommand_RequestStadiumInfo::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RequestStadiumInfo. Returns the constant string 'AccountCommand_RequestStadiumInfo'. |
| `014e8980` | AccountCommand_RequestStadiumInfo::~AccountCommand_RequestS… | confirmed | Stores the AccountCommand_RequestStadiumInfo vftable into this, then calls base destructor 01407bb0. Does not free the object. |
| `014e8a00` | AccountCommand_RequestStadiumInfo::Clone | confirmed | Virtual slot [17] of AccountCommand_RequestStadiumInfo. Allocates 0xc bytes via 00d833a6; if non-null it constructs the object with 014e88c0, then calls vtable… |
| `014e8a70` | AccountCommand_RequestStadiumInfo::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RequestStadiumInfo (NetworkCommand source RTTI); on null it asserts 'clone' at AccountComma… |
| `014e8bd0` | AccountCommand_RequestStadiumInfo::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_RequestStadiumInfo. Stores its vftable, calls base destructor 01407bb0, then frees the object through _free when bit 0 of th… |
| `014e8c30` | AccountCommand_RequestStadiumInfo::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RequestStadiumInfo. Writes 'Request Stadium Info Account Command' through 01241650, then calls base summary 01407e70. |
| `014e8c60` | AccountCommand_RequestStadiumInfo::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RequestStadiumInfo. Writes the 36-byte string 'Request Stadium Info Account Command' through 01241650 and returns. |
| `014e8ac0` | AccountCommand_RequestStadiumInfo::Read | likely | Virtual slot [9] of AccountCommand_RequestStadiumInfo. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads one integer into this+8 thr… |
| `014e8b10` | AccountCommand_RequestStadiumInfo::Write | likely | Virtual slot [10] of AccountCommand_RequestStadiumInfo. Calls the slot-2 virtual, calls base Write 01407d30, appends the integer at this+8 through 013c23f0, th… |
| `014e8950` | AccountCommand_RequestStadiumInfo::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RequestStadiumInfo. Returns global DAT_01cbe8f8. |
| `014e8960` | AccountCommand_RequestStadiumInfo::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RequestStadiumInfo. Stores its argument into global DAT_01cbe8f8. |

**AccountCommand_RequestUpdateStadium** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e8460` | AccountCommand_RequestUpdateStadium::AccountCommand_Request… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable, sets the string member capacity at +0x20 to 0xf, its size at +0x1c to 0 and its inline byte at … |
| `014e84f0` | AccountCommand_RequestUpdateStadium::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RequestUpdateStadium. Returns the constant string 'AccountCommand_RequestUpdateStadium'. |
| `014e8530` | AccountCommand_RequestUpdateStadium::~AccountCommand_Reques… | confirmed | Stores the AccountCommand_RequestUpdateStadium vftable into this, frees the string buffer at param_1[3] when its capacity (param_1[8]) is above 15, resets capa… |
| `014e85b0` | AccountCommand_RequestUpdateStadium::Clone | confirmed | Virtual slot [17] of AccountCommand_RequestUpdateStadium. Allocates 0x24 bytes via 00d833a6; if non-null it constructs the object with 014e8460, then calls vta… |
| `014e86a0` | AccountCommand_RequestUpdateStadium::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_RequestUpdateStadium. Calls the body destructor 014e8530, then frees the object through _free when bit 0 of the flag argumen… |
| `014e87b0` | AccountCommand_RequestUpdateStadium::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RequestUpdateStadium (NetworkCommand source RTTI); on null it asserts 'clone' at AccountCom… |
| `014e8870` | AccountCommand_RequestUpdateStadium::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RequestUpdateStadium. Writes 'Request Update Stadium Account Command' through 01241650, then calls base summary 01407e70. |
| `014e88a0` | AccountCommand_RequestUpdateStadium::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RequestUpdateStadium. Writes the 38-byte string 'Request Update Stadium Account Command' through 01241650 and returns. |
| `014e83d0` | AccountCommand_RequestUpdateStadium::Read | likely | Virtual slot [9] of AccountCommand_RequestUpdateStadium. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads a string-like member at t… |
| `014e8420` | AccountCommand_RequestUpdateStadium::Write | likely | Virtual slot [10] of AccountCommand_RequestUpdateStadium. Calls the slot-2 virtual, calls base Write 01407d30, writes the string member at this+8 through 013c2… |
| `014e8500` | AccountCommand_RequestUpdateStadium::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RequestUpdateStadium. Returns global DAT_01cbe8f4. |
| `014e8510` | AccountCommand_RequestUpdateStadium::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RequestUpdateStadium. Stores its argument into global DAT_01cbe8f4. |

**AccountCommand_RespondActivateStadiumPiece** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e7f80` | AccountCommand_RespondActivateStadiumPiece::AccountCommand_… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable to AccountCommand_RespondActivateStadiumPiece and zeroes the field at +8. Returns the object. |
| `014e8000` | AccountCommand_RespondActivateStadiumPiece::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RespondActivateStadiumPiece. Returns the constant string 'AccountCommand_RespondActivateStadiumPiece'. |
| `014e8040` | AccountCommand_RespondActivateStadiumPiece::~AccountCommand… | confirmed | Stores the AccountCommand_RespondActivateStadiumPiece vftable into this, then calls base destructor 01407bb0. Does not free the object. |
| `014e8140` | AccountCommand_RespondActivateStadiumPiece::Clone | confirmed | Virtual slot [17] of AccountCommand_RespondActivateStadiumPiece. Allocates 0xc bytes via 00d833a6; if non-null it constructs the object with 014e7f80, then cal… |
| `014e81b0` | AccountCommand_RespondActivateStadiumPiece::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RespondActivateStadiumPiece (NetworkCommand source RTTI); on null it asserts 'clone' at Acc… |
| `014e8310` | AccountCommand_RespondActivateStadiumPiece::ScalarDeletingD… | confirmed | Virtual slot [0] of AccountCommand_RespondActivateStadiumPiece. Stores its vftable, calls base destructor 01407bb0, then frees the object through _free when bi… |
| `014e8370` | AccountCommand_RespondActivateStadiumPiece::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RespondActivateStadiumPiece. Writes 'Respond Update Stadium Account Command' (39 bytes) through 01241650, then calls base s… |
| `014e83a0` | AccountCommand_RespondActivateStadiumPiece::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RespondActivateStadiumPiece. Writes the 38-byte string 'Respond Update Stadium Account Command' through 01241650 and return… |
| `014e8200` | AccountCommand_RespondActivateStadiumPiece::Read | likely | Virtual slot [9] of AccountCommand_RespondActivateStadiumPiece. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads one integer into t… |
| `014e8250` | AccountCommand_RespondActivateStadiumPiece::Write | likely | Virtual slot [10] of AccountCommand_RespondActivateStadiumPiece. Calls the slot-2 virtual, calls base Write 01407d30, appends the integer at this+8 through 013… |
| `014e8010` | AccountCommand_RespondActivateStadiumPiece::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RespondActivateStadiumPiece. Returns global DAT_01cbe8f0. |
| `014e8020` | AccountCommand_RespondActivateStadiumPiece::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RespondActivateStadiumPiece. Stores its argument into global DAT_01cbe8f0. |
| `014e80b0` | AccountCommand_RespondActivateStadiumPiece::BuildAndQueue | speculative | Virtual slot [15] of AccountCommand_RespondActivateStadiumPiece. Allocates 0x14 bytes (00d833a6), zeroes them (012f9eb0), stores 0xe0 in the first dword (012f8… |

**AccountCommand_RespondStadiumInfo** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e7b10` | AccountCommand_RespondStadiumInfo::AccountCommand_RespondSt… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable, zeroes the integer fields at +8, +0xc and +0x10, clears the +0x14 member (size 0 at +0x28, capa… |
| `014e7ba0` | AccountCommand_RespondStadiumInfo::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RespondStadiumInfo. Returns the constant string 'AccountCommand_RespondStadiumInfo'. |
| `014e7be0` | AccountCommand_RespondStadiumInfo::~AccountCommand_RespondS… | confirmed | Stores the AccountCommand_RespondStadiumInfo vftable into this, frees the member string buffer at param_1[6] when its capacity (param_1[0xb]) is above 15, rese… |
| `014e7c60` | AccountCommand_RespondStadiumInfo::Clone | confirmed | Virtual slot [17] of AccountCommand_RespondStadiumInfo. Allocates 0x30 bytes via 00d833a6; if non-null it constructs the object with 014e7b10, then calls vtabl… |
| `014e7d50` | AccountCommand_RespondStadiumInfo::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_RespondStadiumInfo. Calls the body destructor 014e7be0, then frees the object through _free when bit 0 of the flag argument … |
| `014e7e60` | AccountCommand_RespondStadiumInfo::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RespondStadiumInfo (NetworkCommand source RTTI); on null it asserts 'clone' at AccountComma… |
| `014e7f30` | AccountCommand_RespondStadiumInfo::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RespondStadiumInfo. Writes 'Respond Stadium Info Account Command' through 01241650, then calls base summary 01407e70. |
| `014e7f60` | AccountCommand_RespondStadiumInfo::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RespondStadiumInfo. Writes the 36-byte string 'Respond Stadium Info Account Command' through 01241650 and returns. |
| `014e7a30` | AccountCommand_RespondStadiumInfo::Read | likely | Virtual slot [9] of AccountCommand_RespondStadiumInfo. Calls the slot-4 virtual on the archive, base Read 01407cf0, then reads integers into this+8, this+0xc a… |
| `014e7ab0` | AccountCommand_RespondStadiumInfo::Write | likely | Virtual slot [10] of AccountCommand_RespondStadiumInfo. Calls the slot-2 virtual, calls base Write 01407d30, appends the integers at +8, +0xc and +0x10 through… |
| `014e7980` | AccountCommand_RespondStadiumInfo::BuildAndQueue | speculative | Virtual slot [15] of AccountCommand_RespondStadiumInfo. Allocates 0x14 bytes (00d833a6), zeroes them (012f9eb0), stores 0xe1 in the first dword (012f8c70), pas… |
| `014e7bb0` | AccountCommand_RespondStadiumInfo::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RespondStadiumInfo. Returns global DAT_01cbe8ec. |
| `014e7bc0` | AccountCommand_RespondStadiumInfo::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RespondStadiumInfo. Stores its argument into global DAT_01cbe8ec. |

**AccountCommand_RespondUpdateStadium** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e74e0` | AccountCommand_RespondUpdateStadium::AccountCommand_Respond… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable to AccountCommand_RespondUpdateStadium and zeroes the field at +8. Returns the object. |
| `014e7560` | AccountCommand_RespondUpdateStadium::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_RespondUpdateStadium. Returns the constant string 'AccountCommand_RespondUpdateStadium'. |
| `014e75a0` | AccountCommand_RespondUpdateStadium::~AccountCommand_Respon… | confirmed | Stores the AccountCommand_RespondUpdateStadium vftable into this, then calls base destructor 01407bb0. Does not free the object. |
| `014e76a0` | AccountCommand_RespondUpdateStadium::Clone | confirmed | Virtual slot [17] of AccountCommand_RespondUpdateStadium. Allocates 0xc bytes via 00d833a6; if non-null it constructs the object with 014e74e0, then calls vtab… |
| `014e7710` | AccountCommand_RespondUpdateStadium::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_RespondUpdateStadium (NetworkCommand source RTTI); on null it asserts 'clone' at AccountCom… |
| `014e7870` | AccountCommand_RespondUpdateStadium::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_RespondUpdateStadium. Stores its vftable, calls base destructor 01407bb0, then frees the object through _free when bit 0 of … |
| `014e78d0` | AccountCommand_RespondUpdateStadium::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_RespondUpdateStadium. Writes 'Respond Update Stadium Account Command' through 01241650, then calls base summary 01407e70. |
| `014e7900` | AccountCommand_RespondUpdateStadium::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_RespondUpdateStadium. Writes the 38-byte string 'Respond Update Stadium Account Command' through 01241650 and returns. |
| `014e7760` | AccountCommand_RespondUpdateStadium::Read | likely | Virtual slot [9] of AccountCommand_RespondUpdateStadium. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads one integer into this+8 t… |
| `014e77b0` | AccountCommand_RespondUpdateStadium::Write | likely | Virtual slot [10] of AccountCommand_RespondUpdateStadium. Calls the slot-2 virtual, calls base Write 01407d30, appends the integer at this+8 through 013c23f0, … |
| `014e7570` | AccountCommand_RespondUpdateStadium::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_RespondUpdateStadium. Returns global DAT_01cbe8e8. |
| `014e7580` | AccountCommand_RespondUpdateStadium::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_RespondUpdateStadium. Stores its argument into global DAT_01cbe8e8. |
| `014e7610` | AccountCommand_RespondUpdateStadium::BuildAndQueue | speculative | Virtual slot [15] of AccountCommand_RespondUpdateStadium. Allocates 0x14 bytes (00d833a6), zeroes them (012f9eb0), stores 0xe2 in the first dword (012f8c70), p… |

**AccountCommand_SetAchievementList** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014ea570` | AccountCommand_SetAchievementList::~AccountCommand_SetAchie… | confirmed | Stores the AccountCommand_SetAchievementList vftable into this, deletes the element pointers of the vector at +0xc through 014ea4c0, frees the vector storage a… |
| `014ea610` | AccountCommand_SetAchievementList::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_SetAchievementList. Returns the constant string 'AccountCommand_SetAchievementList'. |
| `014ea650` | AccountCommand_SetAchievementList::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_SetAchievementList. Calls the body destructor 014ea570, then frees the object through _free when bit 0 of the flag argument … |
| `014ea670` | AccountCommand_SetAchievementList::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_SetAchievementList. Writes 'Set Achievement List Account Command' through 01241650 (37 bytes), then calls base summary 0140… |
| `014ea3e0` | AccountCommand_SetAchievementList::Write | likely | Virtual slot [10] of AccountCommand_SetAchievementList. Calls the slot-2 virtual, calls base Write 01407d30, writes the vector at this+8 through 014ea2f0 (Writ… |
| `014ea620` | AccountCommand_SetAchievementList::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_SetAchievementList. Returns global DAT_01cbe90c. |
| `014ea630` | AccountCommand_SetAchievementList::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_SetAchievementList. Stores its argument into global DAT_01cbe90c. |
| `014ea110` | — | unknown | Virtual slot [15] of AccountCommand_SetAchievementList. Gets the object from 012d46a0, allocates 0x14 bytes (00d833a6) and zeroes them (012f9eb0) when the allo… |

**AccountCommand_SetOptIn** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e9b80` | AccountCommand_SetOptIn::AccountCommand_SetOptIn | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable to AccountCommand_SetOptIn, clears the byte at +8 and the byte at +9, and zeroes the field at +0… |
| `014e9c00` | AccountCommand_SetOptIn::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_SetOptIn. Returns the constant string 'AccountCommand_SetOptIn'. |
| `014e9c40` | AccountCommand_SetOptIn::~AccountCommand_SetOptIn | confirmed | Stores the AccountCommand_SetOptIn vftable into this, then calls base destructor 01407bb0. Does not free the object. |
| `014e9d50` | AccountCommand_SetOptIn::Clone | confirmed | Virtual slot [17] of AccountCommand_SetOptIn. Allocates 0x10 bytes via 00d833a6; if non-null it constructs the object with 014e9b80, then calls vtable slot [18… |
| `014e9dc0` | AccountCommand_SetOptIn::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_SetOptIn (NetworkCommand source RTTI); on null it asserts 'clone' at AccountCommandSetOptIn… |
| `014e9f90` | AccountCommand_SetOptIn::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_SetOptIn. Stores its vftable, calls base destructor 01407bb0, then frees the object through _free when bit 0 of the flag arg… |
| `014e9ff0` | AccountCommand_SetOptIn::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_SetOptIn. Writes 'Set Opt In Account Command' through 01241650, calls base summary 01407e70, then writes 'Opt In?: %d' from… |
| `014ea0f0` | AccountCommand_SetOptIn::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_SetOptIn. Writes the 26-byte string 'Set Opt In Account Command' through 01241650 and returns. |
| `014e9e20` | AccountCommand_SetOptIn::Read | likely | Virtual slot [9] of AccountCommand_SetOptIn. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads integers through 013c24d0 that are st… |
| `014e9ec0` | AccountCommand_SetOptIn::Write | likely | Virtual slot [10] of AccountCommand_SetOptIn. Calls the slot-2 virtual, calls base Write 01407d30, appends the byte at +8, the byte at +9 and the value at +0xc… |
| `014e9c10` | AccountCommand_SetOptIn::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_SetOptIn. Returns global DAT_01cbe908. |
| `014e9c20` | AccountCommand_SetOptIn::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_SetOptIn. Stores its argument into global DAT_01cbe908. |
| `014e9cd0` | AccountCommand_SetOptIn::BuildAndQueue | speculative | Virtual slot [15] of AccountCommand_SetOptIn. Gets the object from 012d46a0, allocates 0x14 bytes (00d833a6), zeroes them (012f9eb0) when the allocation is non… |

**AccountCommand_UploadAvatarImage** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e96f0` | AccountCommand_UploadAvatarImage::AccountCommand_UploadAvat… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable and zeroes the fields at +8 and +0xc. Returns the object. |
| `014e9770` | AccountCommand_UploadAvatarImage::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_UploadAvatarImage. Returns the constant string 'AccountCommand_UploadAvatarImage'. |
| `014e97b0` | AccountCommand_UploadAvatarImage::~AccountCommand_UploadAva… | confirmed | Stores the AccountCommand_UploadAvatarImage vftable into this, frees the buffer at param_1[3] when non-null, zeroes +8, then calls base destructor 01407bb0. Do… |
| `014e9870` | AccountCommand_UploadAvatarImage::Clone | confirmed | Virtual slot [17] of AccountCommand_UploadAvatarImage. Allocates 0x10 bytes via 00d833a6; if non-null it constructs the object with 014e96f0, then calls vtable… |
| `014e98e0` | AccountCommand_UploadAvatarImage::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_UploadAvatarImage (NetworkCommand source RTTI); on null it asserts 'clone' at AccountComman… |
| `014e9aa0` | AccountCommand_UploadAvatarImage::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_UploadAvatarImage. Calls the body destructor 014e97b0, then frees the object through _free when bit 0 of the flag argument i… |
| `014e9b30` | AccountCommand_UploadAvatarImage::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_UploadAvatarImage. Writes 'Upload Avatar Image Account Command' through 01241650, then calls base summary 01407e70. |
| `014e9b60` | AccountCommand_UploadAvatarImage::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_UploadAvatarImage. Writes the 35-byte string 'Upload Avatar Image Account Command' through 01241650 and returns. |
| `014e9950` | AccountCommand_UploadAvatarImage::Read | likely | Virtual slot [9] of AccountCommand_UploadAvatarImage. Calls the slot-4 virtual on the archive, then base Read 01407cf0, reads an integer into this+8 through 01… |
| `014e99d0` | AccountCommand_UploadAvatarImage::Write | likely | Virtual slot [10] of AccountCommand_UploadAvatarImage. Calls the slot-2 virtual, calls base Write 01407d30, appends the byte count at +8 through 013c23f0, writ… |
| `014e9780` | AccountCommand_UploadAvatarImage::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_UploadAvatarImage. Returns global DAT_01cbe904. |
| `014e9790` | AccountCommand_UploadAvatarImage::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_UploadAvatarImage. Stores its argument into global DAT_01cbe904. |

**AccountCommand_ValidateUsername** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e93b0` | AccountCommand_ValidateUsername::AccountCommand_ValidateUse… | confirmed | Calls 01407af0 (base constructor) first, then sets the vftable, sets the string member capacity at +0x20 to 0xf, its size at +0x1c to 0, its inline byte at +0x… |
| `014e9440` | AccountCommand_ValidateUsername::GetTypeName | confirmed | Virtual slot [11] of AccountCommand_ValidateUsername. Returns the constant string 'AccountCommand_ValidateUsername'. |
| `014e9480` | AccountCommand_ValidateUsername::~AccountCommand_ValidateUs… | confirmed | Stores the AccountCommand_ValidateUsername vftable into this, frees the member string buffer at param_1[3] when its capacity (param_1[8]) is above 15, resets c… |
| `014e9500` | AccountCommand_ValidateUsername::Clone | confirmed | Virtual slot [17] of AccountCommand_ValidateUsername. Allocates 0x28 bytes via 00d833a6; if non-null it constructs the object with 014e93b0, then calls vtable … |
| `014e95f0` | AccountCommand_ValidateUsername::ScalarDeletingDtor | confirmed | Virtual slot [0] of AccountCommand_ValidateUsername. Calls the body destructor 014e9480, then frees the object through _free when bit 0 of the flag argument is… |
| `014e9630` | AccountCommand_ValidateUsername::CloneInto | confirmed | Virtual slot [18] override. Runtime-casts param_2 to AccountCommand_ValidateUsername (NetworkCommand source RTTI); on null it asserts 'clone' at AccountCommand… |
| `014e96a0` | AccountCommand_ValidateUsername::PrintSummary | confirmed | Virtual slot [19] of AccountCommand_ValidateUsername. Writes 'Validate Username Account Command' through 01241650, then calls base summary 01407e70. |
| `014e96d0` | AccountCommand_ValidateUsername::PrintLabel | confirmed | Virtual slot [20] of AccountCommand_ValidateUsername. Writes the 33-byte string 'Validate Username Account Command' through 01241650 and returns. |
| `014e92d0` | AccountCommand_ValidateUsername::Read | likely | Virtual slot [9] of AccountCommand_ValidateUsername. Calls the slot-4 virtual on the archive, then base Read 01407cf0, then reads the string member at this+8 t… |
| `014e9360` | AccountCommand_ValidateUsername::Write | likely | Virtual slot [10] of AccountCommand_ValidateUsername. Calls the slot-2 virtual, calls base Write 01407d30, writes the member at this+8 through 013c2860, append… |
| `014e9220` | AccountCommand_ValidateUsername::BuildAndQueue | speculative | Virtual slot [15] of AccountCommand_ValidateUsername. Gets the object from 012d46a0, allocates 0x14 bytes (00d833a6), zeroes them (012f9eb0), stores 0xc1 in th… |
| `014e9450` | AccountCommand_ValidateUsername::GetSharedValue | speculative | Virtual slot [12] of AccountCommand_ValidateUsername. Returns global DAT_01cbe900. |
| `014e9460` | AccountCommand_ValidateUsername::SetSharedValue | speculative | Virtual slot [13] of AccountCommand_ValidateUsername. Stores its argument into global DAT_01cbe900. |

**(no class)** (26)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014e77f0` | AllocAndConstruct_014e74e0 | speculative | Allocates 0xc bytes through 00d833a6. If non-null it calls 014e74e0 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fram… |
| `014e7850` | RegisterCreator_21d | speculative | Calls 013ce130(0x21d, 014e77f0) and then 013ce230(0x21d, 014e77f0). Takes no arguments and returns nothing. |
| `014e7cd0` | AllocAndConstruct_014e7b10 | speculative | Allocates 0x30 bytes through 00d833a6. If non-null it calls 014e7b10 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e7d30` | RegisterCreator_21b | speculative | Calls 013ce130(0x21b, 014e7cd0) and then 013ce230(0x21b, 014e7cd0). Takes no arguments and returns nothing. |
| `014e7d70` | CopyStringMember14Out | speculative | Takes this and a destination std::string. Sets the destination's capacity to 0xf, its size to 0 and its inline byte to 0, then assigns the whole string member … |
| `014e7de0` | AssignStringMember14 | speculative | Assigns the string argument (passed by value on the stack) into the member string at this+0x14 with 00420ff1 (basic_string::assign). Frees the argument buffer … |
| `014e8290` | AllocAndConstruct_014e7f80 | speculative | Allocates 0xc bytes through 00d833a6. If non-null it calls 014e7f80 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fram… |
| `014e82f0` | RegisterCreator_219 | speculative | Calls 013ce130(0x219, 014e8290) and then 013ce230(0x219, 014e8290). Takes no arguments and returns nothing. |
| `014e8620` | AllocAndConstruct_014e8460 | speculative | Allocates 0x24 bytes through 00d833a6. If non-null it calls 014e8460 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e8680` | RegisterCreator_21c | speculative | Calls 013ce130(0x21c, 014e8620) and then 013ce230(0x21c, 014e8620). Takes no arguments and returns nothing. |
| `014e86c0` | CopyStringMember8Out | speculative | Takes this and a destination std::string. Sets the destination's capacity to 0xf, its size to 0 and its inline byte to 0, then assigns the whole string member … |
| `014e8730` | AssignStringMember8 | speculative | Assigns the string argument (passed by value on the stack) into the member string at this+8 with 00420ff1 (basic_string::assign). Frees the argument buffer thr… |
| `014e8b50` | AllocAndConstruct_014e88c0 | speculative | Allocates 0xc bytes through 00d833a6. If non-null it calls 014e88c0 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fram… |
| `014e8bb0` | RegisterCreator_21a | speculative | Calls 013ce130(0x21a, 014e8b50) and then 013ce230(0x21a, 014e8b50). Takes no arguments and returns nothing. |
| `014e8ee0` | AssignListFromTemp | speculative | Takes this, a value at param_2 and a heap pointer param_3. Erases or moves the range at this+0xc..+0x10 into the container at this+8 through 0041f5e6 (after bo… |
| `014e8f90` | ClearMember8ReturnArg | speculative | Calls 005f15f0 on this+8 and returns its second argument unchanged. |
| `014e9100` | AllocAndConstruct_014e8e80 | speculative | Allocates 0x18 bytes through 00d833a6. If non-null it calls 014e8e80 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e9160` | RegisterCreator_218 | speculative | Calls 013ce130(0x218, 014e9100) and then 013ce230(0x218, 014e9100). Takes no arguments and returns nothing. |
| `014e9570` | AllocAndConstruct_014e93b0 | speculative | Allocates 0x28 bytes through 00d833a6. If non-null it calls 014e93b0 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e95d0` | RegisterCreator_1e5 | speculative | Calls 013ce130(0x1e5, 014e9570) and then 013ce230(0x1e5, 014e9570). Takes no arguments and returns nothing. |
| `014e9a20` | AllocAndConstruct_014e96f0 | speculative | Allocates 0x10 bytes through 00d833a6. If non-null it calls 014e96f0 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e9a80` | RegisterCreator_1f7 | speculative | Calls 013ce130(0x1f7, 014e9a20) and then 013ce230(0x1f7, 014e9a20). Takes no arguments and returns nothing. |
| `014e9ac0` | CopyBlobToString | speculative | Takes this (with a byte count at +8 and a byte pointer at +0xc) and a destination std::string. Resets the destination's capacity to 0xf, its size to 0 and its … |
| `014e9f10` | AllocAndConstruct_014e9b80 | speculative | Allocates 0x10 bytes through 00d833a6. If non-null it calls 014e9b80 (constructor) on the block and returns the object; otherwise returns 0. Sets up an SEH fra… |
| `014e9f70` | RegisterCreator_1bc | speculative | Calls 013ce130(0x1bc, 014e9f10) and then 013ce230(0x1bc, 014e9f10). Takes no arguments and returns nothing. |
| `014ea4c0` | DeletePolymorphicElements | speculative | Walks the pointer vector at this+0xc..+0x10. For each non-null element it calls the element's vtable slot 0 with flag 1 (scalar deleting destructor, which free… |

### `net.clientupdate`

**BaseClient** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `009392c0` | BaseClient::HandleClientUpdate 🌐 | confirmed | Reads update subtype and dispatches validated payloads into client state; destroys each temporary packet object. |

**ClientUpdatePacketAddAbility** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e730` | ClientUpdatePacketAddAbility::ClientUpdatePacketAddAbility 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x16) (family 0x26, sub-opcode 22), installs ClientUpdatePacketAddAbility::vftable an… |
| `0089e7a0` | ClientUpdatePacketAddAbility::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketAddAbility vtable is… |

**ClientUpdatePacketAddEffectTag** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e410` | ClientUpdatePacketAddEffectTag::ClientUpdatePacketAddEffect… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x10) (family 0x26, sub-opcode 16), installs ClientUpdatePacketAddEffectTag::vftable … |
| `0089e470` | ClientUpdatePacketAddEffectTag::~ClientUpdatePacketAddEffec… | confirmed | Destructor body: writes ClientUpdatePacketAddEffectTag::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e4d0` | ClientUpdatePacketAddEffectTag::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor for the family-38 sub-16 packet: restores ClientUpdatePacketAddEffectTag::vftable, BaseClientUpdatePacket::vftable and BasePack… |

**ClientUpdatePacketAddProfileAbilitySetApl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089dfc0` | ClientUpdatePacketAddProfileAbilitySetApl::ClientUpdatePack… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0xf) (family 0x26, sub-opcode 15), installs ClientUpdatePacketAddProfileAbilitySetApl… |
| `0089e020` | ClientUpdatePacketAddProfileAbilitySetApl::~ClientUpdatePac… | confirmed | Destructor body: writes ClientUpdatePacketAddProfileAbilitySetApl::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and … |
| `0089e080` | ClientUpdatePacketAddProfileAbilitySetApl::ScalarDeletingDe… | confirmed | MSVC scalar deleting destructor for the family-38 sub-15 packet: restores ClientUpdatePacketAddProfileAbilitySetApl::vftable, BaseClientUpdatePacket::vftable a… |

**ClientUpdatePacketCoinCount** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e670` | ClientUpdatePacketCoinCount::ClientUpdatePacketCoinCount 🌐 | confirmed | Packet constructor taking one value: calls the client-update base constructor FUN_0089d180(0x13) (family 0x26, sub-opcode 19), installs ClientUpdatePacketCoinC… |
| `0089e6e0` | ClientUpdatePacketCoinCount::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketCoinCount vtable is … |

**ClientUpdatePacketCollectionAddEntry** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089db30` | ClientUpdatePacketCollectionAddEntry::ClientUpdatePacketCol… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(10) (family 0x26, sub-opcode 10), installs ClientUpdatePacketCollectionAddEntry::vfta… |
| `0089dba0` | ClientUpdatePacketCollectionAddEntry::~ClientUpdatePacketCo… | confirmed | Destructor body: writes ClientUpdatePacketCollectionAddEntry::vftable, then the embedded icon member's base vtable at param_1[4] (Loggable::vftable), then Base… |
| `0089dc10` | ClientUpdatePacketCollectionAddEntry::ScalarDeletingDestruc… | likely | Thin scalar deleting destructor: calls FUN_0089dba0 (the full destructor, which restores the packet vtable and the embedded icon member vtable) and then calls … |

**ClientUpdatePacketCollectionRemove** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089da00` | ClientUpdatePacketCollectionRemove::ClientUpdatePacketColle… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(9), which installs BasePacket::vftable, writes family opcode 0x26 into param_1[1] and… |
| `0089da60` | ClientUpdatePacketCollectionRemove::~ClientUpdatePacketColl… | confirmed | Destructor body: writes ClientUpdatePacketCollectionRemove::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns… |
| `0089dac0` | ClientUpdatePacketCollectionRemove::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor for the family-38 sub-9 collection-remove packet: writes ClientUpdatePacketCollectionRemove::vftable, BaseClientUpdatePacket::v… |

**ClientUpdatePacketCollectionRemoveEntry** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089dc30` | ClientUpdatePacketCollectionRemoveEntry::ClientUpdatePacket… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0xb) (family 0x26, sub-opcode 11) and then installs ClientUpdatePacketCollectionRemov… |
| `0089dc90` | ClientUpdatePacketCollectionRemoveEntry::~ClientUpdatePacke… | confirmed | Destructor body: writes ClientUpdatePacketCollectionRemoveEntry::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and re… |
| `0089dcf0` | ClientUpdatePacketCollectionRemoveEntry::ScalarDeletingDest… | confirmed | MSVC scalar deleting destructor for the family-38 sub-11 packet: restores ClientUpdatePacketCollectionRemoveEntry::vftable, BaseClientUpdatePacket::vftable and… |

**ClientUpdatePacketCollectionStart** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089d990` | ClientUpdatePacketCollectionStart::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor of the family-38 sub-8 collection-start packet: writes ClientUpdatePacketCollectionStart::vftable, then BaseClientUpdatePacket:… |

**ClientUpdatePacketDeleteProfile** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e220` | ClientUpdatePacketDeleteProfile::ClientUpdatePacketDeletePr… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x14) (family 0x26, sub-opcode 20), installs ClientUpdatePacketDeleteProfile::vftable… |
| `0089e280` | ClientUpdatePacketDeleteProfile::~ClientUpdatePacketDeleteP… | confirmed | Destructor body: writes ClientUpdatePacketDeleteProfile::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e2e0` | ClientUpdatePacketDeleteProfile::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor for the family-38 sub-20 packet: restores ClientUpdatePacketDeleteProfile::vftable, BaseClientUpdatePacket::vftable and BasePac… |

**ClientUpdatePacketDoneSendingPreloadCharacters** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e8c0` | ClientUpdatePacketDoneSendingPreloadCharacters::ClientUpdat… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1a) (family 0x26, sub-opcode 26), installs ClientUpdatePacketDoneSendingPreloadChar… |
| `0089e920` | ClientUpdatePacketDoneSendingPreloadCharacters::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived preload-marker vtable is not restored … |

**ClientUpdatePacketImmediateActivationCheck** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089ea30` | ClientUpdatePacketImmediateActivationCheck::ClientUpdatePac… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1e) (family 0x26, sub-opcode 30), zeroes the dword at +0xc through the 9-byte helpe… |
| `0089ea90` | ClientUpdatePacketImmediateActivationCheck::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the cla… |

**ClientUpdatePacketImmediateActivationFailed** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089eae0` | ClientUpdatePacketImmediateActivationFailed::ClientUpdatePa… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x1d) (family 0x26, sub-opcode 29), zeroes the dword at +0xc through FUN_0101cd40 and… |
| `0089eb40` | ClientUpdatePacketImmediateActivationFailed::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the cla… |

**ClientUpdatePacketRemoveEffectTag** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e540` | ClientUpdatePacketRemoveEffectTag::ClientUpdatePacketRemove… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x11) (family 0x26, sub-opcode 17), installs ClientUpdatePacketRemoveEffectTag::vftab… |
| `0089e5a0` | ClientUpdatePacketRemoveEffectTag::~ClientUpdatePacketRemov… | confirmed | Destructor body: writes ClientUpdatePacketRemoveEffectTag::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns. |
| `0089e600` | ClientUpdatePacketRemoveEffectTag::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor for the family-38 sub-17 packet: restores ClientUpdatePacketRemoveEffectTag::vftable, BaseClientUpdatePacket::vftable and BaseP… |

**ClientUpdatePacketSetGrandfatheredStatus** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e970` | ClientUpdatePacketSetGrandfatheredStatus::ClientUpdatePacke… 🌐 | confirmed | Packet constructor taking one byte: calls the client-update base constructor FUN_0089d180(0x1b) (family 0x26, sub-opcode 27), installs ClientUpdatePacketSetGra… |
| `0089e9e0` | ClientUpdatePacketSetGrandfatheredStatus::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived vtable is not restored because the cla… |

**ClientUpdatePacketUpdateActionBarSlot** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e7f0` | ClientUpdatePacketUpdateActionBarSlot::ClientUpdatePacketUp… 🌐 | confirmed | Packet constructor that also builds an embedded ActionBarSlot: calls the client-update base constructor FUN_0089d180(0x19) (family 0x26, sub-opcode 25), stores… |
| `0089e860` | ClientUpdatePacketUpdateActionBarSlot::Dtor | likely | Destructor-shaped cleanup: destroys the embedded ActionBarSlot member at +0x18 by calling FUN_0101cd70 with that member as this (it rewrites ActionBarSlot::vft… |

**ClientUpdatePacketUpdateLocation** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089dd60` | ClientUpdatePacketUpdateLocation::ClientUpdatePacketUpdateL… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0xc) (family 0x26, sub-opcode 12), installs ClientUpdatePacketUpdateLocation::vftable… |
| `0089ddc0` | ClientUpdatePacketUpdateLocation::~ClientUpdatePacketUpdate… | confirmed | Destructor body: writes ClientUpdatePacketUpdateLocation::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and returns; … |
| `0089de20` | ClientUpdatePacketUpdateLocation::ScalarDeletingDestructor | confirmed | MSVC scalar deleting destructor for the family-38 sub-12 location packet: restores ClientUpdatePacketUpdateLocation::vftable, BaseClientUpdatePacket::vftable a… |

**ClientUpdatePacketUpdateProfileAbilitySetApl** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e0f0` | ClientUpdatePacketUpdateProfileAbilitySetApl::ClientUpdateP… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0x18) (family 0x26, sub-opcode 24), installs ClientUpdatePacketUpdateProfileAbilitySe… |
| `0089e150` | ClientUpdatePacketUpdateProfileAbilitySetApl::~ClientUpdate… | confirmed | Destructor body: writes ClientUpdatePacketUpdateProfileAbilitySetApl::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object a… |
| `0089e1b0` | ClientUpdatePacketUpdateProfileAbilitySetApl::ScalarDeletin… | confirmed | MSVC scalar deleting destructor for the family-38 sub-24 packet: restores ClientUpdatePacketUpdateProfileAbilitySetApl::vftable, BaseClientUpdatePacket::vftabl… |

**ClientUpdatePacketUpdateProfileExperience** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089de90` | ClientUpdatePacketUpdateProfileExperience::ClientUpdatePack… 🌐 | confirmed | Packet constructor: calls the client-update base constructor FUN_0089d180(0xe) (family 0x26, sub-opcode 14) and then installs ClientUpdatePacketUpdateProfileEx… |
| `0089def0` | ClientUpdatePacketUpdateProfileExperience::~ClientUpdatePac… | confirmed | Destructor body: writes ClientUpdatePacketUpdateProfileExperience::vftable, then BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object and … |
| `0089df50` | ClientUpdatePacketUpdateProfileExperience::ScalarDeletingDe… | confirmed | MSVC scalar deleting destructor for the family-38 sub-14 packet: restores ClientUpdatePacketUpdateProfileExperience::vftable, BaseClientUpdatePacket::vftable a… |

**ClientUpdatePacketUpdateProfileRank** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0089e350` | ClientUpdatePacketUpdateProfileRank::ClientUpdatePacketUpda… 🌐 | confirmed | Packet constructor taking two values: calls the client-update base constructor FUN_0089d180(0x12) (family 0x26, sub-opcode 18), installs ClientUpdatePacketUpda… |
| `0089e3c0` | ClientUpdatePacketUpdateProfileRank::Dtor | likely | Destructor-shaped cleanup: writes BaseClientUpdatePacket::vftable, then BasePacket::vftable into the object; the derived ClientUpdatePacketUpdateProfileRank vt… |

**(no class)** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c23e50` | RequestFriendsPositions 🌐 | confirmed | Constructs CommandPacketFriendsPositionRequest and sends it through the gateway wrapper when the client command service is active. |
| `00925550` | BoundedReaderRunBody_0091b080 🌐 | likely | Builds a bounded reader over the buffer (param_2 start, param_3 length), runs 0091b080 on the record at param_1 through it, and returns 1 when the reader error… |
| `00938cf0` | GatePositionBaselineRefresh | likely | Reads elapsed time since the saved timestamp. If elapsed is at most 200 ms, returns before either position callback. For elapsed 201 through 5000 ms, calls the… |
| `00921460` | ReadRecord0xe4_Keep | speculative | Allocates a 0xe4-byte owner object through 00d833a6 and initialises it with 00ae1f70 (or uses 0 when the allocation fails), reads fixed fields from the reader … |
| `00921510` | ReadRecord0xe4_Discard | speculative | Allocates a 0xe4-byte owner object through 00d833a6, initialises it with 00ae1f70, reads fixed fields from the reader param_1 through 00911d60, destroys the ob… |
| `00910fb0` | — | unknown | If arg1 and arg2 are nonzero: builds cursor {arg2,arg2,arg2+arg3,arg3}; thiscall FUN_008d81b0 on arg1 with &cursor; thiscall FUN_008dadd0 on arg1+0xc with &cur… |

### `net.login`

**ClientAccountManager** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00715ec0` | ClientAccountManager::Run | confirmed | Per-frame state machine of the account manager: loops while the state dword at this+0x22270 is below 0xfd and switches on it (cases 3,5,6,7,8,10,0xb..0x34), dr… |
| `00719330` | ClientAccountManager::OnAccountInfo | confirmed | Result callback of the account-info query (this, context, result, payload pointer). For result 1 it copies payload[0], payload[1] and payload[2] into this+0x22… |
| `00719550` | ClientAccountManager::OnRequestCharacterList | confirmed | Reply callback for the account character list (this, context, result, list pointer). For result 1 or 3 it clears the selected-character pointer at *(this+0x222… |
| `007198b0` | — | unknown | Character-delete reply callback (this, flag byte, guid low dword, guid high dword). It always logs OnRequestCharacterDelete: result=0x%0x and Guid=%llu. When t… |

**ClientLoginManager** (14)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00952780` | ClientLoginManager::ResetFlags10to13 | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 2): clears the bytes at this+0x10, +0x11, +0x12 and +0x13. |
| `009527a0` | ClientLoginManager::ReportUnableToConnect | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 3): clears the byte at this+0x11, then calls virtual slot 0x40 of the service found through 00769710 (+4) wi… |
| `009527c0` | ClientLoginManager::OnForcedDisconnect | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 6, two args): gets the service from 00769710; when the object at [svc+4]+0x280 reports nonzero through its v… |
| `00952be0` | ClientLoginManager::ForwardCompletion7 | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 7, five params where param_5 points to two dwords): clears the byte at +0x13 and calls vtable slot 3 (+0xc) … |
| `009532f0` | ClientLoginManager::Ctor | confirmed | Thiscall ctor (ECX = this, arg = pointer stored at +8): installs the Login::ExternalLoginHandler vtable and then ClientLoginManager::vftable at [this]; sets th… |
| `00953390` | ClientLoginManager::DtorBody | confirmed | Dtor body (ECX = this): stores ClientLoginManager::vftable; when the object at +4 is non-null releases it through its vtable slot 0 with flag 1; releases the I… |
| `00953bc0` | ClientLoginManager::ScalarDeletingDtor | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 0, flag arg): calls 00953390 (ClientLoginManager::DtorBody), frees this when bit 0 of the flag is set, and r… |
| `009542d0` | ClientLoginManager::OnLoginAuthReply | confirmed | Thiscall virtual (ClientLoginManager::vftable slot 4, five args): logs the authentication result with LoginServer authentication=%d; on failure reports Login S… |
| `009566c0` | ClientLoginManager::OnPacketD3 🌐 | confirmed | Checks that the supplied buffer contains at least one 16-bit word and that the first word equals 0xd3; on a match, calls 00956100 with the manager receiver and… |
| `00952830` | ClientLoginManager::ClearFlag13 | likely | Thiscall virtual (ClientLoginManager::vftable slot 14): clears the byte at this+0x13. |
| `00956100` | ClientLoginManager::ParseNameSuggestionReply 🌐 | likely | Parses the supplied byte range, builds a temporary five-slot NameSuggestion list, calls manager virtual slot 4 with decoded values, then destroys the temporary… |
| `00952820` | ClientLoginManager::Slot13ServiceProbe | speculative | Virtual (ClientLoginManager::vftable slot 13): calls 00769710 and discards the result; returns nothing. |
| `00952c10` | — | unknown | Thiscall virtual (ClientLoginManager::vftable slot 9): clears the byte at +0x13, calls vtable slot 6 (+0x18) of the object at this+8 with param_4, param_5 and … |
| `00954160` | — | unknown | Virtual (ClientLoginManager::vftable slot 5, no args): calls 006f9e60 with the address of the IString at this+0x28. |

**Engine::NetworkQueryUserNameRegistration** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124edb0` | Engine::NetworkQueryUserNameRegistration::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'QueryUserNameRegistration : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:273'. |
| `0124edc0` | Engine::NetworkQueryUserNameRegistration::ScalarDeletingDtor | confirmed | Slot [0] of Engine::NetworkQueryUserNameRegistration::vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argume… |
| `0124ed20` | Engine::NetworkQueryUserNameRegistration::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::NetworkQueryUserNameRegistration::vftable into the object; when the int argument … |
| `0124ed90` | Engine::NetworkQueryUserNameRegistration::TypeName | likely | Slot [1] of Engine::NetworkQueryUserNameRegistration::vftable. Returns the constant C string 'QueryUserNameRegistration'. |

**Engine::NetworkRequestPasswordLogin** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124e870` | Engine::NetworkRequestPasswordLogin::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'RequestPasswordLogin : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:144'. |
| `0124e880` | Engine::NetworkRequestPasswordLogin::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `0124e7e0` | Engine::NetworkRequestPasswordLogin::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable and then Engine::NetworkRequestPasswordLogin::vftable into the object; when the int argument i… |
| `0124e850` | Engine::NetworkRequestPasswordLogin::TypeName | likely | Slot [1] of Engine::NetworkRequestPasswordLogin::vftable. Returns the constant C string 'RequestPasswordLogin'. |
| `0124f610` | Engine::NetworkRequestPasswordLogin::ExecuteCredentialFields | likely | Slot [2] of Engine::NetworkRequestPasswordLogin::vftable. Calls FUN_012e0ba0, gets the thread-owned object from FUN_0124f2e0; if non-null it takes FUN_012eb250… |

**Engine::NetworkRequestSessionIDLogin** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0124f270` | Engine::NetworkRequestSessionIDLogin::DebugName | confirmed | Slot [4] of its vftable. Returns the constant C string 'RequestSessionIDLogin : 0 : ..\common\coreclient\engine\network\networkhandlers.cc:130'. |
| `0124f280` | Engine::NetworkRequestSessionIDLogin::ScalarDeletingDtor | confirmed | Slot [0] of its vftable. Stores Scaleform::IScaleformActionHandler::vftable into *this and, when bit 0 of the byte argument is set, calls _free(this) through t… |
| `0124f1e0` | Engine::NetworkRequestSessionIDLogin::ctor | likely | Returns this. Stores Scaleform::IScaleformActionHandler::vftable then Engine::NetworkRequestSessionIDLogin::vftable into the object; when the int argument is n… |
| `0124f250` | Engine::NetworkRequestSessionIDLogin::TypeName | likely | Slot [1] of Engine::NetworkRequestSessionIDLogin::vftable. Returns the constant C string 'RequestSessionIDLogin'. |
| `0124f520` | — | unknown | Slot [2] of Engine::NetworkRequestSessionIDLogin::vftable. Calls FUN_012e0ba0, gets the thread-owned object from FUN_0124f2e0; if non-null it takes the C strin… |

**ExternalLoginApi** (9)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fa9f80` | ExternalLoginApi::CharacterDeleteRequest 🌐 | confirmed | thiscall(this, arg2 key pair): copies two dwords from arg2 into a message with opcode dword 9; mode 0 calls 00fa7bb0 (TCP binary send), mode 1 calls 00fa8ca0 w… |
| `00faa0a0` | ExternalLoginApi::CharacterTransferRequest 🌐 | confirmed | thiscall(this, arg2, arg3, arg4): builds a message with opcode dword 0x12 from arg2 and arg3 (or the optional arg4 object when non-null); mode 0 calls 00fa7f70… |
| `00fa9110` | — 🌐 | unknown | thiscall(this, arg2, arg3): local opcode dword = 3; mode at this+4: mode 0 calls 00fa7900(&opcode) and then, when arg2 is above zero, calls the vtable slot +0x… |
| `00fa91a0` | — 🌐 | unknown | fastcall(this): local opcode dword = 0xb; mode at this+4: mode 0 calls 00fa7c80(&opcode) (header-only TCP send); mode 1 calls 00fa3370(&opcode, CharacterSelect… |
| `00fa91e0` | — 🌐 | unknown | fastcall(this): local opcode dword = 0xd; mode at this+4: mode 0 calls 00fa7e60(&opcode) (header-only TCP send); mode 1 calls 00fa34d0(&opcode, ServerListReque… |
| `00fa9620` | — 🌐 | unknown | fastcall(this): if this+0x18 (credential count) is below 1 and no credentials are set in this+0xc, logs the no-credentials message and calls vtable +0x14 on th… |
| `00fa9db0` | — 🌐 | unknown | thiscall(this, payload array arg2, server id arg3): builds a message with opcode dword 5, copies the payload bytes (arg2 data at +4, length +8) into a temporar… |
| `00fa9eb0` | — 🌐 | unknown | thiscall(this, arg2 pointer pair, arg3, arg4): builds a message with opcode dword 7 and the two dwords from arg2 plus arg3/arg4; mode 0 calls 00fa7ae0 (TCP bin… |
| `00fa9fe0` | — 🌐 | unknown | thiscall(this, arg2, arg3): builds a message with opcode dword 0x10 and the two dwords arg2/arg3; mode 0 calls 00fa7d90 (TCP binary send), mode 1 calls 00fa8e2… |

**Login::ExternalLoginApi** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fa2d40` | Login::ExternalLoginApi::DtorBody | confirmed | Fastcall with this in ECX. Stores the Login::ExternalLoginApi vtable at this, then releases two embedded SoeUtil::IStringFixed<char,32> members: the one at +0x… |
| `00fa3c90` | Login::ExternalLoginApi::~ExternalLoginApi | confirmed | Thiscall with one byte flag argument (ret 4). Calls 00fa2d40 (the ExternalLoginApi destructor body) on this, then calls _free(this) when bit 0 of the flag is s… |

**Login::ExternalLoginHandler** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00952680` | Login::ExternalLoginHandler::ScalarDeletingDtor | confirmed | Thiscall virtual (Login::ExternalLoginHandler::vftable slot 0, one flag byte arg): stores the vftable at [this], frees this with _free (00d82463) when bit 0 of… |
| `00952670` | Login::ExternalLoginHandler::VtableInit | likely | Fastcall (ECX = this): stores Login::ExternalLoginHandler::vftable at [this] and returns. No other field is written; the same body appears as a bare constructo… |

**Login::ExternalLoginTcpApi** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fa2fd0` | Login::ExternalLoginTcpApi::DtorBody | confirmed | Fastcall with this in ECX, with an SEH frame. Stores the Login::ExternalLoginTcpApi vtable at this, at this+4 (0187a704) and at this+0xa88 (0187a6e8). Calls 00… |
| `00fa97d0` | Login::ExternalLoginTcpApi::OnIncomingPacket 🌐 | confirmed | Virtual method (TCP vtable slot 16): feeds the incoming buffer through 0101adc0 into a string, reads the leading packet name, and dispatches by string compare:… |
| `00fa9c60` | Login::ExternalLoginTcpApi::OnConnectThunk | confirmed | Adjustor thunk: adds 0xa88 to ECX (bytes 81 c1 88 0a 00 00) and jumps to 00fa9620 (e9 rel32). |
| `00fa3070` | AdjustThunk_00fa3070 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa3fb0 (the TcpApi scalar deleting destructor). |
| `00fa3080` | AdjustThunk_00fa3080 | likely | Adjustor thunk: sub ecx,4 then jmp 00fa3fb0 (the TcpApi scalar deleting destructor). Followed by three int3 padding bytes. |
| `00fa3090` | AdjustThunk_00fa3090 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa3050 (a thunk to 010d7f60). |
| `00fa30a0` | AdjustThunk_00fa30a0 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa3040 (a thunk to 010d8170). |
| `00fa30b0` | AdjustThunk_00fa30b0 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa0b00 (a thunk to 010d8350). |
| `00fa30c0` | AdjustThunk_00fa30c0 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa0af0 (a thunk to 010d8780). |
| `00fa30d0` | AdjustThunk_00fa30d0 | likely | Adjustor thunk: sub ecx,0xa88 then jmp 00fa3060 (a thunk to 010d7f70). |
| `00fa3fb0` | Login::ExternalLoginTcpApi::~ExternalLoginTcpApi | likely | Thiscall with one byte flag argument (ret 4). Calls 00fa2fd0 (the TcpApi destructor body) on this, then calls _free(this) when bit 0 of the flag is set, and re… |
| `00fa30e0` | ClearFlagForwardSlot0c_TcpApi | speculative | Fastcall with this in ECX. Stores 0 in the byte at this+0xa90. If the pointer at this+0xa94 is non-null, calls its vtable slot +0xc (thiscall, receiver = that … |
| `00fa3100` | ClearFlagForwardSlot08_TcpApi | speculative | Fastcall with this in ECX. Stores 0 in the byte at this+0xa90. If the pointer at this+0xa94 is non-null, calls its vtable slot +0x8 (thiscall, receiver = that … |

**Login::ExternalLoginUdpApi** (15)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fa2e60` | Login::ExternalLoginUdpApi::DtorBody | confirmed | Fastcall with this in ECX, using an SEH frame. Stores the Login::ExternalLoginUdpApi vtable at this, at +0xc0 (0187a670), at +0xc4 (0187a664) and at +0xbf0 (01… |
| `00fa3da0` | Login::ExternalLoginUdpApi::ExternalLoginUdpApi | confirmed | Thiscall with five stack arguments (ret 0x14). Calls 00d33270 (CryptoBaseApi constructor) with the string 'LoginUdp_6' and the other arguments, then sets field… |
| `00fa3e50` | Login::ExternalLoginUdpApi::~ExternalLoginUdpApi | confirmed | Thiscall with one byte flag argument (ret 4). Calls 00fa2e60 (the UdpApi destructor body) on this, then calls _free(this) when bit 0 of the flag is set, and re… |
| `00fa9c50` | Login::ExternalLoginUdpApi::OnConnectThunk | confirmed | Adjustor thunk: adds 0xbf0 to ECX (bytes 81 c1 f0 0b 00 00) and jumps to 00fa9620 (e9 rel32). |
| `00fa2f10` | AdjustThunk_00fa2f10 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa2ef0 (a thunk to 010d5890). No frame; returns through the target. |
| `00fa2f20` | AdjustThunk_00fa2f20 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa3e50 (the ExternalLoginUdpApi scalar deleting destructor). |
| `00fa2f30` | AdjustThunk_00fa2f30 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa2ee0 (a thunk to 010d5d40). |
| `00fa2f40` | AdjustThunk_00fa2f40 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa0ae0 (a thunk to 010d6950). |
| `00fa2f50` | AdjustThunk_00fa2f50 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa0ad0 (a thunk to 010d6be0). |
| `00fa2f60` | AdjustThunk_00fa2f60 | likely | Adjustor thunk: sub ecx,0xbf0 then jmp 00fa2f00 (a thunk to 010d58a0). |
| `00fa2f70` | AdjustThunk_00fa2f70 | likely | Adjustor thunk: sub ecx,0xc0 then jmp 00fa3e50 (the ExternalLoginUdpApi scalar deleting destructor). |
| `00fa2f80` | AdjustThunk_00fa2f80 | likely | Adjustor thunk: sub ecx,0xc4 then jmp 00fa3e50 (the ExternalLoginUdpApi scalar deleting destructor). |
| `00fa2f90` | ClearFlagForwardSlot0c_UdpApi | speculative | Fastcall with this in ECX. Stores 0 in the byte at this+0xbf8. If the pointer at this+0xbfc is non-null, calls its vtable slot +0xc (thiscall, receiver = that … |
| `00fa2fb0` | ClearFlagForwardSlot08_UdpApi | speculative | Fastcall with this in ECX. Stores 0 in the byte at this+0xbf8. If the pointer at this+0xbfc is non-null, calls its vtable slot +0x8 (thiscall, receiver = that … |
| `00fa9220` | — 🌐 | unknown | Virtual method (UDP vtable slot 18): switches on the first byte of the buffer (arg1, arg2 length; 0 when empty). Case 2 builds a stack Array<unsigned char> and… |

**Login::GameServerData** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00faa7a0` | Login::GameServerData::DestructorBody | confirmed | fastcall(this): resets the vtables down the base chain (RefCounted, GameServerData, IString, IStringFixed<char,256>, IStringFixed<char,64>), releases the four … |
| `00faa970` | Login::GameServerData::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, arg2 byte flags): calls the destructor body 00faa7a0, then frees the object through the CRT free when flag bit 0 is set, and returns … |
| `00faa650` | Login::GameServerData::CompareKeyFields | likely | Virtual slot 2 (thiscall arg2 other, arg3 byte mask): compares the dwords at +8, +0x60, +0x178, +0x1198, +0x17c of both records and the flag byte at +0x1194 un… |
| `00faa710` | — | unknown | Virtual slot 3 (thiscall arg2 other): compares the dwords at +8, +0x60, +0x178, +0x17c and the string fields +0x10, +0x184 (through 0079e880), and the bits of … |
| `00faa8d0` | — | unknown | fastcall(this): installs the RefCounted and GameServerData vtables, sets the refcount word to 1, and initialises the embedded StringFixed<64>, StringFixed<256>… |
| `00faaa30` | — | unknown | Virtual slot 4 (thiscall arg2 source, arg3 byte mask): copies the masked flag bits of +0x1194, the dwords at +0x60, +0x178, +0x17c, +0x1198, and the string mem… |

**LoginCommand** (11)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01441c70` | LoginCommand::LoginCommand | confirmed | Fastcall constructor (this in ECX). Calls the base constructor 0140c090 on this (ECX is unchanged at the call, per bytes) and stores the LoginCommand vtable at… |
| `01441cf0` | LoginCommand::TypeName | confirmed | Virtual slot 11 with no arguments. Returns the address of the literal LoginCommand (018ea6c4). |
| `01441d30` | LoginCommand::~LoginCommand | confirmed | Destructor body (fastcall, this in ECX, no deleting part). Stores LoginCommand::vftable at +0, then calls the base destructor 0140c160 with ECX = this and retu… |
| `01441d90` | LoginCommand::Clone | confirmed | Virtual slot 17 (fastcall, this in ECX). Allocates 4 bytes with operator new (00d833a6); when non-null default-constructs the block with 01441c70; then calls s… |
| `01441e20` | LoginCommand::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls slot 4 on this with (param_2, &param_2) and returns 0 when that fails; calls 0140c2a0 with the reader (bas… |
| `01441e60` | LoginCommand::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls 013c26e0 with the text Starting LoginCommand (a no-op that returns 1, … |
| `01441f30` | LoginCommand::ScalarDeletingDestructor | confirmed | Scalar deleting destructor (thiscall, one stack argument, ret 4). Stores LoginCommand::vftable at +0, calls the base destructor 0140c160 on this, then calls _f… |
| `01441f90` | LoginCommand::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads ECX from the argument (the std::string receiving text) and appends the 13-byte literal Login Command (018ea7… |
| `01441eb0` | LoginCommand::Create | likely | Factory with no arguments. Calls operator new with 4 bytes (00d833a6); returns 0 when that returns null, otherwise default-constructs the block with 01441c70 a… |
| `01441d00` | LoginCommand::GetStaticDword | speculative | Virtual slot 12 with no arguments. Returns the dword stored in the global DAT_01cbe3e8. |
| `01441d10` | LoginCommand::SetStaticDword | speculative | Virtual slot 13 with one argument. Stores the argument into the global dword DAT_01cbe3e8. |

**LoginCommand_DisplayMOTD** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b9b80` | LoginCommand_DisplayMOTD::Read | confirmed | Calls vftable slot 4 (header read) on the stream; then reads, in order, a std::string at this+4 (SerialReadString), an int at this+0x20, an int at this+0x24 (S… |
| `014b9c00` | LoginCommand_DisplayMOTD::Write | confirmed | Calls vftable slot 2 (header write) on the stream, then writes a std::string at this+4 (SerialAppendString), an int at this+0x20 and an int at this+0x24 (Seria… |

**LoginCommand_ForgotPassword** (8)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b9690` | LoginCommand_ForgotPassword::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested. Calls… |
| `014b96e0` | LoginCommand_ForgotPassword::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b9730` | LoginCommand_ForgotPassword::LoginCommand_ForgotPassword | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, stores LoginCommand_ForgotPassword::vftable into *this, set… |
| `014b97c0` | LoginCommand_ForgotPassword::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_ForgotPassword. |
| `014b9800` | LoginCommand_ForgotPassword::~LoginCommand_ForgotPassword | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_ForgotPassword::vftable into *this, frees the buffer at +8 when its capacity (+0x1c)… |
| `014b9880` | LoginCommand_ForgotPassword::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x20-byte block (00d833a6). When non-null it constructs it with 014b9730 and keeps … |
| `014b97d0` | LoginCommand_ForgotPassword::ReadGlobal01cbe744 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe744. |
| `014b97e0` | LoginCommand_ForgotPassword::WriteGlobal01cbe744 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe744. |

**LoginCommand_ForgotPasswordResults** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b90f0` | LoginCommand_ForgotPasswordResults::LoginCommand_ForgotPass… | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, stores LoginCommand_ForgotPasswordResults::vftable into *th… |
| `014b9170` | LoginCommand_ForgotPasswordResults::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_ForgotPasswordResults. |
| `014b91b0` | LoginCommand_ForgotPasswordResults::~LoginCommand_ForgotPas… | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_ForgotPasswordResults::vftable into *this, then calls 01441d30 (login base destructo… |
| `014b92e0` | LoginCommand_ForgotPasswordResults::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0xc-byte block (00d833a6). When non-null it constructs it with 014b90f0 and keeps t… |
| `014b9350` | LoginCommand_ForgotPasswordResults::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_ForgotPasswordResults (00d8d382 with both RTTI descri… |
| `014b93b0` | LoginCommand_ForgotPasswordResults::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested. Calls… |
| `014b9430` | LoginCommand_ForgotPasswordResults::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b9530` | LoginCommand_ForgotPasswordResults::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores LoginCommand_ForgotPasswordResults::vftable into *this, calls 01441d30 (login base destructor body), … |
| `014b9590` | LoginCommand_ForgotPasswordResults::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Login Results' plus newline (length 0xe) through 01241650, calls the base dump thunk 01441e10 (jumps t… |
| `014b9650` | LoginCommand_ForgotPasswordResults::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Results Login Command' (length 0x15) to it through 01241650 (… |
| `014b9240` | LoginCommand_ForgotPasswordResults::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). First calls 012d46a0 (its pseudocode argument is the stack-cookie artifact). It news a 0x14-byte blo… |
| `014b9180` | LoginCommand_ForgotPasswordResults::ReadGlobal01cbe740 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe740. |
| `014b9190` | LoginCommand_ForgotPasswordResults::WriteGlobal01cbe740 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe740. |

**LoginCommand_GetChallenge** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b8ca0` | LoginCommand_GetChallenge::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local receiving the version; its result is tested. Calls 014… |
| `014b8cf0` | LoginCommand_GetChallenge::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b8d50` | LoginCommand_GetChallenge::LoginCommand_GetChallenge | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, stores LoginCommand_GetChallenge::vftable into *this, sets … |
| `014b8de0` | LoginCommand_GetChallenge::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_GetChallenge (mov eax,018ffb0c; ret). |
| `014b8e20` | LoginCommand_GetChallenge::~LoginCommand_GetChallenge | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_GetChallenge::vftable into *this, frees the buffer at +8 when its capacity (+0x1c) e… |
| `014b8ea0` | LoginCommand_GetChallenge::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x20-byte block (00d833a6). When non-null it constructs it with 014b8d50 and keeps … |
| `014b8f90` | LoginCommand_GetChallenge::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b8e20 (the LoginCommand_GetChallenge destructor body) on this, frees this with _free when bit 0 of … |
| `014b8fd0` | LoginCommand_GetChallenge::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_GetChallenge (00d8d382 with both RTTI descriptors); o… |
| `014b9030` | LoginCommand_GetChallenge::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument, ret 4). Appends 'Get Login Challenge' plus newline (length 0x14) through 01241650, calls the base dump thunk 014… |
| `014b90d0` | LoginCommand_GetChallenge::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Get Challenge Login Command' (length 0x1b) to it through 0124… |
| `014b8df0` | LoginCommand_GetChallenge::ReadGlobal01cbe73c | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe73c (mov eax,[01cbe73c]; ret). |
| `014b8e00` | LoginCommand_GetChallenge::WriteGlobal01cbe73c | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe73c. |

**LoginCommand_GuestLogin** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b8770` | LoginCommand_GuestLogin::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b8810` | LoginCommand_GuestLogin::Dump | confirmed | Virtual slot 19 (one stack argument, ret 4). Appends 'Guest Login Challenge' plus newline (length 0x16) to the string argument through 01241650, then calls the… |
| `014b8840` | LoginCommand_GuestLogin::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Guest Login Command' (length 0x13) to it through 01241650 (st… |
| `014b8860` | LoginCommand_GuestLogin::~LoginCommand_GuestLogin | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_GuestLogin::vftable into *this, frees the buffer at +0x14 when its capacity (+0x28) … |
| `014b8910` | LoginCommand_GuestLogin::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_GuestLogin. |
| `014b8970` | LoginCommand_GuestLogin::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested. Then … |
| `014b89e0` | LoginCommand_GuestLogin::LoginCommand_GuestLogin | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, stores LoginCommand_GuestLogin::vftable into *this, calls 0… |
| `014b8ae0` | LoginCommand_GuestLogin::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b8860 (the LoginCommand_GuestLogin destructor body) on this, frees this with _free when bit 0 of th… |
| `014b8b00` | LoginCommand_GuestLogin::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x2c-byte block (00d833a6). When non-null it constructs it with 014b89e0 and keeps … |
| `014b8b70` | LoginCommand_GuestLogin::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_GuestLogin (00d8d382 with both RTTI descriptors); on … |
| `014b8920` | LoginCommand_GuestLogin::ReadGlobal01cbe738 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe738. |
| `014b8930` | LoginCommand_GuestLogin::WriteGlobal01cbe738 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe738. |

**LoginCommand_RegisterUsername** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `013c15d0` | LoginCommand_RegisterUsername::LoginCommand_RegisterUsername | confirmed | Constructor body (fastcall, this in ecx): calls FUN_01441c70 for the base part, stores the LoginCommand_RegisterUsername vtable at +0, initialises two empty st… |
| `013c1670` | LoginCommand_RegisterUsername::GetName | confirmed | Virtual slot [11] of the class: returns the literal LoginCommand_RegisterUsername (bytes b8 64 04 8e 01 c3: mov eax,0x018e0464; ret). |
| `013c1840` | LoginCommand_RegisterUsername::ScalarDeletingDtor | confirmed | Virtual slot [0] of the class (bytes 56 8b f1 / e8 / f6 44 24 08 01 / 74 09 / 56 / e8 / 83 c4 04 / 8b c6 / 5e c3): saves esi, calls the destructor body FUN_013… |
| `013c18a0` | LoginCommand_RegisterUsername::CopyFieldsTo | confirmed | Virtual slot [18] of the class: dynamic-casts param_2 from NetworkCommand to LoginCommand_RegisterUsername via FUN_00d8d382 (RTTI_Type_Descriptor pair); on fai… |
| `013c1920` | LoginCommand_RegisterUsername::TraceFields | confirmed | Virtual slot [19] of the class: logs the line Register Username Login Command (level 0x20), calls thunk_FUN_0140c290 on param_2, then formats and logs Username… |
| `013c1a30` | LoginCommand_RegisterUsername::TraceName | confirmed | Virtual slot [20] of the class: calls FUN_01241650 with the text Register Username Login Command and the level 0x1f, then returns (bytes 8b 4c 24 04 / 6a 1f / … |
| `013c14a0` | LoginCommand_RegisterUsername::Deserialize | likely | Virtual slot [9] of the class: calls base slot [4] with (param_2, &param_2), then FUN_01441e20(param_2), then reads a string into this+4 (FUN_013c28d0). When t… |
| `013c1680` | LoginCommand_RegisterUsername::GetSharedDword | speculative | Virtual slot [12] of the class: returns the 32-bit global DAT_01cbdd78 (bytes a1 78 dd cb 01 c3: mov eax,[0x01cbdd78]; ret). Slot [13] writes the same global. |
| `013c1690` | LoginCommand_RegisterUsername::SetSharedDword | speculative | Virtual slot [13] of the class: stores its 32-bit argument into the global DAT_01cbdd78 (bytes 8b 44 24 04 a3 78 dd cb 01 c2 04 00: mov eax,[esp+4]; mov [0x01c… |
| `013c1750` | LoginCommand_RegisterUsername::CreateDefaultAndSink | speculative | Virtual slot [17] of the class: allocates 0x40 bytes with FUN_00d833a6; when that succeeds it constructs a LoginCommand_RegisterUsername with FUN_013c15d0 (oth… |
| `013c1550` | — | unknown | Virtual slot [10] of the class: calls base slot [2] with param_2, writes the trace text Starting LoginCommand_RegisterUsername with FUN_013c26e0, calls FUN_014… |
| `013c16b0` | — | unknown | Destructor body (fastcall): re-stores the LoginCommand_RegisterUsername vtable at +0, releases the heap buffers of the strings at +0x20 (capacity at +0x38) and… |

**LoginCommand_RegisterUsernameResults** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b8170` | LoginCommand_RegisterUsernameResults::LoginCommand_Register… | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, then stores LoginCommand_RegisterUsernameResults::vftable i… |
| `014b81f0` | LoginCommand_RegisterUsernameResults::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_RegisterUsernameResults. |
| `014b8230` | LoginCommand_RegisterUsernameResults::~LoginCommand_Registe… | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_RegisterUsernameResults::vftable into *this, then calls 01441d30 (login base destruc… |
| `014b8360` | LoginCommand_RegisterUsernameResults::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0xc-byte block (00d833a6). When non-null it constructs it with 014b8170 and keeps t… |
| `014b83d0` | LoginCommand_RegisterUsernameResults::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_RegisterUsernameResults (00d8d382 with both RTTI desc… |
| `014b8430` | LoginCommand_RegisterUsernameResults::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested. Calls… |
| `014b84b0` | LoginCommand_RegisterUsernameResults::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b85b0` | LoginCommand_RegisterUsernameResults::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Stores LoginCommand_RegisterUsernameResults::vftable into *this, calls 01441d30 (login base destructor body)… |
| `014b8610` | LoginCommand_RegisterUsernameResults::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Login Results' plus newline (length 0xe) through 01241650, calls the base dump thunk 01441e10 (jumps t… |
| `014b86d0` | LoginCommand_RegisterUsernameResults::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Results Login Command' (length 0x15) to it through 01241650 (… |
| `014b82c0` | LoginCommand_RegisterUsernameResults::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). First calls 012d46a0 (its pseudocode argument is the stack-cookie artifact). It news a 0x14-byte blo… |
| `014b8200` | LoginCommand_RegisterUsernameResults::ReadGlobal01cbe734 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe734. |
| `014b8210` | LoginCommand_RegisterUsernameResults::WriteGlobal01cbe734 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe734. |

**LoginCommand_RespondToChallenge** (12)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b6ff0` | LoginCommand_RespondToChallenge::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0 (a no-op), calls 014… |
| `014b74a0` | LoginCommand_RespondToChallenge::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Respond to Login Challenge' plus newline (length 0x1b) through 01241650, calls the base dump thunk 014… |
| `014b77a0` | LoginCommand_RespondToChallenge::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Respond To Challenge Login Command' (length 0x22) to it throu… |
| `014b77c0` | LoginCommand_RespondToChallenge::~LoginCommand_RespondToCha… | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_RespondToChallenge::vftable into *this, then tidies its ten string members in revers… |
| `014b79f0` | LoginCommand_RespondToChallenge::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_RespondToChallenge. |
| `014b7f40` | LoginCommand_RespondToChallenge::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b77c0 (the LoginCommand_RespondToChallenge destructor body) on this, frees this with _free when bit… |
| `014b7f60` | LoginCommand_RespondToChallenge::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x138-byte block (00d833a6). When non-null it constructs it with 014b7cd0 and keeps… |
| `014b7a00` | LoginCommand_RespondToChallenge::ReadGlobal01cbe730 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe730. |
| `014b7a10` | LoginCommand_RespondToChallenge::WriteGlobal01cbe730 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe730. |
| `014b7a50` | — | unknown | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested. Then … |
| `014b7cd0` | — | unknown | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, stores LoginCommand_RespondToChallenge::vftable into *this,… |
| `014b7fd0` | — | unknown | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_RespondToChallenge (00d8d382 with both RTTI descripto… |

**LoginCommand_Results** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b6590` | LoginCommand_Results::~LoginCommand_Results | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_Results::vftable into *this, frees the buffer at +0x2c when its capacity (+0x40) exc… |
| `014b6650` | LoginCommand_Results::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_Results. |
| `014b6690` | LoginCommand_Results::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), passes trace text to 013c26e0, calls 01441e60 (log… |
| `014b6890` | LoginCommand_Results::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b6590 (the LoginCommand_Results destructor body) on this, frees this with _free when bit 0 of the f… |
| `014b68d0` | LoginCommand_Results::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Login Results' plus newline (length 0xe) through 01241650, calls the base dump thunk 01441e10 (jumps t… |
| `014b6a90` | LoginCommand_Results::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Results Login Command' (length 0x15) to it through 01241650 (… |
| `014b6ab0` | LoginCommand_Results::LoginCommand_Results | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, then stores LoginCommand_Results::vftable into *this, zeroe… |
| `014b6b40` | LoginCommand_Results::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x44-byte block (00d833a6). When non-null it constructs it with 014b6ab0 and keeps … |
| `014b6bb0` | LoginCommand_Results::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument). dynamic_casts the NetworkCommand argument to LoginCommand_Results (00d8d382 with both RTTI descriptors); on nul… |
| `014b6cd0` | LoginCommand_Results::Read | confirmed | Virtual slot 9 (thiscall, one stack argument, ret 4). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version; its result is tested… |
| `014b6460` | LoginCommand_Results::CreateEventAndQueue | likely | Virtual slot 15 (fastcall, no stack arguments, returns 1). First calls 012d46a0 (its pseudocode argument is the stack-cookie artifact). When the byte at this+0… |
| `014b6660` | LoginCommand_Results::ReadGlobal01cbe72c | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe72c. |
| `014b6670` | LoginCommand_Results::WriteGlobal01cbe72c | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe72c. |

**LoginCommand_SendChallenge** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014b5a40` | LoginCommand_SendChallenge::Read | confirmed | Virtual slot 9 (thiscall, one stack argument). Calls vtable slot 4 (+0x10) with the argument and a local that receives the version word; returns 0 when that fa… |
| `014b5ac0` | LoginCommand_SendChallenge::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls vtable slot 2 (+0x8) with the argument (class header), calls 01441e60 (login base write), writes the stri… |
| `014b5b10` | LoginCommand_SendChallenge::LoginCommand_SendChallenge | confirmed | Fastcall constructor (ECX=this) under an EH frame. Calls 01441c70 (login base constructor) on this, then stores LoginCommand_SendChallenge::vftable into *this,… |
| `014b5bb0` | LoginCommand_SendChallenge::GetTypeName | confirmed | Virtual slot 11 (no arguments). Returns a pointer to the literal LoginCommand_SendChallenge. |
| `014b5bf0` | LoginCommand_SendChallenge::~LoginCommand_SendChallenge | confirmed | Fastcall destructor body (ECX=this) under an EH frame. Stores LoginCommand_SendChallenge::vftable into *this. Tidies two std::string members: frees the buffer … |
| `014b5c90` | LoginCommand_SendChallenge::CloneNew | confirmed | Virtual slot 17 (fastcall, no stack arguments). Under an EH frame it news a 0x40-byte block (00d833a6). When non-null it constructs it with 014b5b10 and keeps … |
| `014b5d80` | LoginCommand_SendChallenge::ScalarDeletingDtor | confirmed | Virtual slot 0 (thiscall, one byte flag argument). Calls 014b5bf0 (the LoginCommand_SendChallenge destructor body) on this, frees this with _free when bit 0 of… |
| `014b6290` | LoginCommand_SendChallenge::Clone | confirmed | Virtual slot 18 (thiscall, one stack argument, ret 4). dynamic_casts the NetworkCommand argument to LoginCommand_SendChallenge (00d8d382 with both RTTI descrip… |
| `014b62f0` | LoginCommand_SendChallenge::Dump | confirmed | Virtual slot 19 (thiscall, one stack argument). Appends 'Send Login Challenge' plus newline (length 0x15) through 01241650, calls the base dump thunk 01441e10 … |
| `014b63a0` | LoginCommand_SendChallenge::AppendDescription | confirmed | Virtual slot 20 (one stack argument, ret 4). Loads the argument into ECX and appends the literal 'Send Challenge Login Command' (length 0x1c) to it through 012… |
| `014b5bc0` | LoginCommand_SendChallenge::ReadGlobal01cbe728 | speculative | Virtual slot 12 (no arguments). Returns the 32-bit global at 01cbe728. |
| `014b5bd0` | LoginCommand_SendChallenge::WriteGlobal01cbe728 | speculative | Virtual slot 13 (one stack argument, ret 4). Stores the argument into the 32-bit global at 01cbe728. |
| `014b5da0` | — | unknown | Virtual slot 15 (fastcall, no stack arguments). Under an EH frame it reads a state object from 012efb10 (string at +4 with length +0x14 and capacity +0x18) and… |

**LoginCommand_SendPassword** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `014440e0` | LoginCommand_SendPassword::Write | confirmed | Virtual slot 10 (thiscall, one stack argument). Calls slot 2 on this with param_2, calls the LoginCommand base write 01441e60, then writes in order: string at … |
| `01444900` | LoginCommand_SendPassword::DumpText | confirmed | Virtual slot 19 (thiscall, one stack argument: the std::string receiving text). Appends the line Send Password Login Command with a newline through 01241650, c… |
| `01444bc0` | LoginCommand_SendPassword::AppendDisplayName | confirmed | Virtual slot 20 (one stack argument, ret 4; bytes verified). Loads ECX from the argument (the std::string receiving text) and appends the 27-byte literal Send … |
| `01445180` | LoginCommand_SendPassword::~LoginCommand_SendPassword | confirmed | Destructor body: stores LoginCommand_SendPassword::vftable, releases the heap buffers of the string members (each guarded by capacity > 15), calls the map clea… |
| `01445380` | LoginCommand_SendPassword::GetName | confirmed | Returns the address of the literal LoginCommand_SendPassword (6 bytes of code, no callees). |
| `01445420` | LoginCommand_SendPassword::Deserialize | confirmed | Virtual slot 9 reader. Calls this vtable slot 4 (+0x10) with the reader and a pointer to the stack argument, which receives the version. Then reads by version:… |
| `01445600` | LoginCommand_SendPassword::LoginCommand_SendPassword | confirmed | Constructor (ECX = this): stores LoginCommand_SendPassword::vftable, empties the string members at +0x4 through +0xf4 and the map at +0x3c through 01445590, se… |
| `01445860` | LoginCommand_SendPassword::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01445180, then calls _free on this only when bit 0 of the argument byte is set, and retur… |
| `01445880` | LoginCommand_SendPassword::CreateDefaultAndSink | confirmed | Virtual slot 17: allocates 0x110 bytes through 00d833a6, constructs a default LoginCommand_SendPassword there with 01445600 (result 0 when the allocation fails… |
| `014458f0` | LoginCommand_SendPassword::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts param_2 from NetworkCommand to LoginCommand_SendPassword through 00d8d382; on failure reports the loginCommand assert at line 0x… |
| `01445390` | LoginCommand_SendPassword::GetSharedDword | likely | Returns the 32-bit global DAT_01cbe480 (mov eax,[DAT_01cbe480]; ret). |
| `014453a0` | LoginCommand_SendPassword::SetSharedDword | likely | Stores its 32-bit argument into the global DAT_01cbe480 and returns. |
| `01445a30` | LoginCommand_SendPassword::CreateDefault | likely | Allocates 0x110 bytes through 00d833a6 and constructs a default LoginCommand_SendPassword there through 01445600; returns the new object, or 0 when the allocat… |

**LoginCommand_SendSessionID** (13)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01445b70` | LoginCommand_SendSessionID::Serialize | confirmed | Virtual slot 10 writer (stack argument = writer): calls vtable slot 2 (+0x8) and the base body writer 01441e60, then appends the string at +0x4 and the int map… |
| `01445eb0` | LoginCommand_SendSessionID::TraceFields | confirmed | Virtual slot 19: formats nine lines with _sprintf (SessionID, Platform, Match Challenge Key, Is Parent Session ID as %d from the flag at +0x64, Login Client, L… |
| `01446160` | LoginCommand_SendSessionID::TraceName | confirmed | Virtual slot 20: appends the 29-byte text Send Session ID Login Command through 01241650. |
| `01446180` | LoginCommand_SendSessionID::~LoginCommand_SendSessionID | confirmed | Destructor body: stores LoginCommand_SendSessionID::vftable, releases the heap buffers of the string members (each guarded by capacity > 15), calls the keyed-m… |
| `01446350` | LoginCommand_SendSessionID::GetName | confirmed | Returns the address of the literal LoginCommand_SendSessionID (no callees). |
| `014463b0` | LoginCommand_SendSessionID::Deserialize | confirmed | Virtual slot 9 reader (stack argument = reader). Calls vtable slot 4 (+0x10) for the header and version, then the base body 01441e20, the string at +0x4, and b… |
| `01446740` | LoginCommand_SendSessionID::ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): calls the destructor body 01446180, then calls _free on this when bit 0 of the argument byte is set, and returns th… |
| `01446760` | LoginCommand_SendSessionID::CreateDefaultAndSink | confirmed | Virtual slot 17: allocates 0xf4 bytes through 00d833a6, constructs a default LoginCommand_SendSessionID there with 01446500 (result 0 on allocation failure), t… |
| `014467d0` | LoginCommand_SendSessionID::CopyFieldsTo | confirmed | Virtual slot 18: dynamic-casts param_2 from NetworkCommand to LoginCommand_SendSessionID through 00d8d382; on failure reports the loginCommand assert at line 0… |
| `01446360` | LoginCommand_SendSessionID::GetSharedDword | likely | Returns the 32-bit global DAT_01cbe484. |
| `01446370` | LoginCommand_SendSessionID::SetSharedDword | likely | Stores its 32-bit argument into the global DAT_01cbe484. |
| `014468f0` | LoginCommand_SendSessionID::CreateDefault | likely | Allocates 0xf4 bytes through 00d833a6 and constructs a default LoginCommand_SendSessionID there with 01446500; returns the new object, or 0 when the allocation… |
| `01446500` | — | unknown | Constructor (ECX = this): calls 01441c70 for the base, stores LoginCommand_SendSessionID::vftable, empties the string members (+0x4, +0x30, +0x4c, +0x6c, +0x88… |

**LoginService** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012efb70` | LoginService::ctor | confirmed | Fastcall, ECX = this: stores LoginService::vftable at +0, then initialises eleven string-shaped members (each with length 0 and capacity 0xf, buffers at +0x08,… |
| `012efc90` | LoginService::DtorBody | confirmed | Fastcall, ECX = this: stores LoginService::vftable at +0; if the global DAT_01cbd960 equals this it clears the global; then for each of the eleven string-shape… |
| `012eff30` | LoginService::ScalarDeletingDtor | confirmed | Thiscall with a flag byte: calls the destructor body 012efc90 and frees this only when flag bit 0 is set, then returns this (ret 4). |

**PlayClient** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0090e6a0` | PlayClient::HandleInitializationParameters 🌐 | confirmed | Parses initialization parameters, compares the received environment string with the client's configured environment, and logs mismatches to ClientCriticalError… |
| `00938a50` | PlayClient::CompleteConnectionSetup 🌐 | confirmed | Handles the ready connection state, logs connection or player-data failures, records download statistics, constructs initialization details, and tunnels a gene… |

**SoeUtil::List<Login::ClientGameServerData,-1>** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00fa87d0` | ReadClientGameServerList | confirmed | stdcall-like (arg1 buffer, arg2 length, callback arg3): sets a List<Login::ClientGameServerData> with vtable and a 0xe tag, reads it through 00fa8540; if the r… |

**(no class)** (117)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00953f80` | LoginServerConnect | confirmed | Thiscall (ECX = login handler; arg2 = host text, arg3 = secondary text; ret 0xc): returns 1 when the busy byte at +0x11 is already set. Otherwise it assigns th… |
| `00fa4940` | ReadForceDisconnect 🌐 | confirmed | Thiscall with two stack arguments (a value and the callback). Opens the ForceDisconnect element with 01017500, reads the Reason int with 00fa2200 into a local,… |
| `00fa5ce0` | ReadCharacterCreateReplyEvent6 🌐 | confirmed | Thiscall with the event flag in ECX, a value argument and a callback. Opens the CharacterCreateReply element with 01017500, reads Status with 00fa2260 into the… |
| `00fa7730` | ReadCharacterSelectInfoReply 🌐 | confirmed | stdcall(arg1, callback), ret 8: builds a stack result {Array<uchar,0,1>, List<Login::EntityDetails>}, parses the input with 00fa6e30(arg1, "CharacterSelectInfo… |
| `00fa8880` | ReadServerUpdate 🌐 | confirmed | stdcall(arg1, callback): BeginStruct(ServerUpdate name) via 01017500, reads the Server sub-structure through 00fa7120(arg1, Server), EndStruct via 01016e90(0),… |
| `00fa8e20` | SendNamedServerIdPayloadDeref 🌐 | confirmed | thiscall(this=transport, msg, name): BeginStruct(name) via 01017250, writes ServerId (msg+4) through 00fa5290 or 00fa5150 (flag byte), writes Payload through 0… |
| `00fa94b0` | ReadServerListReply 🌐 | confirmed | stdcall(arg1, callback): BeginStruct(ServerListReply) via 01017500, reads the Servers list of Server entries through 00fa8210 into a local List<Login::ClientGa… |
| `009526e0` | LoginHandlerResetConnection | likely | Thiscall (ECX = this, one arg): when the pointer at this+4 is non-null, calls 00fa9110(arg, 1), then calls the function pointer in the object's vtable slot 0 w… |
| `00952720` | LoginHandlerConnectionCall1000 | likely | Fastcall (ECX = this): when the connection object at this+4 is non-null, calls its vtable slot 2 (+8) with the constant 1000. |
| `00954940` | WriteRecordDwordBytesSub40 | likely | Thiscall (ECX = record, arg = writer): writes the sub-object at +0 through 00911c30 (length-prefixed bytes), then writes the dword at +0x34 (4 bytes), the byte… |
| `00954b40` | WriteShortAndFiveSubobjects | likely | Thiscall (ECX = record, arg = writer): writes the 16-bit value stored at record+4 through 007cc400 (2 bytes), then writes the five sub-objects at +8, +0x3c, +0… |
| `00fa4370` | ReadEntityRecord | likely | Thiscall with the record in ECX (fields at +8 EntityKey (int64 via 00b85b20), +0x10 ServerId (int via 00b85ac0), +0x14 Status (int via 00fa22c0), +0x18 Payload… |
| `00fa43e0` | ReadEntityRecordNoServer | likely | Thiscall with the record in ECX (EntityKey int64 at +8 via 00b85b20, Status int at +0x10 via 00fa2320, Payload base64 text at +0x14 via 00fa4210). Opens the el… |
| `00fa4490` | ReadEntityResultAppData | likely | Thiscall with the record in ECX: Entity int64 at +8 (via 00b85b20 under the name Entity), Server int at +0x10 (via 00b85ac0), Result int at +0x18 (via 00fa23e0… |
| `00fa4bd0` | ReadLoginStatusRecord | likely | Thiscall with the record in ECX (bool at +4, int at +8, bool at +0xc, optional payload pointer at +0x10), a reader state byte at arg, and a begin-struct argume… |
| `00fa5800` | BoundedReadByteDwordDispatch | likely | Thiscall (this in ECX) with three stack arguments: start, length and a callback (ret 0xc). Reads a byte and a dword with 00fa3630 into an event record {byte, d… |
| `00fa61e0` | ReadEntityDetailsList | likely | Cdecl-style with a flag byte, a list-begin name, a list-element name and an element-begin name. Opens the outer element (01017500), clears the list with 00fa60… |
| `00fa6440` | ReadStateDescriptionRecord | likely | Thiscall with the record in ECX, a flag byte and a begin-struct argument. Opens the element (01017500), reads PopulationLevel-style and NameId-style ints with … |
| `00fa6d10` | WriteSessionFingerprintFields | likely | Thiscall with the record in ECX (Session string at +4, Fingerprint string at +0x38), a flag byte at arg+1 and a name argument. Opens the element with 01017250(… |
| `00fa6e30` | ReadEntitiesRecord | likely | Thiscall with the record in ECX (Status int at +4, CanBypassServerLock bool at +8, Entities list at +0xc), a flag byte at arg and an element-name argument. Ope… |
| `00fa7120` | ReadAllowedAccessRecord | likely | Thiscall with the record in ECX (AllowedAccess bool at +4), a flag byte at arg and a begin-struct name argument. Opens the element with 01017500, reads the sub… |
| `00fa74c0` | WriteEntityKeyServerIdPayload | likely | Thiscall with the record in ECX (EntityKey at +8, ServerId at +0x10, payload record pointer at +0x14) and a writer argument; the flag byte at writer+1 selects … |
| `00fa75e0` | WriteEntityServerAppData | likely | Thiscall with the record in ECX (Entity at +8, Server at +0x10, AppData payload pointer at +0x14) and a writer argument whose flag byte selects the formatter. … |
| `00fa7ae0` | SendCharacterLoginRequest 🌐 | likely | Serialises a message through 00fa6980 into an 8 KB packet writer, sends the bytes with transport vtable +0x50(data,len,1), then destroys the writer (009545e0). |
| `00fa7d90` | SendTunnelAppPacketClientToServer 🌐 | likely | Serialises a message through 00fa6b10 into an 8 KB packet writer, sends the bytes with transport vtable +0x50(data,len,1), then destroys the writer (009545e0). |
| `00fa8210` | ReadServerEntryList | likely | 5 stack args: if BeginStruct(arg2) succeeds, loops while BeginStruct(arg4) succeeds, calling 00fa1910 and 00fa7120(arg1, arg5) for each entry, then EndStruct (… |
| `01445070` | ReadInt32PairMap | likely | Deserialises a map: ECX holds a holder whose first dword is the reader; the stack argument is the target map. Reads an element count with 013c2690, then for ea… |
| `01445aa0` | RegisterFactory0x19A | likely | Static initialiser: registers the factory 01445a30 under key 0x19a by calling 013ce130 with (0x19a, factory) and then 013ce230 with the same key and factory. |
| `01445c20` | StringGetCopyAt2C | likely | Returns a copy of the std::string at this+0x2c through the hidden return buffer param_2: initialises the buffer (size 0, capacity 15, first byte 0) and assigns… |
| `01445c90` | StringGetCopyAt48 | likely | Returns a copy of the std::string at this+0x48 through the hidden return buffer param_2 (same body as 01445c20 with offset 0x48). |
| `01445d00` | StringGetCopyAtA0 | likely | Returns a copy of the std::string at this+0xa0 through the hidden return buffer param_2 (same body as 01445c20 with offset 0xa0). |
| `01445d70` | StringGetCopyAtBC | likely | Returns a copy of the std::string at this+0xbc through the hidden return buffer param_2 (same body as 01445c20 with offset 0xbc). |
| `01445de0` | StringGetCopyAtD8 | likely | Returns a copy of the std::string at this+0xd8 through the hidden return buffer param_2 (same body as 01445c20 with offset 0xd8). |
| `01446960` | RegisterFactory0x19B | likely | Static initialiser: registers the factory 014468f0 under key 0x19b by calling 013ce130 with (0x19b, factory) and then 013ce230 with the same key and factory. |
| `014b5d60` | RegisterFactoryKey0x4f | likely | Pushes factory 014b5d00 and key 0x4f, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b6c90` | RegisterFactoryKey0x51 | likely | Pushes factory 014b6c30 and key 0x51, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b8150` | RegisterFactoryKey0x50 | likely | Pushes factory 014b80e0 and key 0x50, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b8590` | RegisterFactoryKey0x199 | likely | Pushes factory 014b8530 and key 0x199, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `014b8c60` | RegisterFactoryKey0x143 | likely | Pushes factory 014b8c00 and key 0x143, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `014b8f70` | RegisterFactoryKey0x4e | likely | Pushes factory 014b8f10 and key 0x4e, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Returns… |
| `014b9510` | RegisterFactoryKey0x1b3 | likely | Pushes factory 014b94b0 and key 0x1b3, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `014b9950` | RegisterFactoryKey0x1b2 | likely | Pushes factory 014b98f0 and key 0x1b2, calls 013ce130, then calls 013ce230 with ECX set to the value 013ce130 returned and the same two stack arguments. Return… |
| `00952740` | ConnectionForward_fa91a0 | speculative | Fastcall: when the pointer at this+4 is non-null, tail-jumps to 00fa91a0 with ECX set to that pointer; otherwise returns. |
| `00952750` | ConnectionForward_fa91e0 | speculative | Fastcall: when the pointer at this+4 is non-null, tail-jumps to 00fa91e0 with ECX set to that pointer; otherwise returns. |
| `00952ba0` | LoginRequestBegin | speculative | Thiscall (ECX = this, two dword args): when the connection at this+4 is non-null and the byte at +0x13 is clear, sets +0x13 to 1, passes the two args as a pair… |
| `00fa0790` | AllocObject11acOrNull | speculative | Fastcall with this in ECX. Clears the byte at this+4 (dword index 1), sets up an SEH frame, calls operator new (00d833a6) with 0x11ac bytes, and when that retu… |
| `00fa08c0` | InitWord15ThenHolder | speculative | Fastcall with this in ECX. Stores 0xf at [this], calls 00fa0790 with this+4 as its receiver, and returns this. |
| `00fa0920` | DispatchEventThenVirtual14 | speculative | Thiscall with one stack argument (ret 4). Stores the byte at arg+4 into this+8. If this+0xc is non-null, calls its vtable slot +0x10 with (this, byte at arg+4,… |
| `00fa0970` | ForwardToListenerVirtual18 | speculative | Thiscall with one stack argument (ret 4). If this+0xc is non-null, calls its vtable slot +0x18 with (this, dword at arg+4), then clears the byte at this+8. |
| `00fa0cb0` | InitHolderReturnSelf | speculative | Fastcall with its argument in ECX. Calls 00fa0790 with the same ECX and returns the argument unchanged in EAX. |
| `00fa5150` | FormatFieldWriteCtx_5150 | speculative | Format-and-write helper with a context value as its first stack argument (kept in a local) and a pointer to one value as its second. Formats the value with 004… |
| `00fa5290` | FormatFieldWriteTagged_5290 | speculative | Same shape as 00fa5150 (format one value with 00402fd0 into a local StringFixed<2048>, copy to a heap buffer), writing with 01017330(context, buffer, length, 0… |
| `00fa53e0` | FormatFieldWriteB_53e0 | speculative | Cdecl with one pointer argument whose first two dwords are formatted by 00402fd0 with the format string at 0175b2f8 into a local SoeUtil::StringFixed<2048>. Co… |
| `00fa5520` | FormatFieldWriteCtx2_5520 | speculative | Format-and-write helper with a context first stack argument and a pointer to two values: formats both with 00402fd0 (format string 0175b2f8) into a local Strin… |
| `00fa5670` | FormatFieldWriteTagged2_5670 | speculative | Same shape as 00fa5520 (two formatted values with 00402fd0, format string 0175b2f8, into a local StringFixed<2048>), writing with 01017330(context, buffer, len… |
| `00fa5880` | BoundedReadByteDwordQwordDispatch6 | speculative | Thiscall with three stack arguments (start, length, callback; ret 0xc). Builds an event record whose first dword is 6, second 0, and two dwords from DAT_01cb4d… |
| `00fa7080` | ReadCountedNodeList | speculative | Cdecl-style (plain ret) with a cursor record as argument. Clears the list in ECX with 00fa3120, reads a dword count from the cursor and for each entry creates … |
| `00fa8350` | ReadStringThenCallback | speculative | Three stack args (arg1 buffer, arg2 length, arg3 callback): builds a 15-capacity string object from the buffer through 00fa6ec0 and calls arg3(&string) when th… |
| `00fa8540` | ReadCountedElementList | speculative | Reads one byte from the buffer as a count into *arg3, falling back to 0 and clamping the cursor when out of input; then calls 00fa7080(&cursor, arg3+1) to read… |
| `00faa180` | CStringToArrayForwarder | speculative | Plain function taking a NUL-terminated string: measures its length, copies the bytes into a temporary SoeUtil Array<unsigned char,0,1> (through 00701cc0 and 00… |
| `00faa240` | CStringArgToArrayForwarder | speculative | Plain function taking (arg1, NUL-terminated string arg2): measures the string, copies its bytes into a temporary Array<unsigned char,0,1> (00701cc0, 00d2e5d0),… |
| `00faa310` | CStringToArray8KForwarder | speculative | Two C-string args (param_1, param_2): measures param_2, builds a temporary SoeUtil Array<unsigned char,8192,1> with an 8 KB inline buffer through 00d2e5d0, the… |
| `00faa420` | CStringTransferForwarder | speculative | Takes (arg1, arg2, arg3 C string): when arg3 is non-null measures it and copies its bytes into a temporary array (00701cc0, 00d2e5d0); then calls 00faa0a0(arg1… |
| `012efec0` | GetOrCreateGlobal_13c | speculative | Argumentless: if the global DAT_01cbd960 is null it allocates 0x13c bytes (00d833a6), constructs the object with 012efb70 and stores the result in DAT_01cbd960… |
| `013c17c0` | CreateObj0x40Default | speculative | Allocates 0x40 bytes with FUN_00d833a6; when that succeeds it constructs a LoginCommand_RegisterUsername with FUN_013c15d0 and returns it; returns 0 on allocat… |
| `013c1820` | RegisterFactory0x198 | speculative | Pushes the address of FUN_013c17c0 and the key 0x198, calls FUN_013ce130(0x198, FUN_013c17c0), moves the returned object into ecx and calls FUN_013ce230, then … |
| `013c1860` | SetStringAt4 | speculative | Assigns the argument std::string to the string at param_1+0x04 (assign(src, 0, npos); bytes 8b 44 24 04 / 6a ff 6a 00 50 / 83 c1 04 / e8 / c2 04 00 pattern wit… |
| `013c1880` | SetStringAt20 | speculative | Assigns the argument std::string to the string at param_1+0x20 (assign(src, 0, npos)). |
| `01445e50` | SetStringAt4 | speculative | Assigns the std::string argument to the string member at this+0x4 (assign(src,0,npos) through 00420ff1). |
| `014b5d00` | AllocConstructObject0x40 | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x40. When the result is non-null it returns the value of 014b5b10 applied to that block; otherwise … |
| `014b6c30` | AllocConstructObject0x44 | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x44. When the result is non-null it returns the value of 014b6ab0 applied to that block; otherwise … |
| `014b6f40` | CopyFourDwordsToThis4 | speculative | Thiscall with one stack argument (ret 4). Copies the four dwords at the argument (16 bytes) into this+4, +8, +0xc and +0x10. |
| `014b70d0` | CopyStringFieldOut0x3c | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string (capacity 0xf, size 0, first byte 0), then assigns… |
| `014b7140` | CopyStringFieldOut0x58 | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0x58 into it through 00420ff1 (… |
| `014b71b0` | CopyStringFieldOut0x74 | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0x74 into it through 00420ff1 (… |
| `014b7220` | CopyStringFieldOut0xac | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0xac into it through 00420ff1 (… |
| `014b7290` | CopyStringFieldOut0xe4 | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0xe4 into it through 00420ff1 (… |
| `014b7300` | CopyStringFieldOut0x100 | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0x100 into it through 00420ff1 … |
| `014b7370` | CopyStringFieldOut0x11c | speculative | Thiscall with one stack argument (a std::string result, returned). Initialises the result as an empty string, then assigns this+0x11c into it through 00420ff1 … |
| `014b73e0` | AssignStringField0x90 | speculative | Thiscall with one stack argument (a std::string, ret 4). Assigns the whole argument string into this+0x90 through 00420ff1. |
| `014b7420` | AssignStringField0x3c | speculative | Thiscall with one stack argument (a std::string, ret 4). Assigns the whole argument string into this+0x3c through 00420ff1. |
| `014b7440` | AssignStringField0x58 | speculative | Thiscall with one stack argument (a std::string, ret 4). Assigns the whole argument string into this+0x58 through 00420ff1. |
| `014b7460` | AssignStringField0x74 | speculative | Thiscall with one stack argument (a std::string, ret 4). Assigns the whole argument string into this+0x74 through 00420ff1. |
| `014b80e0` | AllocConstructObject0x138 | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x138. When the result is non-null it returns the value of 014b7cd0 applied to that block; otherwise… |
| `014b8530` | AllocConstructObject0xc | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0xc. When the result is non-null it returns the value of 014b8170 applied to that block; otherwise r… |
| `014b8c00` | AllocConstructObject0x2c | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x2c. When the result is non-null it returns the value of 014b89e0 applied to that block; otherwise … |
| `014b8f10` | AllocConstructObject0x20 | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x20. When the result is non-null it returns the value of 014b8d50 applied to that block; otherwise … |
| `014b94b0` | AllocConstructObject0xc | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0xc. When the result is non-null it returns the value of 014b90f0 applied to that block; otherwise r… |
| `014b98f0` | AllocConstructObject0x20 | speculative | Under an EH frame, calls 00d833a6 (operator new) with size 0x20. When the result is non-null it returns the value of 014b9730 applied to that block; otherwise … |
| `00954690` | — | unknown | Thiscall (ECX = record, arg = bounded byte cursor): reads two length-prefixed blocks through 00894b10, a length-prefixed block into record+0x6c through 009538e… |
| `00954bd0` | — | unknown | Thiscall (ECX = record, arg = writer): writes four sub-objects at +0, +0x24, +0x48 and +0x6c through 00911c30, then eight dwords at +0x90, +0x94, +0x98, +0x9c,… |
| `00956230` | — 🌐 | unknown | When the manager service is available, copies its supplied byte range into temporary SoeUtil byte-array storage, passes the bytes to 00fa9fe0 for outbound Tunn… |
| `009564a0` | — 🌐 | unknown | When the service exists, constructs a local PacketCheckNameRequest, copies the supplied name strings, filters request rows through 0070d300, sends the request … |
| `00956590` | — 🌐 | unknown | When the manager is present and its guard byte is clear, copies a manager-owned byte range into a temporary byte array, passes that array and the caller value … |
| `00fa4e70` | — | unknown | Cdecl-style with three arguments: param_3 is a byte record (data pointer at +4, length at +8). Base64-encodes the bytes into a local SoeUtil::StringFixed<8192>… |
| `00fa5020` | — | unknown | Cdecl with one pointer argument whose first dword is formatted by 00402fd0 with the format string at 01770978 into a local SoeUtil::StringFixed<2048>. Copies t… |
| `00fa5910` | — | unknown | Cdecl-style bounded wrapper (plain ret): builds a local event whose first dword is 8 and an embedded SoeUtil::Array<unsigned char,0,1> (vptr at local+0x10), re… |
| `00fa5a00` | — | unknown | Cdecl-style bounded wrapper: builds an event whose first dword is 10 and an embedded SoeUtil::Array<unsigned char,0,1>, reads with 00fa37f0 over [start, start+… |
| `00fa5af0` | — | unknown | Cdecl-style bounded wrapper: builds an event whose first dword is 0x11 (17) and an embedded SoeUtil::Array<unsigned char,0,1>, reads with 00fa38a0 over [start,… |
| `00fa5bd0` | — | unknown | Cdecl-style bounded wrapper (plain ret): builds an event whose first dword is 0x13 (19) and an embedded SoeUtil::Array<unsigned char,0,1>, reads with 00fa3920 … |
| `00fa5d80` | — 🌐 | unknown | Cdecl-style (plain ret) with the record in its first argument and a callback in its second. Builds an event whose first dword is 8 and an embedded SoeUtil::Arr… |
| `00fa5e50` | — 🌐 | unknown | Cdecl-style (plain ret) with the record in its first argument and a callback in its second. Builds an event whose first dword is 10 and an embedded SoeUtil::Ar… |
| `00fa5f20` | — 🌐 | unknown | Cdecl-style (plain ret) with the record in its first argument and a callback in its second. Opens the TunnelAppPacketServerToClient element with 01017500, read… |
| `00fa6000` | — | unknown | Cdecl-style (plain ret) with the record in its first argument and a callback in its second. Builds an event whose first dword is 0x13 (19) and an embedded SoeU… |
| `00fa7650` | — | unknown | Cdecl-style (plain ret) with a start, a length and a callback. Builds an event record whose first dword is 0xc, zeroes a second dword and an embedded SoeUtil::… |
| `00fa7830` | — 🌐 | unknown | thiscall(this=transport, msg), ret 4: creates an 8 KB packet writer (00953c20), serialises msg through 00fa6800 (byte at msg+0, length-prefixed fields at msg+4… |
| `00fa7900` | — 🌐 | unknown | thiscall(this=transport, byte*): creates an 8 KB packet writer, appends the single byte *arg2 (dword value truncated to a byte), sends the writer bytes with tr… |
| `00fa7a10` | — 🌐 | unknown | Serialises a message through 00fa68c0 into an 8 KB packet writer, sends the bytes with transport vtable +0x50(data,len,1), then destroys the writer (009545e0). |
| `00fa7bb0` | — 🌐 | unknown | Serialises a message through 00fa6a50 into an 8 KB packet writer, sends the bytes with transport vtable +0x50(data,len,1), then destroys the writer (009545e0). |
| `00fa7c80` | — 🌐 | unknown | thiscall(this=transport, byte*): creates an 8 KB packet writer, appends the single byte *arg2, sends the writer bytes with transport vtable +0x50(data,len,1), … |
| `00fa7e60` | — 🌐 | unknown | thiscall(this=transport, byte*): creates an 8 KB packet writer, appends the single byte *arg2, sends the writer bytes with transport vtable +0x50(data,len,1), … |
| `00fa7f70` | — | unknown | Serialises a message through 00fa6c20 into an 8 KB packet writer, sends the bytes with transport vtable +0x50(data,len,1), then destroys the writer (009545e0). |
| `00fa80c0` | — 🌐 | unknown | thiscall(this=transport, arg2 unused, arg3=name): serialises the request named by arg3 through 00fa6d10 into a stack 8 KB StringFixed, then calls vtable +0x44 … |
| `00fa89b0` | — 🌐 | unknown | thiscall(this=transport, msg, name): BeginStruct(name) via 01017250, writes ServerId (dword at msg+4) through 00fa5290 or 00fa5150 (chosen by a flag byte), wri… |
| `00fa8b50` | — 🌐 | unknown | thiscall(this=transport, msg, name): serialises the request named by arg3 through 00fa74c0 into a stack 8 KB StringFixed, then calls transport vtable +0x44 wit… |
| `00fa8ca0` | — 🌐 | unknown | thiscall(this=transport, msg, name): BeginStruct(name) via 01017250, writes the EntityKey field (msg+8) through 00fa5670 or 00fa5520 (chosen by a flag byte), t… |
| `00fa8fc0` | — | unknown | thiscall(this=transport, msg, name): serialises the request named by arg3 through 00fa75e0 into a stack 8 KB StringFixed, then calls transport vtable +0x44(dat… |

### `net.namechange`

**NameChange::BaseNameChangePacket** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a14f10` | BaseNameChangePacket_Dtor | confirmed | Destructor of the name-change packet base: runs the EH-protected base teardown FUN_0070a790 (which releases the packet's IString member), then rewrites the obj… |
| `00a14f70` | BaseNameChangePacket_Dtor_00a14f70 | confirmed | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a14fd0` | BaseNameChangePacket_Dtor_00a14fd0 | confirmed | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a15030` | BaseNameChangePacket_Dtor_00a15030 | confirmed | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |
| `00a15090` | BaseNameChangePacket_Dtor_00a15090 | confirmed | Byte-identical copy of the BaseNameChangePacket destructor: base teardown FUN_0070a790, then vtable reset to NameChange::BaseNameChangePacket and BasePacket. |

**NameChange::ChangeNameRequestPacket** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16ab0` | ChangeNameRequestPacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a14fd0 and frees the packet with _free when bit 0 of the flag argument is s… |
| `00a16ad0` | ChangeNameRequestPacket_CtorWithName 🌐 | confirmed | Calls FUN_00a0ff60(3), stores type and identifier dwords at +0xc through +0x14, installs the packet vftable, calls FUN_0070bac0 with ECX=this+0x18 to initializ… |
| `00a16a30` | — 🌐 | unknown | Constructor: base sub-opcode 3, then stores the guid pair and the trailing flag from the arguments (fields at +0xc through +0x14) and initialises the name memb… |

**NameChange::CheckNamePacket** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16000` | CheckNamePacket_Ctor 🌐 | confirmed | Calls the base packet constructor FUN_00a0ff60(1), installs NameChange::CheckNamePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the … |
| `00a16060` | CheckNamePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0, bytes 568bf1 e8a8eeffff f644240801 7409 56 e8...): calls the BaseNameChangePacket destructor copy 00a14f10 and frees… |
| `00a16990` | CheckNamePacket_CtorWithName 🌐 | confirmed | Calls FUN_00a0ff60(1), stores type and identifier dwords at +0xc through +0x14, installs the packet vftable, calls FUN_0070bac0 with ECX=this+0x18 to initializ… |
| `00a16900` | — 🌐 | unknown | Constructor overload: builds the base with sub-opcode 1, stores the two dwords and the trailing flag handed in as arguments (fields at +0xc through +0x14 and t… |

**NameChange::CheckNameResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16080` | CheckNameResponsePacket_Ctor 🌐 | confirmed | Calls FUN_00a0ff60(2), installs NameChange::CheckNameResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and … |
| `00a160e0` | CheckNameResponsePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a14f70 and frees the packet with _free when bit 0 of the flag argument is s… |

**NameChange::CheckNameStatusResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a161e0` | CheckNameStatusResponsePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a15090 and frees the packet with _free when bit 0 of the flag argument is s… |
| `00a16180` | CheckNameStatusResponsePacket_Ctor 🌐 | likely | Calls FUN_00a0ff60(6), installs NameChange::CheckNameStatusResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded membe… |

**NameChange::NameChangeResponsePacket** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00a16100` | NameChangeResponsePacket_Ctor 🌐 | confirmed | Calls FUN_00a0ff60(4), installs NameChange::NameChangeResponsePacket::vftable, then calls FUN_0070bac0 with ECX=this+0x18 to initialize the embedded member and… |
| `00a16160` | NameChangeResponsePacket_ScalarDeletingDtor | confirmed | Scalar deleting destructor (vtable slot 0): delegates to the base destructor copy 00a15030 and frees the packet with _free when bit 0 of the flag argument is s… |

### `ui.account`

**Engine::AccountAcceptFriendRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f190` | Engine::AccountAcceptFriendRequest::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountAddFriend** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128ec50` | Engine::AccountAddFriend::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountAddIgnore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128eef0` | Engine::AccountAddIgnore::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountLoadMMOPlayerImage** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f550` | Engine::AccountLoadMMOPlayerImage::GetActionName | confirmed | Returns the address of the constant string LoadMMOPlayerImage (6 bytes, mov eax,imm32 then ret). |
| `0128f570` | Engine::AccountLoadMMOPlayerImage::GetSourceTag | confirmed | Returns the address of the constant string LoadMMOPlayerImage : 0 : (accounthandlers.cc line 1192); 6 bytes, mov eax,imm32 then ret. |
| `0128f580` | Engine::AccountLoadMMOPlayerImage::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |
| `0128f4e0` | — | unknown | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountLoadMMOPlayerImage::vftable into *this with EH state 0; when the int argument is nonzero… |

**Engine::AccountRejectFriendRequest** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f2e0` | Engine::AccountRejectFriendRequest::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountRemoveFriend** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128eda0` | Engine::AccountRemoveFriend::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountRemoveIgnore** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f040` | Engine::AccountRemoveIgnore::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountRequestContextMenu** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f390` | Engine::AccountRequestContextMenu::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountRequestContextMenu::vftable into *this with EH state 0; when the int argument is nonzero… |
| `0128f400` | Engine::AccountRequestContextMenu::GetActionName | confirmed | Returns the address of the constant string RequestContextMenu (6 bytes, mov eax,imm32 then ret). |
| `0128f420` | Engine::AccountRequestContextMenu::GetSourceTag | confirmed | Returns the address of the constant string RequestContextMenu : 0 : (accounthandlers.cc line 1168); 6 bytes, mov eax,imm32 then ret. |
| `0128f430` | Engine::AccountRequestContextMenu::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountShowFriends** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128eb00` | Engine::AccountShowFriends::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::AccountUsernameRegistration** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128f730` | Engine::AccountUsernameRegistration::Ctor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable then Engine::AccountUsernameRegistration::vftable into *this with EH state 0; when the int argument is nonze… |
| `0128f7a0` | Engine::AccountUsernameRegistration::GetActionName | confirmed | Returns the address of the constant string UserNameRegistration (6 bytes, mov eax,imm32 then ret). |
| `0128f7c0` | Engine::AccountUsernameRegistration::GetSourceTag | confirmed | Returns the address of the constant string UserNameRegistration : 0 : (accounthandlers.cc line 873); 6 bytes, mov eax,imm32 then ret. |
| `0128f7d0` | Engine::AccountUsernameRegistration::ScalarDeletingDtor | confirmed | Stores Scaleform::IScaleformActionHandler::vftable into *this; when bit 0 of the byte argument is set calls _free(this) through thunk 00d82463, then returns th… |

**Engine::FRTWAccountPostLogin** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283820` | Engine::FRTWAccountPostLogin::CallGetterReturnOne | speculative | Virtual slot [2] (13 bytes, ret 8): calls 012d46a0 with no arguments, ignores its result, and returns 1 in EAX. |

**Engine::FRTWAccountQueryPlayerInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01283980` | Engine::FRTWAccountQueryPlayerInfo::VirtualSlot2 | speculative | Virtual slot [2] (29 bytes, ret 8): calls 012eb290 with the name AccountID and default 0 (result discarded), calls 012d46a0 with no arguments (result discarded… |

**Engine::FRTWAccountRespondDBAvatars** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01289580` | — | unknown | Virtual slot [2] response handler (void, ret 8; the first stack argument is the display action, the second is not read). Argument 0 must be an integer list (ty… |

**Engine::FRTWAccountRespondDBAwards** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `012899c0` | — | unknown | Virtual slot [2] response handler (void, ret 8; the first stack argument is the display action, the second is not read). Argument 0 must be an integer list (ty… |

**Engine::FRTWAccountRespondDBHonors** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `01289e00` | — | unknown | Virtual slot [2] response handler (void, ret 8; the first stack argument is the display action, the second is not read). Argument 0 must be an integer list (ty… |

**Engine::FRTWAccountRespondPlayerAvatars** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128b270` | Engine::FRTWAccountRespondPlayerAvatars::StorePlayerAvatars | confirmed | Virtual slot [2] handler (ret 8, returns 1). Argument 0 integer (map key); arguments 1 to 4 integer lists. Builds one 0xC-byte record per element (00d833a6(0xC… |

**Engine::FRTWAccountRespondPlayerHonors** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128bae0` | Engine::FRTWAccountRespondPlayerHonors::StorePlayerHonors | confirmed | Virtual slot [2] handler (ret 8, returns 1). Argument 0 integer (map key); arguments 1 to 4 integer lists. Builds one 0xC-byte record per element: list 1 value… |

**Engine::FRTWAccountRespondPlayerInfo** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128ab60` | Engine::FRTWAccountRespondPlayerInfo::StorePlayerInfo | confirmed | Virtual slot [2] handler (ret 8, returns 1). Reads display-action arguments through 012f8f60 with type-tag checks: argument 0 integer (map key), arguments 1 to… |

**Engine::FRTWAccountRespondPlayerMatches** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128c350` | — | unknown | Reads argument slots 0 to 15 of the display action with type-tag checks (integer, string list, integer list), builds string-list and 0x90-byte record collectio… |

**Scaleform::IScaleformActionHandler** (10)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128e870` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128ea10` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128eb60` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128ecb0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128ee00` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128ef50` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128f0a0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128f1f0` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128f340` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |
| `0128f490` | Scaleform::IScaleformActionHandler::Ctor | likely | Stores Scaleform::IScaleformActionHandler::vftable into *this and returns; 67-byte body with EH frame and stack cookie around one store and no calls. |

**(no class)** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `0128a850` | InitRecord44 | likely | Fastcall initializer: returns this (eax = ECX) after clearing the string at +0x18 (its first byte at +0x1C, size at +0x2C, capacity 0xF at +0x30) and the dword… |
| `0128a8b0` | InitVectorTriples | likely | Fastcall initializer: returns this (eax = ECX) after zeroing three 3-dword vector headers (begin, end, capacity) at +0xC, +0x1C and +0x2C; makes no calls. |
| `0128a920` | InitThreeStrings | speculative | Fastcall initializer: returns this (eax = ECX) after clearing three std::string members at +4, +0x20 and +0x3C (first byte, size at +0x14/+0x30/+0x4C, capacity… |

### `ui.welcome`

**WelcomeInfo::ClaimCodeDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ace4f0` | GetClaimCodeColumnName | confirmed | Maps indices 0 through 4 to Code, Title, Description, ImageSetId, and TintId; returns <error> for other indices. |
| `00ace930` | GetField230 | confirmed | Returns the dword stored at this+0x230. |
| `00ad0d20` | WelcomeInfo::ClaimCodeDataSource::Ctor | confirmed | Installs ClaimCodeDataSource and entry-array vtables, stores the owner pointer, and zeros the array count, capacity, and pointer. |
| `00ad0db0` | WelcomeInfo::ClaimCodeDataSource::Dtor | confirmed | Restores the entry-array vtable, destroys mixed-string rows, frees array storage, clears the pointer, and calls the embedded base destructor. |
| `00ad0e40` | WelcomeInfo::ClaimCodeDataSource::ScalarDeletingDtor | confirmed | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00acfcf0` | — | unknown | Selects one 0x2c-byte row and handles columns 0 through 4 by clearing a string, resolving two string ids, formatting an integer, or looking up a final id. |

**WelcomeInfo::InfoDataSource** (5)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ace4a0` | GetInfoColumnName | confirmed | Returns Treasure Ticket Count for column 0, Did You Know for column 1, and <error> otherwise. |
| `00acfc20` | GetInfoCellValue | confirmed | For one requested column, returns the localized count for key 0x17a or resolves a row string and formats its identifier as <STRING %d> when unresolved. |
| `00ad0bc0` | WelcomeInfo::InfoDataSource::Ctor | confirmed | Installs InfoDataSource and integer-array vtables, stores the owner, initializes array fields and selection to -1, then loads WelcomeTips.txt. |
| `00ad0c70` | WelcomeInfo::InfoDataSource::Dtor | confirmed | Restores the integer-array vtable, clears its count, frees array storage, zeros the pointer, and calls the embedded base destructor. |
| `00ad0d00` | WelcomeInfo::InfoDataSource::ScalarDeletingDtor | confirmed | Calls the InfoDataSource destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |

**WelcomeInfo::LatestContentDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ace560` | GetLatestContentColumnName | confirmed | Returns Title for column 0, Description for column 1, and <error> otherwise. |
| `00ace940` | GetField230 | confirmed | Returns the dword stored at this+0x230. |
| `00acf890` | GetLatestContentCellValue | confirmed | Reads the selected 8-byte row field, asks the string interface to resolve it, and on lookup failure may format <STRING %d> into the output. |
| `00ad0e60` | WelcomeInfo::LatestContentDataSource::Ctor | confirmed | Installs LatestContentDataSource and entry-array vtables, stores the owner pointer, and zeros the array count, capacity, and pointer. |
| `00ad0ef0` | WelcomeInfo::LatestContentDataSource::Dtor | confirmed | Clears the element count, restores the entry-array vtable, frees array storage, zeros the pointer, and calls the embedded base destructor. |
| `00ad0f80` | WelcomeInfo::LatestContentDataSource::ScalarDeletingDtor | confirmed | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |

**WelcomeInfo::WhatsNewDataSource** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00ace460` | GetWhatsNewColumnName | confirmed | Returns Subject for column 0, Body for column 1, and <error> for other indices. |
| `00ace920` | GetField230 | confirmed | Returns the dword stored at this+0x230. |
| `00ad0a70` | WelcomeInfo::WhatsNewDataSource::Ctor | confirmed | Installs the WhatsNewDataSource and entry-array vtables, stores the owner pointer, zeros array state, and parses the local welcome resource. |
| `00ad0b10` | WelcomeInfo::WhatsNewDataSource::Dtor | confirmed | Restores the entry-array vtable, destroys its wide-string rows, frees array storage, clears the pointer, and calls the embedded base destructor. |
| `00ad0ba0` | WelcomeInfo::WhatsNewDataSource::ScalarDeletingDtor | confirmed | Calls the data-source destructor and frees the object when flag bit 0 is set; otherwise returns the object pointer. |
| `00acf280` | — | unknown | Selects row index times 0x20; for field 0 or 1 it clears the wide string at that row offset and returns 1, otherwise returns 0. |

**(no class)** (19)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00acf550` | LoadWeightedWelcomeTips | confirmed | Loads WelcomeTips.txt, splits non-comment rows at '^', stores each weight and length, sums lengths, then chooses a row with the random helper. |
| `00ad0100` | ParseWelcomeText | confirmed | Loads welcome.txt, skips # lines, adds wide-string rows, handles ':' row markers, and compares '&' row identifiers with current client fields before updating t… |
| `00acf2c0` | DestroyWideStringEntryRange | likely | Destroys wide-string members for each selected 0x20-byte row and reduces the array count by the requested amount. |
| `00acf3a0` | ReallocateWhatsNewEntries | likely | Requests new storage for 0x20-byte entries, constructs or copies existing rows, destroys the old row strings, frees old storage, then updates the array pointer. |
| `00acf470` | DestroyWideAndNarrowStringRange | likely | Destroys each 0x2c-byte row's wide and narrow strings in the selected range, then reduces the row count. |
| `00acf970` | EraseWhatsNewEntries | likely | Destroys the rows after the selected interval, copies later 0x20-byte rows over the removed range, and decreases the array count. |
| `00acfa90` | EnsureWhatsNewEntryAtIndex | likely | Returns the existing row when index is in range; otherwise grows capacity, initializes each newly exposed 0x20-byte row, updates count, and returns the request… |
| `00acfb50` | ReallocateClaimCodeEntries | likely | Requests new storage for 0x2c-byte claim-code entries, initializes/copies each live row, destroys old string members, frees old storage, and updates the array … |
| `00acfe80` | AppendWhatsNewEntry | likely | Grows the 0x20-byte row array if needed, increments its count, initializes a new wide-string entry from the supplied source, and returns its pointer. |
| `00acff50` | EnsureDefaultWhatsNewEntry | likely | Ensures a 0x20-byte row exists at the requested index, returns zero on failure, otherwise initializes the row's two wide-string members and returns the result. |
| `00ad04f0` | AppendClaimCodeSourceEntry | likely | Ensures capacity for one 0x2c-byte row, increments the count, initializes its mixed-string entry from the supplied source, and returns the row pointer. |
| `00ad0720` | InsertWhatsNewEntries | likely | Expands the 0x20-byte entry array if required, shifts the tail rows to open a gap, initializes the inserted rows, and returns the first inserted row pointer. |
| `00ad0790` | ApplyWelcomeScreenPayload | likely | Copies pair rows and mixed-string rows from the supplied payload into two data sources, invokes each source's update slot, and copies three trailing values int… |
| `00ad0a60` | AdjustorThunk_00ad0880 | likely | Loads ECX from the pointer at the incoming ECX address, then jumps directly to 00ad0880. |
| `00ad0fa0` | CreateWelcomeDataSources | likely | Allocates and constructs four WelcomeInfo data sources, stores each through a reference-counted pointer helper, and zeros three trailing fields. |
| `00acf300` | — | unknown | Initializes two wide-string members in a 0x20-byte entry, copies the supplied source into the first member, and returns the entry pointer. |
| `00acf4b0` | — | unknown | Initializes a wide string and a narrow string, copies the wide-string source, then copies three dwords from source offsets +0x10 through +0x18 into the entry. |
| `00ad0580` | — | unknown | Moves a requested number of 0x2c-byte claim-code rows between array positions, constructing destination strings and destroying replaced or removed rows. |
| `00ad0880` | — | unknown | Converts two arguments to wide strings, supplies localized MessageOfTheDay text when the first is empty, compares existing rows, then updates or appends the ma… |

## Source shards

[[S0148 00715ec0-00719d20|S0148]] (4), [[S0219 0089d990-008a3c80|S0219]] (47), [[S0239 0090a890-009105b0|S0239]] (1), [[S0240 009105e0-009161d0|S0240]] (1), [[S0243 00920130-00925ec0|S0243]] (3), [[S0247 00934740-00939200|S0247]] (2), [[S0248 009392c0-0093d740|S0248]] (1), [[S0253|S0253]] (24), [[S0254 00954ec0-0095a710|S0254]] (5), [[S0288 00a13190-00a18a10|S0288]] (18), [[S0322 00acd6d0-00ad3250|S0322]] (42), [[S0386 00c1f190-00c26aa0|S0386]] (1), [[S0543|S0543]] (63), [[S0544|S0544]] (45), [[S0658 0124dfe0-01253ee0|S0658]] (14), [[S0668|S0668]] (2), [[S0669|S0669]] (9), [[S0670 0128c350-01290f70|S0670]] (30), [[S0688|S0688]] (4), [[S0727 013be8e0-013c3b90|S0727]] (16), [[S0751|S0751]] (14), [[S0752|S0752]] (32), [[S0773|S0773]] (124), [[S0774|S0774]] (2), [[S0782|S0782]] (147)
