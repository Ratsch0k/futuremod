pub mod dangerous;
pub mod game;
pub mod input;
pub mod ui;
pub mod system;
pub mod matrix;

type LuaResult<T> = Result<T, mlua::Error>;