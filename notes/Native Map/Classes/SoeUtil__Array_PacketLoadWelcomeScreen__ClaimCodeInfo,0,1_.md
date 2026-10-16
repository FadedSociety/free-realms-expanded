# Class: SoeUtil::Array<PacketLoadWelcomeScreen::ClaimCodeInfo,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbab0` | `AllocateStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ClaimCodeInfo,0,1>::A…` | ✅ | virtual | Adjusts requested capacity using the existing capacity field +0x0c and growth policy, returns existing storage when suitable, otherwise allocates param_2 entries at 0x2c bytes each with four-byte alignment. |
| `008bbb50` | `FreeStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ClaimCodeInfo,0,1>::FreeS…` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4, then returns. |
| `008bbb70` | `SwapOrDestroyStorage → SoeUtil::Array<PacketLoadWelcomeScreen::ClaimCodeInfo,0,…` | ✅ | virtual | If the destroy flag is set, calls param_2's virtual +0x0c; otherwise swaps storage pointer, capacity and size fields between the two array objects. |
