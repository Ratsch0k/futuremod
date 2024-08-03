use std::{cell::Ref, fmt, marker::PhantomData, mem::size_of, ops::{AddAssign, MulAssign}, sync::{Arc, Mutex}};

use log::info;
use mlua::{AnyUserData, FromLua, IntoLua, Lua, MetaMethod, OwnedTable, UserData, UserDataMethods};
use nalgebra::{DMatrix, Matrix4, Scalar, Vector3};
use num::{traits::{FromBytes, ToBytes}, Num, One, Zero};

use super::LuaResult;

pub fn create_matrix_library(lua: Arc<Lua>) -> Result<OwnedTable, mlua::Error> {
  let table = lua.create_table()?;

  // Float-based dynamic matrix
  table.set("FloatMatrixType", lua.create_proxy::<MatrixType<i32>>()?)?;
  table.set("zerosFloat", lua.create_function(create_zero_matrix::<f32>)?)?;
  table.set("identityFloat", lua.create_function(create_identity_matrix::<f32>)?)?;
  table.set("newFloat", lua.create_function(create_matrix::<f32>)?)?;

  // Integer-based dynamic matrix
  table.set("IntMatrixType", lua.create_proxy::<MatrixType<i32>>()?)?;
  table.set("zerosInt", lua.create_function(create_zero_matrix::<i32>)?)?;
  table.set("identityInt", lua.create_function(create_identity_matrix::<i32>)?)?;
  table.set("newInt", lua.create_function(create_matrix::<i32>)?)?;

  // Model matrix
  table.set("ModelMatrix", lua.create_proxy::<ModelMatrix>()?)?;
  table.set("newModel", lua.create_function(create_model_matrix)?)?;

  Ok(table.into_owned())
}


/// Trait for types that encapsulate their data in an arc and mutex.
/// Those types can implement this trait and return a cloned reference to their inner
/// data.
trait HasArc<T> where Self: Clone {
  fn get_arc(&self) -> Arc<Mutex<T>>;
}

trait MatrixContext<M>: HasArc<M> {
  /// Call the given function with an immutable reference to the matrix.
  fn with_matrix<F, R>(&self, f: F) -> LuaResult<R> where F: Fn(&M) -> LuaResult<R> {
    match self.get_arc().lock() {
      Ok(matrix) => {
        f(&matrix)
      },
      Err(_) => {
        Err(mlua::Error::RuntimeError("Matrix is locked".to_string()))
      },
    }
  }
  
  /// Call the given function with a mutable reference to the matrix.
  fn with_matrix_mut<F, R>(&self, f: F) -> LuaResult<R> where F: Fn(&mut M) -> LuaResult<R> {
    match self.get_arc().lock() {
      Ok(mut matrix) => {
        f(&mut matrix)
      },
      Err(_) => {
        Err(mlua::Error::RuntimeError("Matrix is locked".to_string()))
      },
    }
  }
}

/// Generic matrix of any size for lua.
/// 
/// Stores the matrix in an arc and mutex to ensure that when getting a new reference
/// to a matrix, the matrix content doesn't have to be copied. Also, the mutex ensures
/// that all references to a matrix can modify it without any weird behavior.
#[derive(Debug)]
struct LuaMatrix<T>(Arc<Mutex<DMatrix<T>>>);

impl<T> Clone for LuaMatrix<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> HasArc<DMatrix<T>> for LuaMatrix<T> {
  fn get_arc(&self) -> Arc<Mutex<DMatrix<T>>> {
      self.0.clone()
  }
}

impl<T> MatrixContext<DMatrix<T>> for LuaMatrix<T> {}


impl<T> LuaMatrix<T> {
  pub fn check_bounds(&self, row: u8, col: u8) -> LuaResult<()> {
    self.with_matrix(|matrix| {
      if row as usize > matrix.nrows() || col as usize > matrix.ncols() {
        return Err(mlua::Error::RuntimeError("Index out of bounds".to_string()));
      }

      Ok(())
    })
  }
}

impl<'a, T: 'static> FromLua<'a> for LuaMatrix<T> {
    fn from_lua(value: mlua::Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
      try_from_userdata::<LuaMatrix<T>>(value, lua)
    }
}

