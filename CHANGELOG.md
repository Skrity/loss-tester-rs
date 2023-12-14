# CHANGELOG

## 2023-12-13

* Fixed TCP implementation: no false positives on losses
* Added COBS encoding
* Moved buffers inside frame handling structures
* Removed length checking
* Allow unlimited speed in StaticLimiter via `-b 0` (mostly useful for TCP)

## 2023-12-14

* Implement burst limiter:

  * uses less amount of sleep primitive
  * allows for serial TCP bitrate of 6gbps+
  * provides more efficient CPU usage
  * pushes more exact bitrate
