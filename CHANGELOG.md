# CHANGELOG

## 2023-12-13

* Fixed TCP implementation: no false positives on losses
* Added COBS encoding
* Moved buffers inside frame handling structures
* Removed length checking
* Allow unlimited speed in StaticLimiter via `-b 0` (mostly useful for TCP)
