# Class: SoeUtil::List<RewardPacketNonMemberRewardGranted::DeferredStars,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b888a0` | `RewardDeferredStarsList_Alloc` | 🟢 | virtual | Allocation hook (vtable slot 2) of the deferred-stars list: calls FUN_006fb9c0, obtains the current memory manager via FUN_006fb860 (virtual slot 1 on it when present, else the manager at DAT_01bdf280, else a global all… |
| `00b888e0` | `RewardDeferredStarsList_Free` | 🟢 | virtual | Deallocation hook (vtable slot 3) of the deferred-stars list: forwards the node pointer to FUN_00401c00 with alignment 4 and returns. |
| `00b88940` | `RewardDeferredStarsList_Append` | 🟢 | method | Appends a node to the list tail: allocates through vtable slot 2 (*param_1+8), writes the old tail (this+8) into node+0xc, clears node+8, links the node both from the old tail and (for an empty list) as head, sets it as… |
| `00b88f80` | `RewardDeferredStarsList_Clear` | 🟢 | method | Empties the deferred-stars list: while the head at this+4 is non-null it unlinks the node (fixing the prev/next fields at node+0xc and node+8 and updating head/tail this+4/this+8), decrements the count at this+0xc and f… |
| `00b89150` | `RewardDeferredStarsList_Destroy` | 🟢 | dtor | Non-virtual list cleanup: reinstalls the list vtable and tail-jumps into the clear routine 00b88f80 (the x86 is a mov vtable store followed by a jmp). |
| `00b89160` | `RewardDeferredStarsList_ScalarDeletingDtor` | 🟢 | dtor | Vtable slot 0 of the deferred-stars list: reinstalls the list vtable, calls the clear routine 00b88f80 and frees the list storage through _free when bit 0 of param_2 is set. |
