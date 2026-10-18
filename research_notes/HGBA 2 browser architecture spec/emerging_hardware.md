# Emerging Compute Hardware: Research Horizon Notes for HGBA-2.0

Research date: 2026-10-13. Every claim has a URL. Items marked **[vendor claim]** are not independently verified. Items marked **[unverified]** could not be confirmed from a primary source during this pass. Older data carries its date.

---

## 0. Bottom line (for the appendix)

| Technology | What really exists (Oct 2026) | Commercial form | Plausible for client browser workloads in 5–10 yrs? |
|---|---|---|---|
| RISC-V client CPUs | First RVA23 silicon is shipping in SBCs and a Framework mainboard (SpacemiT K3). Performance is roughly Cortex-A76 class. | Dev boards, one laptop mainboard | **Yes, as a general-purpose CPU ISA.** This is the only item here that is a real client target. Browsers need an upstream riscv64 tier (JIT, sandbox, media). |
| Photonic compute | Photonic interconnect (co-packaged optics) is commercializing in data centers. Photonic matrix-math compute is limited to lab and early rack products. | Data-center interconnect; a few rack-mounted photonic NPUs | **No** as client compute. It may arrive indirectly, through cheaper cloud inference. |
| Neuromorphic | Research systems (Loihi 2/Hala Point, NorthPole) plus µW–mW edge MCUs (Akida, Pulsar, Speck/Xylo) | Sensor-edge MCUs and IP licenses | **Only as an invisible sensor pre-processor** (wake-word, gesture, eye tracking) behind OS APIs. It does not render pages. |
| Analog in-memory compute | IBM research chips; announced M.2 accelerator from EnCharge; Mythic and Sagence on funding and roadmaps | Early-access cards | **Possible as one more NPU behind the OS ML stack**, reached through WebNN. It needs no browser-specific architecture. |
| Quantum | 100–150-qubit noisy machines; first error-correction milestones; cloud-only, cooled to about 10 mK | Cloud services (IBM, Quantinuum/Oracle, D-Wave) | **No.** Physically and economically incompatible with client devices. The browser-relevant quantum work is post-quantum cryptography, which runs on classical CPUs and already ships. |
| "Neuromorphic / event-driven displays" | Event-based *sensors* are real. Event-driven *display* hardware amounts to patents and papers. Foveated rendering and VRR are real and shipping. | Event-camera sensors; eye-tracked foveation in headsets | **Claims of "neuromorphic rendering" should be treated as speculative.** The real levers are foveation, VRR, and damage-region compositing. |

**Design lesson (Section 7):** Every past hardware wave (GPU, SIMD, NPUs) reached the web through an OS or driver abstraction plus a capability-negotiated, sandboxed standard API (WebGL/WebGPU, Wasm SIMD, WebNN). It did not reach the web through engine-specific coupling. HGBA-2.0 should define *extension points* (pluggable compute backends behind standard APIs, capability discovery, process-isolated device brokers) rather than naming specific exotic devices.

---

## 1. RISC-V client computing

### 1.1 Profile and ISA baseline
- **RVA23 was ratified on 2024-10-21** by the RISC-V International Board. It makes the Vector (RVV 1.0) and Hypervisor extensions mandatory for 64-bit application processors. https://riscv.org/blog/risc-v-announces-ratification-of-the-rva23-profile-standard/ ; https://www.hpcwire.com/off-the-wire/risc-v-announces-ratification-of-the-rva23-profile-standard/
- Embedded Computing Design reports that Android uses RVA23 as the baseline for its RISC-V ABI. https://embeddedcomputing.com/technology/open-source/risc-v-open-source-ip/risc-v-international-reveals-rva23-profile-ratification-in-time-for-the-risc-v-summit-north-america
- Significance for browsers: a mandatory vector unit means JITs and media and crypto libraries can rely on RVV, the way x86-64 relies on SSE2 and ARMv8 relies on NEON. Before RVA23, RVV support was fragmented.

### 1.2 RVV 1.0 hardware (pre-RVA23)
- **SpacemiT K1 / X60 (2024):** 8 cores, RVA22 plus RVV 1.0 with 256-bit VLEN. It ships in the Banana Pi BPI-F3. https://docs.banana-pi.org/en/BPI-F3/SpacemiT_K1_datasheet. CNX puts its single-core performance at about 1.3x a Cortex-A55. https://www.cnx-software.com/?p=130158
- Note that the K1 is **not** RVA23-compliant (Phoronix, 2026-02). https://www.phoronix.com/news/Ubuntu-SpacemiT-K3

