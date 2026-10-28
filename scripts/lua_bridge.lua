-- JSON bridge appended to UI/ScriptsBase.bin by scripts/lua_probe_overlay.py.
-- The harness writes one command object per line to <run>/lua-bridge-commands.jsonl;
-- the client runs in <run>/client, so the paths are relative to its working directory.
-- Answers are appended to <run>/lua-bridge-state.jsonl with the command's own id.
-- Commands: state, click (dialog/radial/housing), chat, interact. Anything else
-- answers an error line. Every command and hook is pcall'd: bad input must not
-- stop the client UI. See notes/Autopilot/T857.md for the client evidence.

if rawget(_G, "__luaBridgeInstalled") then
  return
end
__luaBridgeInstalled = 1

local COMMAND_FILE = "../lua-bridge-commands.jsonl"
local RESULT_FILE = "../lua-bridge-state.jsonl"
local POLL_FRAMES = 15
local MAX_POLL_BYTES = 262144
local MAX_LINE_BYTES = 65536
local MAX_COMMANDS_PER_POLL = 64
local MAX_ROWS = 8
local MAX_WINDOWS = 24

local frames = 0
local readOffset = 0
local pending = ""
local answered = {}
local dialogText = nil
local dialogRows = {}
local radial = { shown = false, buttons = {} }
local housing = { shown = false, buttons = {} }

local function text(value)
  if value == nil then
    return nil
  end
  local ok, rendered = pcall(tostring, value)
  if not ok then
    return nil
  end
  return rendered
end

local function utf8Char(cp)
  if cp < 0x80 then
    return string.char(cp)
  end
  if cp < 0x800 then
    return string.char(0xC0 + math.floor(cp / 0x40), 0x80 + cp % 0x40)
  end
  if cp < 0x10000 then
    return string.char(0xE0 + math.floor(cp / 0x1000),
      0x80 + math.floor(cp / 0x40) % 0x40, 0x80 + cp % 0x40)
  end
  return string.char(0xF0 + math.floor(cp / 0x40000),
    0x80 + math.floor(cp / 0x1000) % 0x40,
    0x80 + math.floor(cp / 0x40) % 0x40, 0x80 + cp % 0x40)
end

