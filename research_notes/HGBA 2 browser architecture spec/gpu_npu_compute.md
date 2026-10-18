# GPU and NPU Compute Access for Browsers, and Heterogeneous Scheduling Realities (as of October 2026)

Research date: 2026-10-13. Every finding carries a source URL. Dated sources are labeled. "(secondary)" marks blog or news sources rather than vendor or standards sources.

## 1. WebGPU shipping status, wgpu/Dawn maturity, compatibility mode, WGSL

### Takeaway
As of October 2026, WebGPU ships by default in Chromium (Dawn) on Windows, macOS, ChromeOS, and most Android. It also ships in Safari 26 on every Apple OS and in Firefox (wgpu) on Windows and macOS. Linux is the gap. Chrome on Linux is limited to Intel Gen12+ and NVIDIA on Wayland, and Firefox on Linux and Android is still Nightly or behind a flag. Chrome 146 shipped an opt-in compatibility mode that targets OpenGL ES 3.1.

### Cited Findings
- The gpuweb Implementation Status wiki was last edited on 2 Oct 2026 (Kai Ninomiya). It gives these Chromium/Dawn versions: Mac, Windows x86/x64 and ChromeOS shipped in 113. Android 12+ on ARM/Qualcomm/Intel shipped in 121, Imagination on Android 16+ in 139, and Samsung Xclipse is TBD. Linux shipped on Intel Gen12+ in 144 and on NVIDIA (driver 535.183.01+) under Wayland in 147. Other Linux GPUs are behind `--enable-unsafe-webgpu`, and so is Windows ARM64. — [gpuweb wiki: Implementation Status](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
- Firefox (wgpu-based) per the same wiki: Windows shipped in 141. Apple Silicon Macs shipped in 145 on macOS 26+ and in 147 on all macOS versions. Intel Macs are Nightly only. Linux is Nightly only, and Mozilla expects to ship it in 2026. Android is behind a flag (`gfx.webgpu.ignore-blocklist`), with Android work also expected in 2026. — [gpuweb wiki](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
- Firefox 141 (July 2025) was the first release with WebGPU on Windows. — [Wikipedia: WebGPU](https://en.wikipedia.org/wiki/WebGPU); [It's FOSS (secondary)](https://itsfoss.com/news/firefox-webgpu-support/)
- Safari/WebKit has WebGPU "supported and enabled by default" in macOS Tahoe 26, iOS 26, iPadOS 26, and visionOS 26. — [gpuweb wiki](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
- Servo's WebGPU is work in progress behind the `dom.webgpu.enabled` pref, and it is also built on wgpu. wgpu therefore backs both Firefox and Servo. — [gpuweb wiki](https://github.com/gpuweb/gpuweb/wiki/Implementation-Status)
- Older coverage described WebGPU as shipping in all major browsers. — [web.dev: WebGPU supported in major browsers](https://web.dev/blog/webgpu-supported-major-browsers); [webgpu.com (secondary)](https://www.webgpu.com/news/webgpu-hits-critical-mass-all-major-browsers/)
  - A May 2026 article says WebGPU is "Baseline everywhere". The wiki's Linux and Android caveats contradict that claim. — [vr.org (secondary)](https://vr.org/articles/webgpu-baseline-2026-three-js-webxr-default)
- Chrome 146 (blog post dated 25 Feb 2026) shipped WebGPU compatibility mode:
  - It is opt-in, via `requestAdapter({ featureLevel: "compatibility" })`.
  - It starts with Android devices on OpenGL ES 3.1. The team is exploring ChromeOS with OpenGL ES 3.1 and Windows with Direct3D 11.
  - On Core-capable hardware the app gets a Core adapter but stays within compat limits unless it enables `"core-features-and-limits"`.
  - Any app that stays within compat limits is also a valid Core app.
  — [Chrome for Developers: What's New in WebGPU 146](https://developer.chrome.com/blog/new-in-webgpu-146); [WebGPU Fundamentals: compatibility mode](https://webgpufundamentals.org/webgpu/lessons/webgpu-compatibility-mode.html)
- Dawn is Chrome's WebGPU implementation. It also acts as Chromium's abstraction layer over Metal, Vulkan, and D3D, and Skia Graphite uses it (see section 6). — [Chromium blog: Introducing Skia Graphite (Jul 2025)](https://blog.google/chromium/introducing-skia-graphite-chromes/)

### Inferences
- On Windows and macOS, HGBA-2.0 can treat WebGPU (WGSL compute) as a baseline GPU target. On Linux and Android it needs a fallback path: compat mode, WebGL2, or CPU.
- WebGPU has two mature native implementations, Dawn (C++) and wgpu (Rust). A new engine can embed either one and pick up the WGSL toolchain (Tint in Dawn, Naga in wgpu) plus security hardening.
- Compat mode is a useful lower capability tier for a capability-graph scheduler. A device can advertise "WebGPU-compat" or "WebGPU-core", and core adds features such as storage textures in more stages and independent blend.

### Gaps
- I did not find a 2026 primary source on wgpu release cadence or version, or a formal Mozilla Linux ship date. The wiki only says "expected 2026".
- I did not retrieve the full list of compat-mode limits.
- WGSL spec maturity (CR status) was not verified in this pass.

## 2. WebNN: spec status, Chromium status, backends, other browsers

### Takeaway
WebNN is a W3C Candidate Recommendation, with an updated CR Snapshot published 22 Jan 2026. In Chromium it is still experimental. The Origin Trial was approved for M145–147, rescheduled to M146–148, and then delayed indefinitely on 24 Feb 2026. Current Chromium backends are Windows ML/ONNX Runtime on Windows, Core ML on macOS, and LiteRT (TFLite) on Windows, ChromeOS, Android, and Linux. CPU works everywhere, while GPU and NPU coverage is limited. DirectML, the original Windows backend, is now in maintenance mode.

### Cited Findings
- On 22 Jan 2026 the W3C published an updated Candidate Recommendation Snapshot of WebNN, with comments due 22 Mar 2026. — [CADE summary](https://cadeproject.org/updates/updated-candidate-recommendation-web-neural-network-webnn-api-published-by-the-world-wide-web-consortium/); [w3c/transitions #769](https://github.com/w3c/transitions/issues/769)
- The W3C transition request cites interest in a Chromium origin trial and a **positive standards position from Mozilla**. — [w3c/transitions #769](https://github.com/w3c/transitions/issues/769)
- Timeline of the blink-dev "Intent to Experiment: WebNN" thread:
  - 19 Dec 2025: approved "LGTM to experiment 145-147 inclusive".
  - 5 Jan 2026: delayed by at least one release.
  - 5 Feb 2026: rescheduled to M146–M148, with Chrome and Edge trial keys available.
  - 13 Feb 2026: Android excluded "due to the implementation's immaturity on that platform", and Android supports CPU inference only.
  - 24 Feb 2026: "we've decided to delay the Origin Trial again and will provide an update here when we have a new starting milestone."
  - The thread also states: "WebNN can currently execute models on CPU across all platforms but support for GPU and NPU execution is more limited."
  — [blink-dev: Intent to Experiment: WebNN](https://groups.google.com/a/chromium.org/g/blink-dev/c/5CWKSChYo98/m/xMw0U5NkAAAJ)
  - Press coverage of Chrome 146 Beta reported WebNN entering origin trial. The blink-dev thread shows it was delayed afterwards. — [Phoronix (secondary)](https://www.phoronix.com/news/Chrome-146-Beta); [Neowin (secondary)](https://www.neowin.net/news/google-promotes-chrome-146-to-the-beta-channel-with-webnn-support-sanitizer-api-and-more/)
- The WebNN implementation status page (© 2026):
  - Backends: **LiteRT** (Windows, ChromeOS, Android, Linux), **Windows ML** (Windows, ONNX ops), and **Core ML** (macOS, MIL ops).
  - Browser columns: Chrome Canary and Edge Canary for LiteRT and Windows ML, and Chrome Canary only for Core ML. All are marked "experimental".
  - The spec defines 95 ops in total.
  - Framework integrations: ONNX Runtime Web (WebNN EP) and LiteRT.js. DirectML is no longer listed.
  — [WebNN implementation status](https://webmachinelearning.github.io/webnn-status/)
- Chromium flags: `#webnn-onnxruntime` selects the Windows ML/ORT backend, and disabling it falls back to TFLite. The `#webnn-coreml` flag has a GPU/NPU variant, and the flags carry expiry milestone 146. — [webnn.io: Chrome flags](https://webnn.io/en/api-reference/browser-compatibility/chrome-flags)
- A WebNN docs PR describes switching Chrome or Edge to the Windows ML/ORT backend by disabling the "DirectML backend for WebNN" flag. — [webnn-docs PR #32](https://github.com/webmachinelearning/webnn-docs/pull/32/files)
- Microsoft Learn says DirectML is in "sustained engineering" and that new feature development has moved to Windows ML. The DirectML GitHub README says "maintenance mode": no new features, but security and compliance fixes continue, and Windows 11 24H2+ users should consider Windows ML. — [Microsoft Learn: DirectML](https://learn.microsoft.com/windows/ai/directml); [Microsoft Learn: WebNN overview](https://learn.microsoft.com/windows/ai/directml/webnn-overview)
- Windows ML acts as a system-level layer that finds, downloads, and registers the best execution provider (EP) for the device's CPU, GPU, or NPU.
  - Apps can set policies such as `PREFER_NPU` with automatic fallback to CPU, using `SetEpSelectionPolicy(OrtExecutionProviderDevicePolicy_PREFER_NPU)` or the Python equivalent.
  - Apps can also enumerate EP devices and append a specific one.
  - An older AMD table maps PREFER_NPU to VitisAI EP, PREFER_GPU to DML EP, and PREFER_CPU to CPU EP.
  — [AMD Ryzen AI docs: Windows ML EPs (vendor)](https://ryzenai.docs.amd.com/en/latest/winml/winml_ep.html); [AMD technical article 2026](https://www.amd.com/en/developer/resources/technical-articles/2026/simplifying-onnx-deployment-with-winml.html)
- LiteRT.js can use WebNN as an accelerator with a compile-time device preference, and it requires JSPI. — [vucense explainer (secondary)](https://vucense.com/dev-corner/litert-js-google-web-ai-inference-2026/)

### Inferences
- In October 2026 there is still no WebNN API that ships by default in a stable browser. Chromium has it behind flags or Canary, the OT is delayed, Mozilla's position is positive with no implementation, and Safari's position was not found.
- A browser reaches the NPU only through OS ML frameworks: Windows ML/ORT EPs (QNN, OpenVINO, VitisAI), Core ML (which decides internally whether to use the ANE), and LiteRT delegates. A browser cannot schedule onto NPU hardware directly. On Apple platforms, Core ML makes the final CPU/GPU/ANE placement, and the caller can only hint via compute units.
- HGBA-2.0 should model the NPU as a capability exposed by an OS-framework "device", with opaque placement and fallback. It should not model the NPU as a schedulable processor.

### Gaps
- I found no update after 24 Feb 2026 on a new WebNN OT milestone. The thread may have been truncated, so this is unverified as of October 2026.
- I did not find WebKit's standards position on WebNN.
- I did not retrieve the authoritative Microsoft page on Windows ML EP selection (learn.microsoft.com new-windows-ml/select-execution-providers), so the details above come from AMD's docs.

## 3. Chrome built-in AI (Gemini Nano) and Edge (Phi-4-mini)

### Takeaway
Chrome ships Translator, Language Detector, and Summarizer as stable since Chrome 138. The Prompt API is stable for the web in Chrome 148 and for extensions since 138, while Writer, Rewriter, and Proofreader are still in trials. The model runs on GPU if VRAM is over 4 GB, or on CPU if the device has 16 GB+ RAM and 4+ cores. NPU use is not documented. Edge's Prompt API uses Phi-4-mini, plus a CPU-capable Aion-1.0-Instruct model from version 150.

### Cited Findings
- Chrome status by API:
  - Translator, Language Detector, and Summarizer are stable in Chrome 138.
  - The Prompt API is on the web in Chrome 148 and in extensions since Chrome 138, with a sampling-parameters OT in 148.
  - Writer and Rewriter are in developer trial.
  - Proofreader is in developer trial or origin trial.
  — [Chrome for Developers: Built-in AI APIs](https://developer.chrome.com/docs/ai/built-in-apis) (page dates are inconsistent; last update shown as Sep 2025, but it lists the Chrome 148 status)
- Requirements:
  - OS: Windows 10/11, macOS 13+, Linux, or ChromeOS on Chromebook Plus. Android and iOS are not supported.
  - Storage: 22 GB free.
  - Hardware: either a GPU with strictly more than 4 GB VRAM, or a CPU path with 16 GB+ RAM and 4+ cores.
  - The current model size is shown at `chrome://on-device-internals`.
  — [Chrome for Developers: Get started with built-in AI](https://developer.chrome.com/docs/ai/get-started)
- A third-party guide says Chrome 148 brought the Prompt API to stable. — [pasqualepillitteri.it (secondary)](https://pasqualepillitteri.it/en/news/3145/gemini-nano-chrome-built-in-ai-client-side-en)
- Edge's Prompt API:
  - It has used Phi-4-mini since Canary/Dev 138.0.3309.2.
  - From 150.0.4070 it can use the prerelease Aion-1.0-Instruct model, described as "smaller, faster... supported on devices with less capable GPUs or no GPU, via CPU-inferencing".
  - Requirements: Windows 10/11 or macOS 13.3+, and 20 GB free. The model is deleted if free space drops below 10 GB.
  — [Microsoft Learn: Prompt API in Edge](https://learn.microsoft.com/microsoft-edge/web-platform/prompt-api)

### Inferences
- Both browsers choose the execution device once, by gating on hardware thresholds (VRAM, RAM, cores), and they offer the CPU path as a second tier. No public docs describe NPU dispatch for Gemini Nano or Phi in the browser. Built-in AI is effectively a "GPU-or-CPU" service run by the browser, with model placement hidden from pages.
- For HGBA-2.0, this supports exposing high-level task APIs (summarize, translate) where the browser owns placement, rather than exposing raw devices.

### Gaps
- I found no official documentation of NPU use by Chrome's or Edge's built-in models.
- A May 2026 privacy controversy about silent model download came from a single secondary source and was not verified.

## 4. Client NPU landscape 2026

### Takeaway
Windows NPUs in 2026 run from about 50 to 85 TOPS (INT8, vendor figures): Intel Panther Lake NPU 5 at about 50, AMD XDNA 2 in Ryzen AI 400 at 60, and Snapdragon X2 Elite Hexagon at 80–85. All of them clear the Copilot+ PC bar of 40 TOPS. Apple publishes no M5 Neural Engine TOPS figure and has moved AI throughput into per-GPU-core Neural Accelerators.

### Cited Findings
- Snapdragon X2 Elite: Hexagon NPU at 80 TOPS, and 85 TOPS on the Extreme and some Elite SKUs (X2E-96-100, X2E-90-100, X2E-84-100). — [giznova (secondary)](https://giznova.in/snapdragon-x2-elite-npu/); [localaimaster NPU TOPS list (secondary)](https://localaimaster.com/blog/npu-tops-list)
- Intel Panther Lake: NPU 5 at about 50 TOPS INT8, with FP8 support added. Intel's platform total of about 180 TOPS includes about 120 GPU TOPS. — [localaimaster (secondary)](https://localaimaster.com/blog/npu-comparison-2026); [tech-insider comparison (secondary)](https://tech-insider.org/panther-lake-vs-apple-m5-vs-snapdragon-x2-elite-2026/)
- AMD Ryzen AI 400 "Gorgon Point" XDNA 2: 60 TOPS. — [localaimaster (secondary)](https://localaimaster.com/blog/npu-tops-list)
- Copilot+ PC NPU minimum: 40 TOPS. — [SolidAITech NPU guide (secondary)](https://www.solidaitech.com/2026/05/npu-neural-processing-unit-complete-guide.html)
- Apple M5 (Oct 2025):
  - 16-core Neural Engine and a 10-core GPU with a Neural Accelerator in each core.
  - Apple claims over 4x peak GPU AI compute versus M4 and over 6x versus M1.
  - 153 GB/s unified memory bandwidth (M4 had 120 GB/s), and up to 32 GB.
  - No TOPS figure is published. Unofficial estimates are about 38–42 TOPS.
  — [Guru3D](https://www.guru3d.com/story/apple-m5-soc-brings-neural-accelerators-to-every-gpu-core); [MacTrast](https://www.mactrast.com/2025/10/apple-unveils-its-m5-chip-featuring-next-generation-gpu-optimized-for-ai-tasks/); [All About Circuits](https://www.allaboutcircuits.com/news/apple-looks-to-the-future-of-ai-with-new-m5-processor/)
- AMD XDNA hardware:
  - The NPU is a 2D array of columns, each with 4 rows of compute tiles and 1 memory tile.
  - It can be partitioned at column boundaries into spatially isolated partitions bound to workload contexts.
  - The Linux stack is the amdxdna kernel driver plus the XRT runtime, with IREE plus Peano (LLVM for AIE) for compilation.
  — [Linux kernel docs: amdxdna](https://docs.kernel.org/6.17/accel/amdxdna/amdnpu.html)

### Inferences
- TOPS figures do not compare across vendors (different precision and sparsity assumptions), and Apple does not disclose them. A spec should treat "NPU present plus framework EP available" as the capability, and leave TOPS out of scheduling decisions.
- Browsers can reach these NPUs only indirectly:
  - Windows: ML (QNN EP for Hexagon, OpenVINO EP for Intel, VitisAI EP for AMD).
  - Apple: Core ML (ANE).
  - Android: LiteRT/NNAPI successors.
  - Linux: no mainstream browser path; amdxdna/IREE and Intel's ivpu exist at the native level only.
- XDNA's spatial partitioning makes NPU multi-tenancy coarse-grained (column partitions bound to contexts). That fits with a browser broker that serializes or batches requests from many origins.

### Gaps
- I could not verify primary vendor pages (Qualcomm, Intel, AMD) for exact TOPS precision in this pass, so the figures come from secondary aggregators.
- Lunar Lake NPU 4 (48 TOPS) was not separately sourced in this pass.

## 5. Dispatch overhead, latency, transfer costs, minimum offload granularity

### Takeaway
Fixed costs are in the microseconds-to-milliseconds range and dominate small tasks:
- A native null-kernel launch costs about 3–7 µs on CUDA/Linux, and 10–80 µs under Windows WDDM.
- WebGPU in a browser adds an IPC hop to the GPU process, and readback with `mapAsync` needs at least a round trip and can wait on unrelated queued work.
- NPU per-inference fixed overhead is large enough that a 0.6B draft model ran about 3.6x slower on the Lunar Lake NPU than on the CPU.

Frame budgets are 16.7 ms at 60 Hz and 8.3 ms at 120 Hz. Offload is worthwhile only for batched, coarse-grained work that does not need same-frame CPU readback.

### Cited Findings
- NVIDIA defines launch latency as the time from the launch API call to kernel start, and shows about 20 µs in its example. In a CUDA Graphs example, wall-clock time was 9.6 µs per kernel (including overheads) against 2.9 µs of kernel execution. CUDA Graphs exist to cut this per-launch cost. — [NVIDIA: CUDA Graphs](https://developer.nvidia.com/blog/cuda-graphs)
- Null-kernel launch overhead is about 3 µs on recent hardware. Under Windows WDDM, null launches vary from 10 to 80 µs, averaging about 20 µs. — [NVIDIA forums: kernel launch latency](https://forums.developer.nvidia.com/t/too-much-time-for-kernel-launch-latency/233645); [NVIDIA forums: very slow kernel launches](https://forums.developer.nvidia.com/t/very-slow-kernel-launches/37345)
- A 2026 preprint (GPUOS) cites null-kernel launches at about 3–7 µs and proposes transparent operation fusion to amortize them. — [arXiv 2604.17861](https://arxiv.org/pdf/2604.17861)
- Dawn wire (Chromium's renderer-to-GPU-process WebGPU transport):
  - The client waits for the server's mapAsync status before resolving, so each readback includes a round trip across the IPC boundary.
  - Large `writeBuffer`/`writeTexture` commands are serialized and sent in chunks.
  — [Dawn commit: mapAsync status](https://dawn.googlesource.com/dawn/+/refs/heads/chromium/5484); [Dawn source history](https://dawn.googlesource.com/dawn/+/0d52f800a1d15c2941c3d177f7a5313bdba0bd79)
- WebKit bug 272804: "GPUBuffer.mapAsync waits for unrelated command buffers", which lengthens readback latency. — [WebKit Bugzilla 272804](https://bugs.webkit.org/show_bug.cgi?id=272804)
- Lunar Lake characterization (community, June 2026): for a 0.6B draft model, the NPU was about 3.6x slower per token than the CPU and about 5x slower than the GPU, which the author attributes to a high fixed per-inference dispatch cost. Using the GPU as target with the NPU as draft gave 0.55–0.74x the throughput of GPU-only. — [OpenVINO discussion #36484](https://github.com/openvinotoolkit/openvino/discussions/36484)
- On Intel AI PC hardware (2024), the NPU beat the GPU on small matrices and matrix-vector ops, while the GPU won on larger matrices. — [arXiv 2409.14803](https://arxiv.org/pdf/2409.14803)
- On mobile Hexagon (ASPLOS 2025), the NPU is strong for INT8 matmul, but FP16 matmul on the NPU ran far slower than CPU INT8. — [Xu et al., Fast On-device LLM Inference with NPUs, ASPLOS'25](https://xumengwei.github.io/files/ASPLOS25-NPU.pdf); [arXiv 2407.05858](https://arxiv.org/html/2407.05858v2)
- Unified memory: Apple M5 has a single 153 GB/s unified memory pool shared by CPU, GPU, and Neural Engine. — [Guru3D](https://www.guru3d.com/story/apple-m5-soc-brings-neural-accelerators-to-every-gpu-core)

### Inferences
- Frame budgets: 1000/60 = 16.7 ms and 1000/120 = 8.3 ms. A WebGPU dispatch has a submit-to-complete-to-readback round trip spanning renderer, GPU process, driver, and back, plus queue contention. This can easily cost a large share of a millisecond or more, so synchronous per-frame readback is risky for any compute that feeds layout or script.
- Proposed granularity rules for the spec. These are design heuristics derived from the evidence above, not measured thresholds.
  1. Keep work on the GPU across stages and avoid CPU readback inside a frame.
  2. Batch many small ops into one submit, in the manner of CUDA Graphs or fused passes.
  3. Use the NPU only for sustained, quantized (INT8/FP8) model inference of meaningful size, never for fine-grained or latency-critical per-frame tasks.
  4. On unified-memory SoCs (Apple silicon, Lunar Lake), transfer cost falls to near zero, but sync and dispatch costs remain.
- A capability-graph scheduler should model each edge with a fixed cost (dispatch or IPC), a per-byte cost (zero on UMA, PCIe on discrete), and a sync cost. It should not model compute throughput alone.

### Gaps
- I found no measured numbers for Vulkan/D3D12/Metal compute dispatch latency, WebGPU submit-to-complete latency in Chrome, or Core ML/ANE invocation overhead. These would need targeted benchmarks.
- PCIe transfer numbers for discrete GPUs, and Lunar Lake on-package memory bandwidth, were not sourced in this pass.

## 6. GPU process architecture, render pipelines, GPU and compute rasterization

### Takeaway
Chromium isolates all GPU access in a GPU process:
- Viz handles compositing and display.
- Dawn serves both WebGPU and Skia Graphite.
- Graphite replaced Ganesh on Apple Silicon Macs in July 2025 and gained about 15% on MotionMark 1.3.
- Windows Graphite was still in progress in late 2025, tested mainly on Dawn D3D11.

Firefox and Servo use WebRender. Linebender's Vello shows fully compute-shader-based 2D rendering on wgpu, and its sparse-strips "Vello Hybrid" variant reached roughly beta quality in Q1 2026.

### Cited Findings
- Graphite has been enabled in Chrome on Apple Silicon Macs since July 2025, gaining nearly 15% on MotionMark 1.3 (MacBook Pro M3) and improving INP and graphics smoothness. It is multi-threaded, and Dawn provides the abstraction over Metal, Vulkan, and D3D. — [Chromium blog: Introducing Skia Graphite](https://blog.google/chromium/introducing-skia-graphite-chromes/); [Phoronix (secondary)](https://www.phoronix.com/news/Chromium-Skia-Graphite)
- A December 2025 skia-discuss post (Michael Ludwig) says Graphite is enabled by default on Chrome for Mac and "making progress on Windows", tested mainly with Dawn's D3D11 backend. — [skia-discuss: status of GPU backends](https://groups.google.com/g/skia-discuss/c/Pd92csb5o4o/m/QlPj80PqAQAJ)
- Vello:
  - Vello is a GPU compute-centric 2D renderer built on wgpu. — [Vello ARCHITECTURE.md](https://skia.googlesource.com/external/github.com/linebender/vello/+/refs/heads/sparse/ARCHITECTURE.md)
  - Linebender's Q1 2026 update (19 Apr 2026) reports Vello 0.8.0 and sparse strips 0.0.7.
  - Vello Hybrid (sparse strips, CPU/GPU split) is "roughly beta", with fast paths that skip coarse rasterization, rectangle special-casing, glyph caching, and faster gradients and compositing.
  - A planned unified "Vello API" was replaced by two crates, AnyRender and imaging.
  — [Linebender in 2026 Q1](https://linebender.org/blog/tmil-25/)

### Inferences
- Chromium's design centralizes GPU work in one privileged process that serves both raster (Graphite) and WebGPU through Dawn. Any accelerator scheduler therefore has to sit in or beside the GPU process, and its cross-process submission cost is part of the dispatch overhead in section 5.
- Graphite's recorder/recording model (record on multiple threads, then submit) and Vello's compute pipeline both point towards a render graph. In that graph, rasterization is a set of GPU compute passes scheduled together with other GPU work, which fits HGBA-2.0's render-graph plus capability-graph framing.
- Vello Hybrid's CPU/GPU split shows that the best placement of 2D rasterization differs by device. This is direct evidence for runtime placement in the scheduler.

### Gaps
- I did not fetch primary docs on Chromium's GPU process/Viz architecture or WebRender in this pass. Canonical references are the [Chromium Viz README](https://chromium.googlesource.com/chromium/src/+/HEAD/components/viz/README.md) and the [WebRender repo](https://github.com/servo/webrender), both listed without verification.
- No current status was found for Graphite on Windows, Linux, or Android in 2026.

## 7. Heterogeneous scheduling systems that could inform a capability-graph scheduler

### Takeaway
Three models are relevant:
- **StarPU (task-based runtime):** the scheduler places multi-variant tasks across devices at runtime.
- **ONNX Runtime/Windows ML (EP selection):** a policy such as PREFER_NPU with a fallback chain, plus graph partitioning across EPs.
- **IREE (compiler plus HAL):** one MLIR-based IR lowered to CPU, GPU (Vulkan/Metal/CUDA/ROCm), and NPU targets via plugins (AMD AIE/XDNA).

### Cited Findings
- StarPU is a runtime for heterogeneous CPU and GPU systems. Applications submit tasks with CPU and/or GPU implementations ("codelets"), and the core library schedules them. Several scheduling strategies are selectable at runtime, and one dynamic approach competes with the highly optimized MAGMA library (CCPE 2010/2011 paper). — [StarPU docs introduction](https://git.microlab.ntua.gr/exa2pro/starpu-max/raw/d32e9b2e2a2bc7b2fd76a287bdf8582e5848554d/doc/doxygen/chapters/introduction.doxy); [Augonnet et al., StarPU, CCPE](https://www.par.univie.ac.at/project/peppher/publications/Published/ccpe10.pdf)
- ONNX Runtime/Windows ML EP selection: device policies (PREFER_NPU, PREFER_GPU, PREFER_CPU) with automatic fallback, or explicit EP-device enumeration. Windows ML downloads and registers EPs per device. — [AMD Ryzen AI WinML docs](https://ryzenai.docs.amd.com/en/latest/winml/winml_ep.html); [AMD: Windows ML on AMD NPU (2026)](https://amd.com/en/developer/resources/technical-articles/2026/ai-model-deployment-using-windows-ml-on-amd-npu.html)
- IREE on NPU: `iree-amd-aie` is an early-phase IREE compiler and runtime plugin for AMD NPUs, built with the LLVM CPU backend plus an external `xrt-lite` HAL driver. Kernel docs say IREE supports graph compilation of ML models for AMD NPU, using Peano. — [nod-ai/iree-amd-aie](https://github.com/nod-ai/iree-amd-aie); [kernel docs: amdxdna](https://docs.kernel.org/6.17/accel/amdxdna/amdnpu.html)
- CUDA Graphs and GPUOS-style fusion amortize per-dispatch cost by turning many small launches into one graph or fused op. — [NVIDIA: CUDA Graphs](https://developer.nvidia.com/blog/cuda-graphs); [arXiv 2604.17861](https://arxiv.org/pdf/2604.17861)

### Inferences
- A "capability graph" for HGBA-2.0 could combine three ideas:
  - StarPU-style multi-implementation tasks, such as a CPU, WebGPU, or ML-framework variant of the same op.
  - An ORT-style policy layer (prefer-power, prefer-latency, prefer-NPU, with a fallback chain), driven by cost models that include the fixed dispatch, transfer, and sync costs from section 5.
  - IREE-style ahead-of-time lowering of shader and ML graphs to per-device executables.
- A hierarchical design follows from the platform constraints. The browser-level broker picks a device class per task graph (CPU, GPU via Dawn/wgpu, or NPU via an OS ML framework). Within the GPU or ML framework, the native runtime does the fine-grained scheduling, because NPUs are reachable only through those frameworks.

### Gaps
- I found no source on IREE multi-device or heterogeneous partitioning (IREE HAL multi-device), oneAPI/SYCL, or recent hierarchical scheduler papers (for example, HEFT-style list scheduling in browsers). These need a follow-up search.
- ORT's graph partitioning across multiple EPs was not documented from a primary Microsoft source in this pass.
