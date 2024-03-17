use std::{mem::{self, size_of_val}, sync::Arc};

use log::debug;
use mlua::{FromLua, IntoLua, Lua, LuaSerdeExt, OwnedTable, UserData};
use serde::Serialize;

use crate::future_cop::{self, global::GetterSetter, state::FUTURE_COP, PLAYER_ARRAY_ADDR};

#[derive(Debug, Clone, Serialize)]
enum GameMode {
  PrecinctAssault,
  CrimeWar,
}

impl From<u32> for GameMode {
    fn from(value: u32) -> Self {
      match value {
        0 => GameMode::CrimeWar,
        _ => GameMode::PrecinctAssault,
      }
    }
}

impl From<&future_cop::GameMode> for GameMode {
    fn from(value: &future_cop::GameMode) -> Self {
      match value {
        future_cop::GameMode::CrimeWar => GameMode::CrimeWar,
        future_cop::GameMode::PrecinctAssault => GameMode::PrecinctAssault,
      }
    }
}


#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GameState {
  pub is_in_mission: bool,
  pub game_mode: GameMode,
  pub player_count: u8,
}



#[derive(Debug)]
struct PlayerEntity {
  player_entity: *mut future_cop::PlayerEntity
}

impl UserData for PlayerEntity {
  fn add_fields<'lua, F: mlua::prelude::LuaUserDataFields<'lua, Self>>(fields: &mut F) {
      fields.add_field_method_get("health", |_, this| {
        Ok(unsafe {
          (*this.player_entity).health.health
        })
      });

      fields.add_field_method_set("health", |_, this, health: i16| {
        unsafe {
          (*this.player_entity).health.health = health;
        }

        Ok(())
      });

      fields.add_field_method_get("positionX", |_, this| {
        Ok(unsafe {
          (*this.player_entity).position_x
        })
      });

      fields.add_field_method_set("positionX", |_, this, position_x: u32| {
        unsafe {
          (*this.player_entity).position_x = position_x;
        }

        Ok(())
      });

      fields.add_field_method_get("positionY", |_, this| {
        Ok(unsafe {
          (*this.player_entity).position_y
        })
      });

      fields.add_field_method_set("positionY", |_, this, position_y: u32| {
        unsafe {
          (*this.player_entity).position_y = position_y;
        }

        Ok(())
      });

      fields.add_field_method_get("positionZ", |_, this| {
        Ok(unsafe {
          (*this.player_entity).position_z
        })
      });

      fields.add_field_method_set("positionZ", |_, this, position_z: u32| {
        unsafe {
          (*this.player_entity).position_z = position_z;
        }

        Ok(())
      });

      fields.add_field_method_get("idleTimer", |_, this| {
        Ok(unsafe{(*this.player_entity).idle_timer})
      });

      fields.add_field_method_set("idleTimer", |_, this, idle_timer: u8| {
        unsafe{(*this.player_entity).idle_timer = idle_timer};
        Ok(())
      });

      fn create_getter_setter<'lua, T, F>(name: &str, fields: &mut F, extractor: fn(*mut future_cop::PlayerEntity) -> *mut T)
      where
        F: mlua::prelude::LuaUserDataFields<'lua, PlayerEntity>,
        T: IntoLua<'lua> + FromLua<'lua> + 'static + Copy,  {
          fields.add_field_method_get(name, move |_, this| {
            let field: T;
            unsafe {field = *extractor(this.player_entity)};
      
            Ok(field)
          });
      
          fields.add_field_method_set(name, move |_, this, value: T| {
            unsafe {
              *extractor(this.player_entity) = value;
            }
      
            Ok(())
          });
      }


      fn get_enemies_killed(player: *mut future_cop::PlayerEntity) -> *mut u16 {
        unsafe {&mut (*(*player).player).enemies_killed}
      }

