use std::{cell::OnceCell, path::{Path, PathBuf}, sync::{Arc, Mutex}, thread, time};

use log::*;
use num;
use windows::{Win32::System::Diagnostics::Debug::OutputDebugStringA, core::{PCSTR, s}, Win32::UI::Input::KeyboardAndMouse::*};
use crate::{future_cop::*, config::Config};
use crate::future_cop::global::*;
mod util;
use util::install_hook;
pub mod server;
mod plugins;
use self::{plugins::PluginManager, util::Hook};
use anyhow::anyhow;


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

struct GlobalPluginManager;

impl GlobalPluginManager {
    pub fn get() -> Arc<Mutex<PluginManager>> {
        let plugin_manager;
        unsafe {plugin_manager = PLUGIN_MANAGER.get().unwrap()};

        return plugin_manager.clone();
    }

    pub fn with_plugin_manager<F, R>(f: F) -> Result<R, anyhow::Error>
    where F: Fn(&PluginManager) -> R {
        let plugin_manager = GlobalPluginManager::get();

        let plugin_manager = match plugin_manager.lock() {
            Ok(m) => m,
            Err(e) => return Err(anyhow!("could not get lock to plugin manager: {:?}", e)),
        };

        Ok(f(&plugin_manager))
    }

    pub fn with_plugin_manager_mut<F, R>(f: F) -> Result<R, anyhow::Error>
    where F: Fn(&mut PluginManager) -> R {
        let plugin_manager;
        unsafe {plugin_manager = PLUGIN_MANAGER.get().unwrap()}

        let mut plugin_manager = match plugin_manager.lock() {
            Ok(m) => m,
            Err(e) => return Err(anyhow!("could not get mutable lock to plugin manager: {:?}", e)),
        };

        Ok(f(&mut plugin_manager))
    }
}

type MissionGameLoop = fn() -> ();

pub fn inject(config: Config) {
    unsafe {
        ORIGINAL_PLAYER_METHOD = install_hook(0x00446800, player_method);
        ORIGINAL_DAMAGE_PLAYER = install_hook(0x00446720, damage_player_hook);
        //FIRST_MISSION_GAME_LOOP_FUNCTION = install_hook(FUN_00406a30_ADDRESS, first_mission_game_loop_function);
        let mut hook = Hook::new(FUN_00406a30_ADDRESS);
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

    let plugin_manager = match PluginManager::new(plugins_directory) {
        Ok(m) => m,
        Err(e) => {
            error!("Couldn't initiate plugin manager: {:?}", e);
            panic!();
        }
    };
    let p = Arc::new(Mutex::new(plugin_manager));
    unsafe { PLUGIN_MANAGER.set(p); }

    server::start_server(config);

    start();
}


fn first_mission_game_loop_function(o: MissionGameLoop) {
    match GlobalPluginManager::get().lock() {
        Ok(manager) => {
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

pub fn start() {
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

        unsafe {
            game_mode = *game_mode_global.get();
        }
        
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


unsafe fn is_precinct_assault() {
    let address = 0x00511e03usize;
    let mode = address as *const u8;

    let msg = format!("IsPrecinctAssault: {}\n\0", *mode);
    OutputDebugStringA(PCSTR(msg.as_ptr()));
}

unsafe fn print_key_bitmap() {
    let address = 0x00511f9cusize;
    let key_bitmap = address as *const u32;

    OutputDebugStringA(PCSTR(format!("KeyBitMap: {:#010x}\n\0", *key_bitmap).as_ptr()));
}


unsafe fn set_health() {
    if FIRST_PLAYER.is_none() {
        //OutputDebugStringA(s!("Cannot modify health as address to player entity is unknown"));
        return;
    }

    let player_entity = {
        let mut d = FIRST_PLAYER;
        if SECOND_PLAYER.is_some() {
            d = SECOND_PLAYER;
        }

        d.unwrap()
    };
    
    let max_health = (*player_entity).health.max_health;

    (*player_entity).health.health = max_health;
}


unsafe fn damage_player_hook(player: *mut PlayerEntity, damage: i32) {
    if ORIGINAL_DAMAGE_PLAYER.is_none() {
        error!("Original DamagePlayer function not found");
        return;
    }
    let original_damage_player = ORIGINAL_DAMAGE_PLAYER.unwrap();

    if CONFIG.is_some() {
        let config = &CONFIG.clone().unwrap();
        let player_id: i32 = {
            let mut id: i32 = -1;

            if FIRST_PLAYER.is_some() && (*FIRST_PLAYER.unwrap()).id == (*player).id {
                id = 1;
            } else if SECOND_PLAYER.is_some() && (*SECOND_PLAYER.unwrap()).id == (*player).id {
                id = 2;
            }

            id
        };
        let second_player_exists = SECOND_PLAYER.is_some();

        let should_negate_damage = 
            (player_id == 1 && config.player_one.invincible) ||
            (player_id == 2 && config.player_two.invincible);

        if should_negate_damage {
            info!("Player {} would have taken {} damage\n\0", player_id, damage);

            return original_damage_player(player, 0);
        }
    }

    return original_damage_player(player, damage);
}