/// Try to convert a userdata value into an instance of T.
/// If value is a userdata of type T, this function returns a clone of the userdata.
/// 
/// Errors if the given lua value is not a userdata and if the userdata is not of type T.
fn try_from_userdata<'a, T: 'static>(value: mlua::Value<'a>, _: &'a Lua) -> mlua::Result<T> where T: Clone {
  let userdata = match value.as_userdata() {
    None => return Err(mlua::Error::RuntimeError("Not userdata".to_string())),
    Some(v) => v,
  };
  
  if !userdata.is::<T>() {
    return Err(mlua::Error::RuntimeError("Not a matrix".to_string()));
  }

  let m: Ref<T> = userdata.borrow()?;

  Ok(m.clone())
}

impl<T: Num + Copy + for<'a> IntoLua<'a> + for<'a> FromLua<'a> + fmt::Debug + AddAssign + 'static + ToBytes + MulAssign> UserData for LuaMatrix<T> {
  fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
      fields.add_field_method_get("ncols", |_, matrix| {
        matrix.with_matrix(|matrix| {
          Ok(matrix.ncols())
        })
      });

      fields.add_field_method_get("nrows", |_, matrix| {
        matrix.with_matrix(|matrix| {
          Ok(matrix.nrows())
        })
      });
  }

  fn add_methods<'lua, M>(methods: &mut M) where M: Sized, M: UserDataMethods<'lua, LuaMatrix<T>> {
    methods.add_method("at", |_, matrix, (row, col): (u8, u8)| -> LuaResult<T> {
      matrix.check_bounds(row, col)?;

      matrix.with_matrix(|matrix| {
        Ok(matrix[(row as usize, col as usize)])
      })
    });

    methods.add_method_mut("set", |_, matrix, (row, col, value): (u8, u8, T)| -> LuaResult<()> {
      matrix.check_bounds(row, col)?;

      matrix.with_matrix_mut(|matrix| {
        matrix[(row as usize, col as usize)] = value;

        Ok(())
      })
    });

    methods.add_meta_method(MetaMethod::Add, |_, matrix, rhs: LuaMatrix<T>| -> LuaResult<LuaMatrix<T>> {
      let result = matrix.with_matrix(|lhs| {
        rhs.with_matrix(|rhs| {
          Ok(lhs + rhs)
        })
      })?;

      Ok(LuaMatrix(Arc::new(Mutex::new(result))))
    });

    methods.add_meta_method(MetaMethod::Mul, |_, matrix, rhs: LuaMatrix<T>| -> LuaResult<LuaMatrix<T>> {
      let result = matrix.with_matrix(|lhs| {
        rhs.with_matrix(|rhs| {
          Ok(lhs * rhs)
        })
      })?;

      Ok(LuaMatrix(Arc::new(Mutex::new(result))))
    });

    methods.add_method("toBytes", |_, matrix, ()| -> LuaResult<Vec<u8>> {
      matrix.with_matrix(|matrix| {
        let nrows = matrix.nrows();
        let ncols = matrix.ncols();

        let mut bytes = Vec::<u8>::new();

        for col_idx in 0..ncols {
          for row_idx in 0..nrows {
            let value = matrix[(row_idx, col_idx)];

            let raw_bytes = value.to_le_bytes();

            bytes.extend_from_slice(raw_bytes.as_ref());
          }
        }

        Ok(bytes)
      })
    })
  }
}

/// Create zero matrix
fn create_zero_matrix<'lua, T: Scalar + Zero>(_: &'lua Lua, (rows, columns): (u8, u8)) -> LuaResult<LuaMatrix<T>> {
  Ok(LuaMatrix(Arc::new(Mutex::new(DMatrix::<T>::zeros(rows as usize, columns as usize)))))
}

/// Create identify matrix
fn create_identity_matrix<'lua, T: Scalar + Zero + One>(_: &'lua Lua, size: u8) -> LuaResult<LuaMatrix<T>> {
  let size = size as usize;

  Ok(LuaMatrix(Arc::new(Mutex::new(DMatrix::<T>::identity(size, size)))))
}