### 1.3 First RVA23 client silicon: SpacemiT K3 (2026)
- **Specs:** 8 RVA23 "X100" cores up to about 2.4 GHz, roughly 60 TOPS AI accelerator **[vendor claim]**, up to 32 GB LPDDR5, 15–25 W, actively cooled. https://www.phoronix.com/news/Ubuntu-SpacemiT-K3 ; https://heise.de/-11167158
- **Boards:** Milk-V Jupiter 2 SBC (up to 32 GB RAM, 256 GB UFS) https://www.cnx-software.com/news/benchmarks/?amp=1 ; K3 Pico-ITX with 10GbE, reviewed by Phoronix in May 2026 https://www.phoronix.com/review/spacemit-k3-pico-itx
- **Laptop:** DeepComputing's **DC-ROMA RISC-V Mainboard III for the Framework Laptop 13** uses the K3. It starts at $699, ships with Ubuntu 26.04, and the first batch was due at the end of June 2026. https://deepcomputing.io/deepcomputing-showcases-dc-roma-risc-v-mainboard-iii-at-framework-nextgen-event/ ; https://www.digitalcitizen.life/deepcomputing-launches-risc-v-mainboard-upgrade-for-framework-laptop-13/ ; https://store.deepcomputing.io/products/dc-roma-risc-v-mainboard-iii-for-framework-laptop-13-early-access-program . Whether volume has shipped is **[unverified]**.
- **OS:** Canonical and SpacemiT announced Ubuntu (26.04 LTS) support for the K3/K1. https://canonical.com/blog/spacemit-announces-availability-of-ubuntu-on-k3-k1-series . Debian 13 "trixie" (2025-08) is the first Debian release with **official riscv64** support. https://lwn.net/Articles/1033474 ; https://www.debian.org/releases/stable/release-notes/whats-new.ja.html
- **Real-world rough edges:** CNX's review of Bianbu OS 4.0 on the K3 documents "what works / what doesn't" (2026-06-21). https://www.cnx-software.com/2026/06/21/spacemit-k3-pico-itx-risc-v-chassis-kit-review-part-2-what-works-what-doesnt-in-bianbu-os-4-0/?amp=1

### 1.4 Performance vs ARM/x86 (2026 data)
- Phoronix: the K3 "at times" matched or beat the Raspberry Pi 500+ (4x Cortex-A76 at 2.4 GHz). It trailed significantly in compilation and led in OpenSSL. https://www.phoronix.com/review/spacemit-k3-pico-itx/3
- Phoronix five-year retrospective: the K3 is more than 8x the HiFive Unmatched (geometric mean), but it **still typically trails the Raspberry Pi 5 and current Intel/AMD desktop CPUs** in raw performance. https://phoronix.com/review/risc-v-5-year-performance/6
- CNX sbc-bench: in 7-zip single-core, the K3 X100 scored 2736 MIPS vs 3136 MIPS for the Pi 5. Multi-core was about 2x the Pi 5. https://www.cnx-software.com/?p=164250
- **Sober summary:** the best RISC-V client chip of 2026 is in the Cortex-A76 class (a 2018-era Arm core design). That is several generations behind Apple M-series, Snapdragon X, and current x86 in single-thread performance, which dominates JS and page-load latency.
- **The pipeline is improving:** Tenstorrent licenses TT-Ascalon (RVA23, RVV 1.0, >2.3 SPECint2017/GHz, >2.5 GHz on Samsung SF4X **[vendor claim]**) as IP, announced 2025-12-04. https://tenstorrent.com/newsroom/tenstorrent-announces-availability-of-tt-ascalon . Alibaba's XuanTie C930 (RVA23, VLEN 256) launched as IP in 2025, and the C950 followed in 2026-03. https://www.theregister.com/2026/03/25/alibaba_damo_xuantie_c950_chip/ . No consumer device with Ascalon or C930 silicon was confirmed as of Oct 2026.

