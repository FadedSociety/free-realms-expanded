# Class: ClientQuickChatManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b95140` | `— ⚠reject` | ⚪ | method | Walks the singly linked row list at this+8 (next pointer at +0x38): for every row it asks the client object at this+0xfec for the row display text (virtual call through the second vtable slot 2 with row+8) into a SoeUti… |
| `00b96050` | `ClientQuickChatManager_RefreshPhraseData` | ✅ | virtual | Virtual method of the quick-chat manager: calls the virtual function at slot 0x38 of the object referenced by this+0xff0, then rebuilds the phrase row labels with 00b95140 on the same this, and finally tail-jumps to 00b… |
| `00b965d0` | `— ⚠reject` | ⚪ | dtor | Destructor of the quick-chat manager: reinstalls ClientQuickChatManager::vftable, destroys the embedded text-index object through 00b96540, releases the three data source pointers at +0xff0, +0xff4 and +0x12988 (calls t… |
| `00b96db0` | `ClientQuickChatManager_Construct` | ✅ | ctor | Constructor of the quick-chat manager: runs the QuickChatManager base constructor FUN_0107c580, stores ClientQuickChatManager::vftable, zeroes the data source and refptr fields (+0xfb4 through +0xfe8 and +0x12988), stor… |
| `00b96f60` | `ClientQuickChatManager_DeletingDestruct` | ✅ | dtor | Scalar deleting destructor: calls the manager destructor 00b965d0 and then _free(this) when bit 0 of param_2 is set. |