/// Create matrix from the given nested list of values.
/// 
/// Each item of the outer list represents one row in the matrix.
/// The nested list of a row, contains the row's values.
/// For example, to build the following matrix
/// ```
/// 1 2 3
/// 4 5 6
/// ```
/// you must call this function as shown:
/// ```rust
/// create_matrix(vec![vec![1, 2, 3], vec![4, 5, 6]])
/// ```
/// 
/// All rows must have the same length, otherwise, this function panics.
fn create_matrix<'lua, T: Scalar>(_: &'lua Lua, data: Vec<Vec<T>>) -> LuaResult<LuaMatrix<T>> {
  let rows = data.len();
  let cols = data.iter().map(|r| r.len()).fold(0, |l, r| l.max(r));
  let data: Vec<T> = data.into_iter().flatten().collect();

  Ok(LuaMatrix(Arc::new(Mutex::new(DMatrix::<T>::from_row_slice(rows, cols, &data)))))
}

#[derive(Debug, Clone, Copy)]
struct MatrixType<T> {
  nrows: u32,
  ncols: u32,
  matrix_type: PhantomData<T>,
}

impl<T> MatrixType<T> {
  pub fn new(nrows: u32, ncols: u32) -> MatrixType<T> {
    MatrixType {
      nrows,
      ncols,
      matrix_type: PhantomData,
    }
  }

  pub fn get_byte_size(&self) -> usize {
    size_of::<T>() * self.nrows as usize * self.ncols as usize
  }
}


impl<T: 'static + ToBytes + FromBytes + Num + Copy + AddAssign + MulAssign + fmt::Debug + for<'a> IntoLua<'a> + for<'a> FromLua<'a>> UserData for MatrixType<T> {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
      methods.add_function("new", |_, (nrows, ncols): (u32, u32)| {
        Ok(MatrixType::<T>::new(nrows, ncols))
      });

      methods.add_method("getByteSize", |_, this, ()| {
        Ok(4 * this.nrows * this.ncols)
      });

      methods.add_method("toBytes", |_, this, matrix: LuaMatrix<T>| {
        let matrix_arc = matrix.get_arc();
        let inner_matrix = matrix_arc.lock().map_err(|_| mlua::Error::RuntimeError("Could not get lock to matrix".to_string()))?;

        let mut bytes = Vec::<u8>::new();

        for row_idx in 0..this.nrows {
          for col_idx in 0..this.ncols {
            let value = &inner_matrix[(row_idx as usize, col_idx as usize)];

            let value_bytes = value.to_le_bytes();
            bytes.extend_from_slice(value_bytes.as_ref());
          }
        }

        Ok(bytes)
      });

      methods.add_method("fromBytes", |lua, this, bytes: Vec<u8>| -> LuaResult<LuaMatrix<T>> {
        info!("Matrix.fromBytes");
        // This entire implementation is really ugly
        // Maybe I should rewrite the implementation for dynamically sized matrix and use fixed types instead.
        // That would avoid a lot of problems but lead to more code.
        let value_byte_size = size_of::<T>();
        let byte_size = this.get_byte_size();
        info!("Expecting {} bytes, each value has size {}", byte_size, value_byte_size);

        if bytes.len() < byte_size {
          return Err(mlua::Error::RuntimeError(format!("{} bytes required to construct the matrix", byte_size)));
        }

        info!("Collecting into values");
        let mut values = Vec::<T>::new();

        for chunk_idx in (0..bytes.len()).step_by(value_byte_size) {
          info!("Converting bytes at {} into value", chunk_idx);
          let mut value_bytes = Vec::<u8>::new();
          for byte_idx in 0..value_byte_size {
            value_bytes.push(bytes[chunk_idx + byte_idx]);
          }

          info!("Converting bytes {:?} into value", value_bytes);

          let value: T = unsafe {
            let value_ptr = bytes.as_ptr().offset(chunk_idx as isize) as *const T;

            *value_ptr
          };

          info!("Converted into {:?}", value);

          values.push(value)
        }

        info!("Collectin value into row-major matrix vector");
        let mut matrix_str = String::from("Matrix\n");
        let mut matrix_data = Vec::<Vec<T>>::new();
        for row_idx in 0..this.nrows as usize {
          matrix_data.push(Vec::<T>::new());
          for col_idx in 0..this.ncols as usize {
            matrix_data[row_idx as usize].push(values[row_idx * this.ncols as usize + col_idx]);
            matrix_str.push_str(format!("{:?} ", values[row_idx * this.ncols as usize + col_idx]).as_str());
          }
          matrix_str.push_str("\n");
        }

        info!("{}", matrix_str);

        info!("Creating matrix from row-major vector");
        create_matrix(lua, matrix_data)
      })
    }
}

