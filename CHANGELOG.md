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

## 2023-12-15

* Add dynamic window size for burst limiter
  Allows for ~1gbps of lossless throughput on UDP

## 2023-12-30

* Change structure to allow for testing harness

## 2023-12-31

* Merge multicast and unicast to UDP
* Add ProtoError enum to handle Sessions
* Support sessions in UDP
