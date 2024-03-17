pub(crate) mod global;
use std::fmt;

use global::*;

pub(crate) mod state;


///////////////////////////////////////////////////////////
// Known addresses
///////////////////////////////////////////////////////////
pub const PLAYER_ARRAY_ADDR: u32 = 0x00511fd0;


///////////////////////////////////////////////////////////
// Enums
///////////////////////////////////////////////////////////
#[derive(Debug, Default)]
pub enum GameMode {
    #[default] CrimeWar,
    PrecinctAssault,
}

impl fmt::Display for &GameMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


///////////////////////////////////////////////////////////
// Statics
///////////////////////////////////////////////////////////
pub static IN_GAME_LOOP: VolatileGlobal<bool> = VolatileGlobal::new(0x004c987c);
pub static IS_TWO_PLAYER: VolatileGlobal::<bool> = VolatileGlobal::new(0x00511f54);
pub static IS_PLAYING: VolatileGlobal::<bool> = VolatileGlobal::new(0x00486248);
pub static GAME_MODE: SelectedGameMode = SelectedGameMode::new(0x00511e03);
pub static SCENE: VolatileGlobal<u8> = VolatileGlobal::new(0x00511fb8);
pub static FRAME_NUMBER: VolatileGlobal<u32> = VolatileGlobal::new(0x00511f40);
pub static MAIN_WINDOW: VolatileGlobal<u32> = VolatileGlobal::new(0x00512db4);
pub static HEAP: VolatileGlobal<u32> = VolatileGlobal::new(0x00512ebc);
pub static FUTURE_COP_MODULE: VolatileGlobal<u32> = VolatileGlobal::new(0x004a005c);
pub static EVENTS: VolatileGlobal<u32> = VolatileGlobal::new(0x00512044);


///////////////////////////////////////////////////////////
// Functions
///////////////////////////////////////////////////////////
pub type DamagePlayer = unsafe fn(*mut PlayerEntity, i32);
pub type EntityMethod = unsafe fn(i32, u32, u32, u32) -> u32;
pub type GameLoop = unsafe fn(i32);
pub type VoidFunction = unsafe fn();

///////////////////////////////////////////////////////////
// Function Addresses
///////////////////////////////////////////////////////////
/// This is the first game function called in the main mission game loop.
pub const FUN_00406a30_ADDRESS: u32 = 0x00406a30;


