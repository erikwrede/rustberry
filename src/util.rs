//! Internal utilities (kept for future use; nothing public yet).

#![allow(dead_code)]

use pyo3::ffi::Py_ssize_t;
use pyo3::prelude::*;
use pyo3::types::{PyAnyMethods, PyTuple};
use pyo3::{ffi, Py};

/// Convenience alias for an owned reference to any Python object.
type PyObject = Py<PyAny>;

/// Allocate a [`PyTuple`] from an [`ExactSizeIterator`] of owned objects
/// without going through `PyTuple::new`'s temporary `Vec`.
///
/// Currently unused but kept around for hot paths that need to bulk-build
/// tuples bypassing the high-level API.
#[inline]
pub fn new_from_iter<'py>(
    py: Python<'py>,
    elements: &mut dyn ExactSizeIterator<Item = PyObject>,
) -> Bound<'py, PyTuple> {
    unsafe {
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyTuple_New(len);
        let tup: Bound<'py, PyTuple> = Bound::from_owned_ptr(py, ptr).cast_into_unchecked();

        let mut counter: Py_ssize_t = 0;
        for obj in elements.take(len as usize) {
            ffi::PyTuple_SetItem(ptr, counter, obj.into_ptr());
            counter += 1;
        }

        assert!(
            elements.next().is_none(),
            "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation."
        );
        assert_eq!(
            len, counter,
            "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation."
        );

        tup
    }
}
