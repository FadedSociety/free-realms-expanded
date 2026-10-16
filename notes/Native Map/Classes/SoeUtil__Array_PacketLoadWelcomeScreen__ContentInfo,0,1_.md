# Class: SoeUtil::Array<PacketLoadWelcomeScreen::ContentInfo,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bb980` | `AllocateStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ContentInfo,0,1>::All…` | ✅ | virtual | Adjusts requested capacity using the existing capacity field +0x0c and growth policy, returns existing storage when suitable, otherwise allocates param_2 entries at eight bytes each with four-byte alignment. |
| `008bba20` | `FreeStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ContentInfo,0,1>::FreeSto…` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4, then returns. |
| `008bba40` | `SwapOrDestroyStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ContentInfo,0,1>…` | ✅ | virtual | If the destroy flag is set, calls param_2's virtual +0x0c; otherwise swaps storage pointer, capacity and size fields between the two array objects. |
| `008c6e30` | `SoeUtil_ArrayContentInfo_Dtor` | ✅ | dtor | Stores SoeUtil::Array<PacketLoadWelcomeScreen::ContentInfo,0,1>::vftable at this+0, zeroes the count at this+8, frees the buffer at this+4 through FUN_00401c00 with alignment 4, and zeroes this+4. Returns void. |
| `008c9130` | `SoeUtil_ArrayContentInfo_ScalarDeletingDtor` | ✅ | dtor | Stores the Array<PacketLoadWelcomeScreen::ContentInfo,0,1> vtable at this+0, zeroes the count at +8, frees the buffer at +4 through FUN_00401c00 (alignment 4) and zeroes +4. When bit 0 of the stack flag byte is set it f… |
