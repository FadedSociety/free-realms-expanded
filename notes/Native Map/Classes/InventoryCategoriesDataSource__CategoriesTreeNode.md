# Class: InventoryCategoriesDataSource::CategoriesTreeNode

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a6d830` | `InventoryCategoriesDataSource::CategoriesTreeNode::KeyNameForFlag` | ✅ | virtual | Returns a pointer to the static string CategoryId when the argument is 0, otherwise to the static string UNKNOWN (20 bytes: cmp dword [esp+4],0; mov eax,CategoryId; je; mov eax,UNKNOWN; ret 4). |
| `00a6e170` | `InventoryCategoriesDataSource::CategoriesTreeNode::CategoriesTreeNode` | ✅ | ctor | Constructor: calls the base constructor 0099ef20, stores param_2 at +0x18, param_4 at +0x1c and param_5 at +0x20, installs the CategoriesTreeNode vtable at offset 0, embeds an IString<char> at +0x24 with empty buffer DA… |
| `00a6e230` | `InventoryCategoriesDataSource::CategoriesTreeNode::NameFieldAddress ?` | 🟡 | virtual | Returns this+0x24, the address of the node's name IString member (lea eax,[ecx+0x24]; ret; 4 bytes). |
| `00a6e240` | `InventoryCategoriesDataSource::CategoriesTreeNode::GetValueAt1c ?` | 🟡 | virtual | Returns the dword at this+0x1c (mov eax,[ecx+0x1c]; ret; 4 bytes). That field is the fourth constructor argument stored by 00a6e170. |
| `00a6e260` | `InventoryCategoriesDataSource::CategoriesTreeNode::ReleaseNameString → Inventor…` | ✅ | function | Releases the IString<char> at +0x24: stores the IString vtable there and, when the length at +0x30 is positive, drops a reference on the buffer at +0x28 (lock-prefixed decrement), freeing through the string's vtable slo… |
| `00a6e2e0` | `InventoryCategoriesDataSource::CategoriesTreeNode::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls 00a6e260 (name string release), then frees this through _free when bit 0 of the flag argument is set; returns this. |
| `00a6e430` | `InventoryCategoriesDataSource::CategoriesTreeNode::FormatValueField` | 🟢 | virtual | Virtual slot 5 (+0x14): for index 0 formats the node field +0x18 and for index 1 the field +0x20 into the output buffer param_3 with format DAT_01770978 through 00402fd0, returning 1. Other indexes return 0. |