      create_getter_setter::<u16, _>("enemiesKilled", fields, get_enemies_killed);
      create_getter_setter("deaths", fields, |player| {unsafe {&mut (*(*player).player).deaths}});
      create_getter_setter("currentAction", fields, |player| {unsafe {&mut (*(*player).player).current_action}});
      create_getter_setter("movementMode", fields, |player| unsafe {&mut (*(*player).player).movement_mode});
      create_getter_setter("currentTargetType", fields, |player| unsafe{&mut (*(*player).player).current_target_type});
      create_getter_setter("currentTarget", fields, |player| unsafe{&mut (*(*player).player).current_target});
      create_getter_setter("lastTarget", fields, |player| unsafe{&mut (*(*player).player).last_target});
      create_getter_setter("gunWeaponTimeout", fields, |player| unsafe{&mut (*(*player).player).gun_weapon_timeout});
      create_getter_setter("heavyWeaponTimeout", fields, |player| unsafe{&mut (*(*player).player).heavy_weapon_ammo});
      create_getter_setter("specialWeaponTimeout", fields, |player| unsafe{&mut (*(*player).player).special_weapon_timeout});
      create_getter_setter("gunWeaponAmmo", fields, |player| unsafe{&mut (*(*player).player).gun_weapon_ammo});
      create_getter_setter("heavyWeaponAmmo", fields, |player| unsafe{&mut (*(*player).player).heavy_weapon_ammo});
      create_getter_setter("specialWeaponAmmo", fields, |player| unsafe{&mut (*(*player).player).special_weapon_ammo});
      create_getter_setter("selectedGunWeapon", fields, |player| unsafe{&mut (*(*player).player).selected_gun_weapon});
      create_getter_setter("selectedHeavyWeapon", fields, |player| unsafe{&mut (*(*player).player).selected_heavy_weapon});
      create_getter_setter("selectedSpecialWeapon", fields, |player| unsafe{&mut (*(*player).player).selected_special_weapon});
      create_getter_setter("playerNumber", fields, |player| unsafe{&mut (*(*player).player).player_number});
      create_getter_setter("accelerationX", fields, |player| unsafe{&mut (*(*player).player).acceleration_x});
      create_getter_setter("accelerationZ", fields, |player| unsafe{&mut (*(*player).player).acceleration_z});
      create_getter_setter("accelerationY", fields, |player| unsafe{&mut (*(*player).player).acceleration_y});
  }

  fn add_methods<'lua, M: mlua::prelude::LuaUserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_method("getMaxHealth", |_, this, ()| {
        Ok(unsafe {
          (*this.player_entity).health.max_health
        })
      })
  }
}


impl GameState {
  pub fn new() -> Self {
    let game_state;
    unsafe {game_state = &FUTURE_COP.state};

    GameState {
      is_in_mission: *game_state.is_playing.get(),
      game_mode: GameMode::from(game_state.game_mode.clone().get()),
      player_count: match *game_state.is_two_player.get() {
        true => 2,
        false => 1,
      },
    }
  }
}

pub fn create_game_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  let functions = lua.create_table()?;

  let get_game_state = lua.create_function(|lua, ()| {
    let state = GameState::new();

    Ok(lua.to_value(&state))
  })?;
  functions.set("getState", get_game_state)?;

  let get_player = lua.create_function(|_, player: u8| {
    debug!("Getting player {}", player);

    if player > 1 {
      return Err(mlua::Error::RuntimeError("Can only get player one or two at the moment.".into()))
    }

    let player_array_item: u32;
    unsafe {
      player_array_item = *((PLAYER_ARRAY_ADDR + Into::<u32>::into(player) * 8) as *const u32);
    }

    if player_array_item == 0 {
      return Err(mlua::Error::RuntimeError("Player doesn't exist".into()));
    }

    let player_entity = future_cop::PlayerEntity::from_address(player_array_item);

    Ok(PlayerEntity {player_entity})
  })?;
  functions.set("getPlayer", get_player)?;

  Ok(functions.into_owned())
}