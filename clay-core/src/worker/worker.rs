use std::{
    marker::PhantomData,
};
use ocl::{self, prm};
use crate::{
    Context,
    Scene, View,
    Screen,
};
use super::{Program};


#[allow(dead_code)]
pub struct Worker<S: Scene, V: View> {
    kernel: ocl::Kernel,
    queue: ocl::Queue,
    phantom: PhantomData<(S, V)>,
}

impl<S: Scene, V: View> Worker<S, V> {
    pub fn new(context: &Context) -> crate::Result<Self> {
        let queue = context.queue().clone();

        let program = Program::<S, V>::new()?.build(context)?;

        // build kernel
        let mut kb = ocl::Kernel::builder();
        kb.program(&program)
        .name("fill")
        .queue(queue.clone())
        .arg(prm::Int2::zero())
        .arg(None::<&ocl::Buffer<u8>>);

        V::args_def(&mut kb);
        S::args_def(&mut kb);

        let kernel = kb.build()?;

        Ok(Self { kernel, queue, phantom: PhantomData })
    }

    pub fn render(
        &mut self,
        screen: &mut Screen,
        scene: &S,
        view: &V,
    ) -> crate::Result<()> {
        let dims = screen.dims();
        let dims = prm::Int2::new(dims.0 as i32, dims.1 as i32);
        self.kernel.set_arg(0, &dims)?;
        self.kernel.set_arg(1, screen.buffer_mut())?;
        let mut i = 2;

        view.args_set(i, &mut self.kernel)?;
        i += V::args_count();

        scene.args_set(i, &mut self.kernel)?;
        //i += S::args_count();

        unsafe {
            self.kernel
            .cmd()
            .global_work_size(screen.dims())
            .enq()?;
        }

        Ok(())
    }
}
