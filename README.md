# timsrust_pyo3


# Python bindings to the timsrust reader

Read bruker data files using python with the speed of rust!

``` shell
# pip install timsrust_pyo3
```

## Usage

If what you want is to read spectra from a file, you can use the
`read_all_spectra` function.

``` python
import timsrust_pyo3

datafile = "tests/data/230711_idleflow_400-1000mz_25mz_diaPasef_10sec.d"
specs = timsrust_pyo3.read_all_spectra(datafile)
```

This will return a list of `Spectrum` objects. Which are the results of
“flattening” the frames into its individual isolation windows.

``` python
specs[0]
```

    Spectrum(
     index=0,
     mz_values=[116.25158757494638, 118.9080472754818, 141.8956868151955, 167.55719727130426, 195.07915917591367, 232.07843727809686, 256.9100001557801, 267.30801795707947, 272.49521871359474, 274.26560752950985...len=1599],
     intensities=[9, 9, 9, 9, 9, 9, 9, 9, 9, 9...len=1599],
     precursor=Precursor(mz=812.5, rt=0.4161, im=1.1620169252468266, charge=None, intensity=None),
     collision_energy=42.80258899676376, isolation_mz=812.5, isolation_width=25)

This shoudl work fine for a lot of cases.

If you want a more … raw representation of the data, we can use the
`FrameReader` class.

``` python
reader = timsrust_pyo3.FrameReader(datafile)
all_frames = reader.read_all_frames()
print(all_frames[0])
print(all_frames[1])
```

    Frame(
     scan_offsets=[0, 0, 0, 0, 0, 0, 0, 0, 0, 0...len=710],
     tof_indices=[7269, 226179, 238688, 283353, 302607, 313423, 320067, 325868, 333879, 334217...len=242412],
     intensities=[20, 35, 20, 89, 45, 115, 57, 57, 113, 98...len=242412],
     index=1, rt=0.33491, acquisition_type=DIAPASEF, ms_level=MS1, quadrupole_settings=QuadrupoleSettings(index=0, scan_starts=[], scan_ends=[], isolation_mz=[], isolation_width=[], collision_energy=[]), intensity_correction_factor=0.013324805457840315)
    Frame(
     scan_offsets=[0, 0, 0, 0, 0, 0, 0, 0, 0, 0...len=710],
     tof_indices=[278272, 230453, 89438, 376113, 223326, 110881, 66872, 316096, 353458, 135560...len=4501],
     intensities=[9, 9, 9, 9, 71, 9, 9, 9, 9, 9...len=4501],
     index=2, rt=0.4161, acquisition_type=DIAPASEF, ms_level=MS2, quadrupole_settings=QuadrupoleSettings(index=1, scan_starts=[34, 370, 535], scan_ends=[370, 535, 708], isolation_mz=[812.5, 612.5, 412.5], isolation_width=[25, 25, 25], collision_energy=[42.80258899676376, 32.284789644012946, 25.174757281553397]), intensity_correction_factor=0.013324805457840315)

Note that here each frame does not have mz and ion mobility values.

lets start with the easy one … each tof index can be converted to a mz
value using the `Metadata` class.

``` python
# We point the metadata to the analysis.tdf file
metadata = timsrust_pyo3.Metadata(datafile + "/analysis.tdf")
mzs = metadata.resolve_mzs(all_frames[0].tof_indices)
print(all_frames[0].tof_indices[:5])
print(" Becomes >>>> ")
print(mzs[:5])
```

    [7269, 226179, 238688, 283353, 302607]
     Becomes >>>>
    [111.70270406113804, 767.4671439486657, 822.673531755788, 1035.4396805033862, 1134.6976413752595]

Now the harder one … ion mobility …

This is because each frame stores the mobility information by using the
`scan_offsets` … which means … all peaks from `scan_offsets[0]` to
`scan_offsets[1]` are from the same scan, and thus have the same ion
mobility. and the 1/k0 value of that scan (0) can be converted as well
by using the `Metadata` class.

``` python
scans = metadata.resolve_scans([1,2,3, 700])
print(" [1,2,3] Becomes >>>> ")
print(scans)
```

     [1,2,3] Becomes >>>>
    [1.3689703808180538, 1.3679407616361072, 1.366911142454161, 0.6492665726375176]

Note that since the tims funnel “elutes” values with larger 1/k0 first,
the first scan is actually the one with highest 1/k0.
