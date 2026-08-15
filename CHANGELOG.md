# Changelog

## [1.0.3](https://github.com/Daminoup88/WattSeal/compare/v1.0.2...v1.0.3) (2026-08-15)


### Features

* **collector:** AMD GPU initialization logs ([14d863e](https://github.com/Daminoup88/WattSeal/commit/14d863ee6cce2757bf044c2e4f70bf1d1fbd2b03))
* **collector:** Better driver error logging ([ab6ae20](https://github.com/Daminoup88/WattSeal/commit/ab6ae20bb3962df38a0dff68d43bba276cba4d79))
* **collector:** Improved runtime logging ([f881a41](https://github.com/Daminoup88/WattSeal/commit/f881a41d4d3a75e1a8ed229727878d78c2514149))
* **collector:** NVIDIA GPU initialization logs and fallback with instant power ([11840f4](https://github.com/Daminoup88/WattSeal/commit/11840f44735f05dc38a88d87e6e55a45d9a3345d))
* **gpu:** AMD GPU flexibility ([a87a1a2](https://github.com/Daminoup88/WattSeal/commit/a87a1a2699f660dea7bb7ec0878305e5f9f6184c))
* **gpu:** GPU power estimation model when only usage available (applies for iGPU as well) ([d792c5f](https://github.com/Daminoup88/WattSeal/commit/d792c5fab1ed368d1228951bd52dc70b96dac971))
* **gpu:** Support NVML power samples as fallback ([ea896e5](https://github.com/Daminoup88/WattSeal/commit/ea896e58c8fed530576b7415bdb2657b1e617131))
* **gpu:** Update GPU TDP table ([dbe3ff4](https://github.com/Daminoup88/WattSeal/commit/dbe3ff478a17b124f4ef460978632d10c63795b6))
* **i18n:** add Simplified Chinese (ZH) UI localization ([#80](https://github.com/Daminoup88/WattSeal/issues/80)) ([db43c62](https://github.com/Daminoup88/WattSeal/commit/db43c621442df949f7704af26e40b63272af02aa))
* Log downloads ([4d87277](https://github.com/Daminoup88/WattSeal/commit/4d87277f90fdcab7a34754a04ae9d91b2ed63721))
* **migration:** Previous GPU entries set to previous GPU 1 ([6a467d6](https://github.com/Daminoup88/WattSeal/commit/6a467d6141690ea7f472c445f37619dadea92caf))
* Remove fixed 5W RAM power ([919f259](https://github.com/Daminoup88/WattSeal/commit/919f259243e6c8c5bcffdd6d6778c896d8d09b39))
* **ui:** app translations Romanian ([c71ac6b](https://github.com/Daminoup88/WattSeal/commit/c71ac6b3165bed00cd352e8e76f0db5482d30cca))
* **ui:** Number formatting ([5e7a53b](https://github.com/Daminoup88/WattSeal/commit/5e7a53bb5272e1dea930e0a48d50d08dc934c863))
* **ui:** Prices in local currency ([ba35a6a](https://github.com/Daminoup88/WattSeal/commit/ba35a6aa6bbe68cb7029795cd738142fe576a019))
* **ui:** Translate settings picklists ([ac6408d](https://github.com/Daminoup88/WattSeal/commit/ac6408d102ba5be78c33eceb4f661f3583451abc))


### Bug Fixes

* **ci:** Add explicit permissions ([29d4eea](https://github.com/Daminoup88/WattSeal/commit/29d4eeab8a849a5448c981361f7736c9b9be8261))
* **collector:** AMD RAPL counter on 64 bits ([4b1784b](https://github.com/Daminoup88/WattSeal/commit/4b1784b8b493c769976001df4809f319fab92a4a))
* **collector:** Double admin pop up if denied ([a0a69bc](https://github.com/Daminoup88/WattSeal/commit/a0a69bc403660ab2f5204b3700b5735457385a36))
* **collector:** Info is uncountable ([a6f5706](https://github.com/Daminoup88/WattSeal/commit/a6f57060582f252204c2f48766489889bc39aa05))
* **collector:** PP0, PP1 and DRAM are now optional ([4116bcd](https://github.com/Daminoup88/WattSeal/commit/4116bcd2e9fc41cfb98fa55dbd1b719f309520df))
* **cpu:** Correct MSR DRAM address ([90ef87e](https://github.com/Daminoup88/WattSeal/commit/90ef87e9ad09c1a48407bbc9f65e7498864072c3))
* Linux `.desktop` file ([47b2908](https://github.com/Daminoup88/WattSeal/commit/47b2908426b2acd24dc9e002b5e263d8ad2567ee))
* **ui:** Apply number formatting ([270bff5](https://github.com/Daminoup88/WattSeal/commit/270bff501d80b62598f2b196a96323efb1ce4efb))
* **ui:** If several GPUs, display the sum ([38e4cbe](https://github.com/Daminoup88/WattSeal/commit/38e4cbe91cccefcc75b6f7f5d60f5eee167015a9))
* **ui:** Report issue link ([ea84299](https://github.com/Daminoup88/WattSeal/commit/ea842994832e2f1c731ca8133238ed551d633c1e))
* **ui:** Small settings alignment fix ([4aa34c0](https://github.com/Daminoup88/WattSeal/commit/4aa34c0c168fc22db0ba95a2d3db7d2b0ba67b17))
* **ui:** Variable duration based power also in batched update ([49dfcdf](https://github.com/Daminoup88/WattSeal/commit/49dfcdf50e41a62be976828c7380edd8bc7b2f43))


### Reverts

* **collector:** Go back to Winring0 ([a578dc7](https://github.com/Daminoup88/WattSeal/commit/a578dc7dbc1dc1a6c506564845196b7128d91985))
