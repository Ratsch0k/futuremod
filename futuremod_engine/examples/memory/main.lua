local dangerous = require("dangerous")
local input = require("input")

-- Memory addresses and offsets for player and weapon ammo
local PLAYER_ARRAY = 0x00511fd0
local PLAYER_ENTITY_OFFEST = 0xac
local GUN_WEAPON_OFFSET = 0x84
local HEAVY_WEAPON_OFFSET = 0x86
local SPECIAL_WEAPON_OFFSET = 0x88

local playerOneAddress = nil

local gunWeaponAmmoAddress = nil
local heavyWeaponAmmoAddress = nil
local specialWeaponAmmoAddress = nil

function onUpdate()
  if playerOneAddress == nil then
    local playerOneEntity = dangerous.readMemory(PLAYER_ARRAY, "int")
    print(`Player 1 Entity: {playerOneEntity}`)

    playerOneAddress = dangerous.readMemory(playerOneEntity + PLAYER_ENTITY_OFFEST, "int")
    print(`Player One: {playerOneAddress}`)

    gunWeaponAmmoAddress = playerOneAddress + GUN_WEAPON_OFFSET
    heavyWeaponAmmoAddress = playerOneAddress + HEAVY_WEAPON_OFFSET
    specialWeaponAmmoAddress = playerOneAddress + SPECIAL_WEAPON_OFFSET
  end

  -- Only refill ammo if the player pressed CTRL + R
  if input.isKeyPressed(input.KeyR) and (input.isKeyPressed(input.KeyLControl) or input.isKeyPressed(input.KeyRControl)) then
    -- Sets the ammo to a high value.
    -- We could get the weapon type and set the ammo to the according max value, but
    -- this is just an example.
    dangerous.writeMemory(gunWeaponAmmoAddress, {0x4c, 0x1d})
    dangerous.writeMemory(heavyWeaponAmmoAddress, {0x4c, 0x1d})
    dangerous.writeMemory(specialWeaponAmmoAddress, {0x4c, 0x1d})
  end
end