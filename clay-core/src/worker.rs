use std::path::Path;

use regex::{Regex, RegexBuilder, Captures};

use ocl;
use ocl_include;

use crate::{
    Context, Screen,
};

use lazy_static::lazy_static;


lazy_static!{
    static ref LOCATION: Regex = RegexBuilder::new(
        r#"^([^:\r\n]*):(\d*):(\d*):"#
    ).multi_line(true).build().unwrap();
}


#[allow(dead_code)]
pub struct Worker {
    kernel: ocl::Kernel,
    queue: ocl::Queue,
}

impl Worker {
    pub fn new(context: &Context) -> crate::Result<Self> {
        let queue = context.queue().clone();

        // load source
        let fs_hook = ocl_include::FsHook::new()
        .include_dir(&Path::new("../clay-core/ocl-src/"))?;
        let mem_hook = ocl_include::MemHook::new();
        let hook = ocl_include::ListHook::new()
        .add_hook(mem_hook)
        .add_hook(fs_hook);
        let node = ocl_include::build(&hook, Path::new("main.c")).unwrap();
        let (src, index) = node.collect();

        // build program
        let program = ocl::Program::builder()
        .devices(context.device())
        .source(src)
        .build(context.context())
        .map_err(|e| {
            let message = LOCATION.replace_all(&e.to_string(), |caps: &Captures| -> String {
                if &caps[1] == "<kernel>" { Ok(()) } else { Err(()) }
                .and_then(|()| caps[2].parse::<usize>().map_err(|_| ()))
                .and_then(|line| {
                    index.search(line - 1 - 1 /* workaround */).ok_or(())
                })
                .and_then(|(path, local_line)| {
                    Ok(format!(
                        "{}:{}:{}:",
                        path.to_string_lossy(),
                        local_line,
                        &caps[3],
                    ))
                })
                .unwrap_or(caps[0].to_string())
            }).into_owned();
            ocl::Error::from(ocl::core::Error::from(message))
        })?;

        // build kernel
        let kernel = ocl::Kernel::builder()
        .program(&program)
        .name("fill")
        .queue(queue.clone())
        .arg(&0i32)
        .arg(&0i32)
        .arg(None::<&ocl::Buffer<u8>>)
        .build()?;

        //let objects = scene.create_buffer(&context)?;

        Ok(Self { kernel, queue })
    }

    pub fn render(&self, screen: &mut Screen) -> crate::Result<()> {
        self.kernel.set_arg(0, &(screen.dims().0 as i32))?;
        self.kernel.set_arg(1, &(screen.dims().1 as i32))?;
        self.kernel.set_arg(2, screen.buffer_mut())?;

        unsafe {
            self.kernel
            .cmd()
            .global_work_size(screen.dims())
            .enq()?;
        }

        Ok(())
    }
}
