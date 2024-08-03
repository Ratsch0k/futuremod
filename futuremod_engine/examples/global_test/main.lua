
local table = {
  some="field",
  other=1.0
}

function onLoad()
  print("Iterate through table")
  for key, value in pairs(table) do
    print(`{key}: {value}`)
  end

  local status, result = pcall(throwError)

  if not status then
    print(`Error: {result}`)
  end
end

function throwError()
  error("some error")
end