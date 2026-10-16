# Class: AssetManagement::AssetDeliveryInterface

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010947d0` | `AssetDeliveryInterface::DrainPendingLists` | 🟢 | virtual | Calls FUN_01094560(this) then tail-jumps (jmp) to FUN_010946b0 with ECX=this; no stack arguments of its own (bytes: push esi; mov esi,ecx; call 01094560; mov ecx,esi; pop esi; jmp 010946b0). |
| `0109c720` | `AssetDeliveryInterface::~AssetDeliveryInterface` | ✅ | dtor | Destructor body: installs the AssetDeliveryInterface vtable, runs the base teardown 00fc45a0, destroys the PendingEntryError list member (0109c5f0) and the PendingEntryData list member (0109b8d0), then calls 00703350. |
| `0109c800` | `AssetDeliveryInterface::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor shape: calls the AssetDeliveryInterface body 0109c720, then calls _free when bit 0 of the stack argument is set; returns the object pointer. |
| `0109c820` | `AssetDeliveryInterface::AssetDeliveryInterface` | ✅ | ctor | Constructor: installs the AssetDeliveryInterface vtable, runs the base constructor 00703540, constructs the PendingEntryData list member (0109b830) and the PendingEntryError list member (0109c550), then calls 00fc4580 w… |
