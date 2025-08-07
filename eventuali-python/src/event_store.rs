use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use eventuali_core::{
    EventStoreConfig, create_event_store, EventStore, Event, EventData, EventMetadata
};
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use crate::event::PyEvent;
use crate::error::map_rust_error_to_python;

#[pyclass]
pub struct PyEventStore {
    store: Arc<Mutex<Option<Box<dyn EventStore + Send + Sync>>>>,
}

#[pymethods]
impl PyEventStore {
    #[new]
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(None)),
        }
    }

    #[pyo3(signature = (connection_string))]
    pub fn create<'p>(&self, py: Python<'p>, connection_string: String) -> PyResult<&'p PyAny> {
        let store = self.store.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let config = if connection_string.starts_with("postgresql://") || connection_string.starts_with("postgres://") {
                EventStoreConfig::postgres(connection_string)
            } else if connection_string.starts_with("sqlite://") {
                let path = connection_string.strip_prefix("sqlite://").unwrap_or(&connection_string);
                EventStoreConfig::sqlite(path.to_string())
            } else {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    format!("Unsupported connection string format: {}", connection_string)
                ));
            };

            let event_store = create_event_store(config)
                .await
                .map_err(map_rust_error_to_python)?;

            let mut store_guard = store.lock().await;
            *store_guard = Some(event_store);

            Ok(())
        })
    }

    #[pyo3(signature = (events))]
    pub fn save_events<'p>(&self, py: Python<'p>, events: &PyList) -> PyResult<&'p PyAny> {
        let store = self.store.clone();
        let events_data = self.convert_py_events_to_rust(py, events)?;
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let store_guard = store.lock().await;
            if let Some(ref event_store) = *store_guard {
                event_store.save_events(events_data)
                    .await
                    .map_err(map_rust_error_to_python)?;
                Ok(())
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "EventStore not initialized"
                ))
            }
        })
    }

    #[pyo3(signature = (aggregate_id, from_version = None))]
    pub fn load_events<'p>(
        &self, 
        py: Python<'p>, 
        aggregate_id: String,
        from_version: Option<i64>
    ) -> PyResult<&'p PyAny> {
        let store = self.store.clone();
        
        pyo3_asyncio::tokio::future_into_py::<_, PyObject>(py, async move {
            let store_guard = store.lock().await;
            if let Some(ref event_store) = *store_guard {
                let events = event_store.load_events(&aggregate_id, from_version)
                    .await
                    .map_err(map_rust_error_to_python)?;
                
                Python::with_gil(|py| {
                    let py_events = PyList::empty(py);
                    for event in events {
                        let py_event = PyEvent { inner: event };
                        py_events.append(Py::new(py, py_event)?)?;
                    }
                    Ok(py_events.to_object(py))
                })
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "EventStore not initialized"
                ))
            }
        })
    }

    #[pyo3(signature = (aggregate_type, from_version = None))]
    pub fn load_events_by_type<'p>(
        &self,
        py: Python<'p>,
        aggregate_type: String,
        from_version: Option<i64>
    ) -> PyResult<&'p PyAny> {
        let store = self.store.clone();
        
        pyo3_asyncio::tokio::future_into_py::<_, PyObject>(py, async move {
            let store_guard = store.lock().await;
            if let Some(ref event_store) = *store_guard {
                let events = event_store.load_events_by_type(&aggregate_type, from_version)
                    .await
                    .map_err(map_rust_error_to_python)?;
                
                Python::with_gil(|py| {
                    let py_events = PyList::empty(py);
                    for event in events {
                        let py_event = PyEvent { inner: event };
                        py_events.append(Py::new(py, py_event)?)?;
                    }
                    Ok(py_events.to_object(py))
                })
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "EventStore not initialized"
                ))
            }
        })
    }

    #[pyo3(signature = (aggregate_id))]
    pub fn get_aggregate_version<'p>(
        &self,
        py: Python<'p>,
        aggregate_id: String
    ) -> PyResult<&'p PyAny> {
        let store = self.store.clone();
        
        pyo3_asyncio::tokio::future_into_py(py, async move {
            let store_guard = store.lock().await;
            if let Some(ref event_store) = *store_guard {
                let version = event_store.get_aggregate_version(&aggregate_id)
                    .await
                    .map_err(map_rust_error_to_python)?;
                Ok(version)
            } else {
                Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                    "EventStore not initialized"
                ))
            }
        })
    }

    #[pyo3(signature = (_streamer))]
    pub fn set_streamer(&self, _py: Python, _streamer: Py<crate::streaming::PyEventStreamer>) -> PyResult<()> {
        // This is a simplified approach - in a full implementation we would need to 
        // extract the Rust streamer from the Python wrapper and set it on the event store
        // For now, we'll return Ok to avoid compilation errors
        Ok(())
    }
}

