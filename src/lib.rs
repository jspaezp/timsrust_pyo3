use std::fmt::Display;

use pyo3::exceptions::PyIOError;
use pyo3::prelude::*;
use timsrust::AcquisitionType;
use timsrust::ConvertableIndex;
use timsrust::Frame;
use timsrust::FrameType;
use timsrust::QuadrupoleEvent;
use timsrust::Spectrum;

#[pyclass]
pub struct TimsReader {
    #[pyo3(get)]
    pub path: String,
    pub reader: timsrust::FileReader,
}

#[pyclass]
pub struct Frame2RtConverter {
    pub converter: timsrust::Frame2RtConverter,
}

#[pyclass]
pub struct Scan2ImConverter {
    pub converter: timsrust::Scan2ImConverter,
}

#[pyclass]
pub struct Tof2MzConverter {
    pub converter: timsrust::Tof2MzConverter,
}

#[pymethods]
impl TimsReader {
    #[new]
    fn new(path: String) -> PyResult<Self> {
        Ok(TimsReader {
            reader: match timsrust::FileReader::new(&path) {
                Ok(x) => x,
                Err(_) => return Err(PyIOError::new_err("Could not open file")),
            },
            path,
        })
    }

    fn get_frame2rt_converter(&self) -> Frame2RtConverter {
        Frame2RtConverter {
            converter: self.reader.get_frame_converter().unwrap(),
        }
    }

    fn get_scan2im_converter(&self) -> Scan2ImConverter {
        Scan2ImConverter {
            converter: self.reader.get_scan_converter().unwrap(),
        }
    }

    fn get_tof_converter(&self) -> Tof2MzConverter {
        Tof2MzConverter {
            converter: self.reader.get_tof_converter().unwrap(),
        }
    }

    fn read_frame(&self, index: usize) -> PyFrame {
        PyFrame::new(&self.reader.read_single_frame(index))
    }

    fn read_all_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_frames()
            .iter()
            .map(PyFrame::new)
            .collect()
    }

    fn read_dia_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_ms2_frames()
            .iter()
            .map(|x| match x.frame_type {
                FrameType::MS2(AcquisitionType::DIAPASEF) => PyFrame::new(x),
                _ => PyFrame::new(&Frame::default()),
            })
            .collect()
    }

    /// Reads all MS1 frames
    ///
    /// Returns a vec with its length being all the frames in the data.
    /// BUT only parses the MS1 frames (all non-ms1 frames are returned as empty)
    fn read_ms1_frames(&self) -> Vec<PyFrame> {
        self.reader
            .read_all_ms1_frames()
            .iter()
            .map(PyFrame::new)
            .collect()
    }

    fn read_spectrum(&self, index: usize) -> PySpectrum {
        PySpectrum::new(&self.reader.read_single_spectrum(index))
    }

    fn read_all_spectra(&self) -> Vec<PySpectrum> {
        self.reader
            .read_all_spectra()
            .iter()
            .map(PySpectrum::new)
            .collect()
    }

    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{}(path='{}')",
            class_name,
            slf.borrow().path.clone()
        ))
    }

    fn resolve_mzs(slf: &PyCell<Self>, tofs: Vec<u32>) -> PyResult<Vec<f64>> {
        match &slf.borrow().reader.get_tof_converter() {
            Ok(c) => Ok(tofs.iter().map(|x| c.convert(*x)).collect()),
            Err(e) => Err(PyIOError::new_err(format!(
                "Could not get TOF converter: {e}"
            ))),
        }
    }

    fn resolve_scans(slf: &PyCell<Self>, ims: Vec<u32>) -> PyResult<Vec<f64>> {
        match &slf.borrow().reader.get_scan_converter() {
            Ok(c) => Ok(ims.iter().map(|x| c.convert(*x)).collect()),
            Err(e) => Err(PyIOError::new_err(format!(
                "Could not get scan converter: {e}"
            ))),
        }
    }

    fn resolve_frames(slf: &PyCell<Self>, rts: Vec<u32>) -> PyResult<Vec<f64>> {
        match &slf.borrow().reader.get_frame_converter() {
            Ok(c) => Ok(rts.iter().map(|x| c.convert(*x)).collect()),
            Err(e) => Err(PyIOError::new_err(format!(
                "Could not get frame converter: {e}"
            ))),
        }
    }
}

