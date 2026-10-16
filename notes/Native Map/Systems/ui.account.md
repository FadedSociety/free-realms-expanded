# System: ui.account

30 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (18)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0128eb00` | `Engine::AccountShowFriends::ScalarDeletingDtor` | Engine::AccountShowFriends | Slot [0] scalar deleting destructor in Engine::AccountShowFriends; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ec50` | `Engine::AccountAddFriend::ScalarDeletingDtor` | Engine::AccountAddFriend | Slot [0] scalar deleting destructor in Engine::AccountAddFriend; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eda0` | `Engine::AccountRemoveFriend::ScalarDeletingDtor` | Engine::AccountRemoveFriend | Slot [0] scalar deleting destructor in Engine::AccountRemoveFriend; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eef0` | `Engine::AccountAddIgnore::ScalarDeletingDtor` | Engine::AccountAddIgnore | Slot [0] scalar deleting destructor in Engine::AccountAddIgnore; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f040` | `Engine::AccountRemoveIgnore::ScalarDeletingDtor` | Engine::AccountRemoveIgnore | Slot [0] scalar deleting destructor in Engine::AccountRemoveIgnore; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f190` | `Engine::AccountAcceptFriendRequest::ScalarDeletingDtor` | Engine::AccountAcceptFriendRequest | Slot [0] scalar deleting destructor in Engine::AccountAcceptFriendRequest; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f2e0` | `Engine::AccountRejectFriendRequest::ScalarDeletingDtor` | Engine::AccountRejectFriendRequest | Slot [0] scalar deleting destructor in Engine::AccountRejectFriendRequest; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f390` | `Engine::AccountRequestContextMenu::Ctor` | Engine::AccountRequestContextMenu | Constructor of the request-context-menu Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f400` | `Engine::AccountRequestContextMenu::GetActionName` | Engine::AccountRequestContextMenu | Virtual slot [1] giving the bare action name of the request-context-menu action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f420` | `Engine::AccountRequestContextMenu::GetSourceTag` | Engine::AccountRequestContextMenu | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f430` | `Engine::AccountRequestContextMenu::ScalarDeletingDtor` | Engine::AccountRequestContextMenu | Slot [0] scalar deleting destructor in Engine::AccountRequestContextMenu; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f550` | `Engine::AccountLoadMMOPlayerImage::GetActionName` | Engine::AccountLoadMMOPlayerImage | Virtual slot [1] giving the bare action name of the load-MMO-player-image action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f570` | `Engine::AccountLoadMMOPlayerImage::GetSourceTag` | Engine::AccountLoadMMOPlayerImage | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f580` | `Engine::AccountLoadMMOPlayerImage::ScalarDeletingDtor` | Engine::AccountLoadMMOPlayerImage | Slot [0] scalar deleting destructor in Engine::AccountLoadMMOPlayerImage; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f730` | `Engine::AccountUsernameRegistration::Ctor` | Engine::AccountUsernameRegistration | Constructor of the username-registration Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f7a0` | `Engine::AccountUsernameRegistration::GetActionName` | Engine::AccountUsernameRegistration | Virtual slot [1] giving the bare action name of the username-registration action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f7c0` | `Engine::AccountUsernameRegistration::GetSourceTag` | Engine::AccountUsernameRegistration | Virtual slot [4] returning the action's source tag. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f7d0` | `Engine::AccountUsernameRegistration::ScalarDeletingDtor` | Engine::AccountUsernameRegistration | Slot [0] scalar deleting destructor in Engine::AccountUsernameRegistration; resets the shared base vtable, frees this when param_2 bit 0 is set, and otherwise returns this. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0128e870` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ea10` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128eb60` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ecb0` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ee00` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128ef50` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f0a0` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f1f0` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f340` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f490` | `Scaleform::IScaleformActionHandler::Ctor` | Scaleform::IScaleformActionHandler | Initializes *this with the Scaleform::IScaleformActionHandler vtable; whether this body is called directly or as a base subobject initializer is not established. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0128c350` | `— ⚠reject` | Engine::FRTWAccountRespondPlayerMatches | Vtable slot [2] handler of the player-matches response: fills the player info dialog for an account and opens it. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |
| `0128f4e0` | `— ⚠reject` | Engine::AccountLoadMMOPlayerImage | Constructor of the load-MMO-player-image Scaleform action. | [[Native Map/Shards/S0670 0128c350-01290f70|S0670]] |

