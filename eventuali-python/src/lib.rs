#![allow(non_local_definitions)]
use pyo3::prelude::*;

mod event_store;
mod event;
mod aggregate;
mod error;
mod streaming;

use event_store::PyEventStore;
use event::PyEvent;
use aggregate::PyAggregate;
use streaming::{PyEventStreamer, PyEventStreamReceiver, PySubscriptionBuilder, PyProjection};

#[pymodule]
fn _eventuali(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEventStore>()?;
    m.add_class::<PyEvent>()?;
    m.add_class::<PyAggregate>()?;
    
    // Register streaming classes
    m.add_class::<PyEventStreamer>()?;
    m.add_class::<PyEventStreamReceiver>()?;
    m.add_class::<PySubscriptionBuilder>()?;
    m.add_class::<PyProjection>()?;
    
    // Register custom exceptions
    error::register_exceptions(py, m)?;
    
    Ok(())
}