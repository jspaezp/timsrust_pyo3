
# Python bindings to the timsrust reader

This is a prototype/work in progress/proof of concept.
I am happy to take requests and ideas.

# Installation

```shell
pip install git+https://github.com/jspaezp/timsrust_pyo3
```

# Usage

```python
>>> import timsrust_pyo3
>>> all_frames = timsrust_pyo3.read_all_frames("some_file.d")
>>> all_frames[0]
PyFrame(index=1, rt=0.33491, frame_type=0, len(scan_offsets)=710, len(tof_indices)=242412, len(intensities)=242412)

>>> reader = timsrust_pyo3.TDFReader("some_file.d")
>>> all_frames = reader.read_all_frames()
>>> tfr.resolve_mzs(all_frames[0].tof_indices)
[...] # list[float]
```

Notes:
1. Frame types are:
    - 0: MS1
    - 1: MS2-DDA-PASEF
    - 2: MS2-DIA-PASEF
    - 3: UNKNOWN (either ms1 or ms2)

## Making a dense representation of the frames

```python

@dataclass
class DenseFrame:
    rt: float
    intensities: list[int]
    mzs: list[float]
    imss: list[float]

    @classmethod
    def from_frame(
        cls, frame: timsrust_pyo3.PyFrame, reader: timsrust_pyo3.TimsReader
    ):
        mzs = reader.resolve_mzs(frame.tof_indices)
        out_imss = [None] * len(mzs)
        last_so = 0
        for ims, so in zip(
            reader.resolve_scans(list(range(1, len(frame.scan_offsets) + 1))),
            frame.scan_offsets,
            strict=True,
        ):
            out_imss[last_so:so] = [ims] * (so - last_so)
            last_so = so

        return cls(
            rt=frame.rt,
            intensities=frame.intensities,
            mzs=mzs,
            imss=out_imss,
        )

file =  "my_favourite_dotd.d"
reader = timsrust_pyo3.TimsReader(file)
allframes = reader.read_all_frames()

df = DenseFrame.from_frame(allframes[0], reader)
```

## Getting the isolation window information for DIA

Right now the best way to get this is using raw sql.
We need to read the `analysis.tdf` file and do the equivalent of
a double join using the frame index from a frame, the `DiaFrameMsMsInfo` and the `DiaFrameMsMsWindows` table.


This is more or less how the `DiaFrameMsMsWindows` table looks like:

```
WindowGroup	ScanNumBegin	ScanNumEnd	IsolationMz	IsolationWidth	CollisionEnergy
1	34	370	812.5	25.0	42.8025889967638
1	370	535	612.5	25.0	32.2847896440129
1	535	708	412.5	25.0	25.1747572815534
2	34	342	837.5	25.0	43.3915857605178
2	342	517	637.5	25.0	33.252427184466
...
```

And this is how the `DiaFrameMsMsInfo` table looks like:

```
Frame	WindowGroup
2	1
3	2
4	3
5	4
6	5
```

```python
import sqlite3
from dataclasses import dataclass

@dataclass
class DiaWindow:
    group: int
    scan_begin: int
    scan_end: int
    isolation_mz: float
    isolation_width: float
    collision_energy: float

    @classmethod
    def mapping_from_sql(cls, sql_file):
        conn = sqlite3.connect(file)
        curr = conn.cursor()
        window_data = curr.execute("SELECT * FROM DiaFrameMsMsWindows").fetchall()
        info_data = curr.execute("SELECT * FROM DiaFrameMsMsInfo").fetchall()
        index_to_group = {frame: group for frame, group in info_data}

        group_to_windows = {}

        for group, *window in window_data:
            window_data = DiaWindow(group, *window)
            group_to_windows.setdefault(group, []).append(window_data)

        return index_to_group, group_to_windows

file =  "my_favourite_dotd.d"

index_to_group, group_to_windows = DiaWindow.mapping_from_sql(file + "/analysis.tdf")
reader = timsrust_pyo3.TimsReader(file)
allframes = reader.read_all_frames()
dia_frames = [f for f in allframes if f.frame_type == 2]
example_frame = dia_frames[0]

mz_ranges = {}

for w in group_to_windows[index_to_group[example_frame.index]]:
    mz_low = w.isolation_mz - w.isolation_width
    mz_high = w.isolation_mz + w.isolation_width
    matching_offsets = example_frame.scan_offsets[w.scan_begin : w.scan_end]

    scan_range = range(matching_offsets[0], matching_offsets[-1])
    mz_ranges[(mz_low, mz_high)] = scan_range

mz_ranges
# {(787.5, 837.5): range(0, 1753), (587.5, 637.5): range(1772, 3637), (387.5, 437.5): range(3638, 4487)}
# This means that the values in example_frame.intensities[0:1753] correspond to the mz range 787.5-837.5
```
