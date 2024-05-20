use pyo3::{Bound, ffi, PyObject, Python};
use pyo3::ffi::Py_ssize_t;
use pyo3::types::PyTuple;

#[inline]
fn new_from_iter<'py>(
    py: Python<'py>,
    elements: &mut dyn ExactSizeIterator<Item = PyObject>,
) -> Bound<'py, PyTuple> {
    unsafe {
        // PyTuple_New checks for overflow but has a bad error message, so we check ourselves
        let len: Py_ssize_t = elements
            .len()
            .try_into()
            .expect("out of range integral type conversion attempted on `elements.len()`");

        let ptr = ffi::PyTuple_New(len);

        // - Panics if the ptr is null
        // - Cleans up the tuple if `convert` or the asserts panic
        let tup = std::mem::transmute(Bound::from_owned_ptr(py, ptr));

        let mut counter: Py_ssize_t = 0;

        for obj in elements.take(len as usize) {
            #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
            ffi::PyTuple_SET_ITEM(ptr, counter, obj.into_ptr());
            #[cfg(any(Py_LIMITED_API, PyPy, GraalPy))]
            ffi::PyTuple_SetItem(ptr, counter, obj.into_ptr());
            counter += 1;
        }

        assert!(elements.next().is_none(), "Attempted to create PyTuple but `elements` was larger than reported by its `ExactSizeIterator` implementation.");
        assert_eq!(len, counter, "Attempted to create PyTuple but `elements` was smaller than reported by its `ExactSizeIterator` implementation.");

        tup
    }
}