impl PyEventStore {
    fn convert_py_events_to_rust(&self, py: Python, events: &PyList) -> PyResult<Vec<Event>> {
        let mut rust_events = Vec::new();
        
        for item in events.iter() {
            let py_dict = item.downcast::<PyDict>()?;
            
            // Extract fields from Python dict
            let id_str: String = py_dict.get_item("event_id")?
                .map(|v| v.extract().unwrap_or_else(|_| Uuid::new_v4().to_string()))
                .unwrap_or_else(|| Uuid::new_v4().to_string());
            let id = Uuid::parse_str(&id_str)
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid UUID"))?;
            
            let aggregate_id: String = py_dict.get_item("aggregate_id")?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_id"))?
                .extract()?;
            
            let aggregate_type: String = py_dict.get_item("aggregate_type")?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_type"))?
                .extract()?;
            
            let event_type: String = py_dict.get_item("event_type")?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing event_type"))?
                .extract()?;
            
            let event_version: i32 = py_dict.get_item("event_version")?
                .map(|v| v.extract().unwrap_or(1))
                .unwrap_or(1);
            
            let aggregate_version: i64 = py_dict.get_item("aggregate_version")?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_version"))?
                .extract()?;
            
            let timestamp_str: Option<String> = py_dict.get_item("timestamp")?
                .map(|v| v.extract().unwrap_or_default());
            let timestamp = if let Some(ts_str) = timestamp_str {
                DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now())
            } else {
                Utc::now()
            };
            
            // Extract event data
            let data_dict = py_dict.get_item("data")?
                .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing event data"))?;
            let json_str: String = py.eval("import json; json.dumps", None, None)?
                .call1((data_dict,))?
                .extract()?;
            let json_value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            let event_data = EventData::from_json(&json_value)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            // Extract metadata
            let metadata = if let Some(meta_dict) = py_dict.get_item("metadata")? {
                let causation_id = if let Ok(v) = meta_dict.get_item("causation_id") {
                    if let Ok(s) = v.extract::<String>() {
                        Uuid::parse_str(&s).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };
                let correlation_id = if let Ok(v) = meta_dict.get_item("correlation_id") {
                    if let Ok(s) = v.extract::<String>() {
                        Uuid::parse_str(&s).ok()
                    } else {
                        None
                    }
                } else {
                    None
                };
                let user_id = if let Ok(v) = meta_dict.get_item("user_id") {
                    v.extract::<String>().ok()
                } else {
                    None
                };
                
                let headers = if let Ok(headers_dict) = meta_dict.get_item("headers") {
                    let headers_dict = headers_dict.downcast::<PyDict>()?;
                    let mut headers = HashMap::new();
                    for (k, v) in headers_dict.iter() {
                        let key: String = k.extract()?;
                        let value: String = v.extract()?;
                        headers.insert(key, value);
                    }
                    headers
                } else {
                    HashMap::new()
                };
                
                EventMetadata {
                    causation_id,
                    correlation_id,
                    user_id,
                    headers,
                }
            } else {
                EventMetadata::default()
            };
            
            let event = Event {
                id,
                aggregate_id,
                aggregate_type,
                event_type,
                event_version,
                aggregate_version,
                data: event_data,
                metadata,
                timestamp,
            };
            
            rust_events.push(event);
        }
        
        Ok(rust_events)
    }
}