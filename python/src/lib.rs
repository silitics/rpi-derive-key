use pyo3::{prelude::*, types::PyBytes};

#[pyclass]
struct DeriverBuilder(::rpi_derive_key::DeriverBuilder);

#[pymethods]
impl DeriverBuilder {
    #[new]
    fn new() -> Self {
        Self(::rpi_derive_key::DeriverBuilder::new())
    }

    #[getter]
    fn get_use_customer_otp(&self) -> bool {
        self.0.use_customer_otp()
    }

    #[setter]
    fn set_use_customer_otp(&mut self, enable: bool) {
        self.0.set_use_customer_otp(enable)
    }

    fn build(&self) -> PyResult<Deriver> {
        match self.0.clone().build() {
            Ok(deriver) => Ok(Deriver(deriver)),
            Err(_) => todo!(),
        }
    }
}

#[pyclass]
struct Deriver(::rpi_derive_key::Deriver);

#[pymethods]
impl Deriver {
    fn derive_key<'py>(&self, py: Python<'py>, size: usize, info: &str) -> PyResult<&'py PyBytes> {
        let mut key = vec![0; size];
        match self.0.derive_key(info, &mut key) {
            Ok(_) => Ok(PyBytes::new(py, &key)),
            Err(_) => todo!(),
        }
    }
}

#[pymodule]
fn rpi_derive_key(_: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<DeriverBuilder>()?;
    m.add_class::<Deriver>()?;
    Ok(())
}
