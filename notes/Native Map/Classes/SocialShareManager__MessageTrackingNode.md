# Class: SocialShareManager::MessageTrackingNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00adf030` | `MessageTrackingNodeBaseCtor` | 🟢 | ctor | Initialises the 0x38-byte base of a tracking-map node: zeroes +0/+4/+8, installs SoeUtil::IString<char>::vftable at +0xc with the static empty buffer 0x1be809c, clears the string length/capacity, zeroes +0x20 and +0x24,… |
| `00adf0b0` | `MessageTrackingNodeBaseDtor` | 🟢 | dtor | Destructor body for the node base: frees the owned buffer pointer at +0x30 through _free, restores the IString vtable at +0xc, and when the string length at +0x18 is positive decrements the shared-buffer reference count… |
