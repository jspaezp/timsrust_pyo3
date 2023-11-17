
# Python bindings to the timsrust reader

This is a prototype/work in progress/proof of concept.
I am happy to take requests and ideas.

# Installation

```
pip install git+https://github.com/jspaezp/timsrust_pyo3
```

# Usage

```
>>> import timsrust_pyo3
>>> all_frames = timsrust_pyo3.read_all_frames("some_file.d")
>>> all_frames[0]
PyFrame(index=1, rt=0.33491, frame_type=0, len(scan_offsets)=710, len(tof_indices)=242412, len(intensities)=242412)

>>> reader = timsrust_pyo3.TDFReader("some_file.d")
>>> all_frames = reader.read_all_frames()
>>> tfr.resolve_mzs(all_frames[0].tof_indices)
[...] # list[float]
```
