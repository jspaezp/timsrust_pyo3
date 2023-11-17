use std::collections::HashMap;

use crate::timsrust::Spectrum;
use crate::timsrust::Frame;
use crate::timsrust::FrameType;
use crate::timsrust::ConvertableIndex;
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use timsrust;
use timsrust::AcquisitionType;
use timsrust::QuadrupoleEvent;

#[pyclass]
struct TimsReader {
    pub path: String,
    pub reader: timsrust::FileReader,
}

#[pymethods]
impl TimsReader {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        use pyo3::exceptions::PyIOError;
        let reader = timsrust::FileReader::new(&path);
        match reader {
            Ok(x) => Ok(TimsReader { reader: x, path: path }),
            Err(x) => Err(PyIOError::new_err("Could not open file")),
            
        }
    }
    fn read_all_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_frames()
            .iter()
            .map(|x| PyFrame::new(x))
            .collect()
    }

    // fn read_scan(&self, index: usize) -> PySpectrum {
    //     self.reader.read_all
    // }

    fn read_frame(&self, index: usize) -> PyFrame {
        PyFrame::new(&self.reader.read_single_frame(index))
    }

    fn read_dia_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_frames()
            .iter()
            .filter(|x| x.frame_type == FrameType::MS2(AcquisitionType::DIAPASEF))
            .map(|x| PyFrame::new(x))
            .collect()
    }

    fn read_ms1_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_ms1_frames()
            .iter()
            .map(|x| PyFrame::new(x))
            .collect()
    }

    // TODO implement on the python end
    // fn dia_frame_table(&self) -> HashMap<String, Vec<usize>> {
    //     let frametable = &self.reader.dia_frame_table;
    //     let mut frametable_out: HashMap<String, Vec<usize>> = HashMap::new();

    //     frametable_out.insert("frame".to_string(), frametable.frame.to_owned());
    //     frametable_out.insert("group".to_string(), frametable.group.to_owned());

    //     frametable_out
    // }

    // fn dia_frame_msms_windows(&self, py: Python) -> PyResult<PyObject> {
    //     let msms_frame_window_table = &self.reader.dia_frame_msms_table;

    //     let key_vals: &[(&str, PyObject)] = &[
    //         (
    //             "group",
    //             msms_frame_window_table.group.to_owned().to_object(py),
    //         ),
    //         (
    //             "scan_start",
    //             msms_frame_window_table.scan_start.to_owned().to_object(py),
    //         ),
    //         (
    //             "scan_end",
    //             msms_frame_window_table.scan_end.to_owned().to_object(py),
    //         ),
    //         (
    //             "mz_center",
    //             msms_frame_window_table.mz_center.to_owned().to_object(py),
    //         ),
    //         (
    //             "mz_width",
    //             msms_frame_window_table.mz_width.to_owned().to_object(py),
    //         ),
    //     ];

    //     let dict = key_vals.into_py_dict(py);
    //     Ok(dict.into())
    // }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!("{}(path=???)", class_name))
    }
    fn resolve_mzs(slf: &PyCell<Self>, tofs: Vec<u32>) -> Vec<f64> {
        let converter = slf.borrow()
            .reader
            .get_tof_converter().unwrap();
        
        tofs.iter().map(|x| converter.convert(*x)).collect()
    }
}

#[pyclass]
struct PySpectrum {
    pub mz_values: Vec<f64>,
    pub intensities: Vec<f64>,
    pub index: usize,
    pub precursor: PyPrecursor,
}

#[pyclass]
struct PyPrecursor {
    pub mz: f64,
    pub im: f64,
    pub charge: usize,
    pub intensity: f64,
    pub index: usize,
    pub frame_index: usize,
}

impl PyPrecursor {
    fn new(precursor: &timsrust::Precursor) -> Self {
        PyPrecursor {
            mz: precursor.mz.to_owned(),
            im: precursor.im.to_owned(),
            charge: precursor.charge.to_owned(),
            intensity: precursor.intensity.to_owned(),
            index: precursor.index.to_owned(),
            frame_index: precursor.frame_index.to_owned(),
        }
    }
}

impl PySpectrum {
    fn new(scan: &Spectrum) -> Self {
        let precursor = match scan.precursor {
            QuadrupoleEvent::Precursor(x) => PyPrecursor::new(&x),
            QuadrupoleEvent::None => PyPrecursor {
                mz: 0.0,
                im: 0.0,
                charge: 0,
                intensity: 0.0,
                index: 0,
                frame_index: 0,
            },
        };
        PySpectrum {
            mz_values: scan.mz_values.to_owned(),
            intensities: scan.intensities.to_owned(),
            index: scan.index.to_owned(),
            precursor: precursor,
        }
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
            FrameType::MS2(x) => {
                match x {
                    AcquisitionType::DDAPASEF => 1,
                    AcquisitionType::DIAPASEF => 2,
                    AcquisitionType::Unknown => 3,
                }
            }
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
    let fr = timsrust::FileReader::new(&a);
    let fr = TimsReader {
        reader: fr.unwrap(),
        path: a,
    };

    let out: Vec<PyFrame> = fr.read_all_frames();
    Ok(out)
}

/// A Python module implemented in Rust.
#[pymodule]
fn timsrust_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_all_frames, m)?)?;
    m.add_class::<TimsReader>()?;
    m.add_class::<PyFrame>()?;
    Ok(())
}
