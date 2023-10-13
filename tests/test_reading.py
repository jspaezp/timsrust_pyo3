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


def test_mz_resolution(shared_datadir):
    file = str(shared_datadir / "230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d")

    reader = timsrust_pyo3.TDFReader(file)
    allframes = reader.read_all_frames()
    resolved = reader.resolve_mzs(allframes[0].tof_indices())
    assert len(resolved) == 242412
    assert all(isinstance(mz, float) for mz in resolved)
