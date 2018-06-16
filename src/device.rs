// Copyright 2017 GFX developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use cocoa::base::id;
use cocoa::foundation::{NSUInteger};
use objc::runtime::{Object, BOOL, YES, NO};
use objc_foundation::{NSString, INSString};

use super::*;

use libc;

use block::{Block, ConcreteBlock};

use std::ffi::CStr;
use std::path::Path;
use std::ptr;

use foreign_types::ForeignType;


#[allow(non_camel_case_types)]
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum MTLFeatureSet {
    iOS_GPUFamily1_v1 = 0,
    iOS_GPUFamily2_v1 = 1,
    iOS_GPUFamily1_v2 = 2,
    iOS_GPUFamily2_v2 = 3,
    iOS_GPUFamily3_v1 = 4,
    iOS_GPUFamily1_v3 = 5,
    iOS_GPUFamily2_v3 = 6,
    iOS_GPUFamily3_v2 = 7,
    iOS_GPUFamily1_v4 = 8,
    iOS_GPUFamily2_v4 = 9,
    iOS_GPUFamily3_v3 = 10,
    iOS_GPUFamily4_v1 = 11,
    tvOS_GPUFamily1_v1 = 30000,
    tvOS_GPUFamily1_v2 = 30001,
    tvOS_GPUFamily1_v3 = 30002,
    tvOS_GPUFamily2_v1 = 30003,
    macOS_GPUFamily1_v1 = 10000,
    macOS_GPUFamily1_v2 = 10001,
    macOS_ReadWriteTextureTier2 = 10002,
    macOS_GPUFamily1_v3 = 10003,
    macOS_GPUFamily1_v4 = 10004,
    macOS_GPUFamily2_v1 = 10005,
}

#[allow(non_camel_case_types)]
#[repr(u64)]
#[derive(Copy, Clone, Debug)]
pub enum MTLArgumentBuffersTier {
    tier1 = 0,
    tier2 = 1,
}

bitflags! {
    struct MTLPipelineOption: NSUInteger {
        const ArgumentInfo   = 1 << 0;
        const BufferTypeInfo = 1 << 1;
    }
}

#[link(name = "Metal", kind = "framework")]
extern {
    fn MTLCreateSystemDefaultDevice() -> *mut MTLDevice;
}

#[allow(non_camel_case_types)]
type dispatch_data_t = id;
#[allow(non_camel_case_types)]
type dispatch_queue_t = id;
#[allow(non_camel_case_types)]
type dispatch_block_t = *const Block<(), ()>;

#[cfg_attr(any(target_os = "macos", target_os = "ios"),
           link(name = "System", kind = "dylib"))]
#[cfg_attr(not(any(target_os = "macos", target_os = "ios")),
           link(name = "dispatch", kind = "dylib"))]
#[allow(improper_ctypes)]
extern {
    static _dispatch_main_q: dispatch_queue_t;

    fn dispatch_data_create(
        buffer: *const libc::c_void,
        size: libc::size_t,
        queue: dispatch_queue_t,
        destructor: dispatch_block_t
    ) -> dispatch_data_t;
}

/*type MTLNewLibraryCompletionHandler = extern fn(library: id, error: id);
type MTLNewRenderPipelineStateCompletionHandler = extern fn(renderPipelineState: id, error: id);
type MTLNewRenderPipelineStateWithReflectionCompletionHandler = extern fn(renderPipelineState: id, reflection: id, error: id);
type MTLNewComputePipelineStateCompletionHandler = extern fn(computePipelineState: id, error: id);
type MTLNewComputePipelineStateWithReflectionCompletionHandler = extern fn(computePipelineState: id, reflection: id, error: id);*/

pub enum MTLDevice {}

foreign_obj_type! {
    type CType = MTLDevice;
    pub struct Device;
    pub struct DeviceRef;
}

impl Device {
    pub fn system_default() -> Device {
        unsafe { Device(MTLCreateSystemDefaultDevice()) }
    }
}

/*class!(DeviceRef {
    fn name(&self) -> &NSString;
});*/

#[no_mangle]
#[link_section="__DATA,__objc_imageinfo,regular,no_dead_strip"]
static info_version: u32 = 0;
#[no_mangle]
#[link_section="__DATA,__objc_imageinfo,regular,no_dead_strip"]
static info_flags: u32 = 64;

