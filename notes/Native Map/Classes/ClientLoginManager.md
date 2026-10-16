# Class: ClientLoginManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009552e0` | `ClientLoginManager::Virtual08 ?` | 🟡 | virtual | Clears two object flags, conditionally copies a byte range, initializes a fixed string temporary, then invokes a virtual function at object offset 0x14. |
| `00955460` | `ClientLoginManager::BuildEntriesFromLinkedList ?` | 🟡 | virtual | If a service is available, walks a linked list, allocates one node per item, copies a value and two flag bits, then invokes virtual slot 2 with a local buffer. |
| `00956100` | `ClientLoginManager::ParseNameSuggestionReply` | 🟢 | virtual | Parses the supplied byte range, builds a temporary five-slot NameSuggestion list, calls manager virtual slot 4 with decoded values, then destroys the temporary list. |
| `009566c0` | `ClientLoginManager::OnPacketD3` | ✅ | handler | Checks that the supplied buffer contains at least one 16-bit word and that the first word equals 0xd3; on a match, calls 00956100 with the manager receiver and packet buffer. |
| `00957530` | `ClientLoginManager::BuildAccountCharacterDetails` | 🟢 | virtual | Calls 00769710 with no stack arguments to obtain the manager service; when its service pointer is present, initializes a temporary ten-slot AccountInfoCharacterDetails list, walks supplied records, links eligible entrie… |
