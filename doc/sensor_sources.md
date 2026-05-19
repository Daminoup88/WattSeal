# Sensor sources

The sources of the metrics collected and estimated by WattSeal for every sensor and OS:

1. CPU Metrics

  Windows
   * Library: scaphandre-driver-rs (custom Rust wrapper for the Scaphandre driver).
   * Driver: Scaphandre RAPL Driver (requires admin rights once for installation).
   * Source: Model-Specific Registers (MSR).
       * Intel: Reads MSR_PKG_ENERGY_STATUS (0x611), MSR_PP0_ENERGY_STATUS (0x639), MSR_PP1_ENERGY_STATUS (0x641), and
         MSR_DRAM_ENERGY_STATUS (0x618) ([Intel 64 and IA-32 Architectures Software Developer’s Manual, Volume 3B, Section 14.9](https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3b-part-2-manual.pdf)).
       * AMD: Reads ENERGY_PKG_MSR (0xC001029B) and ENERGY_CORE_MSR (0xC001029A) ([AMD E-SMI Documentation](https://rocm.docs.amd.com/projects/amdsmi/en/docs-6.0.0/esmi_lib_readme_link.html)).
   * Energy counter conversion:
       * The MSR energy registers return a raw counter value (64-bit from EDX:EAX with only 32 bits used).
       * Read the energy unit from MSR_RAPL_POWER_UNIT (Intel) or ENERGY_PWR_UNIT_MSR (AMD):
         `energy_unit = 1 / 2^energy_unit_raw` where `energy_unit_raw` is bits 8-12 of the MSR ([Intel manual chapter 14.9.1]((https://www.intel.com/content/dam/www/public/us/en/documents/manuals/64-ia-32-architectures-software-developer-vol-3b-part-2-manual.pdf))).
       * Convert the raw counter to energy:
         `energy_joules = raw_energy_counter * energy_unit`.
       * The measurement uses two samples and takes the difference:
         `delta_energy_joules = energy_joules_current - energy_joules_previous`.
   * Calculation:
       * The energy is extracted per RAPL domain (pkg, pp0, pp1, dram) by reading:
         * Intel: MSR_PKG_ENERGY_STATUS, MSR_PP0_ENERGY_STATUS, MSR_PP1_ENERGY_STATUS, MSR_DRAM_ENERGY_STATUS.
         * AMD: ENERGY_PKG_MSR, ENERGY_CORE_MSR.

  Linux
   * Library: Standard file system access (needs sudo).
   * Source: [Linux Kernel powercap framework documentation](https://www.kernel.org/doc/html/next/power/powercap/powercap.html).
       * Path: /sys/class/powercap/intel-rapl:0/energy_uj.
   * Calculation: Directly reads microjoules ($\mu J$) from the sysfs interface and calculates the power over the measurement interval.

  Fallback (OS without RAPL support or insufficient permissions, currently always on macOS)
   * Library: sysinfo (for CPU usage).
   * Source: Hardcoded TDP (Thermal Design Power) lookup table in estimation.rs.
   * Calculation: Non-linear power estimation $P = P_{idle} + (P_{peak} - P_{idle}) \times \text{usage}^{1.6}$ where usage is the percentage of CPU utilization, TDP_idle is assumed to be 20% and TDP_peak 125% of TDP (coefficients based on measurements on laptops and non-linear model from a [Google article on server power estimation](https://dl.acm.org/doi/epdf/10.1145/1273440.1250665)).

  ---

  2. GPU Metrics (snapshot for now on AMD and NVIDIA, energy counter available on NVIDIA)

  NVIDIA (Windows & Linux)
   * Library: nvml-wrapper.
   * Source: NVIDIA Management Library (NVML).
   * Metrics: Direct power usage in milliwatts (mW) (should switch to energy), GPU utilization, and Memory (VRAM) utilization.

  AMD (Windows only)
   * Library: adlx.
   * Source: AMD Display Library X (ADLX).
   * Metrics: Accesses the "Performance Monitoring Services" to retrieve real-time power (mW) and usage statistics.

  Intel (Windows only)
   * Library: Win32 Performance Data Helper (PDH).
   * Source: Windows Performance Counter: \\GPU Engine(*)\\Utilization Percentage. Power consumption comes from PP1.

  ---

  3. RAM Metrics
   * Library: sysinfo.
   * Power Estimation: Fixed at a constant 5W for the entire memory bank, should be per-stick constant or DRAM ([Scaphandre doc](https://hubblo-org.github.io/scaphandre-documentation/explanations/rapl-domains.html)).

  ---

  4. Disk Metrics
   * Library: sysinfo, real read/written bytes during the sampling period.
   * Source: Disk I/O throughput counters.
   * Power Estimation:
       * SSD: $0.05W (\text{idle}) + (\text{Throughput MB/s} \times 0.015)$
       * HDD: $3.00W (\text{idle}) + (\text{Throughput MB/s} \times 0.035)$
       * Unknown: $0.30W (\text{idle}) + (\text{Throughput MB/s} \times 0.02)$
       Based on throughput and disk type (SSD vs HDD), arbitrary coefficients, should be refined, [A Comparative Study of HDD and SSD RAIDs' Impact on Server Energy Consumption](https://par.nsf.gov/servlets/purl/10050305)

  ---

  5. Network Metrics
   * Library: sysinfo, real sent/received bytes during the sampling period.
   * Source: Network interface throughput counters.
   * Power Estimation:
       * Formula: $0.2W (\text{idle}) + (\text{Throughput MB/s} \times 0.01)$
       Capped at 3W per interface, should be refined with more data and sources, the CPU (already measured) accounts for the vast majority of the consumption under network load for packet processing: [Understanding Power Efficiency of TCP/IP Packet Processing over 10GbE](https://www.researchgate.net/publication/228358314_Understanding_Power_Efficiency_of_TCPIP_Packet_Processing_over_10GbE).

  ---

  6. Process Attribution

   * Library: sysinfo (for per-process CPU/RAM/Disk usage).
   * Logic:
       * CPU Share: $\frac{\text{Process CPU Usage}}{\text{Total CPU Usage}} \times \text{Total CPU Power}$.
       * GPU Share: $\frac{\text{Process GPU Usage}}{\text{Total GPU Usage}} \times \text{Total GPU Power}$.
       * Final Attribution: The process power is the sum of its attributed compute power (CPU + GPU), weighted
          against the total system power. Custom calculation to encourage responsible usage: if only one process is using the CPU and GPU, it is attributed 100% of the power consumption.
