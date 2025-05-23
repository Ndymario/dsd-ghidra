use std::{
    ffi::{c_char, CString},
    ptr::{self, NonNull},
    slice,
};

use anyhow::Result;

use crate::list::UnsafeList;

pub trait TryIntoUnsafe {
    type UnsafeType;

    /// Removes ownership of `self` by leaking its data into an FFI-compatible type. Ownership must be retrieved later so it
    /// can be dropped.
    fn try_into_unsafe(self) -> Result<Self::UnsafeType>;
}

pub trait TryIntoSafe {
    type SafeType;

    /// # Safety
    ///
    /// Takes ownership of `self` which must have been generated by [`TryIntoUnsafe::try_into_unsafe`].
    unsafe fn try_into_safe(self) -> Result<Self::SafeType>;
}

impl<T> TryIntoUnsafe for Vec<T>
where
    T: TryIntoUnsafe,
{
    type UnsafeType = UnsafeList<T::UnsafeType>;

    fn try_into_unsafe(self) -> Result<Self::UnsafeType> {
        let len = self.len() as u32;
        let boxed = self.into_iter().map(|item| item.try_into_unsafe()).collect::<Result<Box<[_]>>>()?;
        let ptr = Box::into_raw(boxed) as *mut T::UnsafeType;

        // JNA attempts to access any non-null pointer it receives from the FFI, so we must force empty Vecs to be null instead
        // of a dangling pointer.
        let ptr = if ptr == NonNull::dangling().as_ptr() { ptr::null_mut() } else { ptr };
        Ok(UnsafeList { ptr, len })
    }
}

impl<T> TryIntoSafe for UnsafeList<T>
where
    T: TryIntoSafe,
{
    type SafeType = Vec<T::SafeType>;

    unsafe fn try_into_safe(self) -> Result<Self::SafeType> {
        let ptr = if self.ptr.is_null() { NonNull::dangling().as_ptr() } else { self.ptr };
        let slice = slice::from_raw_parts_mut(ptr, self.len as usize);
        let boxed = Box::from_raw(slice);
        let vec = Vec::from(boxed).into_iter().map(|overlay| overlay.try_into_safe()).collect::<Result<Vec<_>>>()?;

        Ok(vec)
    }
}

#[repr(C)]
#[derive(Clone, Debug)]
pub struct UnsafeString(*mut c_char);

impl TryIntoUnsafe for String {
    type UnsafeType = UnsafeString;

    fn try_into_unsafe(self) -> Result<Self::UnsafeType> {
        Ok(UnsafeString(CString::new(self)?.into_raw()))
    }
}

impl TryIntoSafe for UnsafeString {
    type SafeType = String;

    unsafe fn try_into_safe(self) -> Result<Self::SafeType> {
        let cstr = CString::from_raw(self.0);
        Ok(cstr.to_str()?.to_string())
    }
}

macro_rules! into_unsafe_impl {
    ($($t:ty)*) => ($(
        impl TryIntoUnsafe for $t {
            type UnsafeType = $t;

            fn try_into_unsafe(self) -> Result<Self::UnsafeType> {
                Ok(self)
            }
        }

        impl TryIntoSafe for $t {
            type SafeType = $t;

            unsafe fn try_into_safe(self) -> Result<Self::SafeType> {
                Ok(self)
            }
        }
    )*)
}

into_unsafe_impl! { u8 u16 u32 }
