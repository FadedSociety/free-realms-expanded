# Class: MoveAndSelectTargetControllerHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bb4d0` | `MoveAndSelectTargetControllerHandler::LoadDefaultPair` | 🟢 | virtual | Writes the two global dwords at object offsets +8 and +0x0c and returns. |
| `00934870` | `MoveAndSelectTargetControllerHandler::SelectPlayer` | 🟢 | virtual | Reads the candidate target and its position. When both target guards pass, constructs CommandPacketSelectPlayer and sends it only if the gateway pointer at this+0x280 is non-null. It then invokes vtable slot +0xa0 with … |