### 1.5 Android on RISC-V
- Google's 2023 plan was a riscv64 NDK ABI built on RVA22 plus vector extensions, with finalization targeted for "later 2023." https://opensource.googleblog.com/2023/10/android-and-risc-v-what-you-need-to-know.html
- In 2024, Google removed RISC-V support from the Android Common Kernel (ACK) while saying "Android will continue to support RISC-V." https://hackster.io/news/google-removes-risc-v-support-from-the-android-common-kernel-but-denies-abandoning-the-isa-c4ccab33a929
- FOSDEM 2026: community and vendor AOSP ports boot on modern RISC-V hardware. Full device bring-up and CTS/VTS compliance are **still goals, not achieved**. https://deepcomputing.io/fosdem-2026-recap-bringing-android-aosp-to-risc-v/ ; https://fosdem.org/schedule/event/SXX8HE-open_source_risc-v_aosp_porting_progress_challenges_and_upstream_work
- I found no confirmation that a finalized riscv64 NDK ABI or a GMS-certified RISC-V Android phone existed as of Oct 2026 **[unverified]**.

### 1.6 Browser availability on RISC-V
- **V8:** the RISC-V backend has been upstream since Feb 2021. It runs the full test suites but can be broken by upstream changes. https://github.com/v8-riscv/v8/wiki
- **Chromium:** as of 2025-01, RISC-V International stated that "Chromium still does not officially support the RISC-V architecture." Community builds suffered slow rendering and stuttering video. https://riscv.org/?p=101244 . Alibaba DAMO published Chromium optimizations for XuanTie (2025-01-08), same URL. Chromium removed the riscv64 Cronet build from CI. https://chromium.googlesource.com/chromium/src/+/ae826ca03431c9a745180ae27c222f64f757a5cf . Chromium build configuration has RVV and Zba/Zbb/Zbs flags, off by default. https://kdab.searchfox.org/firefox-main/source/third_party/chromium/build/config/riscv.gni . There is **no official Google Chrome binary for riscv64**. Distributions (openEuler and others) carry ports. https://www.openeuler.org/en/blog/RISC-V/
- **Firefox:** the SpiderMonkey riscv64 JIT/Wasm backend landed around Fx 110–111, contributed and maintained by PLCT Lab. https://spidermonkey.dev/blog/2023/02/16/newsletter-firefox-110-111.html . The JIT has been **enabled by default for riscv64 since Firefox 143** (bug 1982266). https://bugzilla.mozilla.org/show_bug.cgi?id=1982266 . Stability issues remain open (bug 1984989, tab crashes on Fx 143 riscv64). https://bugzilla.mozilla.org/show_bug.cgi?id=1984989 . Ubuntu's Mozilla team builds riscv64 Firefox. https://edge.launchpad.net/~mozillateam/+archive/ubuntu/firefox-next/+build/28139423 . riscv64 is a Tier-3 / distro-supported target, not a Mozilla-shipped one.
- **Implication for HGBA-2.0:** RISC-V is the one emerging target that needs **concrete engine work**: a JIT backend, Wasm SIMD lowering to RVV (VLEN-agnostic), a seccomp/sandbox syscall policy, media codecs, and GPU drivers (Imagination BXM-class GPUs on SpacemiT, with Mesa maturity varying). An architecture that keeps ISA-specific code isolated (JIT backends, assembly stubs, and SIMD kernels behind portable intrinsics such as Highway or xsimd) lowers the cost.

---

## 2. Photonic computing

### 2.1 Interconnect (real, commercializing in data centers)
- **Lightmatter Passage M1000** (announced 2025-03-31): a 3D photonic interposer larger than 4,000 mm² with 114 Tbps aggregate optical bandwidth and 256 fibers. Built on GlobalFoundries Fotonix with Amkor, availability "summer 2025." **Passage L200/L200X** co-packaged optics: 32/64 Tbps. https://lightmatter.co/press-release/lightmatter-unveils-passage-m1000-photonic-superchip-worlds-fastest-ai-interconnect/ ; https://www.datacenterdynamics.com/en/news/lightmatter-unveils-m1000-and-l200-passage-photonic-interconnects
- Lightmatter and GUC announced a partnership (2026-01) to bring Passage CPO to market. Its latest disclosed funding is a $400M Series D (2024-10, $4.4B valuation). No named Passage production customer was confirmed. https://stockanalysis.com/private/lightmatter/ ; https://www.photonics.com/Articles/Lightmatter_Raises_154M_Series_C/a69054
- **Celestial AI → Marvell:** announced 2025-12 at about $3.25B upfront and **completed 2026-02-02**. The Photonic Fabric chiplet offers 16 Tbps. Marvell expects meaningful revenue in the second half of FY2028. https://investor.marvell.com/news-events/press-releases/detail/1005/marvell-completes-acquisition-of-celestial-ai ; https://www.lightreading.com/ai-machine-learning/marvell-to-scoop-up-celestial-ai-for-3-25b ; https://evertiq.com/news/2026-02-03-marvell-completes-acquisition-of-celestial-ai

