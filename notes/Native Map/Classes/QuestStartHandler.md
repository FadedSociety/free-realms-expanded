# Class: QuestStartHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a99220` | `QuestStartHandler::DismissEndScreen` | ✅ | handler | Reads the object at arg+4 and tests a chain of state bytes in sub-objects at +0x11c, +0x130, +0x120, +0x124, +0x134 and +0x128; picks a reason code 1, 6, 7, 8, 9, 0x11 or 0x13 by the first set flag, passes it to 008c5d9… |
