# Class: OnlineHelpWindow

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00994470` | `OnlineHelpWindowDtor` | ✅ | dtor | Resets OnlineHelpWindow subobject vtables, frees the browser library member, and releases embedded strings and fixed-string members. |
| `009946d0` | `OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus4` | ✅ | virtual | Subtracts 4 from ECX, then jumps to the scalar deleting destructor wrapper at 009947b0. |
| `009946e0` | `OnlineHelpWindow::ScalarDeletingDtorAdjustorMinus8` | ✅ | virtual | Subtracts 8 from ECX, then jumps to the scalar deleting destructor wrapper at 009947b0. |
| `009947b0` | `OnlineHelpWindow::ScalarDeletingDtor` | ✅ | dtor | Calls the full OnlineHelpWindow destructor and conditionally frees the object based on the deleting flag. |
| `00994980` | `— ⚠reject` | ⚪ | ctor | Installs browser-handler, singleton, and online-help vtables; initializes embedded strings and state; creates the EmbeddedWebBrowserLibrary member and registers callbacks. |
| `00995060` | `OnlineHelpWindow::HandleUrlCommand` | ✅ | virtual | Parses the current browser command string, branches on frgp://, visitHouse/, and teleportTo/ forms, then dispatches help handling, house entry, or a parsed position to the client movement interface. |
| `00995620` | `OnlineHelpWindow::OnNavigationResult` | ✅ | virtual | For a successful callback, runs the browser-action predicate and stores state 3 before reporting status 1; otherwise reports status 3. |
| `009959f0` | `OnlineHelpWindow::Slot2FloatStateTick` | 🟢 | virtual | Virtual (slot [2]) with ECX=this and a float stack arg. If the float equals DAT_017ec67c: when [this+0x10]==3 it calls 00995660 and 009940f0 with ECX=this-4 and returns (ret 8). When [this+0x10]==4 it sets [this+0x10]=5… |
| `00995a70` | `OnlineHelpWindow::Slot3IfFalseSetState4 ? ⚠rename` | 🟡 | virtual | Virtual (slot [3]) with ECX=this and two stack args (undefined4, char); ret 8. If the char argument [esp+8] is zero, calls 009954d0 with ECX=this and push 4. Otherwise returns without action. |
