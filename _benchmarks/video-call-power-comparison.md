---
layout: post
title: "Which Video Call App Kills Your Battery Fastest?"
description: "Real 1-hour power measurements of Teams, Google Meet, Discord, and Slack on Windows — ranked in watts, with the one game changer setting."
date: 2026-03-19
author: "Damien PHILIPPE"
tags: [power-consumption, teams, discord, google-meet, slack, electricity, benchmark, windows, video-call]
keywords: "video call power consumption, Google Meet power usage, Teams power consumption, Discord electricity use, Slack power draw, compare video conferencing app watts, WattSeal benchmark, video call battery drain"
image: /og-image.webp
image_alt: "Video conferencing app power consumption ranking measured with WattSeal"
breadcrumbs: false
---

<style>
.bench-table {
  width: 100%;
  border-collapse: collapse;
  margin: 1.5rem 0;
  font-size: 0.95rem;
}
.bench-table th {
  background: #1e1e2e;
  color: #cdd6f4;
  padding: 0.75rem 1rem;
  text-align: left;
  font-weight: 600;
  border-bottom: 2px solid #45475a;
  white-space: nowrap;
}
.bench-table td {
  padding: 0.65rem 1rem;
  border-bottom: 1px solid #313244;
  color: #cdd6f4;
}
.bench-table tr:last-child td { border-bottom: none; }
.bench-table tr:hover td { background: #1e1e2e; }
.bench-table .row-red    td { background: rgba(243,139,168,.08); }
.bench-table .row-orange td { background: rgba(250,179,135,.06); }
.bench-table .row-yellow td { background: rgba(249,226,175,.05); }
.bench-table .row-green  td { background: rgba(166,227,161,.06); }
.bench-table .row-idle   td { background: rgba(88,91,112,.15); color: #7f849c; }
.bar-wrap { display:flex; align-items:center; gap:.6rem; }
.bar { height:10px; border-radius:4px; display:inline-block; min-width:4px; }
.bar-red    { background:#f38ba8; }
.bar-orange { background:#fab387; }
.bar-yellow { background:#f9e2af; }
.bar-green  { background:#a6e3a1; }
.bar-idle   { background:#585b70; }
.watt-val   { font-weight:700; white-space:nowrap; }
.note-box {
  background: #1e1e2e;
  border-left: 3px solid #89b4fa;
  padding: 0.9rem 1.2rem;
  margin: 1.5rem 0;
  border-radius: 0 6px 6px 0;
  font-size: 0.92rem;
  color: #bac2de;
}
.tip-box {
  background: #1e1e2e;
  border-left: 3px solid #a6e3a1;
  padding: 0.9rem 1.2rem;
  margin: 1.5rem 0;
  border-radius: 0 6px 6px 0;
  color: #cdd6f4;
}
.tip-box ul {
  margin: 0.6rem 0 0 0;
  padding-left: 1.2rem;
}
.tip-box li {
  margin-bottom: 0.45rem;
  line-height: 1.5;
}
.jump-wrap {
  text-align: center;
  margin: 0.5rem 0 1.5rem;
}
.jump-link {
  display: inline-block;
  padding: 0.55rem 1.3rem;
  background: #313244;
  color: #cdd6f4 !important;
  border-radius: 6px;
  font-weight: 600;
  text-decoration: none !important;
  font-size: 0.95rem;
  border: 1px solid #45475a;
  transition: background 0.15s;
}
.jump-link:hover { background: #45475a; color: #cdd6f4 !important; }
</style>

Your laptop is on video calls for hours every day. Have you ever wondered what that's actually costing in electricity, and whether switching apps would make any difference?

This benchmark answers the core question: which video call app kills your battery fastest and by how many watts.

<div class="jump-wrap">
  <a href="#results" class="jump-link">⚡ Jump straight to the results</a>
</div>

We ran every major video conferencing app through identical 1-hour calls and measured the real hardware power draw using [WattSeal](https://wattseal.com). The results reveal a clear outlier, one setting change that beats switching apps entirely, and a genuinely surprising winner at the efficient end.

---

## How we measured

**Hardware:** Gaming laptop, Intel Core i7-8750H, NVIDIA GTX 1060, Windows 11. Fast connection (~1 Gbps) so every app ran at full quality.

**Conditions:** 1-hour 1-on-1 video call, camera and mic on, no background effects, all default settings. Machine rebooted before each session.

**Tool:** [WattSeal](https://wattseal.com) reads directly from hardware energy counters (CPU RAPL + GPU NVML), with per-process attribution — so we know exactly how many watts each app is responsible for, not just the machine total.

**Attribution:** Each app's power includes its own process plus everything it wakes up: the Windows audio engine, the display compositor, embedded browser engines, and the OS services driven by the call. We subtract the stable background activity (tools, idle OS processes) that runs regardless. That background measured at **8.4 W ± 0.05 W across all seven sessions**, confirming the machine's baseline was rock-steady throughout.

**Idle baseline:** Firefox open with no tabs, no call running — 5.72 W attributed. That's the floor everything else is compared against.

All raw data and procedures are on [GitHub](https://github.com/Daminoup88/WattSeal-benchmark).

---

## Results {#results}

<table class="bench-table">
  <thead>
    <tr>
      <th>App</th>
      <th>Attributed power</th>
      <th>vs. idle (5.72 W)</th>
      <th></th>
    </tr>
  </thead>
  <tbody>
    <tr class="row-red">
      <td>🔴 Google Meet — default</td>
      <td class="watt-val">50.08 W</td>
      <td>+44.4 W</td>
      <td><div class="bar-wrap"><div class="bar bar-red" style="width:170px"></div></div></td>
    </tr>
    <tr class="row-orange">
      <td>🟠 Google Meet — reframing <strong>OFF</strong></td>
      <td class="watt-val">41.06 W</td>
      <td>+35.3 W</td>
      <td><div class="bar-wrap"><div class="bar bar-orange" style="width:139px"></div></div></td>
    </tr>
    <tr class="row-yellow">
      <td>🟡 Slack Desktop</td>
      <td class="watt-val">33.06 W</td>
      <td>+27.3 W</td>
      <td><div class="bar-wrap"><div class="bar bar-yellow" style="width:112px"></div></div></td>
    </tr>
    <tr class="row-yellow">
      <td>🟡 Discord Desktop</td>
      <td class="watt-val">32.64 W</td>
      <td>+26.9 W</td>
      <td><div class="bar-wrap"><div class="bar bar-yellow" style="width:111px"></div></div></td>
    </tr>
    <tr class="row-yellow">
      <td>🟡 Discord Web (Firefox)</td>
      <td class="watt-val">32.64 W</td>
      <td>+26.9 W</td>
      <td><div class="bar-wrap"><div class="bar bar-yellow" style="width:111px"></div></div></td>
    </tr>
    <tr class="row-green">
      <td>🟢 Teams Web (Firefox)</td>
      <td class="watt-val">31.50 W</td>
      <td>+25.8 W</td>
      <td><div class="bar-wrap"><div class="bar bar-green" style="width:107px"></div></div></td>
    </tr>
    <tr class="row-green">
      <td>🟢 Zoom Desktop</td>
      <td class="watt-val">31.23 W</td>
      <td>+25.5 W</td>
      <td><div class="bar-wrap"><div class="bar bar-green" style="width:106px"></div></div></td>
    </tr>
    <tr class="row-green">
      <td>🟢 Teams Desktop</td>
      <td class="watt-val">30.85 W</td>
      <td>+25.1 W</td>
      <td><div class="bar-wrap"><div class="bar bar-green" style="width:105px"></div></div></td>
    </tr>
    <tr class="row-idle">
      <td>⚪ Idle baseline (Firefox open)</td>
      <td class="watt-val">5.72 W</td>
      <td>—</td>
      <td><div class="bar-wrap"><div class="bar bar-idle" style="width:19px"></div></div></td>
    </tr>
  </tbody>
</table>

<div class="note-box">
💡 <strong>Quality check:</strong> Subtracting each app's attributed power from the total machine draw leaves a residual of 8.36–8.45 W across all seven benchmarks — a variation of just 0.09 W. The background was perfectly stable throughout, so every number here is directly comparable.
</div>

---

## The Google Meet problem — and the 10-second fix

Google Meet with default settings draws **50.08 W**, more than 60% higher than Teams Desktop at the same call quality. The cause is an AI feature called **auto-reframing**: enabled by default, it adjusts the video crop to keep you centered at the start of the meeting. Sounds harmless. It burns a sustained 9 extra watts, tracks your face during the whole meeting, without even reframing.

Turning it off takes about 10 seconds: three-dot menu → Settings → Video → disable "Auto-framing." Power drops to 41.06 W right away. That single change saves more electricity than switching from Slack all the way down to Teams Desktop.

Even with reframing disabled, Google Meet still sits well above the rest of the pack at 41.06 W. Google's browser-based video pipeline carries more inherent overhead than the other clients tested. We ran Meet in multiple browsers to rule out Firefox as the cause — consumption was consistently high across all of them.

Over a year of heavy use, that gap is real: default Meet runs around €36/year in electricity costs while Teams Desktop hovers around €22, at EU average rates. Not life-changing money, but your battery and its lifespan notice the extra heat regardless.

---

## The surprisingly tight race everyone else is running

Strip out Google Meet and you get six very different apps crammed into a 2.2-watt range.

**Slack Desktop (33.06 W)** leads for the wrong reasons: it offloads more compositing to Windows' display manager than anyone else. `dwm.exe` alone pulls 6.1 W during a Slack call, the highest of any app tested.

**Discord Desktop and Discord Web (32.64 W each)** come in tied — exactly. Same number, down to the hundredth of a watt. Whether you run Discord as an Electron app or open it in Firefox, the hardware ends up doing the same amount of work. Discord manages its rendering pipeline so completely internally that the outer shell barely matters.

**Teams Web (31.50 W)** and **Zoom Desktop (31.23 W)** are essentially neck and neck in the middle.

And then there's **Teams Desktop (30.85 W)**, quietly sitting at the bottom of the chart as the most efficient app in the entire benchmark. Yes, Teams Desktop — the bloated enterprise app that IT departments spend half their lives updating, that spawns seventeen processes when you open it, and that nobody asked for. It turns out Microsoft's unloved desktop client is the power efficiency champion of the group. We did not see that coming either.

The underlying reason is actually straightforward: for a 1-on-1 call, every app is doing roughly the same work — encode your camera once, decode one incoming stream. Architecture choices shuffle that work between processes, but the total barely changes. There is a natural floor around 31 W and most apps are sitting right on it.

---

<div class="tip-box">
<strong>🟢 Practical takeaways</strong>
<ul>
  <li><strong>Disable auto-reframing in Google Meet</strong> if you use it regularly. It is the single highest-impact change you can make — 9 W saved instantly, no app switch required.</li>
  <li><strong>Teams Desktop, Zoom, and Teams Web</strong> are the most efficient options if you have a choice.</li>
  <li><strong>Browser vs. native matters less than you would expect.</strong> Discord Desktop and Web are literally identical in power draw. Within any app, the difference is under 2 W.</li>
  <li><strong>AI video features add real load.</strong> Background blur, virtual backgrounds, and noise suppression all increase draw on any app. Disable what you do not actually need.</li>
  <li><strong>Warmer laptop, shorter battery life.</strong> These watt numbers translate directly to heat output and how fast your charge drains on a call.</li>
</ul>
</div>

---

## Measure it yourself

All of these results were produced with [WattSeal](/), a free tool that shows real-time power consumption per application using hardware energy counters. No smart plug required, works on Windows, macOS, and Linux, and takes about two minutes to set up.

Raw data, benchmark procedures, and full per-process breakdowns are in the [WattSeal-benchmark repository](https://github.com/Daminoup88/WattSeal-benchmark).

---

## Frequently asked questions

### Does browser choice affect Google Meet's consumption?
No. We tested Meet in multiple browsers and saw consistently high consumption across all of them. The overhead is in Google's video pipeline, not the browser.

### Why does the idle baseline show only 5.72 W?
That number is the attributed power of Firefox itself at idle — the portion of system draw caused by the browser being open with no tabs. The machine's total draw at idle is higher (including GPU display output, RAM, storage, and OS services), but those components run regardless of what app is open, so we exclude them from the comparison.

### Are these results applicable to other machines?
The absolute watt values will differ — a thin-and-light with integrated graphics will draw less overall. The relative ranking between apps should hold: the same architectural choices drive the same trade-offs across hardware generations.

### What about noise suppression and virtual backgrounds?
These add significant overhead on top of what you see above. All tests used default settings with those features off. Expect higher numbers if you use them.

### Will you add multi-participant tests?
Yes. That is where server routing kicks in and differences between apps tend to grow. We will publish those numbers when ready.