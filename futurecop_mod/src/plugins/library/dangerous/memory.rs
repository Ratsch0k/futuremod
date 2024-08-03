use futurecop_hook::types::{Type, MAX_STRING};
use log::debug;
use mlua::{AnyUserDataExt, Lua};

use crate::plugins::library::LuaResult;

fn try_userdata_to_bytes(userdata: &mlua::AnyUserData) -> LuaResult<Vec<u8>> {
  userdata.call_method("toBytes", ())
    .map_err(|e| mlua::Error::RuntimeError(format!("Could not convert userdata into bytes: {}", e)))
}

/// Lua function to write arbitrary to a arbitrary memory address.
/// 
/// **Very unsafe**.
/// 
/// Wrong usage can easily lead to a panic.
pub fn write_memory_function<'lua>(_: &'lua Lua, (address, data): (u32, mlua::Value)) -> Result<(), mlua::Error> {
  debug!("Write memory to {}, value: {:?}", address, data);

  // Verify that the byte list if valid, before doing any unsafe operations
  let bytes: Vec<u8> = match data {
    mlua::Value::Table(byte_array) => {
      debug!("Writing byte array");
      // Lua array start with index 1
      let mut index = 1;

      let mut bytes: Vec<u8> = Vec::new();
      while byte_array.contains_key(index)? {
        let value: mlua::Value = byte_array.get(index)?;

        match value {
          mlua::Value::Integer(byte) => {
            if byte < 0 || 0xff < byte {
              return Err(mlua::Error::RuntimeError("supply the memory to write as a byte array (each item must be between 0 and 255".to_string()));
            }

            bytes.push(byte as u8);
          },
          t => return Err(mlua::Error::RuntimeError(format!("unsupported argument, provide memory as a byte array. Expected array, got {:?}", t))),
        }
        index += 1;
      }

      bytes
    },
    mlua::Value::Integer(value) => {
      debug!("Writing integer");
      value.to_le_bytes().to_vec()
    },
    mlua::Value::Number(value) => {
      debug!("Writing number");
      let value = value as f32;

      value.to_le_bytes().to_vec()
    },
    mlua::Value::String(value) => {
      debug!("Writing string");
      value.as_bytes().to_vec()
    },
    mlua::Value::UserData(userdata) => {
      debug!("Writing userdata");
      try_userdata_to_bytes(&userdata)?
    }
    _ => return Err(mlua::Error::RuntimeError("invalid argument. following types are supported: table, number, integer, string".to_string()))
  };

  debug!("Writing data: {:?}", bytes);

  let memory = address as *mut u8;

  debug!("Writing {:?} to {}", bytes, address);
  unsafe {
    for index in 0..bytes.len() {
      let address_to_write = memory.add(index);
      let byte_to_write = bytes[index];

      *address_to_write = byte_to_write;
    }
  }

  Ok(())
}

/// Read any memory address and convert it to the given type in lua.
pub fn read_memory_function<'lua>(lua: &'lua Lua, (address, type_name): (u32, String)) -> Result<mlua::Value<'lua>, mlua::Error> {
  debug!("Read memory address {} with type {}", address, type_name);
  let value_type = match Type::try_from_str(type_name.as_str()) {
    Some(t) => t,
    None => return Err(mlua::Error::RuntimeError("unsupported type".to_string()))
  };

  let value: mlua::Value;
  unsafe {
    value = match value_type {
      Type::Float => mlua::Value::Number(*(address as *const f32) as f64),
      Type::String => {
        let mut string_bytes: Vec<u8> = Vec::new();
        let string_pointer = address as *const u8;
  
        for i in 0..MAX_STRING {
          let current_value = *(string_pointer.add(i.into()));
          
          if current_value == 0 {
            break;
          }
  
          string_bytes.push(current_value);
        }
  
        mlua::Value::String(lua.create_string(string_bytes.as_slice())?)
      },
      Type::Void => mlua::Value::Nil,
      Type::Integer => mlua::Value::Integer(*(address as *const i32)),
      Type::Short => mlua::Value::Integer((*(address as *const i16)).into()),
      Type::Byte => mlua::Value::Integer((*(address as *const i8)).into()),
      Type::UnsignedInteger => mlua::Value::Integer(TryInto::<i32>::try_into(*(address as *const u32)).unwrap()),  // TODO: Properly handle error
      Type::UnsignedShort => mlua::Value::Integer((*(address as *const u16)).into()),
      Type::UnsignedByte => mlua::Value::Integer((*(address as *const u8)).into()),
    }
  }

  Ok(value)
}