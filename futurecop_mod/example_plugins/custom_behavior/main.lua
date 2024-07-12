local dangerous = require("dangerous")
local bit32 = require("bit32")
local math = require("math")

local customBehaviorFunctionNative = nil
local behaviorA0Function = nil
local renderObject = nil

local instances = {}

function healingStationGetSize(event, arg2, arg3, arg4)
  return 0x120
end

function healingStationInit(event, obj, dataRefs, arg4)
  -- Call default behavior function to initialize the base model
  behaviorA0Function:call(event, obj, dataRefs, arg4)

  -- Initialize rest of the behavior data
  local secondModelRef = getSecondObjectRefFromDataRefs(dataRefs)

  dangerous.writeMemory(obj + 0xa0, secondModelRef)
  dangerous.writeMemory(obj + 0xa4, 0x010000)
  dangerous.writeMemory(obj + 0xa8, 0)
  dangerous.writeMemory(obj + 0xac, 0)
  dangerous.writeMemory(obj + 0xb0, 0)
  dangerous.writeMemory(obj + 0xb4, 0x010000)
  dangerous.writeMemory(obj + 0xb8, 0)
  dangerous.writeMemory(obj + 0xbc, 0)
  dangerous.writeMemory(obj + 0xc0, 0)
  dangerous.writeMemory(obj + 0xc4, 0x010000)

  local basePositionAddr = obj + 0x50
  local posX = dangerous.readMemory(basePositionAddr, "int")
  local posY = dangerous.readMemory(basePositionAddr + 0x4, "int")
  local posZ = dangerous.readMemory(basePositionAddr + 0x8, "int")

  -- Change texture offset to change appearance of station
  dangerous.writeMemory(obj + 0x92, {0x90})

  dangerous.writeMemory(obj + 0xc8, posX)
  dangerous.writeMemory(obj + 0xcc, posY + 0xd00)
  dangerous.writeMemory(obj + 0xd0, posZ)
  dangerous.writeMemory(obj + 0xd4, posX)
  dangerous.writeMemory(obj + 0xd8, posY + 0xd00)
  dangerous.writeMemory(obj + 0xdc, posZ)

  -- We don't have to store everything about an object in native memory
  -- By referencing the object's ids we can manage some of the state in lua

  -- Get id of healing station and store in instances table
  local id = dangerous.readMemory(obj + 0xc, "int")
  instances[id] = {
    ["address"]=obj,
    ["coolDownTimer"]=0,
    ["coolDown"]=0x2ff,
    ["triggeredByPlayer"]=nil,
  }

  return 0
end

function getInstance(obj)
  local id = dangerous.readMemory(obj + 0xc, "int")

  return instances[id]
end

function getSecondObjectRefFromDataRefs(dataRef)
  local resourceRef = dangerous.readMemory(dataRef + 4, "int")

  local secondModelRef = dangerous.readMemory(resourceRef + 16, "int")

  return dangerous.readMemory(secondModelRef, "int")
end

function healingStationUpdate(event, obj, gameSpeed, _)
  local instance = getInstance(obj)

  if (instance.coolDownTimer > 0) then
    instance.coolDownTimer = math.max(instance.coolDownTimer - gameSpeed, 0)
  end

  if (instance.triggeredByPlayer) then
    healPlayer(instance.triggeredByPlayer)
    instance.triggeredByPlayer = nil
    instance.coolDownTimer = instance.coolDown
  end

  return 0
end

function healPlayer(player)
  local playerHealthAddr = player + 0x1c

  local maxHealth = dangerous.readMemory(playerHealthAddr + 0x2, "short")
  local maxHealthByte1 = bit32.band(maxHealth, 0xff)
  local maxHealthByte2 = bit32.rshift(bit32.band(maxHealth, 0xff00), 8)
  dangerous.writeMemory(playerHealthAddr, {maxHealthByte1, maxHealthByte2})
end

function healingStationRender(event, obj, arg3, arg4)
  -- Render base
  behaviorA0Function:call(event, obj, arg3, arg4)

  -- Get address to second model data
  local secondModelRefAddr = obj + 0xa0;
  local valuePtr = obj + 0x100;

  -- Render second object if healing station is not in cool down
  local instance = getInstance(obj)

  if (instance.coolDownTimer <= 0) then
    renderObject:call(secondModelRefAddr, valuePtr, 1)
  end

  print("End of render")

  return 0
end

function didPlayerTrigger(triggerEntity)
  local behaviorType = dangerous.readMemory(triggerEntity + 0x16, "short")

  -- Check if colliding entity is a player entity
  if (behaviorType ~= 1) then
    return false
  end

  -- Check if player presses action key
  local player = dangerous.readMemory(triggerEntity + 0xac, "int")
  local currentAction = dangerous.readMemory(player + 0x30, "int")

  if (bit32.band(currentAction, 0x02000000) == 0) then
    return false
  end

  return true
end

function healingStationTriggered(event, obj, triggerEntity, _)

  local instance = getInstance(obj)

  if (instance.coolDownTimer > 0) then
    return 1
  end

  if (didPlayerTrigger(triggerEntity)) then
    instance.triggeredByPlayer = triggerEntity
  end

  return 1
end

function healingStationDefault(event, arg2, arg3, arg4)
  return behaviorA0Function:call(event, arg2, arg3, arg4)
end

local healingStationSwitch = {
  [1] = healingStationGetSize,
  [2] = healingStationInit,
  [3] = healingStationRender,
  [6] = healingStationUpdate,
  [10] = healingStationTriggered,
}

function customBehaviorFunction(event, arg2, arg3, arg4)
  local handler = healingStationSwitch[event]

  if (handler) then
    return handler(event, arg2, arg3, arg4)
  end

  return behaviorA0Function:call(event, arg2, arg3, arg4)

end

function getBehaviorFunctionHook(original, entityType)
  if entityType == 0x6f then
    local customAddress = customBehaviorFunctionNative:getAddress()

    return customAddress
  end

  return original(entityType)
end

function onLoad()
  behaviorA0Function = dangerous.getNativeFunction(0x0041a420, {"int", "int", "int", "int"}, "int");
  renderObject = dangerous.getNativeFunction(0x004280a0, {"int", "int", "int"}, "int")
  customBehaviorFunctionNative = dangerous.createNativeFunction({"int", "int", "int", "int"}, "int", customBehaviorFunction)
  dangerous.hook(0x0041a950, {"int"}, "int", getBehaviorFunctionHook)
end