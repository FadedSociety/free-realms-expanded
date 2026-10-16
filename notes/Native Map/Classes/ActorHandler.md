# Class: ActorHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00798db0` | `ActorHandler::Dtor` | ✅ | dtor | Stores ActorHandler::vftable (017e952c) at this+0 and returns. Seven bytes (c7 01 imm32; c3), no other field touched. |
| `00798e80` | `ActorHandler::ReleaseIString` | 🟢 | virtual | Virtual method of the ActorHandler family at vtable slot [6]. When its 4th stack argument is greater than 0, atomically decrements the refcount header at (2nd argument - 4) and, when it reaches zero, calls the IString<c… |
