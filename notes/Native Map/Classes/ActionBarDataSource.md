# Class: ActionBarDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c48f60` | `ActionBarDataSource_GetColumnName` | ✅ | virtual | Column-name accessor (vtable slot 1 of the ActionBarDataSource subobject vtable): a switch over the column index returns the literal names Assigned (0), Icon ID (1), Name (2), Description (3), Quantity (4), Mana Cost (5… |
| `00c48fe0` | `ActionBarDataSource_NotifyOwner` | 🟢 | virtual | Vtable slot 6 of the ActionBarDataSource subobject at +0x254: loads the owning object from ecx-0x254, adjusts this to that owner and tail-jumps to the owner's virtual slot 4 (offset +0x10), i.e. it forwards a change not… |
| `00c49030` | `ActionBarDataSource_GetValue` | ✅ | virtual | ActionBarDataSource vtable slot 4. It obtains a service/object through 00769710 and follows fields +0x3e0, +0x504 and a +0x4a8 hash chain; it bounds param_2 below 9, passes it to 01046190, then switches on param_3 (0..7… |
| `00c492a0` | `ActionBarDataSource::~ActionBarDataSource` | ✅ | dtor | Destructor body: installs the four ActionBarDataSource base-subobject vtables (+0, +0x30, +0x34 and the +0x254 subobject), destroys the embedded list-like member with this=this+0x254 through 0081b7d0, then runs the two … |
| `00c49350` | `ActionBarDataSource_SecondaryScalarDeletingDtor` | 🟢 | dtor | Eleven-byte adjustor thunk (bytes 81 e9 54 02 00 00 e9 ...): subtracts 0x254 from ecx and jumps to the scalar deleting destructor 00c49420, i.e. the deleting destructor entry for the base subobject embedded at +0x254. |
| `00c49360` | `— ⚠reject` | ⚪ | ctor | Constructor taking two arguments: registers the data source name 'BaseClient.ActionBar' via 00922890, runs another base constructor 0081b470, stores the arguments at +0x254 (index 0x95, the embedded subobject/owner refe… |
| `00c49420` | `ActionBarDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the 0183212c vtable): runs the destructor body 00c492a0 and frees the object with CRT _free when bit 0 of the flag is set; returns the object pointer otherwise. |
