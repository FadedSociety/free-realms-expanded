# Class: GuiApp

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00436eb1` | `GuiAppShutdownUI` | 🟢 | method | When the dword at +0x174 is negative, calls FUN_00431f07 with ECX=this+0x1a8, then GApplication::ShutdownControlManager (00405ddc) with ECX=this, then clears the byte at +0x1a0. Byte-verified sequence: mov eax,[esi+0x17… |
| `0043eb16` | `GuiApp::InitializeControlsA` | ✅ | method | Guarded control-manager initialisation (runs only while this+0x174 < 0): calls the variant setup FUN_0043D9CD(param_2..param_5); on failure reports the wide string 'GFC Error: GApplication::Initialize - control manager … |
| `0043ebc0` | `GuiApp::InitializeControlsB` | ✅ | method | Guarded control-manager initialisation (this+0x174 < 0): calls the variant setup FUN_0043DC9A(param_2,param_3,param_4); on failure reports 'GFC Error: GApplication::Initialize - control manager failed to initialize' thr… |
| `0043ec67` | `GuiApp::InitializeControlsC` | ✅ | method | Guarded control-manager initialisation (this+0x174 < 0): calls the variant setup FUN_0043DE42(param_2,param_3,param_4) (this handled by the callee); on failure reports 'GFC Error: GApplication::Initialize - control mana… |
| `0043fcfb` | `GuiApp::InitializeControlsFromStoredSize` | ✅ | method | Runs only while this+0x174 is negative. Reads signed size words at +0x398/+0x39C, using globals DAT_01BC6974/DAT_01BC73B0 when zero, then calls 0043F98D. On failure it substitutes defaults for dimensions allowed by byte… |
| `00822d00` | `GuiApp::StoreValueAndSentinel ?` | 🟡 | virtual | thiscall: ECX is the value. Stores ECX through the third stack pointer argument when it is non-NULL; always stores the dword DAT_01b35614 through the second stack pointer argument. Returns with ret 8. |
| `00823840` | `GuiApp::HandleEventRecord ?` | 🟡 | virtual | thiscall (ECX = this, one stack argument: an event record). Calls 00418c10(record) first. If the record's 16-bit id at +0x1c equals DAT_01bc71ec: stores (record+4 != 0) as a byte at object+0x48 of the block at this+0x3b… |
