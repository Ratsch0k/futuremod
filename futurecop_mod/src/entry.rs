use std::{cell::OnceCell, path::{Path, PathBuf}, sync::{Arc, Mutex}, thread, time};

use log::*;
use num;
use windows::{Win32::System::Diagnostics::Debug::OutputDebugStringA, core::{PCSTR, s}, Win32::UI::Input::KeyboardAndMouse::*};
use crate::{config::Config, futurecop::*, input::KeyState, plugins::plugin_manager::GlobalPluginManager};
use crate::futurecop::global::*;
use crate::util::install_hook;
use crate::{plugins::PluginManager, util::Hook};
use crate::server;

static mut CONFIG: Option<Config> = None;

static mut IS_ATTACHED: bool = false;
static mut ORIGINAL_MENU_LOOP: Option<GameLoop> = None;
static mut ORIGINAL_PLAYER_METHOD: Option<EntityMethod> = None;
static mut PLAYER_ENTITY_ADDRESS: Option<u32> = None;
static mut FIRST_PLAYER: Option<*mut PlayerEntity> = None;
static mut SECOND_PLAYER: Option<*mut PlayerEntity> = None;
static mut ORIGINAL_DAMAGE_PLAYER: Option<DamagePlayer> = None;
static mut FIRST_MISSION_GAME_LOOP_FUNCTION: Option<VoidFunction> = None;

static mut PLUGIN_MANAGER: OnceCell<Arc<Mutex<PluginManager>>> = OnceCell::new();



type MissionGameLoop = fn() -> ();

/// Main entry function of the entire mod.
/// 
/// Sets some always active hooks, configures and initializes global services (e.g. PluginManager) and starts the server.
pub fn main(config: Config) {
    unsafe {
        ORIGINAL_PLAYER_METHOD = install_hook(0x00446800, player_method);
        //FIRST_MISSION_GAME_LOOP_FUNCTION = install_hook(FUN_00406a30_ADDRESS, first_mission_game_loop_function);
        let mut hook = Hook::new(FUN_00406A30_ADDRESS);
        let _ = hook.set_hook(first_mission_game_loop_function as u32).map_err(|_| warn!("Could not hook game loop"));


        CONFIG = Some(config.clone());
    }

    let plugins_directory = config.plugins_directory.clone().map(PathBuf::from).unwrap_or(
        match std::env::current_dir() {
            Ok(path) => Path::join(&path, "plugins"),
            Err(e) => {
                error!("could not determine mods directory: could not get the current directory: {:?}", e);
                panic!("could not get the current directory: {:?}", e);
            },
        }
    );

    // Initialize global plugin manager or panic
    match GlobalPluginManager::initialize(plugins_directory) {
        Err(e) => {
            panic!("error while initializing the global plugin manager: {}", e);
        },
        Ok(_) => (),
    }

    server::start_server(config);

    mod_loop();
}



fn first_mission_game_loop_function(o: MissionGameLoop) {
    // Update the current key state
    let key_states = KeyState::new();
    match key_states.update() {
        Ok(_) => (),
        Err(e) => error!("Error while updating the key state: {}", e.to_string()),
    }

    match GlobalPluginManager::get().lock() {
        Ok(manager) => {
            // Then call onUpdate
            manager.on_update();
        }
        Err(e) => {
            error!("error while getting a lock to the plugin manager to call on_update: {:?}", e)
        },
    }

    o();
}

fn is_key_pressed(vkey: i32) -> bool {
        let key_state: i16;
        unsafe {key_state = GetAsyncKeyState(vkey)};

        return key_state != 0;
}