/// Special matrix that represents a models location, rotation, and scale in the world.
/// When converted into bytes this matrix uses the same memory layout as FutureCop.
/// 
/// FutureCop uses the following memory layout:
/// ```
/// +---------------------------------+
/// | Rotation and Scale (3x3 matrix) |
/// |        Stored per row           |
/// |  (m0x0, m0x1, m0x2), (m1x0,...) |
/// +---------------------------------+
/// | Location (3D Vector)            |
/// +---------------------------------+
/// | Location (3D Vector)            |
/// +---------------------------------+
/// ```
#[derive(Debug)]
struct ModelMatrix(Arc<Mutex<Matrix4<f32>>>);

impl ModelMatrix {
  /// Creates a new model matrix initialized as an identify matrix
  pub fn new() -> ModelMatrix {
    ModelMatrix(Arc::new(Mutex::new(Matrix4::identity())))
  }

  /// Creates a new model matrix from a slice in row-major order.
  pub fn from_slice(slice: &[f32]) -> ModelMatrix {
    ModelMatrix(Arc::new(Mutex::new(Matrix4::from_row_slice(slice))))
  }
}

impl Clone for ModelMatrix {
  fn clone(&self) -> Self {
      Self(self.0.clone())
  }
}

impl HasArc<Matrix4<f32>> for ModelMatrix {
  fn get_arc(&self) -> Arc<Mutex<Matrix4<f32>>> {
      self.0.clone()
  }
}

impl MatrixContext<Matrix4<f32>> for ModelMatrix {}

impl ModelMatrix {
  pub fn check_bounds(&self, row: u8, col: u8) -> LuaResult<()> {
    self.with_matrix(|matrix| {
      if row as usize > matrix.nrows() || col as usize > matrix.ncols() {
        return Err(mlua::Error::RuntimeError("Index out of bounds".to_string()));
      }

      Ok(())
    })
  }
}

impl<'a> FromLua<'a> for ModelMatrix {
  fn from_lua(value: mlua::Value<'a>, lua: &'a Lua) -> mlua::Result<Self> {
    try_from_userdata::<ModelMatrix>(value, lua)
  }
}

impl UserData for ModelMatrix {
  fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
      fields.add_field_method_get("ncols", |_, matrix| {
        matrix.with_matrix(|matrix| {
          Ok(matrix.ncols())
        })
      });

