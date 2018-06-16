// Copyright 2017 GFX developers
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// http://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

extern crate cocoa;
#[macro_use]
extern crate bitflags;
extern crate libc;
#[macro_use]
extern crate log;
#[macro_use]
extern crate objc;
extern crate objc_foundation;
extern crate objc_id;
extern crate block;
#[macro_use]
extern crate foreign_types;

use std::mem;
use std::marker::PhantomData;
use std::ops::Deref;
use std::borrow::{Borrow, ToOwned};

use objc::runtime::{Object, Class, YES, NO};
use cocoa::foundation::NSSize;
use foreign_types::ForeignType;

#[cfg(target_pointer_width = "64")]
pub type CGFloat = libc::c_double;
#[cfg(not(target_pointer_width = "64"))]
pub type CGFloat = libc::c_float;

macro_rules! foreign_obj_type {
    {type CType = $raw_ident:ident;
    pub struct $owned_ident:ident;
    pub struct $ref_ident:ident;
    type ParentType = $parent_ref:ident;
    } => {
        foreign_obj_type! {
            type CType = $raw_ident;
            pub struct $owned_ident;
            pub struct $ref_ident;
        }

        impl ::std::ops::Deref for $ref_ident {
            type Target = $parent_ref;

            fn deref(&self) -> &$parent_ref {
                unsafe { &*(self as *const $ref_ident as *const $parent_ref)  }
            }
        }
    };
    {type CType = $raw_ident:ident;
    pub struct $owned_ident:ident;
    pub struct $ref_ident:ident;
    } => {
        foreign_type! {
            type CType = $raw_ident;
            fn drop = ::obj_drop;
            fn clone = ::obj_clone;
            pub struct $owned_ident;
            pub struct $ref_ident;
        }

        unsafe impl ::objc::Message for $raw_ident {
        }
        unsafe impl ::objc::Message for $ref_ident {
        }

        impl ::std::fmt::Debug for $ref_ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                unsafe {
                    use ::objc_foundation::INSString;
                    // TODO: might leak, not 100% sure...
                    let string: &::objc_foundation::NSString = msg_send![self, debugDescription];
                    write!(f, "{}", string.as_str())
                }
            }
        }

        impl ::std::fmt::Debug for $owned_ident {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                ::std::ops::Deref::deref(self).fmt(f)
            }
        }
    };
}

macro_rules! try_objc {
    {
        $err_name: ident => $body:expr
    } => {
        {
            let mut $err_name: *mut ::objc::runtime::Object = ::std::ptr::null_mut();
            let value = $body;
            if !$err_name.is_null() {
                let desc: *mut Object = msg_send![$err_name, localizedDescription];
                let compile_error: *const ::libc::c_char = msg_send![desc, UTF8String];
                let message = CStr::from_ptr(compile_error).to_string_lossy().into_owned();
                msg_send![$err_name, release];
                return Err(message);
            }
            value
        }
    };
}

macro_rules! class {
    (
        $name:ident {
            $( fn $fun:ident(&self) -> $ret:ty; )*
        }
    ) => (
        $(
            mod $fun {
                const LEN: usize = stringify!($fun).len() + 1;
                #[repr(C)]
                struct SendSyncWrap(*const [u8; LEN]);
                unsafe impl Send for SendSyncWrap {}
                unsafe impl Sync for SendSyncWrap {}

                #[link_section="__TEXT,__objc_methname,cstring_literals"]
                static VALUE : [u8; LEN] = concat!(stringify!($fun), '\0');
                #[link_section="__DATA,__objc_selrefs,literal_pointers,no_dead_strip"]
                pub static REF : SendSyncWrap = SendSyncWrap(&VALUE);
            }
        )*
        mod sel_simple {
            $(
                #[link_section="__TEXT,__objc_methname,cstring_literals"]
                pub static $fun: &'static str = concat!(stringify!($fun), '\0');
            )*
        }
        impl $name {
            $(
                pub fn $fun (&self) -> $ret {
                    use std::mem;
                    use objc;
                    unsafe {
                        let sel = mem::transmute(sel_simple::$fun.as_ptr());
                        objc::__send_message(self, sel, ())
                    }.expect("ObjC message send failure")
                }
            )*
        }
    );
    (
        $name:ident {
            $( fn $fun:ident(&self $(, $param:ident : $ty:ty)+) -> $ret:ty; )*
        }
    ) => (
        mod sel_complex {
            $(
                #[link_section="__TEXT,__objc_methname,cstring_literals"]
                pub static $fun: &'static str = concat!($( stringify!($param), ':' ),+, '\0');
            )*
        }
        impl $name {
            $(
                pub fn $fun (&self $(, $param: $ty)+ ) -> $ret {
                    use std::mem;
                    use objc;
                    unsafe {
                        let sel = mem::transmute(sel_complex::$fun.as_ptr());
                        objc::__send_message(self, sel $(,$param)+)
                    }.expect("ObjC message send failure")
                }
            )*
        }
    );
}


pub struct NSArray<T> {
    _phantom: PhantomData<T>,
}

pub struct Array<T>(*mut NSArray<T>) where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static;
pub struct ArrayRef<T>(foreign_types::Opaque, PhantomData<T>) where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static;

impl<T> Drop for Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    fn drop(&mut self) {
        unsafe {
            msg_send![self.0, release];
        }
    }
}

impl<T> Clone for Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    fn clone(&self) -> Self {
        unsafe {
            Array(msg_send![self.0, retain])
        }
    }
}

