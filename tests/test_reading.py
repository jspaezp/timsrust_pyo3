import timsrust_pyo3


def test_read_all_frames(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    all_frames = timsrust_pyo3.read_all_frames(file)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in all_frames)


def test_file_reader(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")
    reader = timsrust_pyo3.TDFReader(file)
    all_frames = reader.read_all_frames()
    all_frames2 = timsrust_pyo3.read_all_frames(file)

    assert len(all_frames) == len(all_frames2)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in all_frames)

    dia_frames = reader.read_dia_frames()
    assert all(f.frame_type() == 2 for f in dia_frames)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in dia_frames)
    assert len(dia_frames) < len(all_frames)

    ms1_frames = reader.read_ms1_frames()
    assert all(f.frame_type() == 0 for f in ms1_frames)
    assert all(isinstance(f, timsrust_pyo3.PyFrame) for f in ms1_frames)
    assert len(ms1_frames) < len(all_frames)
    assert len(ms1_frames) < len(dia_frames)

    dia_frame_table = reader.dia_frame_table()
    assert set(["group", "frame"]) == set(dia_frame_table)

    msms_windows = reader.dia_frame_msms_windows()
    assert isinstance(msms_windows, dict)
    assert all(len(msms_windows["group"]) == len(w) for w in msms_windows.values())
    assert set(msms_windows.keys()) == set(
        ["group", "scan_start", "scan_end", "mz_center", "mz_width"]
    )


def test_mz_resolution(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")

    reader = timsrust_pyo3.TDFReader(file)
    allframes = reader.read_all_frames()
    resolved = reader.resolve_mzs(allframes[0].tof_indices())
    assert len(resolved) == 242412
    assert all(isinstance(mz, float) for mz in resolved)