      fields.add_field_method_get("nrows", |_, matrix| {
        matrix.with_matrix(|matrix| {
          Ok(matrix.nrows())
        })
      });
  }

  fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
    methods.add_method("at", |_, matrix, (row, col): (u8, u8)| -> LuaResult<f32> {
      matrix.check_bounds(row, col)?;

      matrix.with_matrix(|matrix| {
        Ok(matrix[(row as usize, col as usize)])
      })
    });

    methods.add_method_mut("set", |_, matrix, (row, col, value): (u8, u8, f32)| -> LuaResult<()> {
      matrix.check_bounds(row, col)?;

      matrix.with_matrix_mut(|matrix| {
        matrix[(row as usize, col as usize)] = value;

        Ok(())
      })
    });

    methods.add_meta_method(MetaMethod::Add, |_, matrix, rhs: ModelMatrix| -> LuaResult<ModelMatrix> {
      let result = matrix.with_matrix(|lhs| {
        rhs.with_matrix(|rhs| {
          Ok(lhs + rhs)
        })
      })?;

      Ok(ModelMatrix(Arc::new(Mutex::new(result))))
    });

    methods.add_meta_method(MetaMethod::Mul, |_, matrix, rhs: ModelMatrix| -> LuaResult<ModelMatrix> {
      let result = matrix.with_matrix(|lhs| {
        rhs.with_matrix(|rhs| {
          Ok(lhs * rhs)
        })
      })?;

      Ok(ModelMatrix(Arc::new(Mutex::new(result))))
    });

    methods.add_function("toBytes", |_, (_, matrix): (AnyUserData, AnyUserData)| -> LuaResult<Vec<u8>> {
      let matrix: Ref<ModelMatrix> = matrix.borrow()?;

      matrix.with_matrix(|matrix| {
        let mut bytes = Vec::<u8>::new();

        // Hard coded matrix size because the underlying matrix is always 4x4
        // This loop collects the bytes for the scale and rotation matrix

        // Before collecting a value's byte we must first cast it to i32 because the game
        // doesn't use floats for its model matrices.
        for col_idx in 0..3 {
          for row_idx in 0..3 {
            let value = matrix[(row_idx, col_idx)] as i32;

            let raw_bytes = value.to_le_bytes();

            bytes.extend_from_slice(raw_bytes.as_ref());
          }
        }

        // Next, collect the bytes for the translation part
        let mut rotation_bytes = Vec::<u8>::new();

        rotation_bytes.extend_from_slice(&(matrix[(0, 3)] as i32).to_le_bytes());
        rotation_bytes.extend_from_slice(&(matrix[(1, 3)] as i32).to_le_bytes());
        rotation_bytes.extend_from_slice(&(matrix[(2, 3)] as i32).to_le_bytes());

        // Now push the translation bytes into the bytes two time
        bytes.extend(rotation_bytes.iter());
        bytes.extend(rotation_bytes.iter());

        Ok(bytes)
      })
    });

    methods.add_method("getByteSize", |_, _, ()| -> LuaResult<u32> {
      Ok(4 * 4 * 4)  // Model matrix has the constant size: bytes for u32 * rows * columns
    });

    // Construct a new model matrix from the given vector of bytes
    methods.add_function("fromBytes", |_, bytes: Vec<u8>| -> LuaResult<ModelMatrix> {
      if bytes.len() < 64 {
        return Err(mlua::Error::RuntimeError("Model matrix requires 64 bytes".to_string()));
      }

      let mut values = Vec::<f32>::new();

      for chunk_idx in (0..bytes.len()).step_by(4) {
        // Values are stored as i32 and not as f32.
        // Thus, we first collect four bytes and turn them back into a i32
        let value = i32::from_le_bytes([bytes[chunk_idx], bytes[chunk_idx+1], bytes[chunk_idx+2], bytes[chunk_idx+3]]);

        // Then we can cast it to an f32 and actually push it into the vector
        values.push(value as f32)
      }

      Ok(ModelMatrix::from_slice(&values))
    });

    methods.add_method("translate", |_, matrix, (x, y, z): (f32, f32, f32)| {
      matrix.with_matrix_mut(|matrix| {
        matrix.append_translation_mut(&Vector3::new(x, y, z));

        Ok(())
      })
    });

    methods.add_method("scale", |_, matrix, (x, y, z): (f32, f32, f32)| {
      matrix.with_matrix_mut(|matrix| {
        matrix.append_nonuniform_scaling_mut(&Vector3::new(x, y, z));

        Ok(())
      })
    });

    methods.add_method("uniformScale", |_, matrix, scaling: f32| {
      matrix.with_matrix_mut(|matrix| {
        matrix.append_scaling_mut(scaling);

        Ok(())
      })
    });

    methods.add_method("rotate", |_, matrix, (x, y, z, angle): (f32, f32, f32, f32)| {
      matrix.with_matrix_mut(|matrix| {
        let rotation = Matrix4::from_scaled_axis(&Vector3::new(x, y, z) * angle);

        *matrix = rotation * *matrix;

        Ok(())
      })
    });
  }
}

/// Create a model matrix.
/// 
/// The matrix is initialized with an identify matrix.
fn create_model_matrix(_: &Lua, (): ()) -> LuaResult<ModelMatrix> {
  Ok(ModelMatrix(Arc::new(Mutex::new(Matrix4::identity()))))
}