impl DeviceRef {
    pub fn name2(&self) -> &str {
        struct Foo(*const [u8; 5]);
        unsafe impl Send for Foo {}
        unsafe impl Sync for Foo {}

        let s: &NSString = {
            #[no_mangle]
            #[link_section="__TEXT,__objc_methname,cstring_literals"]
            static OBJC_METHOD_VAR_NAME_ : [u8; 5] = *b"name\0";
            #[no_mangle]
            #[link_section="__DATA,__objc_selrefs,literal_pointers,no_dead_strip"]
            static OBJ_SELECTOR_REFERENCES_: Foo = Foo(&OBJC_METHOD_VAR_NAME_);
            unsafe {
                let selector: objc::runtime::Sel = mem::transmute(OBJ_SELECTOR_REFERENCES_.0);
                objc::__send_message(self, selector, ()).unwrap()
            }
        };

        s.as_str()
    }
    pub fn name(&self) -> &str {
        unsafe {
            let name: &NSString = msg_send![self, name];
            name.as_str()
        }
    }

    pub fn vendor(&self) -> &str {
        unsafe {
            let name: &NSString = msg_send![self, vendorName];
            name.as_str()
        }
    }

    pub fn family_name(&self) -> &str {
        unsafe {
            let name: &NSString = msg_send![self, familyName];
            name.as_str()
        }
    }

    pub fn registry_id(&self) -> u64 {
        unsafe {
            msg_send![self, registryID]
        }
    }

    pub fn max_threads_per_threadgroup(&self) -> MTLSize {
        unsafe {
            msg_send![self, maxThreadsPerThreadgroup]
        }
    }

