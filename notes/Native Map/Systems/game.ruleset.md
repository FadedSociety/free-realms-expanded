# System: game.ruleset

95 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (74)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008d00e0` | `ProjectileParameters::ProjectileParameters` | ProjectileParameters | Construct the projectile parameter record filled by ranged attacks before PlayerUpdateLaunchProjectilePacket is built (the vault note ties this constructor to that packet path). | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `008d0240` | `ProjectileParameters::~ProjectileParameters` | ProjectileParameters | Tear down a projectile parameter record including its two Target sub-objects and strings (16 unwind funclets reference this body). | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `00bb9b10` | `IRequirementManager::IRequirementManager` | IRequirementManager | Initializes the requirement-manager base object. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9e40` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Cleans up the base creator subobject during exception unwinding. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bb9e50` | `BasePool::~BasePool` | SoeGems::ObjectFactory<IRequirement,int… | Cleans up the base object-pool subobject during exception unwinding. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbad80` | `ExpressionNodeBase::Cleanup → Requirements::ExpressionNodeBase::~ExpressionNode…` | ExpressionNodeBase | Runs the base cleanup used by expression-node unwind paths. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbadd0` | `ExpressionNodeBase::Cleanup → Requirements::ExpressionNodeBase::~ExpressionNode…` | ExpressionNodeBase | Runs the base cleanup used by expression-node unwind paths. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb4b0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckCountry using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb520` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckCountry using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb590` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckLanguage using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb600` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckLanguage using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb670` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckLastPlayedProfileOfTypeRank using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb6e0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckLastPlayedProfileOfTypeRank using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb750` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementAlwaysFails using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb7c0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementAlwaysFails using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb830` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckVipRank using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb8a0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckVipRank using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb910` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckCurrentProfile using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb980` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckCurrentProfile using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbb9f0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckRenamePending using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbba60` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckRenamePending using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbad0` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckNudge using RefObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbb40` | `ICreator::InitRegisteredPool` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the registration creator for ClientRequirementCheckNudge using ObjectPool. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbbe0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbc30` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbc80` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbcd0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbd20` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbd70` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbdc0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbe10` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbe60` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbeb0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbf00` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbf50` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbfa0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbbff0` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbc040` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbc090` | `ICreator::Init` | SoeGems::ObjectFactory<IRequirement,int… | Initializes the generic requirement-factory creator base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd0f0` | `RefCounted::~RefCounted` | SoeUtil::RefCounted | Runs a shared base destructor exposed in the ExpressionNodeBase vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd160` | `RefCounted::~RefCounted` | SoeUtil::RefCounted | Runs shared base cleanup used by IRequirement expression-node unwind paths. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd2a0` | `IRequirement::CallSlot14AndUpdateList` | IRequirement | Shared IRequirement virtual implementation also present in the listed base-template and concrete requirement vtables. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd2f0` | `RefCounted::~RefCounted` | SoeUtil::RefCounted | Shared deleting destructor exposed through IRequirement and its base vtables. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd360` | `BaseDefinitionManager::~BaseDefinitionManager` | BaseDefinitionManager | Shared base deleting destructor exposed through IRequirementManager's slot 0. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd3c0` | `ExpressionNodeBase::ExpressionNodeBase` | Requirements::ExpressionNodeBase | Initializes an expression node's reference-counted base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd4d0` | `ObjectFactory::InitRequirementFactory` | SoeGems::ObjectFactory<IRequirement,int> | Initializes ObjectFactory<IRequirement,int> and its embedded registration maps. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd6e0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckCountry RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd740` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckCountry ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd7a0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckLanguage RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd800` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckLanguage ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd860` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckLastPlayedProfileOfTypeRank RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd8c0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckLastPlayedProfileOfTypeRank ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd920` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementAlwaysFails RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd980` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementAlwaysFails ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd9e0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckVipRank RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbda40` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckVipRank ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbdaa0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckCurrentProfile RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbdb00` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckCurrentProfile ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbdb60` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckRenamePending RefObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbdbc0` | `ICreator::~ICreator` | SoeGems::ObjectFactory<IRequirement,int… | Scalar deleting destructor used by the ClientRequirementCheckRenamePending ObjectPool creator vtable. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `010463c0` | `ContentPackManager::IsContentUnlocked` | ContentPackManager | Returns whether a content pack id is unlocked: always unlocked for id < 1, otherwise the stored flag of that pack, and locked (0) with a log line when the pack is unknown. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01046410` | `ContentPackManager::SetFlagById` | ContentPackManager | Virtual slot 5 of ContentPackManager that writes the per-pack flag byte read by IsContentUnlocked (010463c0) and reports whether it changed. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `010469e0` | `ContentPackManager::LoadDefinitions` | ContentPackManager | Virtual slot 1 of ContentPackManager: loads ContentPacks.txt from the content directory and rebuilds the content-pack definitions. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01046cf0` | `ContentPackManager::DeleteDefinitions` | ContentPackManager | Virtual slot 2 of ContentPackManager: deletes all loaded content-pack definitions and logs how many remain. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047b80` | `ContentPackManager::DtorBody` | ContentPackManager | Destructor body of ContentPackManager; it clears and destroys the ContentPack map member at +0x14 and the name string at +4, then resets the vtable to the base class. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047d50` | `ContentPackManager::Ctor` | ContentPackManager | Constructor of ContentPackManager: builds its base vtables, the name string, and the ContentPack map at +0x14, and logs creation. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047e00` | `ContentPackManager::ScalarDeletingDtor` | ContentPackManager | Scalar deleting destructor of ContentPackManager (vtable slot 0). | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01047e20` | `ContentPackManager::LoadDefinitionsFromBuffer` | ContentPackManager | Virtual slot 3 of ContentPackManager: loads the content-pack lock table from a text definitions buffer, creating each pack's unlock flag in the ContentPack map. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `013c5920` | `RuleSetData::RuleSetData` | RuleSetData | Constructor of the RuleSetData persistent component; the vtable chain shows it derives PersistentComponent -> PersistentBase. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c59b0` | `RuleSetData::GetTypeName` | RuleSetData | Virtual type-name accessor used by the persistent-component/type machinery to identify this class. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c59f0` | `RuleSetData::~RuleSetData` | RuleSetData | Destructor of RuleSetData: deletes the owned expression tree and unwinds the base vtables. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5a90` | `RuleSetData::FormatDebug` | RuleSetData | Debug/trace format method: renders the child expression into the stream followed by its boolean evaluation result. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5ae0` | `RuleSetData::FormatDebugTyped` | RuleSetData | Typed (gvt:) variant of the ruleset debug/trace formatter; the 'gvt:' prefix matches the game-value-type node names used across the expression-node region (0142xxxx-0149xxxx). | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5b70` | `RuleSetData::Dump` | RuleSetData | Debug dump of a RuleSetData value into a text stream (labels, ID, expression tree, closing). | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## 🟢 likely (12)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008adff0` | `Wrapper_00adac80 → ClientRepeatingActivity_Dtor_SEHWrapper` |  | Forwarding wrapper to the ClientRepeatingActivity teardown body; the wrapper does not identify its containing type. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `008d0310` | `ProjectileParameters::ScalarDeletingDestructor` | ProjectileParameters | Deleting destructor entry of the ProjectileParameters vtable (slot [0] of 01802a0c): run the destructor body and optionally free the record. | [[Native Map/Shards/S0226 008ca9e0-008d08b0|S0226]] |
| `00bb9b80` | `BaseDefinitionManager::~BaseDefinitionManager ⚠downgrade` | BaseDefinitionManager | Restores the base vtable while unwinding an IRequirementManager object. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bba490` | `RequirementSet::~RequirementSet` | RequirementSet | Runs the RequirementSet destructor body through its wrapper. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbc690` | `RequirementSet::~RequirementSet` | RequirementSet | Implements the scalar deleting destructor for a RequirementSet. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `01046650` | `NegateIsUnlocked_01046650` |  | Negated form of ContentPackManager::IsContentUnlocked (010463c0): true when the content pack is not unlocked. | [[Native Map/Shards/S0571 01044480-0104a2b0|S0571]] |
| `01082040` | `RequirementSetCtor` | RequirementSet | Constructor of the RequirementSet object (class name from RTTI); its game role is not decoded here. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010821b0` | `RequirementSetDtorBody` | RequirementSet | Destructor body of RequirementSet: releases the requirement node lists. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `010821d0` | `RequirementSetScalarDeletingDtor` | RequirementSet | Scalar deleting destructor (vtable slot 0) of RequirementSet. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |
| `013c5c00` | `CreateRuleSetData` |  | Factory that news and constructs a 12-byte RuleSetData object; registered as type id 5 by 013c5c60. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5c60` | `RegisterRuleSetDataFactory` |  | One of the type-registration stubs emitted for every persistent value type (FUN_013ce130/FUN_013ce230 have 500+ callers each); registers RuleSetData's factory at type id 5. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5c80` | `RuleSetData::~RuleSetData` | RuleSetData | Scalar deleting destructor for RuleSetData, used by the deleting-destructor call in 013c59f0 and by delete sites. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## 🟡 speculative (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c59c0` | `GetRuleSetGlobal ?` |  | Getter for a static word (singleton/flag) used by the ruleset code adjacent to RuleSetData methods. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c59d0` | `SetRuleSetGlobal ?` |  | Setter paired with the global getter for the ruleset subsystem's static word. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5a70` | `SetExpressionTree ?` |  | Setter for the expression-tree pointer that neighbouring RuleSetData::Dump (013c5b70) prints from the same +8 field. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5a80` | `GetExpressionTree ?` |  | Getter for the expression-tree pointer read by RuleSetData::Dump (013c5b70). | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5b50` | `GetRuleSetID ?` |  | Getter for the RuleSetID field that RuleSetData::Dump (013c5b70) prints after the "RuleSetID:" label. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5b60` | `SetRuleSetID ?` |  | Setter for the RuleSetID field printed by RuleSetData::Dump (013c5b70). | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c5ca0` | `RuleSetData::Matches ? ⚠rename` | RuleSetData | Composite boolean predicate over a RuleSetData's fields and two virtual checks; likely a match/equality test used by the ruleset evaluator. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb9e60` | `— ⚠reject` | SoeGems::ObjectFactory<IRequirement,int… | Provides the conditional-delete wrapper for the object-pool base. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |
| `00bbd580` | `— ⚠reject` | SoeGems::ObjectFactory<IRequirement,int> | Releases registered requirement pools during object-factory teardown. | [[Native Map/Shards/S0367 00bb7790-00bbdbc0|S0367]] |

