use timsrust::converters::{
    ConvertableDomain, Frame2RtConverter, Scan2ImConverter, Tof2MzConverter,
};

use pyo3::prelude::*;

#[derive(Clone)]
#[pyclass(name = "Frame2RtConverter")]
pub struct PyFrame2RtConverter {
    pub converter: timsrust::converters::Frame2RtConverter,
}

impl From<&Frame2RtConverter> for PyFrame2RtConverter {
    fn from(x: &Frame2RtConverter) -> Self {
        PyFrame2RtConverter {
            converter: x.clone(),
        }
    }
}

impl ConvertableDomain for PyFrame2RtConverter {
    fn convert<T: Into<f64> + Copy>(&self, x: T) -> f64 {
        self.converter.convert(x)
    }
    fn invert<T: Into<f64> + Copy>(&self, value: T) -> f64 {
        self.converter.invert(value)
    }
}

#[derive(Clone)]
#[pyclass(name = "Scan2ImConverter")]
pub struct PyScan2ImConverter {
    pub converter: timsrust::converters::Scan2ImConverter,
}

impl From<&Scan2ImConverter> for PyScan2ImConverter {
    fn from(x: &Scan2ImConverter) -> Self {
        PyScan2ImConverter { converter: *x }
    }
}

impl ConvertableDomain for PyScan2ImConverter {
    fn convert<T: Into<f64> + Copy>(&self, x: T) -> f64 {
        self.converter.convert(x)
    }
    fn invert<T: Into<f64> + Copy>(&self, value: T) -> f64 {
        self.converter.invert(value)
    }
}

#[derive(Clone)]
#[pyclass(name = "Tof2MzConverter")]
pub struct PyTof2MzConverter {
    pub converter: timsrust::converters::Tof2MzConverter,
}

impl From<&Tof2MzConverter> for PyTof2MzConverter {
    fn from(x: &Tof2MzConverter) -> Self {
        PyTof2MzConverter { converter: *x }
    }
}

impl ConvertableDomain for PyTof2MzConverter {
    fn convert<T: Into<f64> + Copy>(&self, x: T) -> f64 {
        self.converter.convert(x)
    }
    fn invert<T: Into<f64> + Copy>(&self, value: T) -> f64 {
        self.converter.invert(value)
    }
}
