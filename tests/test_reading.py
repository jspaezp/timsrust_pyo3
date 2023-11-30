from dataclasses import dataclass
import sqlite3
import timsrust_pyo3


def test_read_all_frames(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    all_frames = timsrust_pyo3.read_all_frames(file)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in all_frames)


def test_file_reader(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    reader = timsrust_pyo3.TimsReader(file)
    all_frames = reader.read_all_frames()
    all_frames2 = timsrust_pyo3.read_all_frames(file)

    assert len(all_frames) == len(all_frames2)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in all_frames)

    dia_frames = reader.read_dia_frames()
    assert all(f.frame_type == 2 for f in dia_frames if f.intensities)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in dia_frames)
    assert len(dia_frames) == len(all_frames)

    ms1_frames = reader.read_ms1_frames()
    assert all(f.frame_type == 0 for f in ms1_frames if f.intensities)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in ms1_frames)
    assert len(ms1_frames) == len(all_frames)
    assert len(ms1_frames) == len(dia_frames)


def test_dda_file_reading(shared_datadir):
    file = str(shared_datadir / "dda_test.d")
    reader = timsrust_pyo3.TimsReader(file)
    specs = reader.read_all_spectra()
    assert len(specs) > 0


def test_mz_resolution(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")

    reader = timsrust_pyo3.TimsReader(file)
    allframes = reader.read_all_frames()
    resolved = reader.resolve_mzs(allframes[0].tof_indices)
    assert len(resolved) == 242412
    assert all(isinstance(mz, float) for mz in resolved)


def test_frame_converters(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    reader = timsrust_pyo3.TimsReader(file)
    allframes = reader.read_all_frames()

    resolved_mzs = reader.resolve_mzs(allframes[0].tof_indices)

    assert len(resolved_mzs) == 242412
    assert all(isinstance(mz, float) for mz in resolved_mzs)

    resolved_scans = reader.resolve_scans(
        list(range(1, len(allframes[0].scan_offsets) + 1))
    )
    assert len(resolved_scans) == 710
    assert resolved_scans[0] >= 1.36
    assert resolved_scans[0] <= 1.37

    assert resolved_scans[-1] >= 0.81
    assert resolved_scans[-1] <= 0.82


def test_flattening(shared_datadir):
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

    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    reader = timsrust_pyo3.TimsReader(file)
    allframes = reader.read_all_frames()

    _ = DenseFrame.from_frame(allframes[0], reader)
    dense_frames = [DenseFrame.from_frame(f, reader) for f in allframes]

    for f in dense_frames:
        assert len(f.intensities) == len(f.mzs)
        assert len(f.intensities) == len(f.imss)
        assert all(
            isinstance(i, int) for i in f.intensities
        ), f"Not all intensities are ints ({set([type(i) for i in f.intensities])})"
        assert all(isinstance(m, float) for m in f.mzs), "Not all mzs are floats"
        assert all(isinstance(i, float) for i in f.imss), "Not all imss are floats"
        assert all(i >= 0 for i in f.intensities)


def test_dia_info_mapping(shared_datadir):
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
            conn = sqlite3.connect(sql_file)
            curr = conn.cursor()
            window_data = curr.execute("SELECT * FROM DiaFrameMsMsWindows").fetchall()
            info_data = curr.execute("SELECT * FROM DiaFrameMsMsInfo").fetchall()
            index_to_group = {frame: group for frame, group in info_data}

            group_to_windows = {}

            for group, *window in window_data:
                window_data = DiaWindow(group, *window)
                group_to_windows.setdefault(group, []).append(window_data)

            return index_to_group, group_to_windows

    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    index_to_group, group_to_windows = DiaWindow.mapping_from_sql(
        file + "/analysis.tdf"
    )

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

    ## This tests that the scan ranges are correct
    for mz_range, scan_range in mz_ranges.items():
        example_frame.intensities[scan_range[0] : scan_range[-1]]
        assert (len(scan_range) - 1) == len(
            example_frame.intensities[scan_range[0] : scan_range[-1]]
        )
