# wmparc
White matter parcellation as proposed in "Cortex parcellation associated whole white matter parcellation in individual subjects".

Build programm with `cargo build --release`

Usage: {} trk-file [options]
  trk-file      Fibertracking results in TrackVis format. Tracts have to be in the same space as the cortex parcellation image.
  
  options:
  -n, --nifti   Path to nifti image that represents the cortex parcellation [required].
  -o, --output  Path to the output file [optional].
  -h, --help    Print the help menu.