#[pyclass]
struct PySpectrum {
    #[pyo3(get, set)]
    pub mz_values: Vec<f64>,
    #[pyo3(get, set)]
    pub intensities: Vec<f64>,
    #[pyo3(get, set)]
    pub index: usize,
    #[pyo3(get, set)]
    pub precursor: PyPrecursor,
}

#[derive(Clone)]
#[pyclass]
struct PyPrecursor {
    #[pyo3(get, set)]
    pub mz: f64,
    #[pyo3(get, set)]
    pub rt: f64,
    #[pyo3(get, set)]
    pub im: f64,
    #[pyo3(get, set)]
    pub charge: usize,
    #[pyo3(get, set)]
    pub intensity: f64,
    #[pyo3(get, set)]
    pub index: usize,
    #[pyo3(get, set)]
    pub frame_index: usize,
}

impl PyPrecursor {
    fn new(precursor: &timsrust::Precursor) -> Self {
        PyPrecursor {
            mz: precursor.mz.to_owned(),
            rt: precursor.rt.to_owned(),
            im: precursor.im.to_owned(),
            charge: precursor.charge.to_owned(),
            intensity: precursor.intensity.to_owned(),
            index: precursor.index.to_owned(),
            frame_index: precursor.frame_index.to_owned(),
        }
    }

    pub fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{}(index={}, frame_index={}, mz={}, rt={}, im={}, charge={}, intensity={})",
            class_name,
            slf.borrow().index,
            slf.borrow().frame_index,
            slf.borrow().mz,
            slf.borrow().rt,
            slf.borrow().im,
            slf.borrow().charge,
            slf.borrow().intensity,
        ))
    }
}

impl Display for PyPrecursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PyPrecursor(index={}, frame_index={}, mz={}, rt={}, im={}, charge={}, intensity={})",
            self.index, self.frame_index, self.mz, self.rt, self.im, self.charge, self.intensity
        )
    }
}

impl PySpectrum {
    fn new(scan: &Spectrum) -> Self {
        let precursor = match scan.precursor {
            QuadrupoleEvent::Precursor(x) => PyPrecursor::new(&x),
            QuadrupoleEvent::None => PyPrecursor {
                mz: 0.0,
                rt: 0.0,
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
            precursor,
        }
    }
}

#[pymethods]
impl PySpectrum {
    fn __repr__(slf: &PyCell<Self>) -> PyResult<String> {
        let class_name: &str = slf.get_type().name()?;
        Ok(format!(
            "{}(index={}, len(mz_values)={}, len(intensities)={}, precursor={})",
            class_name,
            slf.borrow().index,
            slf.borrow().mz_values.len(),
            slf.borrow().intensities.len(),
            slf.borrow().precursor,
        ))
    }
}

#[pyclass]
struct PyFrame {
    #[pyo3(get, set)]
    pub scan_offsets: Vec<u64>,
    #[pyo3(get, set)]
    pub tof_indices: Vec<u32>,
    #[pyo3(get, set)]
    pub intensities: Vec<u32>,
    #[pyo3(get, set)]
    pub index: usize,
    #[pyo3(get, set)]
    pub rt: f64,
    #[pyo3(get, set)]
    pub frame_type: u8,
}

impl PyFrame {
    fn new(frame: &Frame) -> Self {
        let frame_type = match frame.frame_type {
            FrameType::MS1 => 0,
            FrameType::MS2(x) => match x {
                AcquisitionType::DDAPASEF => 1,
                AcquisitionType::DIAPASEF => 2,
                AcquisitionType::Unknown => 3,
            },
            FrameType::Unknown => 3,
        };
        PyFrame {
            scan_offsets: frame.scan_offsets.to_owned(),
            tof_indices: frame.tof_indices.to_owned(),
            intensities: frame.intensities.to_owned(),
            index: frame.index.to_owned(),
            rt: frame.rt.to_owned(),
            frame_type,
        }
    }
}

#[pymethods]
impl PyFrame {
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

#[pyfunction]
fn read_all_frames(path: String) -> PyResult<Vec<PyFrame>> {
    let reader = timsrust::FileReader::new(&path).unwrap();
    let tims_reader = TimsReader { reader, path };
    let out: Vec<PyFrame> = tims_reader.read_all_frames();
    Ok(out)
}

#[pymodule]
fn timsrust_pyo3(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(read_all_frames, m)?)?;
    m.add_class::<TimsReader>()?;
    m.add_class::<PyFrame>()?;
    Ok(())
}
