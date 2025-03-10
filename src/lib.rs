#![feature(fn_traits)]
#![feature(unboxed_closures)]
#![feature(tuple_trait)]

pub mod array;
pub mod async_tokio;
pub mod closure;
pub mod ctor;
pub mod io;
pub mod option;
pub mod string;

pub use lean4_macro::Lean4;
pub use lean4_macro::Lean4Inductive;

pub use lean4_sys;
use lean4_sys::{
    b_lean_obj_arg, lean_alloc_external, lean_external_class, lean_get_external_data, lean_object,
    lean_register_external_class,
};

#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Lean4Obj(pub *mut lean_object);

unsafe impl Send for Lean4Obj {}
unsafe impl Sync for Lean4Obj {}

impl From<*mut lean_object> for Lean4Obj {
    fn from(ptr: *mut lean_object) -> Self {
        Self(ptr)
    }
}

impl From<Lean4Obj> for *mut lean_object {
    fn from(obj: Lean4Obj) -> Self {
        obj.0
    }
}

/// you must have #[repr(transparent)] or #[repr(C)] on your struct
/// and #[no_mangle] on your static mut lean4_object_*
/// or else you will get a **segfault**
pub trait Lean4Object
where
    Self: Sized,
{
    /// remember to inline this function
    fn get_registed_class() -> &'static mut *mut lean_external_class;

    /// lean4 will call this after its internal RC is dropped
    unsafe extern "C" fn finalize(s: *mut std::ffi::c_void) {
        let bx = Box::from_raw(s as *mut Self);
        drop(bx);
    }
    /// lean4 will call this when contains nested lean objects
    unsafe extern "C" fn foreach(_: *mut std::ffi::c_void, _: b_lean_obj_arg) {}

    #[inline(always)]
    fn into_lean_object_ptr(self) -> Lean4Obj {
        unsafe {
            if Self::get_registed_class().is_null() {
                *Self::get_registed_class() =
                    lean_register_external_class(Some(Self::finalize), Some(Self::foreach))
            }
            let boxed_self = Box::new(self);
            let leaked = Box::leak(boxed_self);
            let leaked_ptr_c_void = leaked as *mut _ as *mut std::ffi::c_void;
            Lean4Obj(lean_alloc_external(
                *Self::get_registed_class(),
                leaked_ptr_c_void,
            ))
        }
    }

    #[inline(always)]
    fn from_lean_object_ptr(ptr: Lean4Obj) -> &'static mut Self {
        unsafe {
            let s = lean_get_external_data(ptr.0) as *mut Self;
            assert!(s != std::ptr::null_mut());
            &mut *s
        }
    }
}
