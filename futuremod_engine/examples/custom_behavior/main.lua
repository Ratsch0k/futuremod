local dangerous = require("dangerous")
local bit32 = require("bit32")
local math = require("math")
local matrix = require("matrix")

--------------------------------------
--- Globals used in the entire plugin
--------------------------------------

local customBehaviorFunctionNative = nil
local behaviorA0Function = nil
local renderObject = nil
local entityDefinition = nil
local behaviorDataDefinition = nil
local playerEntityDefinition = nil
local actorDataRefDefinition = nil
local playerDefinition = nil

--- All healing station instances.
--- Uses the entitiy's ID to identify an instance.
--- An instance contains data associated with a specific instance
--- of a healing station. We could also store this information in memory.
local instances: InstanceTable = {}
type EntityInstance = {
  coolDown: number,
  coolDownTimer: number,
  triggeredByPlayer: PlayerEntity?,
  targetPlayer: number,
  address: number,
}
type InstanceTable = {[number]: EntityInstance}

-------------------------------------------------
--- Type definitions of native data
-------------------------------------------------

--- Actor data is stored in the mission file
type BehaviorData = {
  actorId: number,
  posX: number,
  posY: number,
  posZ: number,
  rotation: number,
  targetPlayer: number,
  coolDown: number,
}

--- Entity data
type Entity = {
  id: number,
  modelMatrix: number,
  posX: number,
  posY: number,
  posZ: number,
  textureOffset: number,
  hoverItemModelRef: number,
  hoverItemMatrix: number,
}

--- References to the different data section as the actor is stored in mission files
type ActorDataRefs = {
  behaviorFunctionAddress: number,
  resourceData: number,
  behaviorData: number,
  actorData: number,
}


type PlayerEntity = {
  playerId: number,
  behavior: number,
  health: number,
  maxHealth: number,
  player: number,
}

type Player = {
  currentAction: number,
}

--- Called when the plugin is loaded.
--- Initilalizses globals and sets up hook for the plugin to work properly.
function onLoad()
  entityDefinition = dangerous.createNativeStructDefinition({
    id={type="int", offset=0x8},
    modelMatrix={type=matrix.ModelMatrix, offset=0x2c},
    posX={type="int",offset=0x50},
    posY={type="int",offset=0x54},
    posZ={type="int",offset=0x58},
    textureOffset={type="short",offset=0x92},
    hoverItemModelRef={type="int",offset=0xa0},
    hoverItemMatrix={type=matrix.ModelMatrix, offset=0xa4},
  })
  behaviorDataDefinition = dangerous.createNativeStructDefinition({
    behavior={type="int", offset=0x00},
    actorId={type="int", offset=0x04},
    posX={type="int", offset=0x08},
    posY={type="int", offset=0x0c},
    posZ={type="int", offset=0x10},
    unknown1={type="int", offset=0x14},  -- You don't have to specify unknown or unimportant fields 
    unknown2={type="int", offset=0x18},  -- because you can use the offset to skip such fields
    unknown3={type="int", offset=0x1c},
    unknown4={type="int", offset=0x20},
    rotation={type="short", offset=0x24},  -- The next fields are custom fields specifically for this custom behavior
    targetPlayer={type="short", offset=0x26},
    coolDown={type="int", offset=0x28},
  })
  actorDataRefDefinition = dangerous.createNativeStructDefinition({
    behaviorFunctionAddress={type="int", offset=0x00},
    resourceData={type="int", offset=0x04},
    behaviorData={type="int", offset=0x08},
    actorData={type="int", offset=0x0c},
  })
  playerEntityDefinition = dangerous.createNativeStructDefinition({
    playerId={type="short", offset=0x14},
    behavior={type="short", offset=0x16},
    health={type="ushort", offset=0x1c},
    maxHealth={type="ushort", offset=0x1e},
    player={type="int", offset=0xac},
  })
  playerDefinition = dangerous.createNativeStructDefinition({
    currentAction={type="uint", offset=0x30},
  })
  behaviorA0Function = dangerous.getNativeFunction(0x0041a420, {"int", "int", "int", "int"}, "int");
  renderObject = dangerous.getNativeFunction(0x004280a0, {"int", "int", "int"}, "int")
  customBehaviorFunctionNative = dangerous.createNativeFunction({"int", "int", "int", "int"}, "int", healingStationBehavior)
