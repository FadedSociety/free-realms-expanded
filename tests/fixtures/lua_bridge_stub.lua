-- Stub UI for scripts/lua_bridge.lua. The bridge hooks the same global tables the
-- client defines (DialogHandler, radialMenu, HousingRadialMenu, OnUpdate); this
-- script fakes them and prints every UI action the bridge triggers.
-- Usage (from a disposable <run>/client directory):
--   lua tests/fixtures/lua_bridge_stub.lua <scenario> <absolute bridge path>
-- The command and result files are the bridge's ../lua-bridge-*.jsonl, so the
-- scenario directory is <run>. See tests/test_lua_bridge.py.

local scenario = arg[1] or "base"
local bridge = arg[2]
if not bridge then
  error("usage: lua_bridge_stub.lua <scenario> <bridge.lua>")
end

local calls = {}
local closest = "-1"

local function record(name, value)
  calls[#calls + 1] = tostring(name) .. "\t" .. tostring(value)
end

Ui = {
  GetAtlasName = function()
    return "Cobblestone"
  end,
  GetPlayerLoc = function()
    return 120.5, 3.25, -44.75
  end,
  GetPlayerHeading = function()
    return 1.5
  end,
  DoClosestInteraction = function()
    record("interact")
  end,
  ProcessChatCommand = function(value)
    record("chat", value)
  end,
  DialogSelect = function(index)
    record("dialog", index)
  end,
  RadialMenuClicked = function(index)
    record("radial", index)
  end,
}

DialogHandler = {
  isDialogShown = false,
  SetDialogText = function(self, value)
    self.text = value
  end,
  SetChoiceText = function(self, index, value)
    self[index] = value
  end,
  Show = function(self)
    self.isDialogShown = true
  end,
  Hide = function(self)
    self.isDialogShown = false
  end,
  SelectOption = function(self, index)
    Ui.DialogSelect(index)
  end,
}

Housing = {
  OnRadialMenuClick = function(event)
    record("housing", event)
  end,
}

local function menu(onClick)
  local m = {
    isShown = false,
    WindowName = "Main.wndRadialMenu",
    setButtonCount = function(self, count)
      self.count = count
    end,
    setButton = function(self, index, icon, label, event)
      self[index] = { label, event }
    end,
    onPopup = function(self)
      self.isShown = true
    end,
    hide = function(self)
      self.isShown = false
    end,
  }
  m.OnUserEvent = function(name, value)
    if name == "radialMenuClick" then
      onClick(value)
      m:hide()
    end
  end
  return m
end

radialMenu = menu(function(value)
  Ui.RadialMenuClicked(value)
end)
HousingRadialMenu = menu(function(value)
  Housing.OnRadialMenuClick(value)
end)
Target = { isShown = false, lastTarget = " " }
UIWindow = { openedWindows = {} }
InventoryHandler = { wndName = "Main.wndInventory", isShown = true }
DsTable = {
  Find = function()
    return {
      GetData = function(_, _, _)
        return closest
      end,
    }
  end,
}

function OnUpdate()
end

dofile(bridge)

if scenario == "partial" then
  -- The test wrote a JSON line without its newline. One poll buffers the partial
  -- line, then completing the line makes the same bytes process exactly once.
  for _ = 1, 16 do
    OnUpdate()
  end
  local file = assert(io.open("../lua-bridge-commands.jsonl", "ab"))
  file:write("\n")
  file:close()
  for _ = 1, 16 do
    OnUpdate()
  end
else
  if scenario == "base" then
    closest = "Samantha"
  elseif scenario == "radial" then
    radialMenu:setButtonCount(2)
    radialMenu:setButton(0, 10, "Talk", 7)
    radialMenu:setButton(1, 11, "Barter", 8)
    radialMenu:onPopup()
  elseif scenario == "housing" then
    HousingRadialMenu:setButtonCount(2)
    HousingRadialMenu:setButton(0, 10, "Up", 1001)
    HousingRadialMenu:setButton(1, 11, "Rotate", 1002)
    HousingRadialMenu:onPopup()
  elseif scenario == "dialog" then
    DialogHandler:SetDialogText("Samantha greets you.")
    DialogHandler:SetChoiceText(0, "Cooking: Strong Arm Stew", 18, 0)
    DialogHandler:SetChoiceText(1, "Harvesting", 18, 0)
    DialogHandler:SetChoiceText(2, "Goodbye", 18, 0)
    DialogHandler:Show()
  elseif scenario == "closed" then
    DialogHandler:SetDialogText("The page body.")
    DialogHandler:SetChoiceText(0, "Goodbye", 18, 0)
    DialogHandler:Show()
    DialogHandler:Hide()
  else
    error("unknown scenario: " .. tostring(scenario))
  end

  for _ = 1, 16 do
    OnUpdate()
  end
end

for i = 1, #calls do
  print(calls[i])
end
