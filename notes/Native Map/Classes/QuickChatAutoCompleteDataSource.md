# Class: QuickChatAutoCompleteDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cdf760` | `QuickChatAutoCompleteDataSource::Constructor` | ✅ | ctor | Initializes the BaseClient.QuickChat.AutoComplete path, stores its context, installs three quick-chat data-source vtables, initializes the list, and initializes an embedded string. |
| `00cdf8c0` | `QuickChatAutoCompleteDataSource::ScalarDeletingDestructor` | ✅ | dtor | Calls the autocomplete data-source destructor body, frees this when flag bit 0 is set, and otherwise returns this. |
