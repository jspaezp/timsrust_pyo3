use std::sync::Arc;

use pyo3::exceptions::PyIOError;
use pyo3::types::PyString;
use std::path::PathBuf;

use crate::timsrust_converters::{PyFrame2RtConverter, PyScan2ImConverter, PyTof2MzConverter};
use crate::timsrust_enums::{PyAcquisitionType, PyMSLevel};
use pyo3::prelude::*;
use std::fmt::Display;
use timsrust::converters::ConvertableDomain;
use timsrust::readers::MetadataReader;
use timsrust::QuadrupoleSettings;
use timsrust::{Frame, Metadata, Precursor, Spectrum};

#[derive(Clone, Debug, PartialEq)]
#[pyclass(name = "QuadrupoleSettings")]
pub struct PyQuadrupoleSettings {
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub scan_starts: Vec<usize>,
    #[pyo3(get)]
    pub scan_ends: Vec<usize>,
    #[pyo3(get)]
    pub isolation_mz: Vec<f64>,
    #[pyo3(get)]
    pub isolation_width: Vec<f64>,
    #[pyo3(get)]
    pub collision_energy: Vec<f64>,
}

impl Display for PyQuadrupoleSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "QuadrupoleSettings(index={}, scan_starts={}, scan_ends={}, isolation_mz={}, isolation_width={}, collision_energy={})",
            self.index,
            format_slice(&self.scan_starts),
            format_slice(&self.scan_ends),
            format_slice(&self.isolation_mz),
            format_slice(&self.isolation_width),
            format_slice(&self.collision_energy),
        )
    }
}

#[pymethods]
impl PyQuadrupoleSettings {
    pub fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!(
            "{}(index={}, scan_starts={}, scan_ends={}, isolation_mz={}, isolation_width={}, collision_energy={})",
            class_name,
            slf.borrow().index,
            format_slice(&slf.borrow().scan_starts),
            format_slice(&slf.borrow().scan_ends),
            format_slice(&slf.borrow().isolation_mz),
            format_slice(&slf.borrow().isolation_width),
            format_slice(&slf.borrow().collision_energy),
        ))
    }
}

impl From<&QuadrupoleSettings> for PyQuadrupoleSettings {
    fn from(x: &QuadrupoleSettings) -> Self {
        PyQuadrupoleSettings {
            index: x.index,
            scan_starts: x.scan_starts.to_owned(),
            scan_ends: x.scan_ends.to_owned(),
            isolation_mz: x.isolation_mz.to_owned(),
            isolation_width: x.isolation_width.to_owned(),
            collision_energy: x.collision_energy.to_owned(),
        }
    }
}

impl From<Arc<QuadrupoleSettings>> for PyQuadrupoleSettings {
    fn from(x: Arc<QuadrupoleSettings>) -> Self {
        PyQuadrupoleSettings {
            index: x.index,
            scan_starts: x.scan_starts.to_owned(),
            scan_ends: x.scan_ends.to_owned(),
            isolation_mz: x.isolation_mz.to_owned(),
            isolation_width: x.isolation_width.to_owned(),
            collision_energy: x.collision_energy.to_owned(),
        }
    }
}

#[pyclass(name = "Frame")]
pub struct PyFrame {
    #[pyo3(get)]
    pub scan_offsets: Vec<usize>,
    #[pyo3(get)]
    pub tof_indices: Vec<u32>,
    #[pyo3(get)]
    pub intensities: Vec<u32>,
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub rt: f64,
    #[pyo3(get)]
    pub acquisition_type: PyAcquisitionType,
    #[pyo3(get)]
    pub ms_level: PyMSLevel,
    #[pyo3(get)]
    pub quadrupole_settings: PyQuadrupoleSettings,
    #[pyo3(get)]
    pub intensity_correction_factor: f64,
}

