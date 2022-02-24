use glm::{ext::translate, transpose, vec3, Matrix3, Vec2, Vector2};
use glutin::{self, PossiblyCurrent};

use std::ffi::CStr;

pub mod gl {
    // pub use self::Gl as Gl;
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

pub struct Gl {
    pub gl: self::gl::Gl,
    pub program: u32,
    pub rect_vbo: u32,
    pub rect_vao: u32,
}

pub fn load(gl_context: &glutin::Context<PossiblyCurrent>) -> Gl {
    let gl = gl::Gl::load_with(|ptr| gl_context.get_proc_address(ptr) as *const _);

    let version = unsafe {
        let data = CStr::from_ptr(gl.GetString(gl::VERSION) as *const _)
            .to_bytes()
            .to_vec();
        String::from_utf8(data).unwrap()
    };
    println!("OpenGL version {}", version);
    unsafe {
        let program = {
            let vs = gl.CreateShader(gl::VERTEX_SHADER);
            gl.ShaderSource(
                vs,
                1,
                [VS_SRC.as_ptr() as *const _].as_ptr(),
                std::ptr::null(),
            );
            gl.CompileShader(vs);

            let fs = gl.CreateShader(gl::FRAGMENT_SHADER);
            gl.ShaderSource(
                fs,
                1,
                [FS_SRC.as_ptr() as *const _].as_ptr(),
                std::ptr::null(),
            );
            gl.CompileShader(fs);

            let program = gl.CreateProgram();
            gl.AttachShader(program, vs);
            gl.AttachShader(program, fs);
            gl.LinkProgram(program);
            gl.DeleteShader(vs);
            gl.DeleteShader(fs);
            program
        };

        gl.UseProgram(program);

        let mut vb = std::mem::zeroed();
        gl.GenBuffers(1, &mut vb);
        gl.BindBuffer(gl::ARRAY_BUFFER, vb);
        gl.BufferData(
            gl::ARRAY_BUFFER,
            (RECT_DATA.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
            RECT_DATA.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let vao = if gl.BindVertexArray.is_loaded() {
            let mut vao = std::mem::zeroed();
            gl.GenVertexArrays(1, &mut vao);
            gl.BindVertexArray(vao);
            vao
        } else {
            0
        };

        let pos_attrib = gl.GetAttribLocation(program, b"position\0".as_ptr() as *const _);
        let color_attrib = gl.GetAttribLocation(program, b"color\0".as_ptr() as *const _);
        gl.VertexAttribPointer(
            pos_attrib as gl::types::GLuint,
            2,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            std::ptr::null(),
        );
        gl.VertexAttribPointer(
            color_attrib as gl::types::GLuint,
            3,
            gl::FLOAT,
            0,
            5 * std::mem::size_of::<f32>() as gl::types::GLsizei,
            (2 * std::mem::size_of::<f32>()) as *const () as *const _,
        );
        gl.EnableVertexAttribArray(pos_attrib as gl::types::GLuint);
        gl.EnableVertexAttribArray(color_attrib as gl::types::GLuint);

        let array = [
            vec3(1.0, 0.0, 1.0),
            vec3(0.0, 1.0, 1.0),
            vec3(0.0, 0.0, 1.0),
        ];
        let trans = glm::Matrix3::from_array(&array);
        gl.ProgramUniformMatrix3fv(
            program,
            0,
            1,
            gl::FALSE,
            trans.as_array().as_ptr() as *const GLfloat,
        );
        Gl {
            gl,
            program,
            rect_vbo: vb,
            rect_vao: vao,
        }
    }
}

impl Gl {
    #[inline(always)]
    pub fn draw_frame(&self) {
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLE_STRIP, 0, 4);
        }
    }

    #[inline(always)]
    pub fn clear(&self, color: [f32; 4]) {
        unsafe {
            self.gl.ClearColor(color[0], color[1], color[2], color[3]);
            self.gl.Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn update_uniform_mat3(&self, mat: glm::Mat3) {
        unsafe {
            self.gl.ProgramUniformMatrix3fv(
                self.program,
                0,
                1,
                gl::FALSE,
                mat.as_array().as_ptr() as *const GLfloat,
            );
        }
    }

    #[inline(always)]
    pub fn draw_rect_vertex(&self, vertices: &[f32]) {
        unsafe {
            self.gl.BindBuffer(gl::ARRAY_BUFFER, self.rect_vbo);
            self.gl.BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as gl::types::GLsizeiptr,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            self.draw_frame();
        }
    }

    #[inline(always)]
    pub fn draw_rect(&self) {
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    #[inline(always)]
    pub fn draw_rect_frame(&self) {
        unsafe {
            self.gl.DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }
}

#[rustfmt::skip]
static VERTEX_DATA: [f32; 15] = [
    -0.5, -0.5,  1.0,  0.0,  0.0,
     0.0,  0.5,  0.0,  1.0,  0.0,
     0.5, -0.5,  0.0,  0.0,  1.0,
];

#[rustfmt::skip]
static RECT_DATA: [f32; 20] = [
    -1.0, 1.0, 0.1, 0.1, 0.3,
    1.0, 1.0, 0.1, 0.1, 0.3,
    -1.0, -1.0, 0.1, 0.1, 0.3,
    1.0, -1.0, 0.1, 0.1, 0.3,
];

const VS_SRC: &'static [u8] = b"
#version 430 core
precision mediump float;
in vec2 position;
in vec3 color;
layout(location=0) uniform mat3 trans;
out vec3 v_color;
void main() {
    gl_Position = vec4((trans*vec3(position, 1.0)).xy,0.0, 1.0);
    v_color = color;
}
\0";

const FS_SRC: &'static [u8] = b"
#version 430 core
precision mediump float;
in vec3 v_color;
void main() {
    gl_FragColor = vec4(v_color, 0.1);
}
\0";

pub use self::context_tracker::{ContextCurrentWrapper, ContextId, ContextTracker, ContextWrapper};
use self::gl::types::GLfloat;

#[allow(dead_code)] // Not used by all examples
mod context_tracker {
    use glutin::{
        self, Context, ContextCurrentState, ContextError, NotCurrent, PossiblyCurrent,
        WindowedContext,
    };
    use takeable_option::Takeable;

    pub enum ContextWrapper<T: ContextCurrentState> {
        Headless(Context<T>),
        Windowed(WindowedContext<T>),
    }

    impl<T: ContextCurrentState> ContextWrapper<T> {
        pub fn headless(&mut self) -> &mut Context<T> {
            match self {
                ContextWrapper::Headless(ref mut ctx) => ctx,
                _ => panic!(),
            }
        }

        pub fn windowed(&mut self) -> &mut WindowedContext<T> {
            match self {
                ContextWrapper::Windowed(ref mut ctx) => ctx,
                _ => panic!(),
            }
        }

        fn map<T2: ContextCurrentState, FH, FW>(
            self,
            fh: FH,
            fw: FW,
        ) -> Result<ContextWrapper<T2>, (Self, ContextError)>
        where
            FH: FnOnce(Context<T>) -> Result<Context<T2>, (Context<T>, ContextError)>,
            FW: FnOnce(
                WindowedContext<T>,
            )
                -> Result<WindowedContext<T2>, (WindowedContext<T>, ContextError)>,
        {
            match self {
                ContextWrapper::Headless(ctx) => match fh(ctx) {
                    Ok(ctx) => Ok(ContextWrapper::Headless(ctx)),
                    Err((ctx, err)) => Err((ContextWrapper::Headless(ctx), err)),
                },
                ContextWrapper::Windowed(ctx) => match fw(ctx) {
                    Ok(ctx) => Ok(ContextWrapper::Windowed(ctx)),
                    Err((ctx, err)) => Err((ContextWrapper::Windowed(ctx), err)),
                },
            }
        }
    }

    pub enum ContextCurrentWrapper {
        PossiblyCurrent(ContextWrapper<PossiblyCurrent>),
        NotCurrent(ContextWrapper<NotCurrent>),
    }

    impl ContextCurrentWrapper {
        fn map_possibly<F>(self, f: F) -> Result<Self, (Self, ContextError)>
        where
            F: FnOnce(
                ContextWrapper<PossiblyCurrent>,
            ) -> Result<
                ContextWrapper<NotCurrent>,
                (ContextWrapper<PossiblyCurrent>, ContextError),
            >,
        {
            match self {
                ret @ ContextCurrentWrapper::NotCurrent(_) => Ok(ret),
                ContextCurrentWrapper::PossiblyCurrent(ctx) => match f(ctx) {
                    Ok(ctx) => Ok(ContextCurrentWrapper::NotCurrent(ctx)),
                    Err((ctx, err)) => Err((ContextCurrentWrapper::PossiblyCurrent(ctx), err)),
                },
            }
        }

        fn map_not<F>(self, f: F) -> Result<Self, (Self, ContextError)>
        where
            F: FnOnce(
                ContextWrapper<NotCurrent>,
            ) -> Result<
                ContextWrapper<PossiblyCurrent>,
                (ContextWrapper<NotCurrent>, ContextError),
            >,
        {
            match self {
                ret @ ContextCurrentWrapper::PossiblyCurrent(_) => Ok(ret),
                ContextCurrentWrapper::NotCurrent(ctx) => match f(ctx) {
                    Ok(ctx) => Ok(ContextCurrentWrapper::PossiblyCurrent(ctx)),
                    Err((ctx, err)) => Err((ContextCurrentWrapper::NotCurrent(ctx), err)),
                },
            }
        }
    }

    pub type ContextId = usize;
    #[derive(Default)]
    pub struct ContextTracker {
        current: Option<ContextId>,
        others: Vec<(ContextId, Takeable<ContextCurrentWrapper>)>,
        next_id: ContextId,
    }

    impl ContextTracker {
        pub fn insert(&mut self, ctx: ContextCurrentWrapper) -> ContextId {
            let id = self.next_id;
            self.next_id += 1;

            if let ContextCurrentWrapper::PossiblyCurrent(_) = ctx {
                if let Some(old_current) = self.current {
                    unsafe {
                        self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                )
                            })
                        })
                        .unwrap()
                    }
                }
                self.current = Some(id);
            }

            self.others.push((id, Takeable::new(ctx)));
            id
        }

        pub fn remove(&mut self, id: ContextId) -> ContextCurrentWrapper {
            if Some(id) == self.current {
                self.current.take();
            }

            let this_index = self
                .others
                .binary_search_by(|(sid, _)| sid.cmp(&id))
                .unwrap();
            Takeable::take(&mut self.others.remove(this_index).1)
        }

        fn modify<F>(&mut self, id: ContextId, f: F) -> Result<(), ContextError>
        where
            F: FnOnce(
                ContextCurrentWrapper,
            )
                -> Result<ContextCurrentWrapper, (ContextCurrentWrapper, ContextError)>,
        {
            let this_index = self
                .others
                .binary_search_by(|(sid, _)| sid.cmp(&id))
                .unwrap();

            let this_context = Takeable::take(&mut self.others[this_index].1);

            match f(this_context) {
                Err((ctx, err)) => {
                    self.others[this_index].1 = Takeable::new(ctx);
                    Err(err)
                }
                Ok(ctx) => {
                    self.others[this_index].1 = Takeable::new(ctx);
                    Ok(())
                }
            }
        }

        pub fn get_current(
            &mut self,
            id: ContextId,
        ) -> Result<&mut ContextWrapper<PossiblyCurrent>, ContextError> {
            unsafe {
                let this_index = self
                    .others
                    .binary_search_by(|(sid, _)| sid.cmp(&id))
                    .unwrap();
                if Some(id) != self.current {
                    let old_current = self.current.take();

                    if let Err(err) = self.modify(id, |ctx| {
                        ctx.map_not(|ctx| {
                            ctx.map(|ctx| ctx.make_current(), |ctx| ctx.make_current())
                        })
                    }) {
                        // Oh noes, something went wrong
                        // Let's at least make sure that no context is current.
                        if let Some(old_current) = old_current {
                            if let Err(err2) = self.modify(old_current, |ctx| {
                                ctx.map_possibly(|ctx| {
                                    ctx.map(
                                        |ctx| ctx.make_not_current(),
                                        |ctx| ctx.make_not_current(),
                                    )
                                })
                            }) {
                                panic!(
                                    "Could not `make_current` nor `make_not_current`, {:?}, {:?}",
                                    err, err2
                                );
                            }
                        }

                        if let Err(err2) = self.modify(id, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(|ctx| ctx.make_not_current(), |ctx| ctx.make_not_current())
                            })
                        }) {
                            panic!(
                                "Could not `make_current` nor `make_not_current`, {:?}, {:?}",
                                err, err2
                            );
                        }

                        return Err(err);
                    }

                    self.current = Some(id);

                    if let Some(old_current) = old_current {
                        self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                )
                            })
                        })
                        .unwrap();
                    }
                }

                match *self.others[this_index].1 {
                    ContextCurrentWrapper::PossiblyCurrent(ref mut ctx) => Ok(ctx),
                    ContextCurrentWrapper::NotCurrent(_) => panic!(),
                }
            }
        }
    }
}
