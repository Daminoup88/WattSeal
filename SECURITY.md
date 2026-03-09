# Security Policy

## Supported Versions

Make sure to always use the latest version of WattSeal, as it includes the most up-to-date security patches and improvements.

## Reporting a Vulnerability

If you discover a security vulnerability in WattSeal, **please do not open a public issue**, share it to other users, or take advantage of it.

Instead, use the [vulnerabilities reporting](https://github.com/Daminoup88/WattSeal/security/advisories/new) feature and fill the form with any relevant information that you have.

We'll acknowledge your report within 48 hours and work with you to understand and address the issue. Once a fix is released, we'll credit you in the release notes (unless you prefer to stay anonymous).

## Scope

WattSeal runs with elevated privileges (admin/root) to access hardware energy counters. We take this responsibility seriously.

---

## WinRing0 Kernel Driver (Windows)

On Windows, WattSeal uses **WinRing0**, a third-party signed kernel-mode driver, to read CPU Model Specific Registers (MSRs). This is currently the only mechanism available to access hardware RAPL energy counters on Windows without building a custom kernel driver (that would be flagged by Windows Defender if not signed). Precise CPU measurements are a core requirement of WattSeal — without them the application cannot fulfil its primary purpose.

### Why it exists

Reading CPU energy registers on Windows requires Ring-0 (kernel) access. WinRing0 is the only widely available signed driver that provides this without requiring test-signing mode. WattSeal uses it in a strictly read-only, targeted manner.

### Security implications

Kernel drivers run at the highest privilege level on the system. WinRing0 exposes generic MSR read/write capability, which goes beyond WattSeal's own read-only needs. This represents an elevated attack surface. While WattSeal constrains its own use of the driver, it cannot fully control what the driver exposes to other processes on the system.

WattSeal does not install WinRing0 as a permanent service. The driver is loaded on demand and its lifecycle is managed by the application. However, driver registration requires writing to the Windows registry and placing the `.sys` file on disk.

### We want to replace it

We are actively seeking a safer alternative — a minimal purpose-built signed driver or an official Windows API. **Contributions and suggestions toward this goal are very welcome.** Until a replacement exists, WinRing0 remains a necessary dependency for full measurement accuracy.

### Your responsibility

By running WattSeal as administrator on Windows and accepting the UAC prompt, **you explicitly consent to loading a third-party kernel-mode driver.** You are responsible for this decision. If you are not comfortable with it, you may run WattSeal without administrator privileges — CPU power readings will fall back to estimates, but no kernel driver will be loaded.

### Removing WinRing0

If WinRing0 has been loaded by WattSeal and you wish to remove it entirely:

1. Open PowerShell **as Administrator**.
2. Stop and remove the service:
   ```powershell
   sc.exe stop WinRing0_1_2_0
   sc.exe delete WinRing0_1_2_0
   ```
3. **Reboot** to fully unload the driver from kernel memory.

### Reporting WinRing0-related issues

If you discover a security vulnerability related to how WattSeal loads or uses WinRing0, or if you know of a safer alternative that could replace it, please report it through the process above or open a GitHub discussion. We treat any such report as high priority.
