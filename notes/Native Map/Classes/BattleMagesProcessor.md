# Class: BattleMagesProcessor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b70cb0` | `BattleMagesProcessor::CreateParticleDefinitionAsset` | ✅ | virtual | Vtable slot 3 allocates 0x68 bytes with 00d833a6; if allocation succeeds, passes the block as this to 0071bed0 with the processor manager pointer, a manager-derived value and flag 1. That constructor initializes Particl… |
| `00b71200` | `— ⚠reject` | ⚪ | ctor | Constructor: takes the manager pointer at +4 from [FUN_00769710()+0x1c], installs AssetManagement::AssetRequestInterface::AssetRequester::vftable then BattleMagesProcessor::vftable at +0, stores the argument at +8, zero… |
| `00b71320` | `BattleMagesProcessor::~BattleMagesProcessor` | ✅ | dtor | Destructor body: rewrites +0 with BattleMagesProcessor::vftable, releases the BattleMagesCommandProcessor at +0x4c through its vtable slot 1 with flag 1 and zeroes that field, releases the camera string with 00b70a00(th… |
| `00b71670` | `BattleMagesProcessor::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the processor destructor FUN_00b71320 (argument-less call in the pseudocode) and, when the flag byte's low bit is set, calls _free on the object before returning it. |
| `00b728a0` | `BattleMagesProcessor::DispatchPacket` | ✅ | virtual | Virtual slot 4 of BattleMagesProcessor: builds a stack BaseBattleMagesPacket temp (ctor FUN_00b6f250), reads the two-u16 inner header from (data, length) with FUN_00b70e40 and switches on the sub id: 2 runs the update-d… |
