pub mod dangerous;
pub mod game;
pub mod input;
pub mod matrix;
pub mod system;
pub mod ui;
pub mod settings;

type LuaResult<T> = Result<T, mlua::Error>;