local function jsonDecode(source)
  local pos = 1

  local function fail(what)
    error(string.format("json %s at byte %d", what, pos), 0)
  end

  local function skip()
    while true do
      local c = string.sub(source, pos, pos)
      if c == " " or c == "\t" or c == "\n" or c == "\r" then
        pos = pos + 1
      else
        return
      end
    end
  end

  local parseValue

  local function parseString()
    pos = pos + 1
    local out = {}
    while true do
      local c = string.sub(source, pos, pos)
      if c == "" then
        fail("unterminated string")
      elseif c == '"' then
        pos = pos + 1
        return table.concat(out)
      elseif c == "\\" then
        local escape = string.sub(source, pos + 1, pos + 1)
        pos = pos + 2
        if escape == '"' or escape == "\\" or escape == "/" then
          out[#out + 1] = escape
        elseif escape == "b" then
          out[#out + 1] = "\b"
        elseif escape == "f" then
          out[#out + 1] = "\f"
        elseif escape == "n" then
          out[#out + 1] = "\n"
        elseif escape == "r" then
          out[#out + 1] = "\r"
        elseif escape == "t" then
          out[#out + 1] = "\t"
        elseif escape == "u" then
          local cp = tonumber(string.sub(source, pos, pos + 3), 16)
          if not cp then
            fail("bad unicode escape")
          end
          pos = pos + 4
          if cp >= 0xD800 and cp <= 0xDBFF then
            if string.sub(source, pos, pos + 1) ~= "\\u" then
              fail("unpaired surrogate")
            end
            local low = tonumber(string.sub(source, pos + 2, pos + 5), 16)
            if not low or low < 0xDC00 or low > 0xDFFF then
              fail("bad surrogate pair")
            end
            pos = pos + 6
            cp = 0x10000 + (cp - 0xD800) * 0x400 + (low - 0xDC00)
          elseif cp >= 0xDC00 and cp <= 0xDFFF then
            fail("unpaired surrogate")
          end
          out[#out + 1] = utf8Char(cp)
        else
          fail("bad escape")
        end
      elseif string.byte(c) < 0x20 then
        fail("control byte in string")
      else
        out[#out + 1] = c
        pos = pos + 1
      end
    end
  end

  local function parseNumber()
    local literal = string.match(string.sub(source, pos), "^-?%d+%.?%d*[eE]?[+-]?%d*")
    if not literal then
      fail("unexpected token")
    end
    local number = tonumber(literal)
    if not number then
      fail("bad number")
    end
    pos = pos + #literal
    return number
  end

  local function parseTable(isObject)
    local result = {}
    pos = pos + 1
    skip()
    if string.sub(source, pos, pos) == (isObject and "}" or "]") then
      pos = pos + 1
      return result
    end
    while true do
      skip()
      local key
      if isObject then
        if string.sub(source, pos, pos) ~= '"' then
          fail("object key")
        end
        key = parseString()
        skip()
        if string.sub(source, pos, pos) ~= ":" then
          fail("object colon")
        end
        pos = pos + 1
        skip()
      end
      local value = parseValue()
      if isObject then
        result[key] = value
      else
        result[#result + 1] = value
      end
      skip()
      local c = string.sub(source, pos, pos)
      if c == (isObject and "}" or "]") then
        pos = pos + 1
        return result
      elseif c ~= "," then
        fail("table separator")
      end
      pos = pos + 1
      skip()
    end
  end

  parseValue = function()
    skip()
    local c = string.sub(source, pos, pos)
    if c == "{" then
      return parseTable(true)
    elseif c == "[" then
      return parseTable(false)
    elseif c == '"' then
      return parseString()
    elseif string.sub(source, pos, pos + 3) == "true" then
      pos = pos + 4
      return true
    elseif string.sub(source, pos, pos + 4) == "false" then
      pos = pos + 5
      return false
    elseif string.sub(source, pos, pos + 3) == "null" then
      pos = pos + 4
      return nil
    end
    return parseNumber()
  end

  local value = parseValue()
  skip()
  if pos <= #source then
    fail("trailing data")
  end
  return value
end

local function jsonString(value)
  local escaped = string.gsub(value, '[%c"\\]', function(c)
    if c == '"' then
      return '\\"'
    elseif c == "\\" then
      return "\\\\"
    elseif c == "\n" then
      return "\\n"
    elseif c == "\r" then
      return "\\r"
    elseif c == "\t" then
      return "\\t"
    end
    return string.format("\\u%04x", string.byte(c))
  end)
  return '"' .. escaped .. '"'
end

local encodeValue

local function isArray(value)
  local count = 0
  for key in pairs(value) do
    if type(key) ~= "number" or key % 1 ~= 0 or key < 1 then
      return false
    end
    count = count + 1
  end
  for i = 1, count do
    if value[i] == nil then
      return false
    end
  end
  return true
end

local function encodeTable(value)
  local parts = {}
  if isArray(value) then
    for i = 1, #value do
      parts[i] = encodeValue(value[i])
    end
    return "[" .. table.concat(parts, ",") .. "]"
  end
  local keys = {}
  for key in pairs(value) do
    if type(key) == "string" then
      keys[#keys + 1] = key
    end
  end
  table.sort(keys)
  for i = 1, #keys do
    parts[i] = jsonString(keys[i]) .. ":" .. encodeValue(value[keys[i]])
  end
  return "{" .. table.concat(parts, ",") .. "}"
end

encodeValue = function(value)
  local kind = type(value)
  if value == nil then
    return "null"
  elseif kind == "boolean" then
    return value and "true" or "false"
  elseif kind == "number" then
    if value ~= value or value == math.huge or value == -math.huge then
      return "null"
    elseif value % 1 == 0 and math.abs(value) < 1e15 then
      return string.format("%.0f", value)
    end
    return string.format("%.9g", value)
  elseif kind == "string" then
    return jsonString(value)
  elseif kind == "table" then
    return encodeTable(value)
  end
  return "null"
end

local function writeResult(result)
  local ok, line = pcall(encodeValue, result)
  if not ok or type(line) ~= "string" then
    line = '{"error":"result encode failed","ok":false}'
  end
  local opened, file = pcall(io.open, RESULT_FILE, "ab")
  if not opened or not file then
    return
  end
  pcall(function()
    file:write(line, "\n")
    file:flush()
  end)
  pcall(file.close, file)
end

local function loadAnswered()
  local opened, file = pcall(io.open, RESULT_FILE, "rb")
  if not opened or not file then
    return
  end
  local data = file:read("*a") or ""
  file:close()
  for line in string.gmatch(data, "[^\n]+") do
    local ok, record = pcall(jsonDecode, line)
    if ok and type(record) == "table" and record.id ~= nil then
      local id = text(record.id)
      if id then
        answered[id] = true
      end
    end
  end
end

local function installDialog()
  local handler = rawget(_G, "DialogHandler")
  if type(handler) ~= "table" then
    return false
  end
  local setText = rawget(handler, "SetDialogText")
  if type(setText) == "function" then
    rawset(handler, "SetDialogText", function(...)
      local args = { ... }
      dialogText = text(args[2])
      dialogRows = {}
      return setText(unpack(args, 1, select("#", ...)))
    end)
  end
  local setChoice = rawget(handler, "SetChoiceText")
  if type(setChoice) == "function" then
    rawset(handler, "SetChoiceText", function(...)
      local args = { ... }
      local index = tonumber(args[2])
      if index == nil then
        index = #dialogRows
      end
      if #dialogRows < MAX_ROWS then
        dialogRows[#dialogRows + 1] = { index = index, text = text(args[3]) or "" }
      end
      return setChoice(unpack(args, 1, select("#", ...)))
    end)
  end
  return true
end

local function installMenu(name, store)
  local menu = rawget(_G, name)
  if type(menu) ~= "table" then
    return false
  end
  local function wrap(key, make)
    local original = rawget(menu, key)
    if type(original) == "function" then
      rawset(menu, key, make(original))
    end
  end
  wrap("setButtonCount", function(original)
    return function(...)
      store.buttons = {}
      return original(...)
    end
  end)
  wrap("setButton", function(original)
    return function(self, index, icon, label, event, ...)
      if #store.buttons < MAX_ROWS then
        store.buttons[#store.buttons + 1] = {
          index = tonumber(index),
          text = text(label) or "",
          event = tonumber(event),
        }
      end
      return original(self, index, icon, label, event, ...)
    end
  end)
  wrap("onPopup", function(original)
    return function(...)
      store.shown = true
      return original(...)
    end
  end)
  wrap("hide", function(original)
    return function(...)
      store.shown = false
      return original(...)
    end
  end)
  store.shown = menu.isShown == true
  return true
end

local function openWindows()
  local names = {}
  local seen = {}
  local function add(name)
    if type(name) ~= "string" or name == "" or seen[name] or #names >= MAX_WINDOWS then
      return
    end
    seen[name] = true
    names[#names + 1] = name
  end
  for name, value in pairs(_G) do
    if type(name) == "string" and type(value) == "table" and value ~= _G then
      local shown = rawget(value, "isShown") == true or rawget(value, "isShown") == 1
        or rawget(value, "isVisible") == true or rawget(value, "visible") == true
        or rawget(value, "active") == true
      if shown then
        add(rawget(value, "wndName") or rawget(value, "windowName")
          or rawget(value, "WindowName") or name)
      end
    end
  end
  local windows = rawget(_G, "UIWindow")
  if type(windows) == "table" and type(rawget(windows, "openedWindows")) == "table" then
    for name in pairs(rawget(windows, "openedWindows")) do
      add(name)
    end
  end
  table.sort(names)
  return names
end

local function buildState()
  local state = {}
  local ui = rawget(_G, "Ui")
  if type(ui) == "table" then
    if type(ui.GetAtlasName) == "function" then
      local ok, zone = pcall(ui.GetAtlasName)
      if ok and type(zone) == "string" and zone ~= "" then
        state.zone = zone
      end
    end
    if type(ui.GetPlayerLoc) == "function" then
      local ok, x, y, z = pcall(ui.GetPlayerLoc)
      if ok and tonumber(x) and tonumber(y) and tonumber(z) then
        state.position = { tonumber(x), tonumber(y), tonumber(z) }
      end
    end
    if type(ui.GetPlayerHeading) == "function" then
      local ok, heading = pcall(ui.GetPlayerHeading)
      if ok and tonumber(heading) then
        state.heading = tonumber(heading)
      end
    end
  end

  local tables = rawget(_G, "DsTable")
  if type(tables) == "table" and type(tables.Find) == "function" then
    local ok, data = pcall(tables.Find, "BaseClient.ClosestInteractionTargetData")
    if ok and data then
      local found, name = pcall(function()
        return data:GetData(0, 0)
      end)
      name = found and text(name) or nil
      if name and name ~= "" and name ~= "-1" then
        state.closest = name
      end
    end
  end

  local target = rawget(_G, "Target")
  if type(target) == "table" then
    local shown = target.isShown == true
    local guid = text(target.lastTarget)
    if shown or (guid and guid ~= "" and guid ~= " ") then
      state.target = { shown = shown }
      if guid and guid ~= "" and guid ~= " " then
        state.target.guid = guid
      end
    end
  end

  local dialog = rawget(_G, "DialogHandler")
  if type(dialog) == "table" then
    local shown = dialog.isDialogShown == true
    if shown or dialogText or #dialogRows > 0 then
      local record = { shown = shown }
      if dialogText then
        record.text = dialogText
      end
      if #dialogRows > 0 then
        local rows = {}
        for i = 1, #dialogRows do
          rows[i] = { index = dialogRows[i].index, text = dialogRows[i].text }
        end
        record.rows = rows
      end
      state.dialog = record
    end
  end

  local function menuState(name, store)
    local menu = rawget(_G, name)
    local shown = store.shown or (type(menu) == "table" and menu.isShown == true)
    if not shown and #store.buttons == 0 then
      return nil
    end
    local record = { shown = shown }
    if #store.buttons > 0 then
      local buttons = {}
      for i = 1, #store.buttons do
        buttons[i] = {
          index = store.buttons[i].index,
          text = store.buttons[i].text,
          event = store.buttons[i].event,
        }
      end
      record.buttons = buttons
    end
    return record
  end

  state.radial = menuState("radialMenu", radial)
  state.housing = menuState("HousingRadialMenu", housing)
  local windows = openWindows()
  if #windows > 0 then
    state.windows = windows
  end
  return state
end

local function findRow(rows, command, label)
  local wanted = tonumber(command.index)
  if wanted ~= nil then
    for i = 1, #rows do
      if rows[i].index == wanted then
        return rows[i]
      end
    end
    error(string.format("no %s row with index %d", label, wanted), 0)
  end
  if type(command.text) ~= "string" or command.text == "" then
    error(string.format("click %s needs text or index", label), 0)
  end
  local wantedText = string.lower(command.text)
  local insensitive, matches = nil, {}
  for i = 1, #rows do
    local rowText = string.lower(rows[i].text or "")
    if rows[i].text == command.text then
      return rows[i]
    elseif rowText == wantedText and insensitive == nil then
      insensitive = rows[i]
    elseif string.find(rowText, wantedText, 1, true) then
      matches[#matches + 1] = rows[i]
    end
  end
  if insensitive then
    return insensitive
  end
  if #matches == 1 then
    return matches[1]
  end
  if #matches > 1 then
    local labels = {}
    for i = 1, math.min(#matches, 5) do
      labels[#labels + 1] = matches[i].text
    end
    error(string.format("ambiguous %s text %q matches: %s", label, command.text,
      table.concat(labels, ", ")), 0)
  end
  error(string.format("no %s row matches %q", label, command.text), 0)
end

local function clickMenu(menuName, store, command)
  local menu = rawget(_G, menuName .. "Menu")
  if type(menu) ~= "table" or (menu.isShown ~= true and not store.shown) then
    error(menuName .. " menu is not shown", 0)
  end
  local row = findRow(store.buttons, command, menuName)
  if row.event == nil then
    error(string.format("%s row %q has no event id", menuName, row.text), 0)
  end
  local ui = rawget(_G, "Ui")
  if type(rawget(menu, "OnUserEvent")) == "function" then
    rawget(menu, "OnUserEvent")("radialMenuClick", row.event)
  elseif type(ui) == "table" and type(ui.RadialMenuClicked) == "function" then
    ui.RadialMenuClicked(row.event)
  else
    error("no radial menu click handler is loaded", 0)
  end
  return { ok = true, target = menuName, index = row.index, text = row.text, event = row.event }
end

local function runCommand(command)
  local name = command.cmd
  if name == "state" then
    local state = buildState()
    state.ok = true
    return state
  elseif name == "click" then
    local target = command.target or "dialog"
    if target == "dialog" then
      local handler = rawget(_G, "DialogHandler")
      if type(handler) ~= "table" or handler.isDialogShown ~= true then
        error("no dialog is shown", 0)
      end
      local row = findRow(dialogRows, command, "dialog")
      if type(handler.SelectOption) ~= "function" then
        error("DialogHandler:SelectOption is unavailable", 0)
      end
      handler:SelectOption(row.index)
      return { ok = true, target = "dialog", index = row.index, text = row.text }
    elseif target == "radial" then
      local handled = clickMenu("radial", radial, command)
      handled.target = "radial"
      return handled
    elseif target == "housing" then
      local handled = clickMenu("HousingRadial", housing, command)
      handled.target = "housing"
      return handled
    end
    error("unknown click target: " .. tostring(target), 0)
  elseif name == "chat" then
    if type(command.text) ~= "string" or command.text == "" or #command.text > 1000 then
      error("chat needs a text of 1 to 1000 bytes", 0)
    end
    local ui = rawget(_G, "Ui")
    if type(ui) ~= "table" or type(ui.ProcessChatCommand) ~= "function" then
      error("Ui.ProcessChatCommand is unavailable", 0)
    end
    ui.ProcessChatCommand(command.text)
    return { ok = true, text = command.text }
  elseif name == "interact" then
    local ui = rawget(_G, "Ui")
    if type(ui) ~= "table" or type(ui.DoClosestInteraction) ~= "function" then
      error("Ui.DoClosestInteraction is unavailable", 0)
    end
    ui.DoClosestInteraction()
    return { ok = true }
  elseif name == "key" or name == "move" then
    error("unsupported: the client Lua cannot inject keys or movement; use interact, click or chat", 0)
  end
  error("unknown command: " .. tostring(name), 0)
end

local function dispatchLine(line)
  if line:match("^%s*$") then
    return
  end
  local parsed, command = pcall(jsonDecode, line)
  if not parsed then
    writeResult({ ok = false, error = "invalid json: " .. (text(command) or "?") })
    return
  end
  if type(command) ~= "table" then
    writeResult({ ok = false, error = "command must be a json object" })
    return
  end
  if type(command.id) ~= "string" and type(command.id) ~= "number" then
    writeResult({ ok = false, error = "command id must be a string or number" })
    return
  end
  local id = text(command.id)
  if id and answered[id] then
    return
  end
  local ran, result = pcall(runCommand, command)
  if not ran then
    result = { ok = false, error = text(result) or "command failed" }
  end
  if type(result) ~= "table" then
    result = { ok = false, error = "command produced no result" }
  end
  result.id = command.id
  result.cmd = command.cmd
  if id then
    answered[id] = true
  end
  writeResult(result)
end

local function poll()
  -- Binary mode: offsets are file bytes, so a hand-written CRLF command file cannot
  -- shift the read cursor the way text-mode translation would.
  local opened, file = pcall(io.open, COMMAND_FILE, "rb")
  if not opened or not file then
    return
  end
  local size = file:seek("end")
  if not size or size < readOffset then
    readOffset = 0
    pending = ""
  end
  file:seek("set", readOffset)
  local chunk = file:read(MAX_POLL_BYTES) or ""
  file:close()
  -- The offset is past every byte read; an unprocessed tail lives in `pending`
  -- and must not be read again. A capped poll leaves complete lines in `pending`
  -- and the next call processes them even when no new bytes arrive.
  readOffset = readOffset + #chunk
  if chunk == "" and pending == "" then
    return
  end
  local buffered = pending .. chunk
  local start = 1
  local commands = 0
  while commands < MAX_COMMANDS_PER_POLL do
    local newline = string.find(buffered, "\n", start, true)
    if not newline then
      break
    end
    local line = string.sub(buffered, start, newline - 1)
    if string.sub(line, -1) == "\r" then
      line = string.sub(line, 1, -2)
    end
    start = newline + 1
    commands = commands + 1
    if #line > MAX_LINE_BYTES then
      writeResult({ ok = false, error = "command line exceeds the byte bound" })
    else
      pcall(dispatchLine, line)
    end
  end
  pending = string.sub(buffered, start)
end

local hooks = { dialog = false, radial = false, housing = false }pcall(function()
  hooks.dialog = installDialog()
  hooks.radial = installMenu("radialMenu", radial)
  hooks.housing = installMenu("HousingRadialMenu", housing)
  loadAnswered()
end)

local update = rawget(_G, "OnUpdate")
local wrapped = false
if type(update) == "function" then
  rawset(_G, "OnUpdate", function(...)
    update(...)
    frames = frames + 1
    if frames % POLL_FRAMES == 0 then
      pcall(poll)
    end
  end)
  wrapped = true
end

writeResult({
  event = "ready",
  version = 1,
  commands = COMMAND_FILE,
  results = RESULT_FILE,
  poll_frames = POLL_FRAMES,
  on_update = wrapped,
  dialog = hooks.dialog,
  radial = hooks.radial,
  housing = hooks.housing,
})
