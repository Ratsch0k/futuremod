use std::sync::Arc;

use mlua::{Lua, LuaSerdeExt, OwnedTable, Value};

use crate::api::{self, ui::{Color, TextPalette, TEXT_PALETTES}};

pub fn create_ui_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  let library = lua.create_table()?;

  let render_text = lua.create_function(|_, (text, pos_x, pos_y, palette): (String, u32, u32, u32)| {
    api::ui::render_text(pos_x, pos_y, TextPalette::from(palette), &text);

    Ok(())
  })?;
  library.set("renderText", render_text)?;

  let render_rectangle = lua.create_function(|lua, (color, pos_x, pos_y, width, height, semi_transparent): (Value, u16, u16, u16, u16, bool)| {
    // Convert the color lua value into the rust type
    let color: Color = lua.from_value(color)?;

    api::ui::render_rectangle(color, pos_x, pos_y, width, height, semi_transparent);

    Ok(())
  })?;
  library.set("renderRectangle", render_rectangle)?;

  for palette in TEXT_PALETTES {
    library.set(format!("Palette{}", palette), Into::<u32>::into(palette))?;
  }

  Ok(library.into_owned())
}