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
pub static ENTITY_LIST_FIRST: VolatileGlobal<u32> = VolatileGlobal::new(0x00499b0c);
pub static ENTITY_LIST_ENTRY: VolatileGlobal<u32> = VolatileGlobal::new(0x00499ad0);
pub static SURFACE: VolatileGlobal<u32> = VolatileGlobal::new(0x00511f64);
pub static SURFACE_COPY: VolatileGlobal<u32> = VolatileGlobal::new(0x00511dc4);
pub static mut RENDER_ITEMS: VolatileGlobal<u32> = VolatileGlobal::new(0x00511dc0);


///////////////////////////////////////////////////////////
// Function Types
///////////////////////////////////////////////////////////
pub type DamagePlayer = unsafe fn(*mut PlayerEntity, i32);
pub type EntityMethod = unsafe fn(i32, u32, u32, u32) -> u32;
pub type GameLoop = unsafe fn(i32);
pub type VoidFunction = unsafe fn();
pub type RenderCharacterFunction = unsafe fn(u32, u32, u32, u32) -> u32;
pub type RenderTextFunction = unsafe fn(*const u8, u32, u32, u32);
pub type RenderRectangleFunction = unsafe fn(u32, u16, u16, u16, u16, u8);
pub type UpdateFunction = unsafe fn (u32, u32, u32) -> u32;
pub type RenderObjectRaw = unsafe fn (u32, u32, u32);
pub type RenderObject = unsafe fn (u32, *mut u32, u32);


///////////////////////////////////////////////////////////
// Function Addresses
///////////////////////////////////////////////////////////
/// This is the first game function called in the main mission game loop.
pub const FUN_00406A30_ADDRESS: u32 = 0x00406a30;
pub const RENDER_CHARACTER_FUNCTION_ADDRESS: u32 = 0x00436130;
pub const RENDER_TEXT_FUNCTION_ADDRESS: u32 = 0x00435f40;
pub const RENDRE_RECTANGLE_FUNCTION_ADDRESS: u32 = 0x00415450;
pub const GET_UPDATE_FUNCTION_OF_BEHAVIOR_ADDRESS: u32 = 0x0041a950;
pub const UPDATE_FUNCTION_BEHAVIOR_0xA0_ADDRESS: u32 = 0x0041a420;
pub const RENDER_OBJECT_ADDRESS: u32 = 0x004284b0;
pub const FUN_004280A0_ADDRESS: u32 = 0x004280a0;


///////////////////////////////////////////////////////////
// Functions
///////////////////////////////////////////////////////////
macro_rules! fn_cast {
    ($address:expr, $t:ty) => {
        std::mem::transmute::<*const (), $t>($address as _)
    };
}

pub fn render_character(character: u32, pos_x: u32, pos_y: u32, palette: u32) -> u32 {
    let fn_ptr = RENDER_CHARACTER_FUNCTION_ADDRESS as *const();
    unsafe {
        let render_character_fn = {std::mem::transmute::<_, RenderCharacterFunction>(fn_ptr)};
        render_character_fn(character, pos_x, pos_y, palette)
    }
}

pub fn render_text(text: *const u8, pos_x: u32, pos_y: u32, palette: u32) {
    unsafe {
        let render_text_fn = fn_cast!(RENDER_TEXT_FUNCTION_ADDRESS, RenderTextFunction);
        render_text_fn(text, pos_x, pos_y, palette);
    }

}

pub fn render_rectangle(color: u32, pos_x: u16, pos_y: u16, width: u16, height: u16, semi_transparent: u8) {
    unsafe {
        let render_rect_fn = fn_cast!(RENDRE_RECTANGLE_FUNCTION_ADDRESS, RenderRectangleFunction);
        render_rect_fn(color, pos_x, pos_y, width, height, semi_transparent);
    }
}

pub fn update_function_behavior_0xa0(arg1: u32, arg2: u32, arg3: u32) -> u32 {
    unsafe {
        let update_fn = fn_cast!(UPDATE_FUNCTION_BEHAVIOR_0xA0_ADDRESS, UpdateFunction);
        update_fn(arg1, arg2, arg3)
    }
}


pub fn render_object_raw(arg1: u32, arg2: u32, arg3: u32) {
    unsafe {
        let render_object_fn = fn_cast!(RENDER_OBJECT_ADDRESS, RenderObjectRaw);
        render_object_fn(arg1, arg2, arg3);
    }
}

pub fn render_object(model_data: u32, value_ref: *mut u32, arg3: u32) {
    unsafe {
        let render_object_fn = fn_cast!(FUN_004280A0_ADDRESS, RenderObject);
        render_object_fn(model_data, value_ref, arg3);
    }
}

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
        let player = address as *mut PlayerEntity;

        player
    }
}


