---
layout: post
title: "How to Measure Your PC's Real Power Consumption (Without a Smart Plug)"
date: 2026-03-12
author: "Damien PHILIPPE"
description: "A step-by-step guide to measuring your PC's total and per-application power consumption using WattSeal — no hardware required."
---

Most power guides tell you to buy a smart plug. WattSeal lets you skip the hardware entirely — it reads energy counters built into your CPU and GPU directly.

## What You'll Need

- A Windows, macOS, or Linux PC
- [WattSeal](/), free and open source

## Step 1 — Download and Run WattSeal

Head to the [download section](#download) and grab the binary for your OS. No installer — just double-click the executable.

WattSeal starts a background daemon automatically on launch. You'll see the tray icon appear.

## Step 2 — Check Your Total System Wattage

Open the WattSeal dashboard. The **System** panel at the top shows:

- **Total watts** — sum of all measured components right now
- **kWh today** — running total since midnight
- **Estimated cost** — based on your configured electricity price

> If you haven't set your electricity tariff yet, go to **Settings → Energy** and enter your local rate in €/kWh or $/kWh.

## Step 3 — Find the Biggest Power Consumers

Switch to the **Applications** tab. WattSeal lists every running process with its attributed watt draw. Sort by **Watts** descending.

Common surprises:
- A background browser tab can draw 5–15 W continuously
- Anti-cheat services often idle at 3–8 W
- Cloud sync clients spike to 20+ W during uploads

## Step 4 — Watch a Specific App

Look at the 

## Step 5 — Check Your Carbon Footprint

Go to **Settings → Carbon** and select your region. WattSeal will recalculate all readings in CO₂ equivalents (grams per hour, kg per day, tonnes per year).

## Tips for Accurate Readings

- Let WattSeal run for at least **5 minutes** before reading averages — startup transients skew short samples.
- For gaming benchmarks, use the **1-minute average** view, not real-time, to smooth frame spikes.
- On laptops, readings reflect **battery draw** which may differ from wall consumption by the efficiency loss of your charger (~85–92% typical).

## Next Steps

- Read our video call power benchmark to see real app comparisons: [Video Call App Power Comparison: Teams, Meet, Discord, Slack (Real WattSeal Data)](/blog/video-call-power-comparison/)
- Browse the [blog](/blog/) for tips on cutting your PC's electricity bill.