    pub fn is_low_power(&self) -> bool {
        unsafe {
            match msg_send![self, isLowPower] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn is_headless(&self) -> bool {
        unsafe {
            match msg_send![self, isHeadless] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn supports_feature_set(&self, feature: MTLFeatureSet) -> bool {
        unsafe {
            match msg_send![self, supportsFeatureSet:feature] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn supports_sample_count(&self, count: NSUInteger) -> bool {
        unsafe {
            match msg_send![self, supportsTextureSampleCount:count] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn d24_s8_supported(&self) -> bool {
        unsafe {
            match msg_send![self, isDepth24Stencil8PixelFormatSupported] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn new_command_queue(&self) -> CommandQueue {
        unsafe {
            msg_send![self, newCommandQueue]
        }
    }

    pub fn new_command_queue_with_max_command_buffer_count(&self, count: NSUInteger) -> CommandQueue {
        unsafe {
            msg_send![self, newCommandQueueWithMaxCommandBufferCount:count]
        }
    }

    pub fn new_default_library(&self) -> Library {
        unsafe {
            msg_send![self, newDefaultLibrary]
        }
    }

    pub fn new_library_with_source(&self, src: &str, options: &CompileOptionsRef) -> Result<Library, String> {
        use cocoa::foundation::NSString as cocoa_NSString;
        use cocoa::base::nil as cocoa_nil;

        unsafe {
            let source = cocoa_NSString::alloc(cocoa_nil).init_str(src);
            let mut err: *mut Object = ptr::null_mut();
            let library: *mut MTLLibrary = msg_send![self, newLibraryWithSource:source
                                                                        options:options
                                                                          error:&mut err];
            if !err.is_null() {
                let desc: *mut Object = msg_send![err, localizedDescription];
                let compile_error: *const libc::c_char = msg_send![desc, UTF8String];
                let message = CStr::from_ptr(compile_error).to_string_lossy().into_owned();
                msg_send![err, release];
                if library.is_null() {
                    return Err(message);
                } else {
                    warn!("Shader warnings: {}", message);
                }
            }

            assert!(!library.is_null());
            Ok(Library::from_ptr(library))
        }
    }

    pub fn new_library_with_file<P: AsRef<Path>>(&self, file: P) -> Result<Library, String> {
        use cocoa::foundation::NSString as cocoa_NSString;
        use cocoa::base::nil as cocoa_nil;

        unsafe {
            let filename = cocoa_NSString::alloc(cocoa_nil)
                .init_str(file.as_ref().to_string_lossy().as_ref());

            let library: *mut MTLLibrary = try_objc!{ err =>
                msg_send![self, newLibraryWithFile:filename
                                             error:&mut err]
            };

            Ok(Library::from_ptr(library))
        }
    }

    pub fn new_library_with_data(&self, library_data: &[u8]) -> Result<Library, String> {
        unsafe {
            let destructor_block = ConcreteBlock::new(|| {}).copy();
            let data = dispatch_data_create(
                library_data.as_ptr() as *const libc::c_void,
                library_data.len() as libc::size_t,
                &_dispatch_main_q as *const _ as dispatch_queue_t,
                &*destructor_block.deref()
            );

            let library: *mut MTLLibrary = try_objc! { err =>
                 msg_send![self, newLibraryWithData:data
                                              error:&mut err]
            };

            Ok(Library::from_ptr(library))
        }
    }

    pub fn new_render_pipeline_state_with_reflection(&self, descriptor: &RenderPipelineDescriptorRef, reflection: &RenderPipelineReflectionRef) -> Result<RenderPipelineState, String> {
        unsafe {
            let reflection_options = MTLPipelineOption::ArgumentInfo | MTLPipelineOption::BufferTypeInfo;

            let pipeline_state: *mut MTLRenderPipelineState = try_objc!{ err =>
                msg_send![self, newRenderPipelineStateWithDescriptor:descriptor
                                                             options:reflection_options
                                                          reflection:reflection
                                                               error:&mut err]
            };

            Ok(RenderPipelineState::from_ptr(pipeline_state))
        }
    }

    pub fn new_render_pipeline_state(&self, descriptor: &RenderPipelineDescriptorRef) -> Result<RenderPipelineState, String> {
        unsafe {
            let pipeline_state: *mut MTLRenderPipelineState = try_objc!{ err =>
                msg_send![self, newRenderPipelineStateWithDescriptor:descriptor
                                                               error:&mut err]
            };

            Ok(RenderPipelineState::from_ptr(pipeline_state))
        }
    }

    pub fn new_compute_pipeline_state(&self, descriptor: &ComputePipelineDescriptorRef) -> Result<ComputePipelineState, String> {
        unsafe {
            let pipeline_state: *mut MTLComputePipelineState = try_objc!{ err =>
                msg_send![self, newComputePipelineStateWithDescriptor:descriptor
                                                               error:&mut err]
            };

            Ok(ComputePipelineState::from_ptr(pipeline_state))
        }
    }

    pub fn new_buffer(&self, length: u64, options: MTLResourceOptions) -> Buffer {
        unsafe {
            msg_send![self, newBufferWithLength:length
                                        options:options]
        }
    }

    pub fn new_buffer_with_data(&self, bytes: *const libc::c_void, length: NSUInteger, options: MTLResourceOptions) -> Buffer {
        unsafe {
            msg_send![self, newBufferWithBytes:bytes
                                        length:length
                                       options:options]
        }
    }

    pub fn new_texture(&self, descriptor: &TextureDescriptorRef) -> Texture {
        unsafe {
            msg_send![self, newTextureWithDescriptor:descriptor]
        }
    }

    pub fn new_sampler(&self, descriptor: &SamplerDescriptorRef) -> SamplerState {
        unsafe {
            msg_send![self, newSamplerStateWithDescriptor:descriptor]
        }
    }

    pub fn new_depth_stencil_state(&self, descriptor: &DepthStencilDescriptorRef) -> DepthStencilState {
        unsafe {
            msg_send![self, newDepthStencilStateWithDescriptor:descriptor]
        }
    }

    pub fn argument_buffers_support(&self) -> Option<MTLArgumentBuffersTier> {
        unsafe {
            let has_arg_buffers: BOOL = msg_send![self, respondsToSelector: sel!(argumentBuffersSupport)];
            if has_arg_buffers == YES {
                Some(msg_send![self, argumentBuffersSupport])
            } else {
                None
            }
        }
    }

    pub fn new_argument_encoder(&self, arguments: &ArrayRef<ArgumentDescriptor>) -> ArgumentEncoder {
        unsafe {
            msg_send![self, newArgumentEncoderWithArguments:arguments]
        }
    }

    pub fn new_heap(&self, descriptor: &HeapDescriptorRef) -> Heap {
        unsafe {
            msg_send![self, newHeapWithDescriptor: descriptor]
        }
    }

    pub fn heap_buffer_size_and_align(&self, length: NSUInteger, options: MTLResourceOptions) -> MTLSizeAndAlign {
        unsafe {
            msg_send![self, heapBufferSizeAndAlignWithLength: length options: options]
        }
    }

    pub fn heap_texture_size_and_align(&self, descriptor: &TextureDescriptorRef) -> MTLSizeAndAlign {
        unsafe {
            msg_send![self, heapTextureSizeAndAlignWithDescriptor: descriptor]
        }
    }
}