///////////////////////////////////////////////////////////
// Structs
///////////////////////////////////////////////////////////
#[derive(Debug)]
#[repr(C)]
pub struct PlayerHealth {
    pub health: i16,
    pub max_health: i16,
    pub unknown1: u32,
    pub unkonwn2: u32,
    pub low_health_threshold: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Player {
    pub field_00_0f: u128,
    pub field_10_11: u16,
    pub field_12_13: u16,
    pub enemies_killed: u16, 
    pub deaths: u16,
    pub field_18_1f: u64,
    pub field_20_2f: u128,
    pub current_action: u32,
    pub field_34_37: u32,
    pub field_38_3b: u32,
    pub field_3c_3f: u32,
    pub movement_mode: u32,
    pub field_44_47: u32,
    pub field_48_4b: u32,
    pub field_4c_4f: u32,
    pub field_50_5f: u128,
    pub field_60_63: u32,
    pub field_64_67: u32,
    pub current_target_type: u32,
    pub current_target: u32,
    pub last_target: u32,
    pub field_74_77: u32,
    pub gun_weapon_timeout: u32,
    pub heavy_weapon_timeout: u32,
    pub special_weapon_timeout: u32,
    pub gun_weapon_ammo: u16,
    pub heavy_weapon_ammo: u16,
    pub special_weapon_ammo: u16,
    pub field_8a_8b: u16,
    pub field_8c_8f: u32,
    pub selected_gun_weapon: u8,
    pub selected_heavy_weapon: u8,
    pub selected_special_weapon: u8,
    pub field_93: u8,
    pub field_94_97: u32,
    pub field_98_9b: u32,
    pub field_9c_9f: u32,
    pub field_a0_a3: u32,
    pub camera_mode: u32,
    pub field_a8_ab: u32,
    pub field_ac_af: u32,
    pub field_b0_bf: u128,
    pub player_number: u8,
    pub field_c1: u8,
    pub field_c2: u8,
    pub field_c3: u8,
    pub field_c4_c7: u32,
    pub field_c8_cb: u32,
    pub field_cc_cf: u32,
    pub field_d0_df: u128,
    pub field_e0_ef: u128,
    pub field_f0_ff: u128,
    pub field_100_10f: u128,
    pub field_110_11f: u128,
    pub field_120_12f: u128,
    pub field_130_13f: u128,
    pub field_140_14f: u128,
    pub field_150_153: u32,
    pub field_154_157: u32,
    pub field_158_15b: u32,
    pub acceleration_x: i32,
    pub acceleration_z: i32,
    pub acceleration_y: i32,
    pub field_168_16b: u32,
    pub field_16c_16f: u32,
    pub field_170_17f: u128,
    pub field_180_18f: u128,
    pub field_190_19f: u128,
    pub field_1a0_1af: u128,
    pub field_1b0_1bf: u128,
    pub field_1c0_1cf: u128,
    pub field_1d0_1df: u128,
    pub field_1e0_1ef: u128,
    pub field_1f0_1ff: u128,
    pub field_200_20f: u128,
    pub field_210_21f: u128,
    pub field_220_22f: u128,
    pub field_230_23f: u128,
    pub field_240_24f: u128,
    pub field_250_25f: u128,
    pub field_260_26f: u128,
    pub field_270_27f: u128,
    pub field_280_28f: u128,
    pub field_290_29f: u128,
    pub field_2a0_2af: u128,
    pub field_2b0_2bf: u128,
    pub field_2c0_2cf: u128,
    pub field_2d0_2df: u128,
    pub field_2e0_2ef: u128,
    pub field_2f0_2ff: u128,
    pub field_300_30f: u128,
    pub field_310_31f: u128,
    pub field_320_327: u64,
    pub field_328_329: u16,
}

#[derive(Debug)]
#[repr(C)]
pub struct PlayerEntity {
    pub parent: u32,
    pub main_method: EntityMethod,
    pub id: u32,
    pub unknown0: u32,
    pub unknown1: u32,
    pub unknown2: u32,
    pub unknown3: u32,
    pub health: PlayerHealth,
    pub unknown7: u128,
    pub unknown8: u128,
    pub unknown9: u32,
    pub position_x: u32,
    pub position_y: u32,
    pub position_z: u32,
    pub unknown10: u128,
    pub unknown11: u128,
    pub unknown12: u128,
    pub unknown13: u128,
    pub unknown14: u64,
    pub rotation:  i32,
    pub player: *mut Player,
    pub unknown15: u32,
    pub unknown16_0: u32,
    pub unknown16_1: u32,
    pub unknown16_2: u32,
    pub unknown16_3: u32,
    pub idle_animation_plays: u32,
    pub unknown17: u32,
    pub unknown18: u8,
    pub unknown19: u8,
    pub unknown20: u8,
    pub idle_timer: u8,
    pub unknown21: u128,
    pub unknown22: u128,
    pub unknown23: u128,
}


impl PlayerEntity {
    /// Create PlayerEntity from the given address.
    /// 
    /// This functions basically takes the address and casts it into a mutable pointer
    /// to a PlayerEntity instance residing at the memory address.
    /// `address` **must** point to a valid instance.
    /// Otherwise, calling this function leads to undefined behavior.
    pub fn from_address(address: u32) -> *mut PlayerEntity {
        let player: *mut PlayerEntity;
        unsafe {
            player = address as *mut PlayerEntity;
        }

        player
    }
}