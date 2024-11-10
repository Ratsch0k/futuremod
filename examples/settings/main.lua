local settings = require("settings")

function onLoad()
  createSettings()

  print(`Creation took {diff} ms`)
end

function createSettings()
  settings:create({
    settings.Section({
      settings.Text("Example Settings"),
      settings.Text("By clicking the button below, you should be able to trigger some code of the plugin. How neat!!!"),
      settings.Button("Click Me"):onClick(function()
        print("Clicked")
      end)
      :withID("button")
    }),
    settings.Section({
      settings.Text("Next Section"),
      settings.Button("Other butotn"):withDisabled(true)
    })
  })
end