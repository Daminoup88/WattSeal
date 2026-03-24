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

<details>
  <summary>Does switching video call apps really save battery?</summary>
  <p>Sometimes — but settings usually matter more than the app name. In this benchmark, the spread between the best and worst default setups was about <strong>19 W</strong>, and one toggle alone changed Google Meet by roughly <strong>9 W</strong>.</p>
</details>

<details>
  <summary>Browser vs desktop: which one is better for battery?</summary>
  <p>It’s usually closer than people expect. In this test, Discord Desktop and Discord Web were identical at <strong>32.64 W</strong>, while Teams Desktop used <strong>30.85 W</strong> and Teams Web used <strong>31.50 W</strong>. Hardware acceleration and compositing matter more than the wrapper.</p>
</details>

<details>
  <summary>Do screen sharing and HD video drain more battery?</summary>
  <p>Yes. Video and screen sharing increase encode/decode and GPU work, so they drain more than audio-only calls. In our 1-on-1 runs, the apps clustered around <strong>31–33 W</strong> once the call was active, which is far above the idle browser baseline of <strong>5.72 W</strong>.</p>
</details>

<script type="application/ld+json">
{
  "@context": "https://schema.org",
  "@type": "FAQPage",
  "mainEntity": [
    {
      "@type": "Question",
      "name": "Which video call app uses the least battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "In this benchmark, Teams Desktop used the least battery at 30.85 W. Teams Web (31.50 W) and Zoom Desktop (31.23 W) were close behind."
      }
    },
    {
      "@type": "Question",
      "name": "Which video call app uses the most battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Google Meet with default settings used the most battery at 50 W. Turning off auto-framing reduced it to 41 W."
      }
    },
    {
      "@type": "Question",
      "name": "Why does Google Meet drain my laptop battery so fast?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Google Meet measured 50 W with default settings, about 9 W higher than the same call with auto-framing off. The main difference is the default AI video feature plus the browser-based pipeline."
      }
    },
    {
      "@type": "Question",
      "name": "Browser vs desktop: which one is better for battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Desktop is slightly better in this group. The gap is usually small because the bigger factor is how the app uses hardware acceleration and system compositing rather than whether it is wrapped as a browser or a desktop client."
      }
    },
    {
      "@type": "Question",
      "name": "Discord vs Slack: which drains more battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Discord is the better choice here. Slack tends to trigger more Windows compositing work, so it usually comes out slightly heavier."
      }
    },
    {
      "@type": "Question",
      "name": "Teams vs Slack: which uses less battery on a laptop?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Teams is the better choice. It stays closer to the efficiency floor, while Slack tends to do a bit more work through the Windows compositor."
      }
    },
    {
      "@type": "Question",
      "name": "Teams vs Zoom: which is better for battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Teams is slightly better here. Both are efficient, but Teams stays a bit closer to the lowest-power end of the group."
      }
    },
    {
      "@type": "Question",
      "name": "Teams vs Google Meet: which drains more battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Teams is clearly better. Google Meet is the heavier option because its default video pipeline and AI features add a lot more overhead."
      }
    },
    {
      "@type": "Question",
      "name": "Zoom vs Google Meet: which drains more battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Zoom is the better choice. Google Meet is much heavier because of its default pipeline and AI video features."
      }
    },
    {
      "@type": "Question",
      "name": "Discord vs Teams: which uses less battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Teams is the better pick. Discord is close, but Teams stays slightly lower because it does less work in the background."
      }
    },
    {
      "@type": "Question",
      "name": "Slack vs Zoom: which uses less battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Zoom is the better choice. Slack generally ends up a bit heavier because of the extra compositing load it creates."
      }
    },
    {
      "@type": "Question",
      "name": "Discord vs Zoom: which drains more battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Neither has a huge advantage here. The result usually comes down to effects, resolution, and whether hardware acceleration is working well."
      }
    },
    {
      "@type": "Question",
      "name": "Slack vs Google Meet: which drains more battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Google Meet is the heavier option. Default AI features and pipeline overhead make it more expensive than Slack in most setups."
      }
    },
    {
      "@type": "Question",
      "name": "Discord Desktop vs Discord Web: which uses less battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "They are effectively tied. Discord’s desktop and web versions behave almost the same because the heavy lifting is the same video pipeline."
      }
    },
    {
      "@type": "Question",
      "name": "Teams Desktop vs Teams Web: which uses less battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Teams Desktop is the better choice. The desktop app sits a bit lower because it handles the call stack more efficiently."
      }
    },
    {
      "@type": "Question",
      "name": "How can I reduce battery drain during video calls?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Turn off background blur, virtual backgrounds, auto-framing, and heavy noise suppression. Lower camera resolution if you can, close extra tabs or apps, and keep hardware acceleration enabled. In this benchmark, one AI feature changed power use by about 9 W."
      }
    },
    {
      "@type": "Question",
      "name": "How to save my battery during video calls?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "The fastest wins are disabling AI video effects, keeping hardware acceleration on, reducing camera or screen-share quality when possible, and avoiding unnecessary multitasking during calls."
      }
    },
    {
      "@type": "Question",
      "name": "Does screen sharing drain battery?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Yes. Screen sharing increases encode work and often keeps the GPU and display compositor busy. Expect higher power draw than a camera-only call."
      }
    },
    {
      "@type": "Question",
      "name": "Do big video calls consume more power?",
      "acceptedAnswer": {
        "@type": "Answer",
        "text": "Usually yes. More participants means more incoming video streams to decode, more UI work, and often higher network activity. Power draw usually rises compared with a 1-on-1 call."
      }
    }
  ]
}
</script>