### 2.2 Photonic compute (lab and early rack products)
- **Lightmatter Envise:** marketed as a photonic AI inference accelerator. Speed claims (for example, 5x an A100 on BERT) are **[vendor claim, never independently verified]**. https://www.aibusiness.com/verticals/lightmatter-announces-envise-a-photonic-accelerator-for-ai . Lightmatter's corporate emphasis has visibly shifted to interconnect (Passage).
- **Lightmatter Nature paper (2025-04):** a six-chip package (four 128×128 photonic tensor cores plus two 12 nm digital control dies) ran ResNet, BERT, and Atari RL near FP32 accuracy at 65.5 TOPS (ABFP16), 78 W electrical. Figures are from secondary summaries; check against the paper. https://aiwiki.ai/wiki/lightmatter ; https://pdpspectra.com/blog/photonic-computing-optical-ai-2026 . Note that 78 W is a data-center power budget, not a phone or laptop budget.
- **Q.ANT (Germany):** photonic "NPU" on PCIe, sold only as a 19" rack server. Gen-1 was orderable in 2024-11. NPU 2.0 was announced in 2025-11, with first shipments targeted for H1 2026 **[shipment unverified]**. https://qant.com/press-releases/q-ant-launches-first-commercial-photonic-processor-for-energy-efficient-high-performance-computing-and-real-time-ai-applications/ ; https://www.yolegroup.com/industry-news/q-ant-unveils-second-generation-photonic-processing-server-to-power-the-next-wave-of-ai-and-hpc/

### 2.3 Photonic ray tracing
- A 2025 *Nature Communications* paper demonstrates a **thin-film lithium-niobate photonic ray-box intersection unit** and claims it is the first photonic counterpart for ray-tracing acceleration, motivated by spatial computing. This is a component-level lab demo, not a renderer. https://ideas.repec.org/a/nat/natcom/v16y2025i1d10.1038_s41467-025-61234-x.html

### 2.4 Client feasibility assessment
- Photonic compute performs analog linear algebra. It needs lasers, modulators, and photodetectors plus electro-optic conversion at each boundary, and it has precision limits (Envise was marketed for inference only because analog is less accurate). https://www.aibusiness.com/verticals/lightmatter-announces-envise-a-photonic-accelerator-for-ai
- Every commercial photonic product as of Oct 2026 is a data-center part (interposers, CPO, rack servers). **No credible client-device photonic compute roadmap exists.** Browsers would feel photonics only indirectly, through cheaper or faster cloud AI and CDN back ends.

---

## 3. Neuromorphic computing

### 3.1 Research-scale systems
- **Intel Hala Point** (2024-04): 1,152 Loihi 2 chips on Intel 4, 1.15 B neurons, 128 B synapses, 2,600 W max, 6U chassis, deployed at Sandia. It remains a **research system**, with about 200 INRC members. https://newelectronics.co.uk/content/news/intel-builds-world-s-largest-neuromorphic-system ; https://www.edge-ai-vision.com/?p=47507
- **Loihi 3:** the only mention is a 2026-01 secondary article citing "preliminary reports" of a late-2026/2027 debut. **[unverified; no Intel source found]** https://investor.wedbush.com/wedbush/article/tokenring-2026-1-28-the-brain-in-the-box-intels-billion-neuron-breakthroughs-signal-the-end-of-the-power-hungry-ai-era
- **Programming model, Lava:** Intel's open-source framework (BSD-3 core, LGPL-2.1 "Magma" backend layers), first released in 2021-10. It is hardware-independent and runs on CPU for development. https://intel.com/content/www/us/en/newsroom/news/intel-unveils-neuromorphic-loihi-2-lava-software.html ; https://intel-ncl.atlassian.net/wiki/spaces/INRC/blog/2021/10/27/1722580993/Lava+v0.1.0+has+been+released . Its current maintenance cadence is **[unverified]**.
- **IBM NorthPole:** strictly a digital near-memory inference chip ("brain-inspired"), not spiking. HPEC 2024: 16 cards ran a 3B-parameter LLM at about 28,356 tok/s with <1 ms/token at 672 W, and IBM claims 72.7x the energy efficiency of the best-matched GPU **[vendor claim]**. https://modha.org/wp-content/uploads/2024/09/NorthPole_HPEC_LLM_2024.pdf . 2025-11 arXiv: a 288-card, 18-server system serving models through watsonx (cloud). https://arxiv.org/pdf/2511.15950v1

