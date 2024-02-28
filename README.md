# DICOMweb client & server using Rust
[![Rust](https://github.com/feliwir/DICOMweb/actions/workflows/rust.yml/badge.svg)](https://github.com/feliwir/DICOMweb/actions/workflows/rust.yml)

This is a DICOMweb server toolkit, which enables writing servers implementing the DICOMweb protocol

## Running

By default the server will run on "http://localhost:8080"
```
cargo run
```
### Uploading data

Use a DICOMweb client to upload / retrieve datasets. E.g. https://dicomweb-client.readthedocs.io/en/latest/usage.html#command-line-interface-cli

### Use with OHIF Viewer

- Clone [OHIF Viewer](https://github.com/OHIF/Viewers)
- Add a config file `platform/app/public/config/local_dicomweb_rs.js`. Depending on your configuration it should look like [this](https://gist.github.com/feliwir/458a63e46ccf57c41cb087dea1fa091e)
- Set `APP_CONFIG` environment variable to our config. E.g. `export APP_CONFIG=config/local_dicomweb_rs.js`
- Build OHIF Viewer like [this](https://docs.ohif.org/deployment/build-for-production/#restore-dependencies--build). 

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