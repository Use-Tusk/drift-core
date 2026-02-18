use pyo3::prelude::*;
use pyo3::types::{PyBool, PyBytes, PyDict, PyFloat, PyInt, PyList, PyString, PyTuple};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue};

pub fn py_to_json_value(value: &Bound<'_, PyAny>) -> PyResult<JsonValue> {
    if value.is_none() {
        return Ok(JsonValue::Null);
    }
    if let Ok(v) = value.extract::<bool>() {
        return Ok(JsonValue::Bool(v));
    }
    if let Ok(v) = value.extract::<i64>() {
        return Ok(JsonValue::Number(JsonNumber::from(v)));
    }
    if let Ok(v) = value.extract::<f64>() {
        return Ok(JsonNumber::from_f64(v)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null));
    }
    if let Ok(v) = value.extract::<String>() {
        return Ok(JsonValue::String(v));
    }
    if let Ok(list) = value.cast::<PyList>() {
        let mut out = Vec::with_capacity(list.len());
        for item in list.iter() {
            out.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(out));
    }
    if let Ok(tuple) = value.cast::<PyTuple>() {
        let mut out = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            out.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(out));
    }
    if let Ok(dict) = value.cast::<PyDict>() {
        let mut out = JsonMap::with_capacity(dict.len());
        for (k, v) in dict.iter() {
            let key = k.extract::<String>()?;
            out.insert(key, py_to_json_value(&v)?);
        }
        return Ok(JsonValue::Object(out));
    }
    Err(pyo3::exceptions::PyTypeError::new_err(
        "unsupported value type for Rust export payload processing",
    ))
}

pub fn json_value_to_py(py: Python<'_>, value: &JsonValue) -> PyResult<Py<PyAny>> {
    match value {
        JsonValue::Null => Ok(py.None()),
        JsonValue::Bool(v) => Ok(PyBool::new(py, *v).to_owned().unbind().into_any()),
        JsonValue::Number(v) => {
            if let Some(i) = v.as_i64() {
                Ok(PyInt::new(py, i).unbind().into_any())
            } else if let Some(u) = v.as_u64() {
                Ok(PyInt::new(py, u).unbind().into_any())
            } else if let Some(f) = v.as_f64() {
                Ok(PyFloat::new(py, f).unbind().into_any())
            } else {
                Ok(py.None())
            }
        }
        JsonValue::String(v) => Ok(PyString::new(py, v).unbind().into_any()),
        JsonValue::Array(arr) => {
            let list = PyList::empty(py);
            for child in arr {
                list.append(json_value_to_py(py, child)?)?;
            }
            Ok(list.unbind().into_any())
        }
        JsonValue::Object(map) => {
            let dict = PyDict::new(py);
            for (k, child) in map {
                dict.set_item(k, json_value_to_py(py, child)?)?;
            }
            Ok(dict.unbind().into_any())
        }
    }
}

pub fn py_any_to_optional_json(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<JsonValue>> {
    match value {
        Some(v) if !v.is_none() => Ok(Some(py_to_json_value(v)?)),
        _ => Ok(None),
    }
}

pub fn py_any_to_optional_bytes(value: Option<&Bound<'_, PyAny>>) -> PyResult<Option<Vec<u8>>> {
    match value {
        Some(v) if !v.is_none() => {
            if let Ok(py_bytes) = v.cast::<PyBytes>() {
                Ok(Some(py_bytes.as_bytes().to_vec()))
            } else {
                Err(pyo3::exceptions::PyTypeError::new_err(
                    "expected bytes or None",
                ))
            }
        }
        _ => Ok(None),
    }
}
