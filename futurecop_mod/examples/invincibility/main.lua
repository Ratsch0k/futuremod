local game = require("game")

function onUpdate()
  local state = game.getState()

  if not state.isInMission then
    return
  end

  local playerOne = game.getPlayer(0)

  healPlayer(playerOne)

  if state.playerCount == 2 then
    local playerTwo = game.getPlayer(1)
    healPlayer(playerTwo)
  end
end

function healPlayer(player)
  player.health = player:getMaxHealth()
end