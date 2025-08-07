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
use uuid::Uuid;

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

    pub fn subscribe<'p>(&self, py: Python<'p>, subscription_dict: &PyDict) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        
        // Extract subscription parameters from dictionary
        let subscription_id = subscription_dict
            .get_item("id")?
            .and_then(|v| v.extract::<String>().ok())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
            
        let aggregate_type_filter = subscription_dict
            .get_item("aggregate_type_filter")?
            .and_then(|v| v.extract::<String>().ok());
            
        let event_type_filter = subscription_dict
            .get_item("event_type_filter")?
            .and_then(|v| v.extract::<String>().ok());
        
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

    #[pyo3(signature = (event, stream_position, global_position))]
    pub fn publish_event<'p>(
        &self, 
        py: Python<'p>, 
        event: &PyEvent, 
        stream_position: u64, 
        global_position: u64
    ) -> PyResult<&'p PyAny> {
        let streamer = self.streamer.clone();
        let event = event.inner.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let streamer_guard = streamer.lock().await;
            streamer_guard.publish_event(event, stream_position, global_position)
                .await
                .map_err(map_rust_error_to_python)?;
            Ok(())
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

    pub fn with_id(mut slf: PyRefMut<Self>, id: String) -> PyRefMut<Self> {
        slf.id = Some(id);
        slf
    }

    pub fn filter_by_aggregate_type(mut slf: PyRefMut<Self>, aggregate_type: String) -> PyRefMut<Self> {
        slf.aggregate_type_filter = Some(aggregate_type);
        slf
    }

    pub fn filter_by_event_type(mut slf: PyRefMut<Self>, event_type: String) -> PyRefMut<Self> {
        slf.event_type_filter = Some(event_type);
        slf
    }

    pub fn build(&self, py: Python<'_>) -> PyResult<PyObject> {
        let py_dict = PyDict::new(py);
        
        let id = self.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
        py_dict.set_item("id", id)?;
        
        if let Some(ref agg_filter) = self.aggregate_type_filter {
            py_dict.set_item("aggregate_type_filter", agg_filter)?;
        }
        
        if let Some(ref event_filter) = self.event_type_filter {
            py_dict.set_item("event_type_filter", event_filter)?;
        }
        
        Ok(py_dict.to_object(py))
    }
}

#[pyclass]
pub struct PyProjection {
    handler: PyObject,
    last_position: Arc<Mutex<Option<u64>>>,
}

#[pymethods]
impl PyProjection {
    #[new]
    pub fn new(handler: PyObject) -> Self {
        Self { 
            handler,
            last_position: Arc::new(Mutex::new(None)),
        }
    }

    pub fn handle_event<'p>(&self, py: Python<'p>, event: &PyEvent) -> PyResult<&'p PyAny> {
        let handler = self.handler.clone();
        let py_event = Py::new(py, PyEvent { inner: event.inner.clone() })?;
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            Python::with_gil(|py| {
                // Call the Python handler function with the event
                if let Ok(_coroutine) = handler.call1(py, (py_event,)) {
                    // If it returns a coroutine, we would need to await it
                    // For now, assume it's a sync function
                    Ok(())
                } else {
                    Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                        "Failed to call projection handler"
                    ))
                }
            })
        })
    }

    pub fn reset<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let last_position = self.last_position.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut pos = last_position.lock().await;
            *pos = None;
            Ok(())
        })
    }

    pub fn get_last_processed_position<'p>(&self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let last_position = self.last_position.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let pos = last_position.lock().await;
            Ok(*pos)
        })
    }

    pub fn set_last_processed_position<'p>(&self, py: Python<'p>, position: u64) -> PyResult<&'p PyAny> {
        let last_position = self.last_position.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let mut pos = last_position.lock().await;
            *pos = Some(position);
            Ok(())
        })
    }
}