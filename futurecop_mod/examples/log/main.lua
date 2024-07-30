local dangerous = require("dangerous")
local util = require("./util")
local game = require("game")

function onLoad()
  util.log("test")

  function load_mission_file_hook(load_mission_file, mission_file)
    print(mission_file)
    local response = load_mission_file(mission_file)
    print(response)

    return response
  end

  dangerous.hook(0x00405ee0, {"string"}, "int", load_mission_file_hook)
  print("main")
end

function onEnable()
  print("on_enabled")
end

function onDisable()
  print("on_disabled")
end

function onUpdate()
  local state = game.getState()

  if not state.isInMission then
    return
  end

  local playerOne = game.getPlayer(0)

  local ammo = {}
  ammo.gunWeapon = playerOne.gunWeaponAmmo
  ammo.heavyWeapon = playerOne.heavyWeaponAmmo
  ammo.specialWeapon = playerOne.specialWeaponAmmo

  print(`Gun Weapon: {ammo.gunWeapon}`)
  print(`Heavy Weapon: {ammo.heavyWeapon}`)
  print(`Special Weapon: {ammo.specialWeapon}`)
end