#[derive(Debug)]
#[repr(C)]
pub struct Position {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}


/// Represents basic entity/actor data.
/// Used by behavior `0xa0`.
#[derive(Debug)]
#[repr(C)]
pub struct BasicEntity {
    pub next_entity: u32,
    pub update_method: u32,
    pub id: u32,
    pub unknown0xc: u32,
    pub unknown0x10: u32,
    pub unknown0x14: u16,
    pub behavior_type: u16,
    pub unknown0x18: u32,
    pub unknown0x1c: u32,
    pub unknown0x20: u16,
    pub unknown0x22: u8,
    pub map_marker: u8,
    pub unknown0x24: u32,
    pub model_data_ref: u32,
    pub model_matrix_0x2c: u32,
    pub model_matrix_0x30: u32,
    pub model_matrix_0x34: u32,
    pub model_matrix_0x38: u32,
    pub model_matrix_0x3c: u32,
    pub model_matrix_0x40: u32,
    pub model_matrix_0x44: u32,
    pub model_matrix_0x48: u32,
    pub model_matrix_0x4c: u32,
    pub position: Position,
    pub unknown0x5c: u32,
    pub unknown0x60: u32,
    pub unknown0x64: u32,
    pub unknown0x68: u32,
    pub unknown0x6c: u32,
    pub unknown0x70: u32,
    pub unknown0x74: u32,
    pub unknown0x78: u32,
    pub unknown0x7c: u32,
    pub unknown0x80: u32,
    pub unknown0x84: u32,
    pub unknown0x88: u32,
    pub unknown0x8c: u32,
    pub unknown0x90: u16,
    pub texture_offset_x: u8,
    pub texture_offset_y: u8,
    pub action_script_ref1: u32,
    pub action_script_ref2: u32,
    pub action_script_ref3: u32,
}

#[derive(Debug)]
#[repr(C)]
pub struct Entity {
    pub next_entity: u32,
    pub update_method: u32,
    pub id: u32,
    pub unknown0xc: u32,
    pub unknown0x10: u32,
    pub unknown0x14: u16,
    pub behavior_type: u16,
    pub unknown0x18: u32,
    pub unknown0x1c: u32,
    pub unknown0x20: u16,
    pub unknown0x22: u8,
    pub map_marker: u8,
    pub unknown0x24: u32,
    pub model_data_ref: u32,
    pub model_matrix_0x2c: u32,
    pub model_matrix_0x30: u32,
    pub model_matrix_0x34: u32,
    pub model_matrix_0x38: u32,
    pub model_matrix_0x3c: u32,
    pub model_matrix_0x40: u32,
    pub model_matrix_0x44: u32,
    pub model_matrix_0x48: u32,
    pub model_matrix_0x4c: u32,
    pub position: Position,
    pub unknown0x5c: u32,
    pub unknown0x60: u32,
    pub unknown0x64: u32,
    pub unknown0x68: u32,
    pub unknown0x6c: u32,
    pub unknown0x70: u32,
    pub unknown0x74: u32,
    pub unknown0x78: u32,
    pub unknown0x7c: u32,
    pub unknown0x80: u32,
    pub unknown0x84: u32,
    pub unknown0x88: u32,
    pub unknown0x8c: u32,
    pub unknown0x90: u32,
    pub unknown0x94: u32,
    pub unknown0x98: u32,
    pub unknown0x9c: u32,
    pub unknown0xa0: u32,
    pub unknown0xa4: u32,
    pub unknown0xa8: u32,
    pub unknown0xac: u32,
    pub unknown0xb0: u32,
    pub unknown0xb4: u32,
    pub unknown0xb8: u32,
    pub unknown0xbc: u32,
    pub unknown0xc0: u32,
    pub unknown0xc4: u32,
    pub unknown0xc8: u32,
    pub unknown0xcc: u32,
    pub unknown0xd0: u32,
    pub unknown0xd4: u32,
    pub unknown0xd8: u32,
    pub unknown0xdc: u32,
    pub unknown0xe0: u32,
    pub unknown0xe4: u32,
    pub unknown0xe8: u32,
    pub unknown0xec: u32,
    pub unknown0xf0: u32,
    pub unknown0xf4: u32,
    pub unknown0xf8: u32,
    pub unknown0xfc: u32,
    pub unknown0x100: u32,
    pub unknown0x104: u32,
    pub unknown0x108: u32,
    pub unknown0x10c: u32,
}