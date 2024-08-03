use std::{cell::Ref, collections::HashMap};

use log::{debug, info};
use mlua::{AnyUserData, AnyUserDataExt, Lua, MetaMethod, UserData};

use futurecop_hook::types::{lua_to_native, native_to_lua, Type};

use crate::plugins::library::LuaResult;


#[derive(Debug, Clone)]
enum FieldType {
  Primitive(Type),
  Complex(String),
}

#[derive(Debug, Clone)]
struct NativeStructField {
  pub offset: u32,
  pub field_type: FieldType,
}

#[derive(Debug, Clone)]
pub struct NativeStruct {
  fields: HashMap<String, NativeStructField>,
  address: u32,
}

impl UserData for NativeStruct {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {      
      methods.add_meta_function(MetaMethod::Index, |lua, (native_struct_userdata, field): (AnyUserData, String)| -> Result<mlua::Value<'lua>, mlua::Error> {
        let native_struct: Ref<NativeStruct> = native_struct_userdata.borrow().map_err(|_| mlua::Error::RuntimeError("Self must be a native struct definition".to_string()))?;

        debug!("Getting field '{}' from native struct at 0x{:x}", field, native_struct.address);
        let native_field = match native_struct.fields.get(&field) {
          Some(field) => field,
          None => {
            debug!("Native struct at 0x{:x} doesn't have field {}", native_struct.address, field);
            return Ok(mlua::Nil);
          },
        };

        let field_ptr = native_struct.address + native_field.offset;

        match &native_field.field_type {
          FieldType::Primitive(primitive) => {
            unsafe {
              let value = *(field_ptr as *const u32);
              native_to_lua(lua, *primitive, value)
            }
          },
          FieldType::Complex(complex) => {
            let complex_type: AnyUserData = native_struct_userdata.named_user_value(&complex)
              .map_err(|e| mlua::Error::RuntimeError(format!("Could not get type {}: {}", complex, e)))?;

            // Call the `getByteSize` method of the value to get the expected amount of byte the type allocates
            let byte_size = complex_type
              .call_method::<_, u32>("getByteSize", ())
              .map_err(|e| mlua::Error::RuntimeError(format!("getByteSize method errored: {}", e)))?;

            let field_ptr = field_ptr as *const u8;
            let mut byte_vec = Vec::<u8>::new();

            // Manually push the expected amount of bytes into the vector.
            // `Vec::from_raw_parts()` could potentially cause issues because the byte array was not created from a Vec
            for byte_idx in 0..byte_size {
              let byte_value = unsafe {*field_ptr.offset(byte_idx as isize)};

              byte_vec.push(byte_value);
            }

            // Call the type's 'fromBytes' function to construct an instance of the type from the bytes
            let f = complex_type.get::<_, mlua::Function>("fromBytes")
              .map_err(|_| mlua::Error::RuntimeError("Type userdata is missing 'fromBytes' function".to_string()))?;
            let value = f.call::<_, mlua::Value>((complex_type, byte_vec))?;

            Ok(value)
          },
        }
      });

      methods.add_meta_function(MetaMethod::NewIndex, |_, (native_struct_userdata, field, value): (AnyUserData, String, mlua::Value)| -> Result<(), mlua::Error> {
        let native_struct: Ref<NativeStruct> = native_struct_userdata.borrow()?;

        debug!("Set field {} of struct at 0x{:x} to {:?}", field, native_struct.address, value);

        let native_field = match native_struct.fields.get(&field) {
          Some(field) => field,
          None => {
            debug!("Struct at 0x{:x} doesn't have field {}", native_struct.address, field);
            return Err(mlua::Error::RuntimeError("Field doesn't exist".to_string()))
          }
        };

        let field_addr = native_struct.address + native_field.offset;

        match &native_field.field_type {
          FieldType::Primitive(primitive) => {
            let native_value = unsafe {
              lua_to_native(*primitive, &value)
                .map_err(|e| mlua::Error::RuntimeError(format!("Could not convert lua value into native: {}", e)))?
            };

            // Report if the lua value was converted into more bytes than expected
            if native_value.len() > 1 {
              debug!("Converted lua value is larger than one double word: {:?}", value);
            }

            match primitive {
              Type::Byte => {
                let field_ptr = field_addr as *mut i8;

                unsafe {
                  *field_ptr = native_value[0] as i8;
                }
              },
              Type::Short => {
                let field_ptr = field_addr as *mut i16;

                unsafe {
                  *field_ptr = native_value[0] as i16;
                }
              },
              Type::UnsignedByte => {
                let field_ptr = field_addr as *mut u8;

                unsafe {
                  *field_ptr = native_value[0] as u8;
                }
              },
              Type::UnsignedShort => {
                let field_ptr = field_addr as *mut u16;

                unsafe {
                  *field_ptr = native_value[0] as u16;
                }
              },
              Type::Integer => {
                let field_ptr = field_addr as *mut i32;

                unsafe {
                  *field_ptr = native_value[0] as i32;
                }
              }
              _ => {
                let field_ptr = field_addr as *mut u32;

                // Only copy the first double word into the field
                unsafe {
                  *field_ptr = native_value[0];
                }
              },
            }
          },
          FieldType::Complex(complex) => {
            // Invoke `toBytes` method of the userdata representing the fields type
            // We invoke the method with the supplied value instead of the actual self.
            // If we would invoke the `toBytes` method of the supplied value, we would allow the plugin to ignore the type information it specified for the field.
            let complex_type: AnyUserData = native_struct_userdata.named_user_value(&complex)
              .map_err(|e| mlua::Error::RuntimeError(format!("Could not get type {}: {}", complex, e)))?;

            let bytes = complex_type.call_method::<mlua::Value, Vec<u8>>("toBytes", value)
              .map_err(|e| mlua::Error::RuntimeError(format!("toBytes function of complex type errored: {}", e)))?;

            // Copy the bytes from the value into the field
            // We don't check the memory access in any way. It is the plugin developer's responsibility to not corrupt memory.
            unsafe {
              let field_ptr = field_addr as *mut u8;

              for byte_idx in 0..bytes.len() {
                let byte = bytes[byte_idx];

                *(field_ptr.offset(byte_idx as isize)) = byte;
              }
            }
          },
        }

        Ok(())
      });
    }
}