### 3.2 Edge and commercial neuromorphic
- **BrainChip Akida:** received its first 2,000 AKD1500 chips in the June 2026 quarter. About 60k units are expected from the run, below forecast because of yield, and commercial shipments are targeted for H2 2026. Revenue is mostly IP licensing (EDGEAI, Lockheed ForwardEdge ASIC). AKD2500 tape-out is planned for 2026-12. https://kalkinemedia.com/au/news/announcements/brainchip-secures-initial-commercial-batch-of-2000-akd1500-neuromorphic-processors-targets-h2-2026-shipments ; https://announcements.asx.com.au/asxpdf/20260727/pdf/0722fx6s9yf2w9.pdf
- **Innatera Pulsar** (launched 2025-05-21): SNN fabric plus RISC-V CPU, CNN, and FFT accelerators. It is a "mass-market neuromorphic MCU" for the sensor edge, with claims of up to 100x lower latency and 500x lower energy **[vendor claim]**. https://www.hpcwire.com/aiwire/2025/05/22/innatera-unveils-pulsar-neuromorphic-microcontroller-for-the-sensor-edge/
- **SynSense:** Speck (event-vision sensor plus 320k-neuron SNN, mW-class) in mass production. Xylo (audio/IMU) shipped from H2 2023. Toolchain is Rockpool (Python). Most data is from 2023. https://www.eenewseurope.com/en/synsense-raises-us10-million-for-smart-vision-sensor/

