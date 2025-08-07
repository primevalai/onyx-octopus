use pyo3::prelude::*;
use pyo3::types::PyDict;
use eventuali_core::{Event as CoreEvent, EventData, EventMetadata};
use uuid::Uuid;
use std::collections::HashMap;

#[pyclass]
#[derive(Clone)]
pub struct PyEvent {
    pub inner: CoreEvent,
}

#[pymethods]
impl PyEvent {
    #[new]
    #[pyo3(signature = (aggregate_id, aggregate_type, event_type, event_version, aggregate_version, data))]
    pub fn new(
        aggregate_id: String,
        aggregate_type: String,
        event_type: String,
        event_version: i32,
        aggregate_version: i64,
        data: String, // Use JSON string instead of PyObject for simplicity
    ) -> PyResult<Self> {
        let json_value: serde_json::Value = serde_json::from_str(&data)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
        
        let event_data = EventData::from_json(&json_value)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;

        let event = CoreEvent::new(
            aggregate_id,
            aggregate_type,
            event_type,
            event_version,
            aggregate_version,
            event_data,
        );

        Ok(PyEvent { inner: event })
    }

    #[staticmethod]
    #[pyo3(signature = (data_dict))]
    pub fn from_dict(data_dict: &PyDict) -> PyResult<Self> {
        // Extract all required fields from the dictionary
        let id_str: String = data_dict.get_item("event_id")
            .ok()
            .and_then(|v| v)
            .and_then(|v| v.extract().ok())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let aggregate_id: String = data_dict.get_item("aggregate_id")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_id"))?
            .extract()?;
        
        let aggregate_type: String = data_dict.get_item("aggregate_type")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_type"))?
            .extract()?;
        
        let event_type: String = data_dict.get_item("event_type")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing event_type"))?
            .extract()?;
        
        let event_version: i32 = data_dict.get_item("event_version")
            .ok()
            .and_then(|v| v)
            .and_then(|v| v.extract().ok())
            .unwrap_or(1);
        
        let aggregate_version: i64 = data_dict.get_item("aggregate_version")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing aggregate_version"))?
            .extract()?;
        
        // Parse event data
        let data_value = data_dict.get_item("data")?
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>("Missing data"))?;
        
        Python::with_gil(|py| {
            let json_module = py.import("json")?;
            let json_str: String = json_module.call_method1("dumps", (data_value,))?.extract()?;
            let json_value: serde_json::Value = serde_json::from_str(&json_str)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            let event_data = EventData::from_json(&json_value)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
            
            // Parse timestamp
            let timestamp = if let Ok(Some(ts_value)) = data_dict.get_item("timestamp") {
                let ts_str: String = ts_value.extract()?;
                chrono::DateTime::parse_from_rfc3339(&ts_str)
                    .map(|dt| dt.with_timezone(&chrono::Utc))
                    .unwrap_or_else(|_| chrono::Utc::now())
            } else {
                chrono::Utc::now()
            };
            
            // Parse metadata
            let metadata = if let Ok(Some(meta_dict)) = data_dict.get_item("metadata") {
                let meta_dict = meta_dict.downcast::<PyDict>()?;
                let causation_id = meta_dict.get_item("causation_id")
                    .ok()
                    .and_then(|v| v)
                    .and_then(|v| v.extract::<String>().ok())
                    .and_then(|s| Uuid::parse_str(&s).ok());
                let correlation_id = meta_dict.get_item("correlation_id")
                    .ok()
                    .and_then(|v| v)
                    .and_then(|v| v.extract::<String>().ok())
                    .and_then(|s| Uuid::parse_str(&s).ok());
                let user_id = meta_dict.get_item("user_id")
                    .ok()
                    .and_then(|v| v)
                    .and_then(|v| v.extract::<String>().ok());
                
                let headers = if let Ok(Some(headers_dict)) = meta_dict.get_item("headers") {
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
            
            let id = Uuid::parse_str(&id_str)
                .map_err(|_| PyErr::new::<pyo3::exceptions::PyValueError, _>("Invalid UUID"))?;
            
            let event = CoreEvent {
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
            
            Ok(PyEvent { inner: event })
        })
    }

    #[getter]
    pub fn id(&self) -> String {
        self.inner.id.to_string()
    }

    #[getter]
    pub fn aggregate_id(&self) -> String {
        self.inner.aggregate_id.clone()
    }

    #[getter]
    pub fn aggregate_type(&self) -> String {
        self.inner.aggregate_type.clone()
    }

    #[getter]
    pub fn event_type(&self) -> String {
        self.inner.event_type.clone()
    }

    #[getter]
    pub fn event_version(&self) -> i32 {
        self.inner.event_version
    }

    #[getter]
    pub fn aggregate_version(&self) -> i64 {
        self.inner.aggregate_version
    }

    #[getter]
    pub fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }

    #[getter]
    pub fn data(&self) -> PyResult<String> {
        match &self.inner.data {
            EventData::Json(value) => Ok(serde_json::to_string(value)
                .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?),
            EventData::Protobuf(_) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "Cannot convert protobuf data to JSON string"
            )),
        }
    }

    pub fn to_dict(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        
        dict.set_item("event_id", self.inner.id.to_string())?;
        dict.set_item("aggregate_id", &self.inner.aggregate_id)?;
        dict.set_item("aggregate_type", &self.inner.aggregate_type)?;
        dict.set_item("event_type", &self.inner.event_type)?;
        dict.set_item("event_version", self.inner.event_version)?;
        dict.set_item("aggregate_version", self.inner.aggregate_version)?;
        dict.set_item("timestamp", self.inner.timestamp.to_rfc3339())?;
        
        // Convert event data
        let data = match &self.inner.data {
            EventData::Json(value) => {
                let json_str = serde_json::to_string(value)
                    .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?;
                let json_module = py.import("json")?;
                json_module.call_method1("loads", (json_str,))?
            },
            EventData::Protobuf(bytes) => {
                let bytes_obj = pyo3::types::PyBytes::new(py, bytes);
                bytes_obj.into()
            },
        };
        dict.set_item("data", data)?;
        
        // Convert metadata
        let metadata_dict = PyDict::new(py);
        if let Some(ref causation_id) = self.inner.metadata.causation_id {
            metadata_dict.set_item("causation_id", causation_id.to_string())?;
        }
        if let Some(ref correlation_id) = self.inner.metadata.correlation_id {
            metadata_dict.set_item("correlation_id", correlation_id.to_string())?;
        }
        if let Some(ref user_id) = self.inner.metadata.user_id {
            metadata_dict.set_item("user_id", user_id)?;
        }
        
        let headers_dict = PyDict::new(py);
        for (key, value) in &self.inner.metadata.headers {
            headers_dict.set_item(key, value)?;
        }
        metadata_dict.set_item("headers", headers_dict)?;
        
        dict.set_item("metadata", metadata_dict)?;
        
        Ok(dict.into())
    }
}