impl From<Frame> for PyFrame {
    fn from(frame: Frame) -> Self {
        let acquisition_type = PyAcquisitionType::from(&frame.acquisition_type);
        let ms_level = PyMSLevel::from(&frame.ms_level);
        let quadrupole_settings = PyQuadrupoleSettings::from(frame.quadrupole_settings.clone());
        PyFrame {
            scan_offsets: frame.scan_offsets,
            tof_indices: frame.tof_indices,
            intensities: frame.intensities,
            index: frame.index,
            rt: frame.rt,
            acquisition_type,
            ms_level,
            quadrupole_settings,
            intensity_correction_factor: frame.intensity_correction_factor,
        }
    }
}

#[pymethods]
impl PyFrame {
    pub fn __repr__(&self) -> String {
        let start_section = format!(
            "index={}, rt={}, acquisition_type={}, ms_level={}, quadrupole_settings={}, intensity_correction_factor={}",
            self.index,
            self.rt,
            self.acquisition_type,
            self.ms_level,
            self.quadrupole_settings,
            self.intensity_correction_factor,
        );
        let arr_section = format!(
            "\n scan_offsets={},\n tof_indices={},\n intensities={}",
            format_slice(&self.scan_offsets),
            format_slice(&self.tof_indices),
            format_slice(&self.intensities),
        );
        format!("Frame({arr_section},\n {start_section})")
    }

    fn get_corrected_intensities(&self) -> Vec<f64> {
        self.intensities
            .iter()
            .map(|x| *x as f64 * self.intensity_correction_factor)
            .collect()
    }
}

#[pyclass(name = "Spectrum")]
pub struct PySpectrum {
    #[pyo3(get)]
    pub mz_values: Vec<f64>,
    #[pyo3(get)]
    pub intensities: Vec<f64>,
    #[pyo3(get)]
    pub precursor: Option<PyPrecursor>,
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub collision_energy: f64,
    #[pyo3(get)]
    pub isolation_mz: f64,
    #[pyo3(get)]
    pub isolation_width: f64,
}

impl From<Spectrum> for PySpectrum {
    fn from(spectrum: Spectrum) -> Self {
        PySpectrum {
            mz_values: spectrum.mz_values,
            intensities: spectrum.intensities,
            precursor: match &spectrum.precursor {
                Some(x) => Some(PyPrecursor::new(x)),
                None => None,
            },
            index: spectrum.index,
            collision_energy: spectrum.collision_energy,
            isolation_mz: spectrum.isolation_mz,
            isolation_width: spectrum.isolation_width,
        }
    }
}

#[pymethods]
impl PySpectrum {
    pub fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!(
            "{}(\n index={},\n mz_values={},\n intensities={},\n precursor={},\n collision_energy={}, isolation_mz={}, isolation_width={})",
            class_name,
            slf.borrow().index,
            format_slice(&slf.borrow().mz_values),
            format_slice(&slf.borrow().intensities),
            match &slf.borrow().precursor {
                Some(x) => format!("{}", x),
                None => "None".to_string(),
            },
            slf.borrow().collision_energy,
            slf.borrow().isolation_mz,
            slf.borrow().isolation_width,
        ))
    }
}

#[pyclass(name = "Metadata")]
pub struct PyMetadata {
    #[pyo3(get)]
    pub path: PathBuf,
    #[pyo3(get)]
    pub rt_converter: PyFrame2RtConverter,
    #[pyo3(get)]
    pub im_converter: PyScan2ImConverter,
    #[pyo3(get)]
    pub mz_converter: PyTof2MzConverter,
    #[pyo3(get)]
    pub compression_type: u8,
    #[pyo3(get)]
    pub lower_rt: f64,
    #[pyo3(get)]
    pub upper_rt: f64,
    #[pyo3(get)]
    pub lower_im: f64,
    #[pyo3(get)]
    pub upper_im: f64,
    #[pyo3(get)]
    pub lower_mz: f64,
    #[pyo3(get)]
    pub upper_mz: f64,
}