### 3.3 Event-based sensors (real) vs event-driven displays (largely not)
- **Prophesee GenX320** (2023): 320×320 event sensor, down to 36 µW, targeting AR/VR and IoT. https://www.vision-systems.com/cameras-accessories/image-sensors/article/14300297/prophesee-launches-event-based-vision-sensor . Tobii partnership on event-based eye tracking for smart eyewear (2025-05). https://www.fiercesensors.com/keyword/prophesee-sa . The Qualcomm Snapdragon collaboration (2023) has **no confirmed shipping phone**. https://www.designnews.com/sensors/this-tiny-sensor-could-be-in-your-next-headset
- **Event-driven *displays*:** the search surfaced only a patent on driving pixels from asynchronous events (https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/10559247) and event-stream tone-mapping research (Frontiers in Neuroscience, 2016; https://hal.sorbonne-universite.fr/hal-01369653). **No commercial event-driven or "neuromorphic" display exists.** Display panels are still scanned with row/column timing. The real power and latency levers are variable refresh rate, panel self-refresh, partial updates, and foveation.
- **Real and shipping in XR:** eye-tracked foveated rendering (Meta Quest Pro ETFR, Vulkan only; Apple Vision Pro). https://developers.meta.com/horizon/documentation/unreal/unreal-eye-tracked-foveated-rendering/ ; https://jig.space/spatial-computing/foveated-rendering . WebXR exposes only *fixed* foveation (`XRWebGLLayer.fixedFoveation`). https://immersive-web.github.io/webxr/#dom-xrwebgllayer-fixedfoveation **[spec link from prior knowledge; not re-checked in this pass]**.

### 3.4 Client browser assessment
- Neuromorphic chips run spiking or sparse networks. They do not run general-purpose code, DOM layout, JS, or rasterization. Their realistic client role is **always-on sensor pre-processing** (keyword spotting, gesture, gaze, presence) inside earbuds, glasses, and phones, exposed to apps through OS APIs. A browser would see them only through existing or future high-level APIs (for example, gaze input in WebXR, or MediaStream processing). **"Neuromorphic rendering" of web content has no hardware or literature basis as of Oct 2026.**

---

## 4. Analog in-memory compute (AIMC)

- **IBM Research (Hermes-class):** a 14 nm chip with 64 PCM-based 256×256 AIMC cores, 63.1 TOPS peak, 9.76 TOPS/W (8-bit), near-software accuracy on ResNet and LSTM. Published in *Nature Electronics* in 2023-08. It is still a **research project**, and I found no product. https://www.theregister.com/2023/08/14/ibm_describes_analog_ai_chip/ ; https://ar5iv.labs.arxiv.org/html/2212.02872
- **EnCharge AI EN100** (announced 2025-05/06): switched-capacitor analog in-memory compute in 16 nm. The M.2 2280 card for laptops claims 200+ TOPS at 8.25 W with 32 GB at 68 GB/s **[vendor claim]**. The PCIe card has 4 NPUs, about 1 POPS, and 128 GB. **No OEM laptop shipping with EN100 was confirmed as of Oct 2026.** https://www.datacenterdynamics.com/en/news/encharge-ai-launches-its-analog-in-memory-en100-ai-accelerator/ ; https://www.eejournal.com/article/encharge-me-up-200-ai-tops-at-only-8-watts/
- **Mythic:** analog flash compute-in-memory. After near-collapse in 2022–23, it raised a $125M round in 2025-12 led by DCVC (investors include Honda and Lockheed Martin). It now targets data-center and edge (robotics, automotive, defense) uses. The M2000 has no confirmed launch. https://www.caproasia.com/?p=63570 ; https://multiples.vc/private-comps/mythic
- **Sagence AI** (formerly Analog Inference; out of stealth in 2024-11): multi-level flash with subthreshold compute. It claims Llama2-70B at 666K tok/s with 10x lower power and 20x lower cost than H100 **[vendor projection, no silicon benchmarks]**. https://www.techradar.com/pro/yet-another-tech-startup-wants-to-topple-nvidia-with-orders-of-magnitude-better-energy-efficiency-sagence-ai-bets-on-analog-in-memory-compute-to-deliver-666k-tokens-s-on-llama2-70b ; https://www.fiercesensors.com/ai/could-analog-unseat-nvidia-inference-sagence-ai-thinks-so
- **Client assessment:** AIMC is the most plausible "exotic" device to appear in a laptop or phone within 5–10 years, but **only as an NPU variant**. Its programming surface would be the OS ML runtime (DirectML/Windows ML, Core ML, NNAPI successors, ONNX Runtime EPs). The browser reaches it through **WebNN**. Its constraints are low-precision (INT4/INT8) weight-stationary inference, drift and calibration, and fixed model capacity. None of these needs a browser architecture change beyond WebNN device selection and tensor interop.

---

## 5. Quantum computing

### 5.1 Hardware status (Oct 2026)
- **Google Willow** (105 qubits): the Nature paper of 2024-12 showed **below-threshold surface-code scaling** (3×3 → 5×5 → 7×7, error rate halved at each step). https://blog.google/technology/research/google-willow-quantum-chip/ . "Quantum Echoes" (OTOC on 65 qubits, *Nature* 2025-10-22) claims the first *verifiable* quantum advantage, about 13,000x faster than classical simulation **[vendor framing; classical comparisons are frequently contested]**. Google says reaching useful scale will require "orders-of-magnitude improvement." https://www.networkworld.com/article/4077876/googles-quantum-chip-achieves-13000x-speed-advantage-over-supercomputers-2.html ; https://chemistry.berkeley.edu/node/2769
- **IBM:** Nighthawk (120 qubits, 218 tunable couplers, about 5,000 two-qubit gates) and Loon (qLDPC fault-tolerance test chip) were announced in 2025-11. IBM expects community-verified quantum advantage by the end of 2026 and runs a public advantage tracker. Its roadmap: Kookaburra (2026), Cockatoo (2027), **Starling (2029: 200 logical qubits, 100M gates, fault-tolerant)**, Blue Jay (2033+: 2,000 logical qubits). https://ibm.com/roadmaps/quantum ; https://siliconangle.com/2025/11/12/ibm-unveils-loon-nighthawk-next-generation-processors-scalable-fault-tolerant-quantum-computing/ ; https://techwireasia.com/2025/11/ibm-introduces-new-processor-as-it-targets-quantum-advantage-by-2026/
- **Quantinuum:** Helios (trapped ion, 98 qubits, 99.921% two-qubit fidelity, 48 logical qubits) launched in 2025-11. Roadmap: Sol (2027, about 100 logical qubits) and Apollo (2029, hundreds of logical qubits). These are S-1 filing targets. An Oracle partnership (2026-08-11) puts Helios in an OCI data center as a managed cloud service. https://www.sec.gov/Archives/edgar/data/0002110105/000162828026032836/quantinuum-sx1.htm
- **D-Wave** (annealing): Advantage2, *Science* 2025-03-12, "Beyond-classical computation in quantum simulation" (spin-glass dynamics vs Frontier). The supremacy claim is **disputed** by researchers using classical tensor-network methods. https://www.techrepublic.com/article/news-d-wave-advantage-2-quantum-computing ; https://www.sec.gov/Archives/edgar/data/1907982/000190798225000054/exhibit991beyondclassicald.htm

### 5.2 Where advantage plausibly exists
- Physics and chemistry simulation (quantum dynamics, molecular structure and NMR), certain sampling and verification tasks, and long-term Shor-type factoring and discrete logs. Optimization and ML advantage remain unproven. None of these are browser workloads. (Sources as above; IBM's tracker covers observable estimation, variational, and classically verifiable tasks: https://techwireasia.com/2025/11/ibm-introduces-new-processor-as-it-targets-quantum-advantage-by-2026/)

### 5.3 Why a quantum co-processor is not a plausible client browser accelerator
1. **Cryogenics:** superconducting qubits operate at about 10 mK in dilution refrigerators drawing 5–10 kW. https://qz.com/quantum-computer-data-center-architecture-explained-051126 . Trapped-ion and neutral-atom systems need ultra-high vacuum and laser systems. Neither fits a phone or laptop.
2. **Access model:** every vendor offers access through cloud job queues (IBM Quantum Platform, OCI/Quantinuum, D-Wave Leap). Round-trips take seconds to hours, compared with a frame budget of 8–16 ms.
3. **Workload mismatch:** browser work (parsing, layout, JS, rasterization, video, crypto) is classical and deterministic. No known quantum speedup applies.
4. **Scale gap:** fault-tolerant machines are targeted for 2029 (IBM Starling, Quantinuum Apollo) as *data-center* systems.
- The **only plausible web touchpoint** is a remote service: an HTTPS API to a cloud QPU. That is ordinary networking and needs no browser architecture.

### 5.4 The browser-relevant quantum topic is classical post-quantum cryptography
- **FIPS 203 (ML-KEM), FIPS 204 (ML-DSA), and FIPS 205 (SLH-DSA)** were published on 2024-08-13 (effective 08-14). **HQC** was selected in 2025-03 as a backup KEM (NISTIR 8545), with its draft standard pending. FN-DSA (FIPS 206) is still in draft. https://csrc.nist.gov/csrc/media/presentations/2025/nist-pqc-project/nist_pqc_the_road_ahead-moody_1.1.pdf ; https://www.encryptionconsulting.com/nist-pqc-standards-fips-203-204-205/ . (Primary: https://csrc.nist.gov/pubs/fips/203/final)
- **Deployed:** Chrome 131+ uses X25519MLKEM768 hybrid key exchange by default on desktop, and Firefox 132+ and Edge 131+ also default to it (secondary sources). An enterprise policy allows opting out. https://unveilscan.unveiltech.com/blog/post-quantum-tls.html ; https://www.encryptionconsulting.com/x25519mlkem768-tls-deployment-guide/ . It runs on ordinary CPUs.
- **Threat timeline:** Gidney (2025-05, arXiv 2505.15917) estimates that RSA-2048 could be broken in under a week with under 1M noisy qubits (0.1% error, 1 µs cycle). 2026 proposals claim about 100k (qLDPC) or about 10k (neutral atoms), both theoretical. https://huggingface.co/papers/2505.15917 ; https://thequantuminsider.com/2026/03/31/q-day-just-got-closer-three-papers-in-three-months-are-rewriting-the-quantum-threat-timeline/
- **HGBA-2.0 action:** make crypto agility (pluggable KEM and signature suites, hybrid by default, PQ certificate and signature readiness, and a WebCrypto ML-KEM/ML-DSA exposure path) a first-class requirement. This is the real "quantum" item for the spec.

---

## 6. VR/AR and display tech relevant to "neuromorphic rendering" claims
- **Real:** eye-tracked dynamic foveated rendering (Quest Pro, Vision Pro), event-based eye-tracking sensors (Prophesee and Tobii), VRR, and panel self-refresh. (Sources in Section 3.3.)
- **Lab or patents only:** event-driven pixel addressing (US patent above), event-stream tone mapping, and photonic ray-box intersection (*Nat. Commun.* 2025).
- **Recommendation:** HGBA-2.0 should model *damage-driven, gaze-aware, refresh-adaptive compositing* (already supported by OS compositors and WebXR) as the "event-driven rendering" story. It should not depend on neuromorphic hardware.

---

## 7. Extension-point lessons from past hardware absorption

| Hardware wave | How it reached the web | Lesson |
|---|---|---|
| GPU compositing (2010) | Chromium added a sandboxed **GPU process** that receives renderer commands and calls GL/D3D via **ANGLE**. It started behind `--enable-accelerated-compositing`. https://blog.chromium.org/2010/08/chromium-graphics-overhaul.html | New hardware enters through a **separate brokered process plus a translation layer**, gated by flags and blocklists before default-on. |
| SIMD | Wasm SIMD (fixed 128-bit) shipped in Chrome 91 (2021-05), Firefox 89, and Safari 16.4 (2023-03). Relaxed SIMD followed. https://v8.dev/features/simd ; https://caniuse.com/wasm-simd | Standardize a **portable lowest-common-denominator** abstraction (128-bit) that engines lower to SSE, NEON, or RVV. Width-specific features come later and stay optional. |
| Modern GPU APIs | WebGPU over Vulkan, Metal, and D3D12. Chrome 113 (2023), Safari 26, and Firefox 141 (Windows) in 2025; Linux and Android still in progress. https://web.dev/blog/webgpu-supported-major-browsers | It takes about a decade from proposal to cross-browser shipping, and platform coverage is uneven even after "shipping." The API abstracts over **OS-native APIs, never over vendor silicon**. |
| NPUs / ML accelerators | **WebNN**: a W3C Candidate Recommendation Draft (2026-08-13), in a Chrome 146 origin trial (2026-02). It has abstract device selection, MLTensor buffer sharing, and transformer ops, and maps to OS ML frameworks. https://www.w3.org/TR/webnn/ ; https://www.phoronix.com/news/Chrome-146-Beta | Exotic accelerators (analog, neuromorphic-as-NPU) would arrive **through the OS ML runtime and then WebNN**. The browser selects a device class ("npu," "gpu," "cpu") and never targets a vendor. |
| PQ crypto | Hybrid KEMs added to TLS stacks (BoringSSL, NSS) and shipped by default with enterprise opt-out. | **Algorithm agility plus hybrid transition**, behind policy switches. |

**Derived extension-point requirements for HGBA-2.0:**
1. **Compute-backend abstraction:** a WebNN-style device-class negotiation plus capability and limits queries. Back ends plug in through OS frameworks, not engine forks.
2. **Brokered device processes:** every new accelerator runs in a sandboxed service process (the GPU-process pattern) with IPC, fault isolation, and blocklisting.
3. **Portable SIMD and vector layer:** ISA-agnostic kernels (128-bit baseline, scalable-vector aware for RVV and SVE). JIT back ends are modular per ISA so riscv64 can be a supported tier.
4. **Feature gating:** flags, origin trials, and driver blocklists before default-on.
5. **Crypto agility:** pluggable KEM and signature suites, with hybrid PQ as the baseline.
6. **Explicit non-goals:** no direct quantum, photonic, or neuromorphic device APIs. These stay behind remote services or OS frameworks unless they appear as a standard device class.

---

## 8. Open gaps and caveats
- Phoronix and ithome pages returned 403 to the fetcher. Their benchmark figures come from search-indexed excerpts.
- I could not confirm: volume shipment of the DC-ROMA Mainboard III, finalization of the Android riscv64 NDK ABI, Q.ANT NPU 2.0 shipments, Loihi 3, the current release cadence of Lava, an OEM laptop with EnCharge EN100, or Lightmatter Passage production customers.
- Treat vendor efficiency multipliers (Envise, Pulsar, NorthPole, EN100, Sagence, D-Wave, Google Echoes) as marketing until independently reproduced.
