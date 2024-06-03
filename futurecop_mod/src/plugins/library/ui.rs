use std::sync::Arc;

use mlua::{Lua, OwnedTable};

use crate::api::{self, ui::{TextPalette, TEXT_PALETTES}};

pub fn create_ui_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  let library = lua.create_table()?;

  let render_text = lua.create_function(|_, (text, pos_x, pos_y, palette): (String, u32, u32, u32)| {
    api::ui::render_text(pos_x, pos_y, TextPalette::from(palette), &text);

    Ok(())
  })?;
  library.set("renderText", render_text)?;

  for palette in TEXT_PALETTES {
    library.set(format!("Palette{}", palette), Into::<u32>::into(palette))?;
  }

  Ok(library.into_owned())
}