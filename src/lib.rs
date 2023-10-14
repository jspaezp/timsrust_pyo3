use std::collections::HashMap;

use crate::timsrust::Frame;
use crate::timsrust::FrameType;
//use crate::timsrust::converters::{Frame2RtConverter, Tof2MzConverter, Scan2ImConverter};
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use timsrust;
use timsrust::ConvertableIndex;
use timsrust::ReadableFrames;

#[pyclass]
struct TDFReader {
    pub reader: timsrust::TDFReader,
}

#[pymethods]
impl TDFReader {
    #[new]
    fn new(path: String) -> Self {
        TDFReader {
            reader: timsrust::TDFReader::new(&path),
        }
    }
    fn read_all_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_frames()
            .iter()
            .map(|x| PyFrame::new(x.clone()))
            .collect()
    }

    fn read_frame(&self, index: usize) -> PyFrame {
        PyFrame::new(&self.reader.read_single_frame(index))
    }

    fn read_dia_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_dia_frames()
            .iter()
            .map(|x| PyFrame::new(x.clone()))
            .collect()
    }

    fn read_ms1_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_ms1_frames()
            .iter()
            .map(|x| PyFrame::new(x.clone()))
            .collect()
    }

    fn dia_frame_table(&self) -> HashMap<String, Vec<usize>> {
        let frametable = &self.reader.dia_frame_table;
        let mut frametable_out: HashMap<String, Vec<usize>> = HashMap::new();

        frametable_out.insert("frame".to_string(), frametable.frame.to_owned());
        frametable_out.insert("group".to_string(), frametable.group.to_owned());

        frametable_out
    }

    fn dia_frame_msms_windows(&self, py: Python) -> PyResult<PyObject> {
        let msms_frame_window_table = &self.reader.dia_frame_msms_table;

        let key_vals: &[(&str, PyObject)] = &[
            (
                "group",
                msms_frame_window_table.group.to_owned().to_object(py),
            ),
            (
                "scan_start",
                msms_frame_window_table.scan_start.to_owned().to_object(py),
            ),
            (
                "scan_end",
                msms_frame_window_table.scan_end.to_owned().to_object(py),
            ),
            (
                "mz_center",
                msms_frame_window_table.mz_center.to_owned().to_object(py),
            ),
            (
                "mz_width",
                msms_frame_window_table.mz_width.to_owned().to_object(py),
            ),
        ];

        let dict = key_vals.into_py_dict(py);
        Ok(dict.into())
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!("{}(path={})", class_name, slf.borrow().reader.path))
    }
    fn resolve_mzs(slf: &PyCell<Self>, tofs: Vec<u32>) -> Vec<f64> {
        tofs.iter()
            .map(|tof| slf.borrow().reader.mz_converter.convert(*tof))
            .collect()
    }
}

#[pyclass]
struct PyFrame {
    pub scan_offsets: Vec<u64>,
    pub tof_indices: Vec<u32>,
    pub intensities: Vec<u32>,
    pub index: usize,
    pub rt: f64,
    pub frame_type: u8,
}

impl PyFrame {
    fn new(frame: &Frame) -> Self {
        let frametype = match frame.frame_type {
            FrameType::MS1 => 0,
            FrameType::MS2DDA => 1,
            FrameType::MS2DIA => 2,
            FrameType::Unknown => 3,
        };
        PyFrame {
            scan_offsets: frame.scan_offsets.to_owned(),
            tof_indices: frame.tof_indices.to_owned(),
            intensities: frame.intensities.to_owned(),
            index: frame.index.to_owned(),
            rt: frame.rt.to_owned(),
            frame_type: frametype,
        }
    }
}

#[pymethods]
impl PyFrame {
    fn rt(&self) -> f64 {
        self.rt
    }
    fn index(&self) -> usize {
        self.index
    }
    fn frame_type(&self) -> u8 {
        self.frame_type
    }
    fn scan_offsets(&self) -> Vec<u64> {
        self.scan_offsets.to_owned()
    }
    fn tof_indices(&self) -> Vec<u32> {
        self.tof_indices.to_owned()
    }
    fn intensities(&self) -> Vec<u32> {
        self.intensities.to_owned()
    }
    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{}(index={}, rt={}, frame_type={}, len(scan_offsets)={}, len(tof_indices)={}, len(intensities)={})",
            class_name,
            slf.borrow().index,
            slf.borrow().rt,
            slf.borrow().frame_type,
            slf.borrow().scan_offsets.len(),
            slf.borrow().tof_indices.len(),
            slf.borrow().intensities.len(),
        ))
    }
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn read_all_frames(a: String) -> PyResult<Vec<PyFrame>> {
    let fr = timsrust::TDFReader::new(&a);

    let out: Vec<PyFrame> = fr
        .read_all_frames()
        .iter()
        .map(|x| PyFrame::new(x.clone()))
        .collect();
    Ok(out)
}

/// A Python module implemented in Rust.
#[pymodule]
fn timsrust_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_all_frames, m)?)?;
    m.add_class::<TDFReader>()?;
    m.add_class::<PyFrame>()?;
    Ok(())
}
