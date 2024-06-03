local ui = require("ui")
local system = require("system")
local math = require("math")

local lastTime = system.getTime()
local fpsMean = 0
local counter = 0
local lastFps = 0

function onUpdate()
  local delta = system.getTime() - lastTime
  local fps = 1000 / delta
  fpsMean += fps

  if counter >= 4 then
    fpsMean = fpsMean / 5
    lastFps = fpsMean

    fpsMean = 0
    counter = 0
  else
    counter += 1
  end

  lastFps = math.round(lastFps)

  ui.renderText(`FPS: {lastFps}`, 0, 0, ui.PaletteWhite)

  lastTime = system.getTime()
end