end

local behaviorHook = nil
--- Called when the user enables this plugin.
--- Hooking and other stuff intrusive things should only be performed if this function is called.
--- Otherwise, the plugin is not enabled.
function onEnable()
  behaviorHook = dangerous.hook(0x0041a950, {"int"}, "int", getBehaviorFunctionHook)
end

--- Unhook hooks when the plugin is disabled.
--- When using hooks, the plugin must ensure that it also unhooks them.
function onDisable()
  behaviorHook:unhook()
end

--- Usually, you should disable the plugin when this function is called.
--- Howver, currently, hooks are irreversible. Therefore, there is nothing to stop.
--- If hooks can be disabled in the future, the `onDisable` function could look like this.
--- (Depends on how the API is implemented)
--[[
function onDisable()
  dangerous.unhook(0x0041a950)
end
]]--

--- Hook for the `getBehaviorFunction` used by the game to fetch the behavior function
--- corresponding to a behavior.
--- 
--- For all behaviors that are not our custom behavior, we pass the arguments to the original function.
function getBehaviorFunctionHook(original, behavior)
  if behavior == 0x6f then
    local customAddress = customBehaviorFunctionNative:getAddress()

    return customAddress
  end

  return original(behavior)
end

--- Returns the approximate size, the healing station needs in memory.
--- We don't actually need this much memory.
function healingStationGetSize(event, arg2, arg3, arg4)
  return 0x120
end

--- Initializes the healing station entity from the entity data from the mission file.
function healingStationInit(event, obj: number, dataRefs, arg4)
  -- Call default behavior function to initialize the base model
  behaviorA0Function:call(event, obj, dataRefs, arg4)

  local actorDataRefs: ActorDataRefs = actorDataRefDefinition:cast(dataRefs)
  local behaviorData: BehaviorData = behaviorDataDefinition:cast(actorDataRefs.behaviorData)
  local entity: Entity = entityDefinition:cast(obj)

  -- Overwrite the main model matrix with a properly rotated matrix
  local mainMatrix = matrix.newModel()
  mainMatrix:uniformScale(0x010000)
  mainMatrix:rotate(0, 1, 0, math.rad(behaviorData.rotation))
  mainMatrix:translate(behaviorData.posX, behaviorData.posY, behaviorData.posZ)
  entity.modelMatrix = mainMatrix

  -- Initialize rest of the behavior data
  local secondModelRef = getSecondObjectRefFromDataRefs(actorDataRefs)

  entity.hoverItemModelRef = secondModelRef

  local posX = entity.posX
  local posY = entity.posY
  local posZ = entity.posZ

  local m = matrix.newModel()

  m:uniformScale(0x010000)
  m:translate(0, -0x1000, 0)
  m:rotate(-1, 0, 0, math.rad(45))
  m:rotate(0, 1, 0, math.rad(behaviorData.rotation))
  m:translate(posX, posY + 0x1700, posZ)
  entity.hoverItemMatrix = m

  -- Change texture offset to change appearance of station
  entity.textureOffset = 0x90

  -- We don't have to store everything about an object in native memory
  -- By referencing the object's ids we can manage some of the state in lua

  -- Get id of healing station and store in instances table
  instances[entity.id] = {
    address=obj,
    coolDownTimer=0,
    coolDown=behaviorData.coolDown,
    triggeredByPlayer=nil,
    targetPlayer=behaviorData.targetPlayer,
  }

  return 0
end

--- Handles update events.
---
--- If the station is currently in cool down, it counts down the cool down.
--- If the station was triggered by a player, it heals the player and starts
--- the count down.
function healingStationUpdate(event: number, obj: number, gameSpeed: number, _)
  local entity: Entity = entityDefinition:cast(obj)
  local instance = instances[entity.id]

  if (instance.coolDownTimer > 0) then
    instance.coolDownTimer = math.max(instance.coolDownTimer - gameSpeed, 0)
  end

  if (instance.triggeredByPlayer) then
    instance.triggeredByPlayer.health = instance.triggeredByPlayer.maxHealth
    instance.triggeredByPlayer = nil
    instance.coolDownTimer = instance.coolDown
  end

  return 0
