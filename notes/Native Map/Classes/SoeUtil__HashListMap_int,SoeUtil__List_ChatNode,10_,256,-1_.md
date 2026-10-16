# Class: SoeUtil::HashListMap<int,SoeUtil::List<ChatNode,10>,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0e290` | `HashListMap::FreeNode` | ✅ | virtual | Frees the supplied node pointer with alignment 4. |
| `00b150e0` | `HashListMap_ChatNode_Destruct` | ✅ | dtor | Destructor body for the chat-node hash list map: stores the HashListMap vtable into *this and then repeatedly runs 00b142a0 on the list head at +8, which unlinks that node from the +8/+0xc chain and decrements the count… |
| `00b15110` | `— ⚠reject` | ⚪ | dtor | Slot [0] scalar deleting destructor of the chat-node hash list map: runs the same teardown as 00b150e0 (installs the vtable, drains the node chain at +8 through 00b142a0) and then frees the object with _free when bit 0 … |
| `00b15160` | `HashListMap_ChatNode_DestructSEH` | ✅ | dtor | Second, SEH-framed copy of the chat-node map destructor: sets up an in-function exception scope, installs the HashListMap vtable, drains the node chain at +8 via 00b142a0, and restores the ExceptionList afterwards. Call… |
| `00b152a0` | `HashListMap_ChatNode_Construct` | ✅ | ctor | Constructor for the chat-node hash list map: zeroes the list head, tail, count and the field at +4, memsets the 256 inline bucket slots at +0x14 to zero (0x400 bytes), and stores two vtables during the body, first SoeUt… |
