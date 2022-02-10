/*!
The opengl shader program module provides idiomatic bindings to OpenGL shaderprograms.

As with everything in graphene, the focus is on a 2D application, so shader reusage, combination, or mega-shaders
are not a usecase we want to cover.

In our scenario OpenGL shaderprograms go through a few stages:
1. Creation
2. Linkage

We do want to model this by providing both a [ShaderProgram](ShaderProgram) and a [LinkedShaderProgram](LinkedShaderProgram) type
which allows compiletime checking for whether
or not a shader was compiled before being attached to a shaderprogram.
*/

use crate::opengl::shader::ShaderType;
use crate::opengl::shader::*;

/// Stores the underlying data of a shaderprogram
///
/// Can only be accessed through the unsafe `[inner](inner)/[inner_mut](inner_mut)` methods of the [ShaderProgram](ShaderProgram) struct.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ShaderProgramInner {
    /// The id of the shaderprogram
    id: gl::types::GLuint,
}

/**
A shaderprogram serves as a pipeline that is run in parallel per texel of the rendering target.

They consist of a multiple stages, some of which can be customised with others that have to be specified.

You can read about the rendering pipeline [here](https://www.khronos.org/opengl/wiki/Rendering_Pipeline_Overview).

For our purposes a shaderprogram has to be setup, linked and can only then be used.
We model this with the newtype patern by having both a ShaderProgram class and a [LinkedShaderProgram](LinkedShaderProgram) class.

# Example
```
let vertex = Shader::new(ShaderType::Vertex)
                    .expect("Unable to create vertex shader")
                    .compile(&vertex_shader_data)
                    .expect("Unable to compile fertex shader");

let fragment = Shader::new(ShaderType::Fragment)
                      .expect("Unable to create fragment shader")
                      .compile(&fragment_shader_data)
                      .expect("Unable to compile fragment shader");

let program = ShaderProgram::new()
                            .expect("Unable to create shader program")
                            .link(vertex, None, None, None, fragment)
                            .expect("Unable to link shader program");

// ...

while (!window.is_closed()) {
    clear();
    program.use();
    render();
}
```
*/
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ShaderProgram {
    inner: ShaderProgramInner,
}

/// Error enum for the failed linkage of a shaderprogram
#[derive(thiserror::Error, Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum ShaderProgramLinkError {
    /// Used if an error occurs in the underlying driver
    #[error("OpenGL error: {0}")]
    #[allow(non_camel_case_types)]
    OpenGL_Error(gl::types::GLenum),
}

impl ShaderProgram {
    /// Attempts to link a shader program with a set of shaders to assemble a pipeline
    /// Only linked shader programs can be used
    ///
    /// # Example
    /// ```
    /// let (vertex, geometry, fragment) = crate_shaders();
    /// let program = ShaderProgram::new().expect("Unable to create shader program");
    /// let linked = program.link(vertex, None, None, Some(geometry), fragment).expect("Unable to link shader program");
    ///
    /// // ...
    ///
    /// linked.use();
    /// render();
    /// ```
    pub fn link(
        self,
        vertex: CompiledShader<{ ShaderType::Vertex }>,
        tess_ctrl: Option<CompiledShader<{ ShaderType::TessControl }>>,
        tess_eval: Option<CompiledShader<{ ShaderType::TessEvaluation }>>,
        geometry: Option<CompiledShader<{ ShaderType::Geometry }>>,
        fragment: CompiledShader<{ ShaderType::Fragment }>,
    ) -> Result<LinkedShaderProgram, ShaderProgramLinkError> {
        unsafe {
            gl::AttachShader(self.get_id(), vertex.get_id());

            match gl::GetError() {
                gl::NO_ERROR => {}
                err => {
                    return Err(ShaderProgramLinkError::OpenGL_Error(err));
                }
            }
        };

        if let Some(tess_ctrl) = tess_ctrl {
            unsafe {
                gl::AttachShader(self.get_id(), tess_ctrl.get_id());
            }
        }

        if let Some(tess_eval) = tess_eval {
            unsafe {
                gl::AttachShader(self.get_id(), tess_eval.get_id());
            }
        }

        if let Some(geometry) = geometry {
            unsafe {
                gl::AttachShader(self.get_id(), geometry.get_id());
            }
        }

        unsafe {
            gl::AttachShader(self.get_id(), fragment.get_id());
        }

        Ok(LinkedShaderProgram {
            inner: LinkedShaderProgramInner { program: self },
        })
    }

    /**
    Retrieves the id of the program.

    # Example
    ```
    let program = ShaderProgram::new().expect("Unable to create shader program");
    assert_eq!(shader.get_id(), 1); // example, YMMV
    ```
    */
    pub fn get_id(&self) -> gl::types::GLuint {
        self.inner.id
    }
}

#[derive(Debug)]
pub struct LinkedShaderProgramInner {
    program: ShaderProgram,
}

#[derive(Debug)]
pub struct LinkedShaderProgram {
    inner: LinkedShaderProgramInner,
}
