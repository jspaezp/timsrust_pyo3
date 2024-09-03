use std::path::Path;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use rayon::iter::ParallelIterator;
use timsrust::readers::FrameReader;
use timsrust::{AcquisitionType, MSLevel};

use crate::timsrust_structs::PyFrame;

#[pyclass(name = "FrameReader")]
pub struct PyFrameReader {
    pub reader: FrameReader,
}

#[pymethods]
impl PyFrameReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        Ok(PyFrameReader {
            reader: match FrameReader::new(Path::new(path)) {
                Ok(x) => x,
                Err(_) => return Err(PyIOError::new_err("Could not open file")),
            },
        })
    }

    pub fn read_frame(&self, index: usize) -> PyResult<PyFrame> {
        match self.reader.get(index) {
            Ok(x) => Ok(PyFrame::from(&x)),
            Err(_) => Err(PyIOError::new_err("Could not read frame, Corrupt frame")),
        }
    }

    pub fn read_all_frames(&self) -> PyResult<Vec<PyFrame>> {
        self.reader
            .get_all()
            .iter()
            .map(|x| match x {
                Ok(x) => Ok(PyFrame::from(x)),
                Err(_) => Err(PyIOError::new_err("Could not read frame, Corrupt frame")),
            })
            .collect()
    }

    pub fn read_dia_frames(&self) -> PyResult<Vec<PyFrame>> {
        self.reader
            .parallel_filter(|x| {
                (x.acquisition_type == AcquisitionType::DIAPASEF) && (x.ms_level == MSLevel::MS2)
            })
            .map(|x| match x {
                Ok(x) => Ok(PyFrame::from(&x)),
                Err(_) => Err(PyIOError::new_err("Could not read frame, Corrupt frame")),
            })
            .collect()
    }

    pub fn read_ms1_frames(&self) -> PyResult<Vec<PyFrame>> {
        self.reader
            .parallel_filter(|x| x.ms_level == MSLevel::MS1)
            .map(|x| match x {
                Ok(x) => Ok(PyFrame::from(&x)),
                Err(_) => Err(PyIOError::new_err("Could not read frame, Corrupt frame")),
            })
            .collect()
    }
}

// Thhe spectrum reader seems hard to implement ...
// `(dyn timsrust::io::readers::spectrum_reader::SpectrumReaderTrait + 'static)` cannot be sent between threads safely
// the trait `Send` is not implemented for `(dyn timsrust::io::readers::spectrum_reader::SpectrumReaderTrait + 'static)`, which is required by `SendablePyClass<PySpectrumReader>
//
// #[pyclass(name = "SpectrumReader")]
// pub struct PySpectrumReader {
//     pub reader: SpectrumReader,
// }
//
// #[pymethods]
// impl PySpectrumReader {
//     #[new]
//     pub fn new(path: &str) -> PyResult<Self> {
//         match SpectrumReader::build().with_path(path).finalize() {
//             Ok(x) => Ok(PySpectrumReader { reader: x }),
//             Err(e) => Err(PyIOError::new_err(e.to_string())),
//         }
//     }
// }
