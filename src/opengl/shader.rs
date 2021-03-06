/*!
The opengl shader module provides idiomatic bindings to OpenGL shaders.

As with everything in graphene, the focus is on a 2D application, so shader reusage, combination, or mega-shaders
are not a usecase we want to cover.

In our scenario OpenGL shaders go through a few stages:
1. Creation
2. Compilation
3. Attachment

We do want to model this by providing both a Shader and a CompiledShader type which allows compiletime checking for whether
or not a shader was compiled before being attached to a ShaderProgram.
*/

/**
Wraps the different OpenGL shader types.

OpenGL supports 6 different types of shaders for varying stages of the pipeline.
You can read more about shaders here: <https://www.khronos.org/opengl/wiki/Shader>
And more about the pipeline here: <https://www.khronos.org/opengl/wiki/Rendering_Pipeline_Overview>

# Examples
```
// creates a compute-shader
let r#type = ShaderType::Compute;
let shader = Shader::new(r#type).expect("Unable to create shader");
```
*/

// sadly you cannot use repr with type aliases, so we cannot use repr(gl::types::GLenum)
// it might be safer to generate to & from versions of this, although I doubt the OpenGL type for GLenum will ever change
#[repr(u32)]
#[derive(Debug, strum_macros::Display, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum ShaderType {
    /// Used for [OpenGL compute shaders](https://www.khronos.org/opengl/wiki/Compute_Shader)
    Compute = gl::COMPUTE_SHADER,
    /// Used for [OpenGL fragment shaders](https://www.khronos.org/opengl/wiki/Fragment_Shader)
    FragmentShader = gl::FRAGMENT_SHADER,
    /// Used for [OpenGL geometry shaders](https://www.khronos.org/opengl/wiki/Geometry_Shader)
    GeometryShader = gl::GEOMETRY_SHADER,
    /// Used for [OpenGL tesselation control shaders](https://www.khronos.org/opengl/wiki/Tessellation_Control_Shader)
    TessControl = gl::TESS_CONTROL_SHADER,
    /// Used for [OpenGL tesselation evaluation shaders](https://www.khronos.org/opengl/wiki/Tessellation_Evaluation_Shader)
    TessEvaluation = gl::TESS_EVALUATION_SHADER,
    /// Used for [OpenGL vertex shaders](https://www.khronos.org/opengl/wiki/Vertex_Shader)
    Vertex = gl::VERTEX_SHADER,
}

/// Stores the underlying data of a shader
///
/// Can only be accessed through the unsafe `[inner](inner)/[inner_mut](inner_mut)` methods of the [Shader](Shader) struct.
#[derive(Debug)]
pub struct ShaderInner {
    /// The id of the shader, generated by OpenGL and valid for the lifetime of the shader
    pub id: gl::types::GLuint,
    /// The type of the shader. See [ShaderType](ShaderType) for more information
    pub r#type: ShaderType,
}

impl PartialEq for ShaderInner {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl Eq for ShaderInner {}

impl std::hash::Hash for ShaderInner {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}

// It doesn't really make sense for shaders to be ordered but there are usecases where you'd want to store them in a set/map
impl PartialOrd for ShaderInner {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ShaderInner {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/**
A shader serves as a program that is executed in parallel for each texel on the screen.

There are different kinds of shaders with different applications.
See [ShaderType](ShaderType) for more information on the different types.

For our purposes a shader has to be setup, compiled and is then attached to a shader program.
We model this with the newtype patern by having both a Shader class and a [CompiledShader](CompiledShader) class.

# Example
```
let shader = Shader::new(ShaderType::Vertex).expect("Unable to create vertex shader");
let compiled = shader.compile(shader_src).expect("Unable to compile vertex shader");
let program = ShaderProgram::new().expect("Unable to create shader program");
let linked = program.link(IntoIter::new(&[shader])).expect("Unable to link shader program");
```
*/
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Shader {
    inner: ShaderInner,
}

impl Shader {
    /// Returns a reference to the inner (private) data of the shader.
    /// Use at your own risk, no guarantees are made to the data itself.
    pub unsafe fn inner(&self) -> &ShaderInner {
        &self.inner
    }

