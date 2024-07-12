use crate::futurecop::{global::GetterSetter, RENDER_ITEMS};


#[repr(C)]
#[derive(Default, Debug)]
pub struct RenderItem {
  pub unknown0x0: u32,
  pub item_type: u8,
  pub unknown0x5: u8,
  pub unknown0x6: u8,
  pub unknown0x7: u8,
  pub sprite_offset_x: u8,
  pub sprite_offset_y: u8,
  pub sprite_width: u8,
  pub sprite_height: u8,
  pub unknown0xc: u8,
  pub unknown0xd: u8,
  pub unknown0xe: u8,
  pub unknown0xf: u8,
  pub unknown0x10: u8,
  pub unknown0x11: u8,
  pub unknown0x12: u8,
  pub unknown0x13: u8,
  pub unknown0x14: u8,
  pub unknown0x15: u8,
  pub unknown0x16: u8,
  pub unknown0x17: u8,
  pub unknown0x18: u8,
  pub color_red: u8,
  pub color_green: u8,
  pub color_blue: u8,
  pub unknown0x1c: u8,
  pub unknown0x1d: u8,
  pub unknown0x1e: u8,
  pub unknown0x1f: u8,
  pub unknown0x20: u8,
  pub unknown0x21: u8,
  pub unknown0x22: u8,
  pub unknown0x23: u8,
  pub unknown0x24: u8,
  pub unknown0x25: u8,
  pub unknown0x26: u8,
  pub unknown0x27: u8,
  pub screen_pos_x: u16,
  pub screen_pos_y: u16,
  pub box_width: u16,
  pub box_height: u16,
  pub unknown0x30: u8,
  pub unknown0x31: u8,
  pub unknown0x32: u8,
  pub unknown0x33: u8,
  pub unknown0x34: u8,
  pub unknown0x35: u8,
  pub unknown0x36: u8,
  pub unknown0x37: u8,
}

pub const EXAMPLE_ITEM: RenderItem = RenderItem {
  unknown0x0: 0x00002528,
  item_type: 0xc4,
  unknown0x5: 0x07,
  unknown0x6: 0x01,
  unknown0x7: 0x03,
  sprite_offset_x: 0xac,
  sprite_offset_y: 0xf5,
  sprite_width: 0xff,
  sprite_height: 0xff,
  unknown0xc: 0xc7,
  unknown0xd: 0xd7,
  unknown0xe: 0xac,
  unknown0xf: 0xd7,
  unknown0x10: 0x4c,
  unknown0x11: 0x6a,
  unknown0x12: 0x5e,
  unknown0x13: 0x03,
  unknown0x14: 0xac,
  unknown0x15: 0x6a,
  unknown0x16: 0x5d,
  unknown0x17: 0x03,
  unknown0x18: 0x74,
  color_red: 0xf0,
  color_green: 0xad,
  color_blue: 0xba,
  unknown0x1c: 0x74,
  unknown0x1d: 0xf0,
  unknown0x1e: 0xad,
  unknown0x1f: 0xba,
  unknown0x20: 0x6c,
  unknown0x21: 0xf0,
  unknown0x22: 0xad,
  unknown0x23: 0xba,
  unknown0x24: 0x74,
  unknown0x25: 0xf0,
  unknown0x26: 0xad,
  unknown0x27: 0xba,
  screen_pos_x: 0xc603,
  screen_pos_y: 0xaf00,
  box_width: 0xffff,
  box_height: 0xffff,
  unknown0x30: 0x98,
  unknown0x31: 0x04,
  unknown0x32: 0xee,
  unknown0x33: 0x00,
  unknown0x34: 0xb7,
  unknown0x35: 0x03,
  unknown0x36: 0xee,
  unknown0x37: 0x00,
};

const TYPE_TRIANGLE: u8 = 0x33;

pub fn render_item(item: RenderItem) {
  unsafe {
    let item_address = RENDER_ITEMS.get().clone();
    RENDER_ITEMS.set(item_address + 0x38);

    let first_field = item_address as *mut u32;
    *first_field = 0;

    let render_item = item_address as *mut RenderItem;
    *render_item = item;
  }
}