impl From<&Metadata> for PyMetadata {
    fn from(x: &Metadata) -> Self {
        PyMetadata {
            path: x.path.to_owned(),
            rt_converter: PyFrame2RtConverter::from(&x.rt_converter),
            im_converter: PyScan2ImConverter::from(&x.im_converter),
            mz_converter: PyTof2MzConverter::from(&x.mz_converter),
            compression_type: x.compression_type,
            lower_rt: x.lower_rt,
            upper_rt: x.upper_rt,
            lower_im: x.lower_im,
            upper_im: x.upper_im,
            lower_mz: x.lower_mz,
            upper_mz: x.upper_mz,
        }
    }
}

#[pymethods]
impl PyMetadata {
    #[new]
    pub fn new(path: PathBuf) -> PyResult<Self> {
        let reader = MetadataReader::new(&path).map_err(|e| PyIOError::new_err(e.to_string()))?;
        Ok(PyMetadata::from(&reader))
    }

    pub fn __repr__(&self) -> String {
        format!("Metadata(path='{}')", self.path.to_str().unwrap_or("None"))
    }

    fn resolve_mzs(&self, tofs: Vec<u32>) -> Vec<f64> {
        tofs.iter().map(|x| self.mz_converter.convert(*x)).collect()
    }

    fn invert_mzs(&self, mzs: Vec<f64>) -> Vec<u32> {
        mzs.iter()
            .map(|x| self.mz_converter.invert(*x) as u32)
            .collect()
    }

    fn resolve_scans(&self, ims: Vec<u32>) -> Vec<f64> {
        ims.iter().map(|x| self.im_converter.convert(*x)).collect()
    }

    fn invert_scans(&self, ims: Vec<f64>) -> Vec<u32> {
        ims.iter()
            .map(|x| self.im_converter.invert(*x) as u32)
            .collect()
    }

    fn resolve_frames(&self, rts: Vec<u32>) -> Vec<f64> {
        rts.iter().map(|x| self.rt_converter.convert(*x)).collect()
    }

    fn invert_frames(&self, rts: Vec<f64>) -> Vec<u32> {
        rts.iter()
            .map(|x| self.rt_converter.invert(*x) as u32)
            .collect()
    }
}

#[derive(Clone)]
#[pyclass(name = "Precursor")]
pub struct PyPrecursor {
    #[pyo3(get)]
    pub mz: f64,
    #[pyo3(get)]
    pub rt: f64,
    #[pyo3(get)]
    pub im: f64,
    #[pyo3(get)]
    pub charge: Option<usize>,
    #[pyo3(get)]
    pub intensity: Option<f64>,
    #[pyo3(get)]
    pub index: usize,
    #[pyo3(get)]
    pub frame_index: usize,
}

impl PyPrecursor {
    fn new(precursor: &Precursor) -> Self {
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
}

impl Display for PyPrecursor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Precursor(mz={}, rt={}, im={}, charge={}, intensity={})",
            self.mz,
            self.rt,
            self.im,
            match self.charge {
                Some(x) => format!("{}", x),
                None => "None".to_string(),
            },
            match self.intensity {
                Some(x) => format!("{}", x),
                None => "None".to_string(),
            }
        )
    }
}

#[pymethods]
impl PyPrecursor {
    pub fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!(
            "{}(mz={}, rt={}, im={}, charge={}, intensity={})",
            class_name,
            slf.borrow().mz,
            slf.borrow().rt,
            slf.borrow().im,
            match slf.borrow().charge {
                Some(x) => format!("{}", x),
                None => "None".to_string(),
            },
            match slf.borrow().intensity {
                Some(x) => format!("{}", x),
                None => "None".to_string(),
            },
        ))
    }
}

fn format_slice<T>(slc: &[T]) -> String
where
    T: Display,
{
    if slc.len() <= 10 {
        format!(
            "[{}]",
            slc.iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    } else {
        format!(
            "[{}...len={}]",
            slc[..10]
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", "),
            slc.len()
        )
    }
}
