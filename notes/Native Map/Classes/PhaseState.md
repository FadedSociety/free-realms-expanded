# Class: PhaseState

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `014690b0` | `PhaseState::RestoreState` | ✅ | virtual | Virtual slot [17] of PhaseState::vftable. Calls FUN_0145c4d0 with the format '%s::restoreState' and the name text (inline buffer at +0x20 when the size word at +0x34 is below 0x10, else the pointer at +0x20), with ECX=t… |
| `014690e0` | `PhaseState::LeaveState` | ✅ | virtual | Virtual slot [18] of PhaseState::vftable. Calls FUN_0145c4d0 with the format '%s::leaveState' and the same inline-or-pointer name text, passes the result to FUN_013508b0 (wa_general trace), loads ECX=[this+8] and calls … |
| `01469120` | `PhaseState::Ctor` | ✅ | ctor | Constructor body. Calls FUN_0145d8c0 first (base state-machine init; its strings include 'Basic StateMachineState'), then stores PhaseState::vftable at +0, zeroes the dwords at +0xF4, +0xF8 and +0xFC, and assigns the li… |
| `014691a0` | `PhaseState::CtorWithMachine` | ✅ | ctor | Constructor overload taking one stack argument (ret 4) that it forwards to FUN_0145d9f0 (base init; its strings include 'mMachine'). Then stores PhaseState::vftable at +0, zeroes the dwords at +0xF4, +0xF8 and +0xFC, an… |
