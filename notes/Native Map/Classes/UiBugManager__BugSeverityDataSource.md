# Class: UiBugManager::BugSeverityDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f89b0` | `UiBugManager::BugSeverityDataSource::ColumnName` | ✅ | virtual | Returns the constant label string Bug Severity. |
| `009f89c0` | `AdjustThisPlus4AndJump010C8F30` | ✅ | virtual | Adds 4 to ECX, then jumps to 010c8f30; the body contains no further call or field logic. |
| `009f9530` | `UiBugManager::BugSeverityDataSource::SetValue` | ✅ | virtual | Gets the selected row, searches the severity table at nested offset +0x80 for its key, passes the matching +4 value and supplied value to a manager virtual setter, and returns 1. |
| `009fc450` | `UiBugManager::BugSeverityDataSource::Dtor` | ✅ | dtor | Restores the data-source vtable at the primary and two embedded offsets, then destroys the embedded user-data table. |
| `009fc600` | `UiBugManager::BugSeverityDataSource::Ctor` | ✅ | ctor | Registers DesignTools.BugSeverity, initializes the primary and two embedded data-source vtables, and stores the manager pointer at +0x254. |
| `009fc680` | `UiBugManager::BugSeverityDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls the severity data-source destructor, frees the object when flag bit 0 is set, and otherwise returns its pointer. |