unsafe impl<T> objc::Message for NSArray<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{}
unsafe impl<T> objc::Message for ArrayRef<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{}

impl<T> Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
 {
    pub fn from_slice(s: &[&T::Ref]) -> Self {
        unsafe {
            let class = Class::get("NSArray").unwrap();
            msg_send![class, arrayWithObjects: s.as_ptr() count: s.len()]
        }
    }
    
    pub fn from_owned_slice(s: &[T]) -> Self {
        unsafe {
            let class = Class::get("NSArray").unwrap();
            msg_send![class, arrayWithObjects: s.as_ptr() count: s.len()]
        }
    }
}

impl<T> foreign_types::ForeignType for Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    type CType = NSArray<T>;
    type Ref = ArrayRef<T>;

    unsafe fn from_ptr(p: *mut NSArray<T>) -> Self {
        Array(p)
    }

    fn as_ptr(&self) -> *mut NSArray<T> {
        self.0
    }
}

impl<T> foreign_types::ForeignTypeRef for ArrayRef<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    type CType = NSArray<T>;
}

impl<T> Deref for Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    type Target = ArrayRef<T>;

    fn deref(&self) -> &ArrayRef<T> {
        unsafe { mem::transmute(self.as_ptr()) }
    }
}

impl<T> Borrow<ArrayRef<T>> for Array<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    fn borrow(&self) -> &ArrayRef<T> {
        unsafe { mem::transmute(self.as_ptr()) }
    }
}

impl<T> ToOwned for ArrayRef<T> where
    T: ForeignType + 'static,
    T::Ref: objc::Message + 'static,
{
    type Owned = Array<T>;

    fn to_owned(&self) -> Array<T> {
        unsafe { Array::from_ptr(msg_send![self, retain]) }
    }
}

pub enum CAMetalDrawable {}

foreign_obj_type! {
    type CType = CAMetalDrawable;
    pub struct CoreAnimationDrawable;
    pub struct CoreAnimationDrawableRef;
    type ParentType = DrawableRef;
}

impl CoreAnimationDrawableRef {
    pub fn texture(&self) -> &TextureRef {
        unsafe {
            msg_send![self, texture]
        }
    }
}

pub enum CAMetalLayer {}

foreign_obj_type! {
    type CType = CAMetalLayer;
    pub struct CoreAnimationLayer;
    pub struct CoreAnimationLayerRef;
}

impl CoreAnimationLayer {
    pub fn new() -> Self {
        unsafe {
            let class = Class::get("CAMetalLayer").unwrap();
            msg_send![class, new]
        }
    }
}

impl CoreAnimationLayerRef {
    pub fn set_device(&self, device: &DeviceRef) {
        unsafe {
            msg_send![self, setDevice:device]
        }
    }

    pub fn pixel_format(&self) -> MTLPixelFormat {
        unsafe {
            msg_send![self, pixelFormat]
        }
    }

    pub fn set_pixel_format(&self, pixel_format: MTLPixelFormat) {
        unsafe {
            msg_send![self, setPixelFormat:pixel_format]
        }
    }

    pub fn drawable_size(&self) -> NSSize {
        unsafe {
            msg_send![self, drawableSize]
        }
    }

    pub fn set_drawable_size(&self, size: NSSize) {
        unsafe {
            msg_send![self, setDrawableSize:size]
        }
    }

    pub fn presents_with_transaction(&self) -> bool {
        unsafe {
            match msg_send![self, presentsWithTransaction] {
                YES => true,
                NO => false,
                _ => unreachable!()
            }
        }
    }

    pub fn set_presents_with_transaction(&self, transaction: bool) {
        unsafe {
            msg_send![self, setPresentsWithTransaction:transaction];
        }
    }

    pub fn set_edge_antialiasing_mask(&self, mask: u64) {
        unsafe {
            msg_send![self, setEdgeAntialiasingMask:mask]
        }
    }

    pub fn set_masks_to_bounds(&self, masks: bool) {
        unsafe {
            msg_send![self, setMasksToBounds:masks]
        }
    }

    pub fn remove_all_animations(&self) {
        unsafe {
            msg_send![self, removeAllAnimations];
        }
    }

    pub fn next_drawable(&self) -> Option<&CoreAnimationDrawableRef> {
        unsafe {
            msg_send![self, nextDrawable]
        }
    }

    pub fn set_contents_scale(&self, scale: CGFloat) {
        unsafe {
            msg_send![self, setContentsScale:scale];
        }
    }
}

mod constants;
mod types;
mod device;
mod texture;
mod sampler;
mod resource;
mod drawable;
mod buffer;
mod renderpass;
mod commandqueue;
mod commandbuffer;
mod encoder;
mod pipeline;
mod library;
mod argument;
mod vertexdescriptor;
mod depthstencil;
mod heap;
mod capturemanager;

pub use constants::*;
pub use types::*;
pub use device::*;
pub use texture::*;
pub use sampler::*;
pub use resource::*;
pub use drawable::*;
pub use buffer::*;
pub use renderpass::*;
pub use commandqueue::*;
pub use commandbuffer::*;
pub use encoder::*;
pub use pipeline::*;
pub use library::*;
pub use argument::*;
pub use vertexdescriptor::*;
pub use depthstencil::*;
pub use heap::*;
pub use capturemanager::*;

#[inline]
unsafe fn obj_drop<T>(p: *mut T) {
    msg_send![(p as *mut Object), release];
}

#[inline]
unsafe fn obj_clone<T: 'static>(p: *mut T) -> *mut T {
    msg_send![(p as *mut Object), retain]
}
