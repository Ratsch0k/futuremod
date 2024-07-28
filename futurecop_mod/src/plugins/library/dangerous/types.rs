use anyhow::bail;
use log::info;
use mlua::Lua;

/// Supported types for lua to/from native conversion.
#[derive(Debug, Clone, Copy)]
pub enum Type {
  String,
  Integer,
  UnsignedInteger,
  Byte,
  UnsignedByte,
  Short,
  UnsignedShort,
  Float,
  Void,
}

impl Type {
  pub fn try_from_str(name: &str) -> Option<Type> {
    let type_value = match name {
      "string" => Type::String,
      "int" => Type::Integer,
      "uint" => Type::UnsignedInteger,
      "float" => Type::Float,
      "void" => Type::Void,
      "short" => Type::Short,
      "ushort" => Type::UnsignedShort,
      "byte" => Type::Byte,
      "ubyte" => Type::UnsignedByte,
      _ => return None,
    };

    Some(type_value)
  }
}

pub const MAX_STRING: u16 = 1024;

/// Convert a native value into its lua value given the type name.
pub unsafe fn native_to_lua<'a>(lua: &'a Lua, lua_type: Type, raw_value: u32) -> Result<mlua::Value<'a>, mlua::Error> {
  let value = match lua_type {
    Type::String => {
      let mut string_bytes: Vec<u8> = Vec::new();
      let string_pointer = raw_value as *const u8;

      for i in 0..MAX_STRING {
        let current_value = *(string_pointer.add(i.into()));
        
        if current_value == 0 {
          break;
        }

        string_bytes.push(current_value);
      }

      mlua::Value::String(lua.create_string(string_bytes.as_slice())?)
    },
    Type::Float => mlua::Value::Number(f64::from(raw_value as f32)),
    Type::Void => mlua::Value::Nil,
    Type::Integer => mlua::Value::Integer(raw_value as i32),
    Type::UnsignedInteger => mlua::Value::Integer(raw_value as i32),
    Type::Short => mlua::Value::Integer(Into::<i32>::into(raw_value as i16)),
    Type::UnsignedShort => mlua::Value::Integer(Into::<i32>::into(raw_value as u16)),
    Type::Byte => mlua::Value::Integer(Into::<i32>::into(raw_value as i8)),
    Type::UnsignedByte => mlua::Value::Integer(Into::<i32>::into(raw_value as u8)),
  };

  Ok(value)
}

/// Convert a lua value into its native representation given a specific lua type.
pub unsafe fn lua_to_native<'a>(lua_type: Type, lua_value: &'a mlua::Value) -> Result<Vec<u32>, anyhow::Error> {
  let actual_type_name = lua_value.type_name();

  let value: Vec<u32> = match lua_type {

    Type::Float => match lua_value.as_f32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not a float", actual_type_name),
    }
    Type::Void => vec![0u32],
    Type::String => match lua_value.as_str() {
      Some(value) => {
        vec![value.as_ptr() as u32]
      },
      None => bail!("value {} is not a string", actual_type_name),
    },
    Type::Integer => match lua_value.as_i32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not an int", actual_type_name),
    },
    Type::UnsignedInteger => match lua_value.as_u32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not an uint", actual_type_name),
    },
    Type::Short => match lua_value.as_i32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not a short", actual_type_name),
    },
    Type::UnsignedShort => match lua_value.as_u32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not a ushort", actual_type_name),
    },
    Type::Byte => match lua_value.as_i32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not a byte", actual_type_name)
    },
    Type::UnsignedByte => match lua_value.as_u32() {
      Some(value) => vec![value as u32],
      None => bail!("value {} is not a ubyte", actual_type_name)
    },
  };

  Ok(value)
}

pub unsafe fn lua_to_native_implied<'a>(value: &'a mlua::Value) -> Result<Vec<u32>, anyhow::Error> {
  let value: Vec<u32> = match value {
    mlua::Value::Nil => vec![0u32],
    mlua::Value::String(value) => {
        vec![value.to_pointer() as u32]
    }
    mlua::Value::Number(value) => {
      vec![*value as f32 as u32]
    },
    mlua::Value::Integer(value) => {
      vec![*value as u32]
    }
    value => bail!("type {} is not supported", value.type_name()),
  };


  Ok(value)
}