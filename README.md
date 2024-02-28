# DICOMweb client & server using Rust
[![Rust](https://github.com/feliwir/DICOMweb/actions/workflows/rust.yml/badge.svg)](https://github.com/feliwir/DICOMweb/actions/workflows/rust.yml)

This is a DICOMweb server toolkit, which enables writing servers implementing the DICOMweb protocol

## Supported features

- [ ] QIDO-RS
  - [x] Support /studies, /series, /instances endpoints
  - [ ] Support includefield queryparameter
- [ ] WADO-RS (missing different representations)
  - [x] Support /metadata endpoint
  - [x] Support /frames endpoint (ignores the framelist currently)
- [ ] STOW-RS (response not valid yet)

## Planned features

- [ ] DicomWeb Retrieve via Zip (Supplement 211)