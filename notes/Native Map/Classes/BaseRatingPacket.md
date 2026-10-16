# Class: BaseRatingPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa5ca0` | `BaseRatingPacket::BaseRatingPacket` | ✅ | ctor | Stores BasePacket::vftable, sets the family opcode field to 0xae (174), stores BaseRatingPacket::vftable and writes the constructor argument into object+0x8. |
| `00aa5d10` | `BaseRatingPacket::~BaseRatingPacket` | ✅ | dtor | Stores BaseRatingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5d60` | `BaseRatingPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseRatingPacket::vftable and BasePacket::vftable, then calls _free on the object when bit 0 of param_2 is set. |
| `00abab40` | `BaseRatingPacket::Dtor` | ✅ | dtor | Destructor body of a BaseRatingPacket-derived object: calls FUN_00abaad0 on the embedded IString-valued HashListMap, stamps the IString vftable at +0xc, releases the buffer at +0x10 when the count at +0x18 is positive (… |
| `00c81880` | `BaseRatingPacket::BaseRatingPacket` | ✅ | ctor | Installs BasePacket then BaseRatingPacket vtables, stores family value 0xae at field 1, and zeros a field at index 2. |
| `00c818e0` | `BaseRatingPacket::Dtor` | ✅ | dtor | Restores BaseRatingPacket::vftable and then BasePacket::vftable. |
| `00c85620` | `BaseRatingPacketDtor` | ✅ | dtor | Destroys the contained rating map, releases the reference-counted string when owned, and restores BaseRatingPacket then BasePacket vtables. |
| `00c86790` | `BaseRatingPacketWithListDtor` | ✅ | dtor | Destroys the embedded RatingDataEntry list, releases the owned string buffer, and restores BaseRatingPacket then BasePacket vtables. |
| `00c869e0` | `— ⚠reject` | ⚪ | handler | Reads a BaseRatingPacket subopcode and dispatches rating data, candidate-info, and other rating replies; housing-related cases update local lookup or housing processor state. |
