# Class: ClientAreaTimerProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be9f70` | `ClientAreaTimerProperty::Ctor` | ✅ | ctor | Stores ClientAreaProperty vtable and initialises the base fields (arguments at +0x18 and +0x1c, constant _DAT_0175b420 at +0x24, zeroes elsewhere), then installs ClientAreaTimerProperty vtable and clears the byte at +0x… |
| `00bea020` | `ClientAreaTimerProperty::DtorBody` | ✅ | dtor | Stores ClientAreaTimerProperty vtable at *this, calls 00ceb3c0 (a stub whose body is only return) with an unwind frame, then stores the ClientAreaProperty vtable and returns. |
| `00ceb330` | `ClientAreaTimerProperty::Load ?` | 🟡 | virtual | Reads StartTimer from the property source and stores its nonzero value as a boolean at object offset +0x38. |
| `00ceb370` | `ClientAreaTimerProperty::Log ?` | 🟡 | virtual | Logs the inherited property fields and StartTimer boolean. |
| `00ceb740` | `ClientAreaTimerProperty::SendTimerPacket ?` | 🟡 | virtual | If StartTimer is true constructs PacketClientAreaTimer(1,-1); otherwise constructs it with (2,-1); sends only when gateway field +0x280 is nonzero. |