    /// Returns a reference to the inner (private) data of the shader.
    /// Use at your own risk, no guarantees are made to the data itself, mutating it is to be considered UB.
    pub unsafe fn inner_mut(&mut self) -> &mut ShaderInner {
        &mut self.inner
    }

    // no get_delete_status since the shader is only ever deleted if it's dropped

    /**
    Retrieves the id of the shader.

    # Example
    ```
    let shader = Shader::new(ShaderType::Compute).expect("Unable to create compute shader");
    assert_eq!(shader.get_id(), 1); // example, YMMV
    ```
    */
    pub fn get_id(&self) -> gl::types::GLuint {
        self.inner.id
    }

    /**
    Retrieves the type of the shader.

    # Example
    ```
    let shader = Shader::new(ShaderType::Compute).expect("Unable to create compute shader");
    assert_eq!(shader.get_type(), ShaderType::Compute);
    ```
    */
    pub fn get_type(&self) -> ShaderType {
        self.inner.r#type
    }
}

/// Error enum for the failed creation of a shader
#[derive(thiserror::Error, Debug, Clone, Eq, PartialEq)]
pub enum ShaderCreationError {
    /// Used if the underlying shadertype is not valid for OpenGL. This ought not to ever happen
    /// but since we strive for idiomatic code it has to be mapped anyway
    #[error("Invalid ShaderType enum: {0}")]
    InvalidEnum(ShaderType),
    /// Used if the underlying OpenGL error is unknown to graphene
    #[error("Unknown Error")]
    Unknown,
}

impl Shader {
    /// Returns a new shader or an error if one occurs in the underlying driver, which shouldn't happen realistically speaking.
    pub fn new(r#type: ShaderType) -> Result<Shader, ShaderCreationError> {
        let id = unsafe { gl::CreateShader(r#type as _) };

        if id == 0 {
            return Err(match unsafe { gl::GetError() } {
                gl::INVALID_ENUM => ShaderCreationError::InvalidEnum(r#type),
                _ => ShaderCreationError::Unknown,
            });
        }

        Ok(Shader {
            inner: ShaderInner { id, r#type },
        })
    }
}

/// Error enum for the failed compilation of a shader
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ShaderCompileError {
    /// Used if the source was passed in improperly formatted UTF8 that couldn't be converted to a c-string
    #[error("The source passed was not properly formatted in UTF8 and could not be converted to a CString: {source}")]
    InvalidUTF8Source {
        #[from]
        /// The underlying c-string conversion error
        source: std::ffi::NulError,
    },
    /// Used if the shader's source couldn't be compiled due to a compilation error.
    #[error("Invalid shader source: {0}")]
    CompilationError(String),
    /// Used if the shader's source couldn't be compiled due to a compilation error and the error message obtained
    /// couldn't be converted to a rust string because it was missing a null byte at the end
    #[error("Shader log's error message didn't contain a null byte at the end")]
    MissingNullByte {
        #[from]
        /// The underlying c-string conversion error
        source: std::ffi::FromVecWithNulError,
    },
    /// Used if the shader's source couldn't be compiled due to a compilation error and the error message obtained
    /// couldn't be converted to a rust string because it was invalid UTF8
    #[error("Shader log's error message wasn't valid UTF8")]
    InvalidUTF8LogSource {
        #[from]
        /// The underlying c-string conversion error
        source: std::ffi::IntoStringError,
    },
    /// Used if the underlying object was not created by OpenGL
    #[error("The underlying object was not created by OpenGL")]
    NotAnOpenGLValue,
    /// Used if the underlying object was not recognised as an OpenGL shader
    #[error("The underlying object was not recognised as an OpenGL shader")]
    NotAShader,
    /// Used if the underlying OpenGL error is unknown to graphene
    #[error("Unknown Error")]
    Unknown,
}

impl Shader {
    /// Compiles the shader and returns a [CompiledShader](CompiledShader) that wraps the current object or returns an error if the operation fails.
    /// Failure is realistic in this situation and can happen in a variety of cases:
    /// 1. The source is invalid
    /// 2. The shader is in an invalid state
    /// 3. An underlying driver issue occurred
    pub fn compile<S: AsRef<str>>(self, src: S) -> Result<CompiledShader, ShaderCompileError> {
        let cstr = std::ffi::CString::new(src.as_ref().as_bytes())?;

        let rc = unsafe {
            let count = 1;
            gl::ShaderSource(self.inner.id, 1, &cstr.as_ptr(), &count);
            gl::GetError()
        };

        match rc {
            gl::NO_ERROR => {}
            gl::INVALID_VALUE => {
                return Err(ShaderCompileError::NotAnOpenGLValue);
            }
            gl::INVALID_OPERATION => {
                return Err(ShaderCompileError::NotAShader);
            }
            // ignoring "INVALID_VALUE if count is less than 0" since we literally hard-coded count to be 1
            _ => return Err(ShaderCompileError::Unknown),
        }

        let rc = unsafe {
            gl::CompileShader(self.inner.id);
            gl::GetError()
        };

        match rc {
            gl::NO_ERROR => {}
            gl::INVALID_VALUE => {
                return Err(ShaderCompileError::NotAnOpenGLValue);
            }
            gl::INVALID_OPERATION => {
                return Err(ShaderCompileError::NotAShader);
            }
            _ => {
                return Err(ShaderCompileError::Unknown);
            }
        }

        let mut compile_status = 0;
        unsafe {
            gl::GetShaderiv(self.inner.id, gl::COMPILE_STATUS, &mut compile_status);
        }

        if compile_status == 0 {
            const CAPACITY: usize = 1024;
            let mut log = Vec::<u8>::with_capacity(CAPACITY);
            let mut length = 0;
            unsafe { gl::GetShaderInfoLog(self.inner.id, CAPACITY as i32, &mut length, log.as_mut_ptr() as *mut i8) };

            unsafe {
                log.set_len(((length + 1) as usize).min(CAPACITY));
            }

            let s = std::ffi::CString::from_vec_with_nul(log)?.into_string()?;

            return Err(ShaderCompileError::CompilationError(s));
        }

        Ok(CompiledShader {
            inner: CompiledShaderInner { shader: self },
        })
    }
}

/// Stores the underlying data of a compiled shader
///
/// Can only be accessed through the unsafe `[inner](inner)/[inner_mut](inner_mut)` methods of the [CompiledShader](CompiledShader) struct.
#[derive(Debug)]
pub struct CompiledShaderInner {
    /// The underlying shader that is being wrapped after compilation
    pub shader: Shader,
}

/**
A compiled shader is the second stage that a shader goes through before being attached to a program and thus destroyed.
If compilation is successful this struct will be returned and will stand in as the future access point for all interaction with the shader.
Only compiled shaders can be attached to programs

# Example
```
let shader = Shader::new(ShaderType::Vertex).expect("Unable to create vertex shader");
let compiled = shader.compile(shader_src).expect("Unable to compile vertex shader");
let program = ShaderProgram::new().expect("Unable to create shader program");
let linked = program.link(IntoIter::new(&[shader])).expect("Unable to link shader program");
```
*/
#[derive(Debug)]
pub struct CompiledShader {
    inner: CompiledShaderInner,
}

impl CompiledShader {
    /// Returns a reference to the inner (private) data of the shader.
    /// Use at your own risk, no guarantees are made to the data itself.
    pub unsafe fn inner(&self) -> &CompiledShaderInner {
        &self.inner
    }

    /// Returns a reference to the inner (private) data of the shader.
    /// Use at your own risk, no guarantees are made to the data itself, mutating it is to be considered UB.
    pub unsafe fn inner_mut(&mut self) -> &mut CompiledShaderInner {
        &mut self.inner
    }

    /**
    Retrieves the id of the compiled shader.

    # Example
    ```
    let shader = Shader::new(ShaderType::Compute).expect("Unable to create compute shader");
    let shader = shader.compile(&shader_src).expect("Unable to compile shader");
    assert_eq!(shader.get_id(), 1); // example, YMMV
    ```
    */
    pub fn get_id(&self) -> gl::types::GLuint {
        self.inner.shader.inner.id
    }

    /**
    Retrieves the type of the compiled shader.

    # Example
    ```
    let shader = Shader::new(ShaderType::Compute).expect("Unable to create compute shader");
    let shader = shader.compile(&shader_src).expect("Unable to compile shader");
    assert_eq!(shader.get_type(), ShaderType::Compute);
    ```
    */
    pub fn get_type(&self) -> ShaderType {
        self.inner.shader.inner.r#type
    }
}

/// Error enum for the failed retrieval of a compiled shader's source's len
#[derive(thiserror::Error, Debug, Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub enum SourceLenRetrievalError {
    /// Used if the underlying object was not created by OpenGL
    #[error("The underlying object was not created by OpenGL")]
    NotAnOpenGLValue,
    /// Used if the underlying object was not recognised as an OpenGL shader
    #[error("The underlying object was not recognised as an OpenGL shader")]
    NotAShader,
    /// Used if GL_SOURCE_LENGTH isn't recognised as an invalid enum
    #[error("GL_SOURCE_LENGTH was not recognised as a valid enum to obtain from OpenGL")]
    InvalidEnum,
    /// Used if the underlying OpenGL error is unknown to graphene
    #[error("Unknown Error")]
    Unknown,
}

impl CompiledShader {
    /// Returns the length of the concatenated string of all sub-strings passed to OpenGL as the shader's source during compilation
    pub fn get_source_len(&self) -> Result<usize, SourceLenRetrievalError> {
        let mut iv = 0;

        let rc = unsafe {
            gl::GetShaderiv(self.get_id(), gl::SHADER_SOURCE_LENGTH, &mut iv);
            gl::GetError()
        };

        match rc {
            gl::NO_ERROR => {}
            gl::INVALID_VALUE => return Err(SourceLenRetrievalError::NotAnOpenGLValue),
            gl::INVALID_OPERATION => return Err(SourceLenRetrievalError::NotAShader),
            gl::INVALID_ENUM => return Err(SourceLenRetrievalError::InvalidEnum),
            _ => return Err(SourceLenRetrievalError::Unknown),
        }

        Ok(iv as usize)
    }
}

/// Error enum for the failed retrieval of a compiled shader's source
#[derive(thiserror::Error, Debug, Clone, Eq, PartialEq)]
pub enum SourceRetrievalError {
    /// Used when the length of the source can't be predetermined, which is necessary to pre-allocate the buffer in which
    /// OpenGL then writes the source string
    #[error("Unable to obtain the length of the source for pre-allocation: {source}")]
    UnableToRetrieveSourceLength {
        #[from]
        /// The underlying error that was caused when trying to read the source's length
        source: SourceLenRetrievalError,
    },
    /// Used when the shader's source contained at least one nul-byte in an invalid position
    #[error("Shader's source contained an nul-byte at an invalid position: {source}")]
    MissingNullByte {
        #[from]
        /// The underlying c-string conversion error
        source: std::ffi::FromVecWithNulError,
    },
    /// Used if the shader's source couldn't be compiled due to a compilation error and the error message obtained
    /// couldn't be converted to a rust string because it was invalid UTF8
    #[error("Shader's source couldn't be converted into UTF8: {source}")]
    InvalidUTF8LogSource {
        #[from]
        /// The underlying c-string conversion error
        source: std::ffi::IntoStringError,
    },
    /// Used if the underlying object was not created by OpenGL
    #[error("The underlying object was not created by OpenGL")]
    NotAnOpenGLValue,
    /// Used if the underlying object was not recognised as an OpenGL shader
    #[error("The underlying object was not recognised as an OpenGL shader")]
    NotAShader,
    /// Used if the underlying OpenGL error is unknown to graphene
    #[error("Unknown Error")]
    Unknown,
}

impl CompiledShader {
    /// Returns a concatenated string of all sub-strings passed to OpenGL as the shader's source during compilation
    pub fn get_source(&self) -> Result<String, SourceRetrievalError> {
        let len = self.get_source_len()?;

        // includes nul-byte
        let mut buffer: Vec<u8> = Vec::with_capacity(len);

        let rc = unsafe {
            gl::GetShaderSource(self.get_id(), len as i32, std::ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
            buffer.set_len(len);
            gl::GetError()
        };

        match rc {
            gl::NO_ERROR => {}
            gl::INVALID_VALUE => return Err(SourceRetrievalError::NotAnOpenGLValue),
            gl::INVALID_OPERATION => return Err(SourceRetrievalError::NotAShader),
            _ => return Err(SourceRetrievalError::Unknown),
        }

        Ok(std::ffi::CString::from_vec_with_nul(buffer)?.into_string()?)
    }
}