end

--- Handles rendering the station and item model.
--- Only renders the item model if the station is not in cool down.
function healingStationRender(event, entityAddr: number, arg3, arg4)
  local entity: Entity = entityDefinition:cast(entityAddr)

  -- Render base
  behaviorA0Function:call(event, entityAddr, arg3, arg4)

  -- Render second object if healing station is not in cool down
  local instance = instances[entity.id]

  if (instance.coolDownTimer <= 0) then
    -- Get address to second model data
    local hoverItemModelPtr = entityAddr + 0xa0
    local valuePtr = entityAddr + 0x100;

    renderObject:call(hoverItemModelPtr, valuePtr, 1)
  end

  return 0
end

--- Handles events when an entity is near the healing station.
---
--- If the healing station's cool down is currently running, it completely ignores all
--- entities.
--- Otherwise, it checks if the nearby entity is a player triggering the station.
--- If so, it stores a reference to the player in the stations instance.
function healingStationTriggered(event: number, entityAddr: number, triggerEntity, _)
  local entity: Entity = entityDefinition:cast(entityAddr)
  local instance = instances[entity.id]

  if (instance.coolDownTimer > 0) then
    return 1
  end

  local playerEntity: PlayerEntity = playerEntityDefinition:cast(triggerEntity)
  if (didPlayerTrigger(instance.targetPlayer, playerEntity)) then
    instance.triggeredByPlayer = playerEntity
  end

  return 1
end

--- Default function for events where we don't need any custom logic.
--- Simply passes all arguments to the behavior function of behavior `0xa0`
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

--- Behavior function of the healing station actor.
--- Depending on the event, this function delegates handling events to the appropriate
--- functions.
---
--- At the beginning of the game, we create a native function wrapper for this function in `onLoad` and
--- pass the wrapper's address when the game asks for the behavior function of our custom behavior.
function healingStationBehavior(event, arg2, arg3, arg4)
  local handler = healingStationSwitch[event]

  if (handler) then
    local success, result = pcall(handler, event, arg2, arg3, arg4)

    if success then
      return result
    else
      print(`Healing station handler for event {event} failed for arguments ({arg2}, {arg3}, {arg4}) with:`)
      print(`{result}`)
      return 0
    end
  end

  return behaviorA0Function:call(event, arg2, arg3, arg4)

end

--- Get the address to the model data of the item model.
--- In the resource references the item model is the second object.
function getSecondObjectRefFromDataRefs(dataRef: ActorDataRefs)
  -- We could also cast these addresses into native struct but we only ever access one field.
  -- So this is easier, faster, and less code.
  local secondModelRef = dangerous.readMemory(dataRef.resourceData + 0x10, "uint")
  local secondModelDataRef = dangerous.readMemory(secondModelRef, "uint")

  return secondModelDataRef
end

--- Check if the given entity is a player entity who tried to trigger the healing station.
---
--- It checks if the entity's beahvior is that of a player. If so, it checks if the player
--- is the targeted player (Player 1 or 2) and if the player currently pressed the action key.
--- If all are true, the function returns true, otherwise it returns false.
--- This function only checks the player entity and does not consider the healing station's state.
function didPlayerTrigger(targetPlayer: number, playerEntity: PlayerEntity): boolean
  -- Check if colliding entity is a player entity
  if (playerEntity.behavior ~= 1) then
    return false
  end

  -- Check if the entity's player ID matches the ID that this healing station searches for
  if (playerEntity.playerId ~= targetPlayer) then
    return false
  end

  -- Cast the player
  local player: Player = playerDefinition:cast(playerEntity.player)

  -- Check if player presses action key
  local currentAction = player.currentAction
  if (bit32.band(currentAction, 0x02000000) == 0) then
    return false
  end

  return true
end