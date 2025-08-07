use pyo3::prelude::*;
use pyo3::types::PyDict;
use eventuali_core::{
    EventStreamer, EventStreamReceiver, Subscription,
    InMemoryEventStreamer
};
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::event::PyEvent;
use crate::error::map_rust_error_to_python;

#[pyclass]
pub struct PyEventStreamer {
    streamer: Arc<Mutex<InMemoryEventStreamer>>,
}

#[pymethods]
impl PyEventStreamer {
    #[new]
    pub fn new(capacity: Option<usize>) -> Self {
        let capacity = capacity.unwrap_or(1000);
        Self {
            streamer: Arc::new(Mutex::new(InMemoryEventStreamer::new(capacity))),
        }
    }

    #[pyo3(signature = (subscription_id, aggregate_type_filter = None, event_type_filter = None))]
    pub fn subscribe<'p>(
        &self, 
        py: Python<'p>, 
        subscription_id: String,
        aggregate_type_filter: Option<String>,
        event_type_filter: Option<String>
    ) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        let subscription = Subscription {
            id: subscription_id,
            aggregate_type_filter,
            event_type_filter,
            from_timestamp: None,
        };
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let streamer_guard = streamer.lock().await;
            let receiver = streamer_guard.subscribe(subscription)
                .await
                .map_err(map_rust_error_to_python)?;
            
            Ok(PyEventStreamReceiver { 
                receiver: Arc::new(Mutex::new(receiver)) 
            })
        })
    }

    #[pyo3(signature = (subscription_id))]
    pub fn unsubscribe<'p>(&self, py: Python<'p>, subscription_id: String) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let streamer_guard = streamer.lock().await;
            streamer_guard.unsubscribe(&subscription_id)
                .await
                .map_err(map_rust_error_to_python)?;
            Ok(())
        })
    }

    #[pyo3(signature = (stream_id))]
    pub fn get_stream_position<'p>(&self, py: Python<'p>, stream_id: String) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let streamer_guard = streamer.lock().await;
            let position = streamer_guard.get_stream_position(&stream_id)
                .await
                .map_err(map_rust_error_to_python)?;
            Ok(position)
        })
    }

    pub fn get_global_position<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let streamer_guard = streamer.lock().await;
            let position = streamer_guard.get_global_position()
                .await
                .map_err(map_rust_error_to_python)?;
            Ok(position)
        })
    }

}

impl PyEventStreamer {
    // Methods moved to pymethods block
}

#[pyclass]
pub struct PyEventStreamReceiver {
    receiver: Arc<Mutex<EventStreamReceiver>>,
}

#[pymethods]
impl PyEventStreamReceiver {
    pub fn recv<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let receiver = self.receiver.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut receiver_guard = receiver.lock().await;
            match receiver_guard.recv().await {
                Ok(stream_event) => {
                    Python::with_gil(|py| {
                        let py_dict = PyDict::new(py);
                        let py_event = PyEvent { inner: stream_event.event };
                        py_dict.set_item("event", Py::new(py, py_event)?)?;
                        py_dict.set_item("stream_position", stream_event.stream_position)?;
                        py_dict.set_item("global_position", stream_event.global_position)?;
                        Ok(py_dict.to_object(py))
                    })
                }
                Err(_) => Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "Channel closed or no more events"
                ))
            }
        })
    }
}

#[pyclass]
pub struct PySubscriptionBuilder {
    id: Option<String>,
    aggregate_type_filter: Option<String>,
    event_type_filter: Option<String>,
}

#[pymethods]
impl PySubscriptionBuilder {
    #[new]
    pub fn new() -> Self {
        Self {
            id: None,
            aggregate_type_filter: None,
            event_type_filter: None,
        }
    }

    pub fn with_id(&mut self, id: String) -> PyResult<()> {
        self.id = Some(id);
        Ok(())
    }

    pub fn filter_by_aggregate_type(&mut self, aggregate_type: String) -> PyResult<()> {
        self.aggregate_type_filter = Some(aggregate_type);
        Ok(())
    }

    pub fn filter_by_event_type(&mut self, event_type: String) -> PyResult<()> {
        self.event_type_filter = Some(event_type);
        Ok(())
    }

    pub fn build(&self) -> PyResult<(String, Option<String>, Option<String>)> {
        let id = self.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        Ok((id, self.aggregate_type_filter.clone(), self.event_type_filter.clone()))
    }
}

#[pyclass]
pub struct PyProjection {
    _handler: PyObject,
}

#[pymethods]
impl PyProjection {
    #[new]
    pub fn new(handler: PyObject) -> Self {
        Self { _handler: handler }
    }

    pub fn handle_event<'p>(&self, py: Python<'p>, _event: &PyEvent) -> PyResult<&'p PyAny> {
        pyo3_asyncio::tokio::future_into_py(py, async move {
            // In a full implementation, this would call the Python handler
            // For now, just return success
            Ok(())
        })
    }
}