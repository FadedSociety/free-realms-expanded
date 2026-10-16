# System: world.area

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (15)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ceae00` | `ClientAreaRoomTypeProperty::WriteRoomTypeId ?` | ClientAreaRoomTypeProperty | Writes the property name RoomTypeId and the field at object offset +0x38 through the supplied serializer helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceae20` | `ClientAreaRoomTypeProperty::Log ?` | ClientAreaRoomTypeProperty | Obtains a formatted base description, writes the inherited field at index 7, and logs RoomTypeId from index 14. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceae70` | `ClientAreaRoomTypeProperty::OnApply ?` | ClientAreaRoomTypeProperty | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the apply helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceaea0` | `ClientAreaRoomTypeProperty::OnRemove ?` | ClientAreaRoomTypeProperty | If the client setting at global offset +0x24c is nonzero, passes fields at +0x1c and +0x38 to the remove helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceaf20` | `ClientAreaPhysicsProperty::Log ?` | ClientAreaPhysicsProperty | Logs inherited property fields plus JumpHeightScalar, GravityScalar, BounceScalar, BounceVelocityCap, and AutoJump. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb120` | `ClientAreaPhysicsProperty::Load ?` | ClientAreaPhysicsProperty | Loads four named physics values with atof, reads AutoJump as a boolean, conditionally updates a live object, and clamps a nested value to 250. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb240` | `ClientAreaPhysicsProperty::Reset ?` | ClientAreaPhysicsProperty | For an active object with type 1, resets selected physics fields at +0x490/+0x498 to the default float and zeros +0x49c/+0x4a0 when property values are non-default. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb2d0` | `ClientAreaPhysicsProperty::Apply ?` | ClientAreaPhysicsProperty | For an active object of type 1, copies physics properties to it; if AutoJump is enabled, sets bit 0x10 at object+0x142 and calls a state helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb330` | `ClientAreaTimerProperty::Load ?` | ClientAreaTimerProperty | Reads StartTimer from the property source and stores its nonzero value as a boolean at object offset +0x38. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb370` | `ClientAreaTimerProperty::Log ?` | ClientAreaTimerProperty | Logs the inherited property fields and StartTimer boolean. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb740` | `ClientAreaTimerProperty::SendTimerPacket ?` | ClientAreaTimerProperty | If StartTimer is true constructs PacketClientAreaTimer(1,-1); otherwise constructs it with (2,-1); sends only when gateway field +0x280 is nonzero. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb810` | `ClientAreaAdvJournalProperty::LoadRegionOverride ?` | ClientAreaAdvJournalProperty | Reads RegionOverride from the property source and stores the supplied integer at object offset +0x38. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb840` | `ClientAreaAdvJournalProperty::Log ?` | ClientAreaAdvJournalProperty | Logs inherited property fields and the RegionOverride value at index 14. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb890` | `ClientAreaAdvJournalProperty::Apply ?` | ClientAreaAdvJournalProperty | For an active area object whose type and pointer fields are valid, checks a helper, then passes RegionOverride to a state tracking helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |
| `00ceb8d0` | `ClientAreaAdvJournalProperty::Remove ?` | ClientAreaAdvJournalProperty | For an active area object whose type and pointer fields are valid, checks a helper, then removes RegionOverride through a state tracking helper. | [[Native Map/Shards/S0422 00ce4e30-00ceb920|S0422]] |

