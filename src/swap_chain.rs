use crate::{Stored, WeaklyStored,
    DeviceId, SwapChainId, TextureId, TextureViewId,
};
use crate::{conv, resource};
use crate::registry::{HUB, Items};
use crate::track::{Tracktion, TrackPermit};

use hal;
use hal::{Device as _Device, Swapchain as _Swapchain};
use log::trace;

use std::{iter, mem};


pub type SwapImageEpoch = u16;

pub(crate) struct SwapChainLink<E> {
    pub swap_chain_id: WeaklyStored<SwapChainId>, //TODO: strongly
    pub epoch: E,
    pub image_index: hal::SwapImageIndex,
}

pub(crate) struct Surface<B: hal::Backend> {
    pub raw: B::Surface,
}

pub(crate) struct Frame<B: hal::Backend> {
    pub texture_id: Stored<TextureId>,
    pub view_id: Stored<TextureViewId>,
    pub fence: B::Fence,
    pub sem_available: B::Semaphore,
    pub sem_present: B::Semaphore,
    pub comb: hal::command::CommandBuffer<B, hal::General, hal::command::MultiShot>,
}

//TODO: does it need a ref-counted lifetime?

pub(crate) struct SwapChain<B: hal::Backend> {
    pub raw: B::Swapchain,
    pub device_id: Stored<DeviceId>,
    pub frames: Vec<Frame<B>>,
    pub acquired: Vec<hal::SwapImageIndex>,
    pub sem_available: B::Semaphore,
    pub command_pool: hal::CommandPool<B, hal::General>,
}

#[repr(C)]
pub struct SwapChainDescriptor {
    pub usage: resource::TextureUsageFlags,
    pub format: resource::TextureFormat,
    pub width: u32,
    pub height: u32,
}

#[no_mangle]
pub extern "C" fn wgpu_swap_chain_get_next_texture(
    swap_chain_id: SwapChainId,
) -> TextureId {
    let mut swap_chain_guard = HUB.swap_chains.write();
    let swap_chain = swap_chain_guard.get_mut(swap_chain_id);
    let device_guard = HUB.devices.read();
    let device = device_guard.get(swap_chain.device_id.value);

    let image_index = unsafe {
        let sync = hal::FrameSync::Semaphore(&swap_chain.sem_available);
        swap_chain.raw.acquire_image(!0, sync).unwrap()
    };

    swap_chain.acquired.push(image_index);
    let frame = &mut swap_chain.frames[image_index as usize];
    unsafe {
        device.raw.wait_for_fence(&frame.fence, !0).unwrap();
    }

    mem::swap(&mut frame.sem_available, &mut swap_chain.sem_available);

    let texture_guard = HUB.textures.read();
    let texture = texture_guard.get(frame.texture_id.value);
    match texture.swap_chain_link {
        Some(ref link) => *link.epoch.lock() += 1,
        None => unreachable!(),
    }

    frame.texture_id.value
}

#[no_mangle]
pub extern "C" fn wgpu_swap_chain_present(
    swap_chain_id: SwapChainId,
) {
    let mut swap_chain_guard = HUB.swap_chains.write();
    let swap_chain = swap_chain_guard.get_mut(swap_chain_id);
    let mut device_guard = HUB.devices.write();
    let device = device_guard.get_mut(swap_chain.device_id.value);

    let image_index = swap_chain.acquired.remove(0);
    let frame = &mut swap_chain.frames[image_index as usize];

    let texture_guard = HUB.textures.read();
    let texture = texture_guard.get(frame.texture_id.value);
    match texture.swap_chain_link {
        Some(ref link) => *link.epoch.lock() += 1,
        None => unreachable!(),
    }

    trace!("transit {:?} to present", frame.texture_id.value);
    let tracktion = device.texture_tracker
        .lock()
        .transit(
            frame.texture_id.value,
            &texture.life_guard.ref_count,
            resource::TextureUsageFlags::PRESENT,
            TrackPermit::EXTEND,
        )
        .unwrap();

    let barrier = match tracktion {
        Tracktion::Keep => None,
        Tracktion::Replace { old } => Some(hal::memory::Barrier::Image {
            states: conv::map_texture_state(old, hal::format::Aspects::COLOR) ..
                (hal::image::Access::empty(), hal::image::Layout::Present),
            target: &texture.raw,
            families: None,
            range: texture.full_range.clone(),
        }),
        Tracktion::Init |
        Tracktion::Extend {..} => unreachable!(),
    };

    unsafe {
        frame.comb.begin(false);
        frame.comb.pipeline_barrier(
            hal::pso::PipelineStage::TOP_OF_PIPE .. hal::pso::PipelineStage::BOTTOM_OF_PIPE,
            hal::memory::Dependencies::empty(),
            barrier,
        );
        frame.comb.finish();

        // now prepare the GPU submission
        device.raw.reset_fence(&frame.fence);
        device.queue_group.queues[0]
            .submit_nosemaphores(iter::once(&frame.comb), Some(&frame.fence));
    }
}