fn handle_player_sprint(player_id: u8, player_entity: &mut PlayerEntity) {
    let player_sprint_key;
    let player: &mut Player;
    
    unsafe {
        player_sprint_key = match &CONFIG {
            None => return,
            Some(c) => match player_id {
                1 => c.player_one.sprint_key,
                2 => c.player_two.sprint_key,
                _ => return,
            }
        };

        player = &mut *player_entity.player;
    };


    if is_key_pressed(player_sprint_key as i32) {
        let old_vel_x = (*player).acceleration_x as f32;
        let old_vel_y = player.acceleration_y as f32;
        let old_vel = (old_vel_x * old_vel_x + old_vel_y * old_vel_y).sqrt();
        let degree = (old_vel_y / old_vel_x).atan();

        let new_vel = num::clamp(old_vel * 2f32, -1_000_000f32, 1_000_000f32);
        let mut new_vel_x = degree.cos() * new_vel;
        if old_vel_x.signum() != new_vel_x.signum() {
            new_vel_x *= -1f32;
        }

        let mut new_vel_y = old_vel_y.signum() * degree.sin() * new_vel;
        if old_vel_y.signum() != new_vel_y.signum() {
            new_vel_y *= -1f32;
        }

        player.acceleration_x = new_vel_x as i32;
        player.acceleration_y = new_vel_y as i32;
    }
}

/// Mod infinite loop.
/// 
/// As long as no plugin exists to allow sprinting, this function is used for simple implementation
/// of a sprinting mod.
/// Every 10 ms set the player's acceleration to a higher values based on the current values.
/// This is janky implementation as we are not actually hooking into player's movement
/// function and instead hope that our function overrides the acceleration after the game's logic
/// clamped it and before the game moved the player.
pub fn mod_loop() {
    loop {
        unsafe {
            if FIRST_PLAYER.is_some() {
                handle_player_sprint(1, &mut *FIRST_PLAYER.unwrap())
            }

            if SECOND_PLAYER.is_some() {
                handle_player_sprint(2, &mut *SECOND_PLAYER.unwrap())
            }
        }

        thread::sleep(time::Duration::from_millis(10));
    }
}

unsafe fn menu_loop_hook(time_delta: i32) {
    OutputDebugStringA(PCSTR(format!("GameLoop({})\n\0", time_delta).as_ptr()));
    match ORIGINAL_MENU_LOOP {
        Some(f) => f(time_delta),
        None => OutputDebugStringA(s!("OriginalGameLoop function not found"))
    }
}

unsafe fn player_method(param1: i32, player_entity: u32, param3: u32, param4: u32) -> u32 {
    if player_entity > 0  {
        if PLAYER_ENTITY_ADDRESS.is_none() {
            PLAYER_ENTITY_ADDRESS = Some(player_entity);
        }

        let player_entity_data = player_entity as *mut PlayerEntity;
        let id = (*player_entity_data).id;
        let game_mode_global = VolatileGlobal::<u32>::new(0x00511e03);
        let game_mode: u32;

        game_mode = *game_mode_global.get();
        
        let mut player: Option<u8> = None;
        match (game_mode, id) {
            (0, 1) => player = Some(0),
            (0, 3) => player = Some(1),
            (_, 1) => player = Some(0),
            (_, 64) => player = Some(1),
            _ => (),
        }

        if player.is_some() && param1 == 2 {
            if player == Some(0) && FIRST_PLAYER.is_none() {
                info!("Player 1 created");
                FIRST_PLAYER = Some(player_entity_data);
            } else if player == Some(1) && SECOND_PLAYER.is_none() {
                info!("Player 2 created");
                SECOND_PLAYER = Some(player_entity_data);
            }
        } else if param1 == 5 {
            if FIRST_PLAYER.is_some() && FIRST_PLAYER.unwrap() as u32 == player_entity {
                info!("Player 1 destroyed");
                FIRST_PLAYER = None;
            }
            if SECOND_PLAYER.is_some() && SECOND_PLAYER.unwrap() as u32 == player_entity {
                info!("Player 2 destroyed");
                SECOND_PLAYER = None;
            }
        }
    }

    match ORIGINAL_PLAYER_METHOD {
        Some(f) => return f(param1, player_entity, param3, param4),
        None => (),
    }

    error!("OriginalPlayerMethod not found");
    return 0;
}
