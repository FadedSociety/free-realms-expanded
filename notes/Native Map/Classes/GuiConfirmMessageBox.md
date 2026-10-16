# Class: GuiConfirmMessageBox

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008236c0` | `GuiConfirmMessageBox::Ctor` | ✅ | ctor | Stores GuiEventHandler::vftable then GuiConfirmMessageBox::vftable at this, zeroes this+4 and this+8, calls 0081eb80 with the string Main.wndConfirm.OnClose and this, and stores its return value at this+0x14. Returns th… |
| `00823730` | `GuiConfirmMessageBox::DtorBody` | 🟢 | dtor | Destructor body: stores GuiConfirmMessageBox::vftable at this, sets a local cleanup flag, calls base teardown 0081bf00 on this, then stores GuiEventHandler::vftable at this. Restores the EH frame and returns. |
| `008237a0` | `GuiConfirmMessageBox::InvokeChoice ?` | 🟡 | virtual | fastcall (ECX = this). Calls 008182e0; when it returns zero calls the callback at this+8 with argument this+0x10 if non-NULL, otherwise calls the callback at this+4 with argument this+0xc if non-NULL. Returns 1 in every… |
| `008237f0` | `GuiConfirmMessageBox::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor shape: calls 00823730 (dtor body) on this, then calls _free (thunk 00d82463) on this only when bit 0 of the flag argument is set; returns this. |
