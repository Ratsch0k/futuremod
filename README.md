# FutureMod
Modding framework and manager for **Future Cop: LAPD**.

FutureMod lets you manage plugins to extend and change the game, as well as write your own mods, called _plugins_, in Lua using FutureMod's modding framework.
The modding frameworks allows plugins to interact with Future Cop's code and even changing Future Cop's code entirely.
For some identified functions and internal game processes, FutureMod offers an API that allows plugins to easily add their own functionality.
Even if FutureMod doesn't offer a custom API, it allows plugins to hook any arbitrary function and interact with the native assembly code.

_FutureMod is the result of my efforts to reverse engineer **Future Cop**._
_When I discover new functions or how the game works, I'm trying to add these functionalities to the plugin API._
_I'm documenting my findings and some of the development process of FutureMod on my [blog](https://blog.simonkurz.de/post/modding-futurecop-introduction/)._

## Install
For now, you have to manually clone this repository and build FutureMod yourself.

As FutureMod is built with [Rust](https://www.rust-lang.org/), you have to install rust beforehand.
You can install Rust be following their [official guide](https://www.rust-lang.org/learn/get-started).

Install FutureMod by running the following commands:
```bash
# Clone the repository
git clone https://github.com/Ratsch0k/futurecop

# Build FutureMod
cargo build --release
```
After FutureMod was build successfully, its executable should be at `<repository>/target/i686-pc-windows-msvc/release/futurecop-mod-injector.exe`.

If you move the executable, make sure to also move the file `futurecop_mod.dll` (located in the same folder as the executable) alongside because it is required by FutureMod.
However, you don't have to move the DLL.
By default, FutureMod expects the DLL to be in the same directory.
If it isn't, it prompts you to manually select the path to the DLL.
You can permanently change the path where FutureMod expects the DLL by adjusting the field `modPath` in the FutureMod's config file.

FutureMod automatically creates the config file the first time you start it or if it cannot find the config file.
The config file is in the same directory as FutureMod and called `config.json`.

## Project Structure
FutureMod consists of two parts: the GUI/injector and the mod.

When the user starts the GUI, it waits for Future Cop to start.
When Future Cop runs, the GUI injects the mod into Future Cop and waits for the mod to initialize itself.
The mod hooks itself into the game, initializes the internal plugin manager, loads and enables installed plugins, and starts a local webserver.
When the mod finishes initializing, the GUI connects to the mod's webserver and now acts as the plugin manager.

FutureMod is written in Rust and consists of three packages:
- `futurecop_injector`: The FutureMod GUI. Injects `futurecop_mod` and allows users to interact with it.
- `futurecop_mod`: The actual mod that is injected into Future Cop to run plugins
- `futurecop_data`: Data and code shared by both `futurecop_injector` and `futurecop_mod`

### GUI/Injector
The GUI powered by [iced](https://iced.rs/) using a partially custom theme.
It consists of several mostly independent views.
Each view is located in the `views` directory at `futurecop_injector/src/view`.

### Mod
The mod manages plugins, the plugin API that allows plugins to interact with Future Cop, and sets up a local webserver that the GUI can connect to.
The mod injects itself into some of the game's logic and processes.
However, all modifications are done in memory, and all game files are left untouched.

All code for managing plugin's is located at `futurecop_mod/src/plugins`.
The API is split into several libraries, each responsible for their own category of interaction.
The code for the API libraries is located at `futurecop_mod/src/plugins/library`.

Lua plugins are powered by [mlua](https://github.com/mlua-rs/mlua) with _Luau_ support.

## Modding Framework
The modding framework allows users to install and manage plugins that extend and change Future Cop.
It provides plugins with an API that allows them to interact with the game in various ways.

Plugins are written in [Lua](https://www.lua.org/).
[Luau](https://luau-lang.org/getting-started) is supported, thus, plugin's can use types to improve the developer experience.

As executing arbitrary code is always associated with a potential security risk, the modding framework tries alleviate most of the risk by running all plugins in an isolated sandboxed environment.
A plugin cannot interact with any other plugin and only has access to a few selected default globals.
They can also require on a selection of safe default libraries.
The modding framework also offers an API, split among various libraries, that plugins can use to interact with the game.
For example, plugins can use the _UI_ library to show text on screen or the _Dangerous_ library for low-level access to the game, such as function hooking and reading memory.

To use a library, the game must specify its library dependencies in its [manifest file](#manifest).
When a user wants to install a plugin, they are shown information about the plugin, such as its dependencies.

As the modding framework allows plugins to directly interact with the games code, it cannot guarantee the security of plugins.
Therefore, specific libraries are marked as dangerous and if the user installs a plugin that depends on such a library the plugin manager will warn them.

### Plugin Structure
A plugin consists of at least two files.
The _manifest_ file contains general information about the plugin, and the _main_ file is the plugin's entry point.

#### Manifest
The manifest file contains general information about the plugin.
It must be in the plugin's root directory and is called `info.toml`.

It contains the following information:
- name
- author
- version
- description
- dependencies

The following is an example manifest file of a plugin called _FPS Display_.
```toml
name = "FPS Display"
version = "0.1.0"
authors = ["Simon Kurz"]
dependencies = ["ui", "system", "math"]
description = "Simple FPS display using the UI library."
```

#### Main File
The main file is the entry point for the plugin and is loaded by the modding framework when the plugin is installed.
It must be located in the plugin's root directory and is called `main.lua` (you can also use the ending `.luau`).

A plugin can define several specific functions that are then called by the modding framework depending on specific plugin or game events.
For example, a plugin can define the function `onUpdate`, which is called every frame when playing a mission.
The following functions are available:
- `onUpdate()`: Called every frame while in a mission
- `onLoad()`: Called when the modding framework loads the plugin
- `onUnload()`: Called the plugin is unloaded
- `onEnable()`: Called when the user enables the plugin
- `onDisable()`: Called when the user disables the plugin
- `onInstall()`: Called when the user installed the plugin. As long as the mod is not uninstalled, this function is only called once
- `onUninstall()`: Called when the user uninstalls the plugin

### API
**Coming**


## Goals
**Coming**