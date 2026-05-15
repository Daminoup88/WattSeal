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

On Windows, WattSeal uses the **Scaphandre RAPL driver**, a minimal signed kernel-mode driver, to read CPU Model Specific Registers (MSRs) required for RAPL energy counters. This replaces the previous generic MSR driver and reduces the exposed surface to the read-only operations WattSeal needs.

### Why it exists

Reading CPU energy registers on Windows requires Ring-0 (kernel) access. The Scaphandre driver provides read-only MSR access focused on RAPL counters, avoiding the generic read/write capabilities of legacy drivers.

### Security implications

Kernel drivers run at the highest privilege level on the system. While the Scaphandre driver is minimal and read-only, it is still privileged code. WattSeal installs it once and then accesses it from user mode; normal operation does not require elevated privileges.

### Installing and removing the driver

To install the driver:

1. Run `WattSeal --install-cpu-driver` **as Administrator**.

To remove the driver:

1. Run `WattSeal --uninstall-cpu-driver` **as Administrator**.

### Reporting driver-related issues

If you discover a security vulnerability related to how WattSeal loads or uses the Scaphandre driver, please report it through the process above. We treat any such report as high priority.