#[derive(Debug, Clone)]
pub enum FieldDefinitionType {
  Primitive(Type),
  Complex(String)
}

#[derive(Debug, Clone)]
pub struct FieldDefinition {
  offset: u32,
  field_type: FieldDefinitionType
}

#[derive(Debug, Clone)]
pub struct NativeStructDefinition {
  fields: HashMap<String, FieldDefinition>
}

fn native_struct_from_definition<'a>(lua: &'a Lua, address: u32, definition_userdata: AnyUserData<'a>) -> LuaResult<AnyUserData<'a>> {
  let definition: Ref<NativeStructDefinition> = definition_userdata.borrow()?;

  let fields = &definition.fields;
  let mut struct_fields: HashMap<String, NativeStructField> = HashMap::new();

  for (key, field_def) in fields.iter() {
    let field_type: FieldType = match field_def.field_type {
      FieldDefinitionType::Primitive(primitive) => FieldType::Primitive(primitive.clone()),
      FieldDefinitionType::Complex(_) => FieldType::Complex(key.clone()),
    };

    struct_fields.insert(key.clone(), NativeStructField{offset: field_def.offset, field_type});
  }

  let native_struct = NativeStruct{
    fields: struct_fields,
    address,
  };

  let native_struct_userdata = lua.create_userdata(native_struct)?;

  for (key, field_def) in fields.iter() {
    if let FieldDefinitionType::Complex(complex) = &field_def.field_type {
      let type_user_value: AnyUserData = definition_userdata.named_user_value(complex)?;
      native_struct_userdata.set_named_user_value(&key, type_user_value)?;
    }
  }

  Ok(native_struct_userdata)
}

impl UserData for NativeStructDefinition {
    fn add_methods<'lua, M: mlua::UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_function("cast", |lua, (definition, address): (AnyUserData, u32)| -> Result<AnyUserData<'lua>, mlua::Error> {
        native_struct_from_definition(lua, address, definition)
      });
    }
}

pub fn create_native_struct_definition_fn<'lua>(lua: &'lua Lua, fields: mlua::Table<'lua>) -> Result<AnyUserData<'lua>, mlua::Error> {
  info!("Creating native struct def");
  let mut native_fields: HashMap<String, FieldDefinition> = HashMap::new();
  
  for pair in fields.clone().pairs::<String, mlua::Table>() {
    let (key, field_definition) = match pair {
      Ok(pair) => pair,
      Err(e) => {
        debug!("Field definition has invalid type");
        return Err(mlua::Error::RuntimeError(format!("Field definition must be a table: {}", e)));
      }
    };

    let offset: u32 = field_definition.get("offset").map_err(|_| mlua::Error::RuntimeError(format!("Field definition of {} is missing 'offset'", key)))?;
    let native_type_id: mlua::Value = field_definition.get("type")
      .map_err(|_| mlua::Error::RuntimeError(format!("Field definition of {} is missing 'type'", key)))?;
    let native_type: FieldDefinitionType = match native_type_id.type_name() {
      "string" => match native_type_id.as_str() {
          Some(native_type_str) => match Type::try_from_str(native_type_str) {
              Some(value) => FieldDefinitionType::Primitive(value),
              None => return Err(mlua::Error::runtime("Unsupported type")),
          }
          None => return Err(mlua::Error::runtime("Could not convert type to string")),
      },
      "userdata" => match native_type_id.as_userdata() {
          Some(userdata) => {
            userdata.get::<_, mlua::Function>("toBytes").map_err(|_| mlua::Error::runtime("Complex type is missing function 'toBytes'"))?;
            userdata.get::<_, mlua::Function>("fromBytes").map_err(|_| mlua::Error::runtime("Complex type is missing function 'fromBytes'"))?;

            FieldDefinitionType::Complex(key.clone())
          },
          None => return Err(mlua::Error::runtime("Could not convert type userdata to userdata"))
      }
      _ => return Err(mlua::Error::runtime("Unsupported type")),
    };

    native_fields.insert(key, FieldDefinition {
      offset,
      field_type: native_type,
    });
  }

  info!("Created the native struct as userdata");
  let definition_userdata = lua.create_userdata(NativeStructDefinition{fields: native_fields})?;

  info!("Setting user values on native struct definition userdata");
  for pair in fields.pairs::<String, mlua::Table>() {
    let (key, field_definition) = match pair {
      Ok(pair) => pair,
      Err(_) => return Err(mlua::Error::runtime("Field definition must be table")),
    };

    let field_definition_type = field_definition.get::<_, mlua::Value>("type")?;
    let field_definition_type_type_name = field_definition_type.type_name();

    if field_definition_type_type_name == "userdata" {
      definition_userdata.set_named_user_value(&key, field_definition_type)?;
    }
  }

  info!("Successfully created native struct definition");
  Ok(definition_userdata)
}

pub fn create_native_struct_fn<'lua>(lua: &'lua Lua, (address, definition_userdata): (u32, AnyUserData<'lua>)) -> Result<AnyUserData<'lua>, mlua::Error> {
  debug!("Create new native struct at 0x{:x}", address);

  native_struct_from_definition(lua, address, definition_userdata)
}