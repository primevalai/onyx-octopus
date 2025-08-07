use pyo3::prelude::*;
use eventuali_core::{Aggregate as CoreAggregate};

#[pyclass]
#[derive(Clone)]
pub struct PyAggregate {
    pub inner: CoreAggregate,
}

#[pymethods]
impl PyAggregate {
    #[new]
    #[pyo3(signature = (id, aggregate_type))]
    pub fn new(id: String, aggregate_type: String) -> Self {
        let aggregate = CoreAggregate::new(id, aggregate_type);
        PyAggregate { inner: aggregate }
    }

    #[staticmethod]
    #[pyo3(signature = (aggregate_type))]
    pub fn new_with_uuid(aggregate_type: String) -> Self {
        let aggregate = CoreAggregate::new_with_uuid(aggregate_type);
        PyAggregate { inner: aggregate }
    }

    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    #[getter]
    pub fn version(&self) -> i64 {
        self.inner.version
    }

    #[getter]
    pub fn aggregate_type(&self) -> String {
        self.inner.aggregate_type.clone()
    }

    pub fn increment_version(&mut self) {
        self.inner.increment_version();
    }

    pub fn is_new(&self) -> bool {
        self.inner.is_new()
    }
}