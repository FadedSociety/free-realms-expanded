# Class: ControllerFishing

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccecf0` | `ControllerFishing::UpdateFrame` | 🟢 | method | Updates fish tension and distance, processes local cast and reel state, manages fishing prompts and timers, creates tactile notifications, and tunnels activity fields when enabled. |
| `00ccf260` | `FishingController::ResetResultState → ControllerFishing::ResetResultState` | 🟢 | method | When the nested runtime object has type 12 and its progress exceeds 250, clears the result string and resets that progress field. |
| `00ccf3c0` | `ControllerFishing::SetState` | 🟢 | method | Selects a UI state from the current activity and local player conditions, updates fishing result state, compares the local actor position with the target, and emits an activity event when needed. |
| `00cd0150` | `ControllerFishing::UpdateFrame` | ✅ | method | Advances fishing state and timers, updates local player UI and fish tension, emits status text and catch presentation, creates fishing effects, and updates result display objects. |
