# Security Policy

## Supported Versions

Make sure to always use the latest version of WattSeal, as it includes the most up-to-date security patches and improvements.

## Reporting a Vulnerability

If you discover a security vulnerability in WattSeal, **please do not open a public issue**, share it to other users, or take advantage of it.

Instead, use the [vulnerabilities reporting](https://github.com/Daminoup88/WattSeal/security/advisories/new) feature and fill the form with any relevant information that you have.

We'll acknowledge your report within 48 hours and work with you to understand and address the issue. Once a fix is released, we'll credit you in the release notes (unless you prefer to stay anonymous).

## Scope

WattSeal may request elevated privileges to install a Windows CPU driver or to access Linux RAPL counters. We take this responsibility seriously.

---

## Scaphandre RAPL Driver (Windows)

On Windows, WattSeal uses **WinRing0**, a third-party signed kernel-mode driver, to read CPU Model Specific Registers (MSRs). This is currently the only mechanism available to access hardware RAPL energy counters on Windows without building a custom kernel driver (that would be flagged by Windows Defender if not signed). Precise CPU measurements are a core requirement of WattSeal – without them the application cannot fulfil its primary purpose.

### Why it exists

Reading CPU energy registers on Windows requires Ring-0 (kernel) access. The Scaphandre driver provides read-only MSR access focused on RAPL counters, avoiding the generic read/write capabilities of legacy drivers.

### Security implications

Kernel drivers run at the highest privilege level on the system. WinRing0 exposes generic MSR read/write capability, which goes beyond WattSeal's own read-only needs. This represents an elevated attack surface. While WattSeal constrains its own use of the driver, it cannot fully control what the driver exposes to other processes on the system.

WattSeal does not install WinRing0 as a permanent service. The driver is loaded on demand and its lifecycle is managed by the application. However, driver registration requires writing to the Windows registry and placing the `.sys` file on disk.

### We want to replace it

We found the scaphandre driver as an alternative: a minimal purpose-built signed driver. However, the signed version is not compatible with AMD devices. This is not acceptable for WattSeal's cross-platform goals. The fixed version is not yet signed.
Until scaphandre can fully replace it, WinRing0 remains a necessary dependency for full measurement accuracy.

### Your responsibility

By running WattSeal as administrator on Windows and accepting the UAC prompt, **you explicitly consent to loading a third-party kernel-mode driver.** You are responsible for this decision. If you are not comfortable with it, you may run WattSeal without administrator privileges – CPU power readings will fall back to estimates, but no kernel driver will be loaded.

### Installing and removing the driver

To install the driver:

1. Run `WattSeal --install-cpu-driver` **as Administrator**.

To remove the driver:

1. Run `WattSeal --uninstall-cpu-driver` **as Administrator**.

### Reporting driver-related issues

If you discover a security vulnerability related to how WattSeal loads or uses the Scaphandre driver, please report it through the process above. We treat any such report as high priority.
