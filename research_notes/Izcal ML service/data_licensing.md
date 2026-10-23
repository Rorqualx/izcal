# Izcal ML service: data licensing

Compiled 21 October 2026. Primary sources were preferred (licence texts, model cards, dataset cards, papers, court and regulator pages). All URLs were accessed on 21 October 2026 unless a publication date is given.
Confidence: **High** = the licence statement or licence text was read at the source (a LICENSE/COPYING file, a model card's licence field, or a dataset card); **Medium** = an official page or a search result quoted the terms but the licence text was not read; **Low** = secondary source only, or not verified.

This strand covers every data set, weight and annotation source the ML service could use for image captioning, OCR, speech recognition and tokenizer/language-model training, under the owner's MIT-only rule of 19 October 2026 and the precedent in `research_notes/Phase 2 reconciliation/licence_audit.md`.

## Summary

- **The rule as written excludes almost every public corpus.** CC0/public domain and CC BY 4.0 are not MIT. Two owner decisions (**Q-D1**, **Q-D2**) decide whether Common Voice/Gutenberg and LibriSpeech/FLEURS/MLS can be used at all.
- **No large image-description corpus has MIT or CC0 images.** Every large corpus is URL-only metadata under CC BY 4.0 (LAION, DataComp, PixelProse, RedCaps, Conceptual Captions), or per-image Flickr/Commons rights (COCO, Visual Genome, Flickr30k, Open Images), or research-only.
- **LAION was taken down on 19 December 2023 after 1,008 suspected CSAM links were found; Re-LAION-5B (30 August 2024) removed 2,236 links.** A browser vendor cannot defend weights trained on it.
- **MIT weights that can ship:** BLIP-2, GIT, Florence-2 and Donut for captioning/document OCR; Moonshine for ASR; Distil-Whisper with a derivative caveat.
- **Whisper's licence conflicts:** the weight card says Apache-2.0; the README says code and weights are MIT. It cannot be treated as verified MIT.
- **No mainstream ASR or OCR engine passes:** Whisper, wav2vec 2.0, Tesseract and PaddleOCR are Apache-2.0; Silero's models are CC BY-NC 4.0; Vosk's models are mixed.
- **Speech:** LibriSpeech, FLEURS and MLS are CC BY 4.0; Common Voice is CC0; TED-LIUM, GigaSpeech and Emilia are non-commercial or access-restricted.
- **Law:** the US Copyright Office (9 May 2025) treats commercial, competing use of vast copyrighted troves, especially through illegal access, as beyond fair use; the EU AI Act Article 53 duties apply from 2 August 2025; the Hamburg LAION ruling (27 September 2024; appeal 10 December 2025) covers research dataset creation, not commercial training.
- **Operationally:** a D0–D5 data ledger, signed manifests, a fail-closed trainer and an independent sample audit are the only credible proof that no non-MIT data entered.

## 1. The rule, and the reading that decides the training plan

**Owner rule (19 October 2026, quoted in the licence audit):** "lets make a mit only, we dont want to be restricted in any way or by any measure, and if so we build our own." Everything Izcal ships must be MIT-licensed, or dual-licensed with MIT as one option (taken under MIT). Anything else that ships, Izcal builds.

**The data clause supplied with this strand:** training data must be **Izcal-commissioned** or **MIT-licensed**, with the licence recorded.

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 1.1 | The rule is about **shipped artifacts**. Model weights ship; the weight licence decides. Training data is an input; the data licence decides. A permissive weight licence does not cure a defective training-data chain, and a permissive data licence does not make a non-MIT weight shippable. | — | Owner rule; [licence audit](../Phase%202%20reconciliation/licence_audit.md) | 2026-10-19 | High |
| 1.2 | **CC0 1.0 and public domain are not MIT.** They impose no conditions, but the rule names MIT. A literal reading excludes Common Voice (CC0) and Project Gutenberg (public domain). This is the single most consequential ambiguity: it removes nearly every large public speech corpus. | — | [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/); [licence audit](../Phase%202%20reconciliation/licence_audit.md), OD-L6 | 2026-10-19 | High |
| 1.3 | **CC BY 4.0 is not MIT.** It permits commercial use and adaptation with attribution, but attribution is a condition and the rule admits no conditions except MIT's. A literal reading excludes LibriSpeech, FLEURS, MLS, COCO annotations, Visual Genome and Open Images annotations. | — | [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) | — | High |
| 1.4 | **Dual licences pass only if MIT is one choice.** "MIT OR Apache-2.0" passes (take MIT). "Apache-2.0" alone fails. "Apache-2.0 OR MIT" passes. "Apache-2.0 OR ISC OR MIT" passes. The audit already applies this test to crates. | — | Owner rule; audit class 1 | 2026-10-19 | High |
| 1.5 | **A dataset's metadata licence is not the images' licence.** LAION, DataComp, PixelProse, RedCaps and Conceptual Captions license only URLs and metadata (often CC BY 4.0); the images remain under their owners' rights. The card is not a rights grant for the pixels. | — | [LAION Re-LAION](https://laion.ai/blog/relaion-5b); [DataComp-1B](https://huggingface.co/datasets/mlfoundations/datacomp_1b) | 2024-08-30 | High |
| 1.6 | **Test-only inputs may have any licence.** OD-L2 allows WPT, test262, oracles and test data that never ship. Evaluation sets are the data analogue: COCO, IAM, Flickr30k and SROIE can be eval benchmarks, not training data. | — | [licence audit](../Phase%202%20reconciliation/licence_audit.md), OD-L2 | 2026-10-19 | High |
| 1.7 | **The audit's class system transfers to data.** Class 1 = MIT/dual-MIT; class 2 = not MIT but ships, so build; class 3 = test-only; class 4 = the standard itself; class 5 = OS. The data ledger in Section 9 uses D0–D5 in the same spirit. | — | [licence audit](../Phase%202%20reconciliation/licence_audit.md) | 2026-10-19 | High |

**Implications:** The ML spec must state its reading of the rule before it chooses a corpus. Two owner decisions unlock almost everything and should be taken first: **Q-D1**, whether CC0/public-domain data counts as commissioned-equivalent (recommended: yes, recorded separately, mirroring OD-L6), and **Q-D2**, whether CC BY 4.0 may train shipped weights (recommended: no; eval only). Without Q-D1 the only clean speech corpus is commissioned recording; without Q-D2 LibriSpeech, FLEURS and MLS are all excluded.

## 2. Model and weight licences: the decisive result

The task asks whether any usable captioning, OCR or ASR weights are MIT. The answer is **yes for captioning and document OCR, and almost no for ASR**. Each finding below is the licence field on the model card — the page from which the weights are actually distributed — not an inference from the code repository.

| # | Model | Task | Weight licence (as stated at the distribution point) | Source | Date | Conf. |
|---|---|---|---|---|---|---|
| 2.1 | **Whisper large-v3** (1,550M) | ASR / translation | **Apache-2.0** on the model card. The GitHub README says "Whisper's code and model weights are released under the MIT License." The two statements conflict; the weights' card is Apache-2.0. | [model card](https://huggingface.co/openai/whisper-large-v3); [README](https://raw.githubusercontent.com/openai/whisper/main/README.md) | paper 2022-12-06 | High (both statements read) |
| 2.2 | **CLIP ViT-L/14** | image-text | **No licence field** on the model card. The code repository is MIT. Absence of a weight licence is not MIT. | [model card](https://huggingface.co/openai/clip-vit-large-patch14); [LICENSE](https://raw.githubusercontent.com/openai/CLIP/main/LICENSE) | 2021 | High (absence read) |
| 2.3 | **SigLIP base** | image-text | **Apache-2.0** | [model card](https://huggingface.co/google/siglip-base-patch16-224) | — | High |
| 2.4 | **BLIP-2 OPT 2.7B** | captioning / VQA | **MIT** (not BSD-3-Clause, the common assumption); the card does not address the base OPT weights' terms | [model card](https://huggingface.co/Salesforce/blip2-opt-2.7b) | — | High |
| 2.5 | **GIT base** | captioning | **MIT** | [model card](https://huggingface.co/microsoft/git-base) | — | High |
| 2.6 | **PaliGemma 3B** | VLM | **Gemma licence** (not MIT) | [model card](https://huggingface.co/google/paligemma-3b-pt-224) | — | High |
| 2.7 | **Florence-2 large** | captioning / OCR / detection | **MIT** | [model card](https://huggingface.co/microsoft/Florence-2-large) | — | High |
| 2.8 | **TrOCR base printed** | OCR | **No licence field** on the model card. The `unilm` repository that holds TrOCR is MIT; the weights themselves carry no stated licence. | [model card](https://huggingface.co/microsoft/trocr-base-printed); [unilm LICENSE](https://raw.githubusercontent.com/microsoft/unilm/master/LICENSE) | — | High (absence read) |
| 2.9 | **Donut base** | document OCR / understanding | **MIT** | [model card](https://huggingface.co/naver-clova-ix/donut-base) | — | High |
| 2.10 | **Moondream2** | VLM | **Apache-2.0** | [model card](https://huggingface.co/vikhyatk/moondream2) | — | High |
| 2.11 | **SmolVLM-Instruct** | VLM | **Apache-2.0** | [model card](https://huggingface.co/HuggingFaceTB/SmolVLM-Instruct) | — | High |
| 2.12 | **Qwen2-VL-7B** | VLM | **Apache-2.0** | [model card](https://huggingface.co/Qwen/Qwen2-VL-7B-Instruct) | — | High |
| 2.13 | **wav2vec2-base-960h** | ASR | **Apache-2.0** | [model card](https://huggingface.co/facebook/wav2vec2-base-960h) | — | High |
| 2.14 | **Moonshine** | ASR | **MIT** — the only mainstream ASR weight set found that passes the rule outright | [model card](https://huggingface.co/UsefulSensors/moonshine) | — | High |
| 2.15 | **Distil-Whisper large-v3** | ASR | **MIT** on the card, but it is a distillation of Whisper (Apache-2.0 on the weight card). The MIT tag may not override the base weights' terms. | [model card](https://huggingface.co/distil-whisper/distil-large-v3) | — | High (card); derivative risk Medium |
| 2.16 | **Silero VAD** | voice activity | **MIT** | [repo licence](https://api.github.com/repos/snakers4/silero-vad) | — | High |
| 2.17 | **Silero models** (STT/TTS) | ASR / TTS | **CC BY-NC 4.0**; only the `base` CIS TTS models are MIT. Non-commercial, so unusable. | [README](https://raw.githubusercontent.com/snakers4/silero-models/master/README.md) | — | High |
| 2.18 | **Vosk API** | ASR toolkit | **Apache-2.0**. Model licences are mixed: Apache-2.0, MIT, AGPL, LGPL-3.0, GPL-3.0 and CC BY-NC-SA 4.0, model by model. | [repo licence](https://api.github.com/repos/alphacep/vosk-api); [model list](https://alphacephei.com/vosk/models) | — | High |
| 2.19 | **Kaldi** | ASR toolkit | **Apache-2.0** (COPYING; GitHub reports NOASSERTION) | [COPYING](https://raw.githubusercontent.com/kaldi-asr/kaldi/master/COPYING) | — | High |
| 2.20 | **Tesseract `tessdata`** | OCR | **Apache-2.0** | [repo licence](https://api.github.com/repos/tesseract-ocr/tessdata) | — | High |
| 2.21 | **PaddleOCR** | OCR | **Apache-2.0** | [repo licence](https://api.github.com/repos/PaddlePaddle/PaddleOCR) | — | High |

**The exact result.** MIT weights exist for **captioning and document OCR**: BLIP-2, GIT, Florence-2 and Donut. MIT weights exist for **one ASR family**: Moonshine, with Distil-Whisper as a derivative-risk caveat. No mainstream ASR weight set passes outright: Whisper is Apache-2.0 at the distribution point (with a conflicting MIT sentence in its README), wav2vec 2.0 is Apache-2.0, Silero's ASR/TTS models are CC BY-NC 4.0, and Vosk's models are mixed. No mainstream OCR **engine** passes: Tesseract and PaddleOCR are Apache-2.0, so their traineddata cannot ship under the rule; they may be black-box oracles only (OD-L2).

**Implications:** The ML spec's M-1 section must list every weight by name, version, distribution URL, licence field and SHA-256, and must show that each is MIT or dual-MIT. The B10 ML inventory should carry the same fields for every shipped classifier. The ASR path has two defensible options: ship **Moonshine** (MIT) after evaluation, or **train Izcal's own ASR** on commissioned/CC0 audio (Section 5); Whisper cannot ship as-is until the Apache/MIT conflict is resolved in writing. For OCR, use **Florence-2 or Donut** (MIT) as the shipped model, or train Izcal's own OCR head; do not ship Tesseract or PaddleOCR traineddata.

## 3. Image captioning data

| # | Source | What it is | Licence actually verified | Source | Date | Conf. |
|---|---|---|---|---|---|---|
| 3.1 | **COCO** | 330K images, 5 captions each | **Annotations CC BY 4.0**; images are from Flickr under **per-image Flickr licences**, not a COCO grant. The terms page did not render; the wording is reproduced by multiple secondary sources. | [cocodataset.org](https://cocodataset.org/#termsofuse) | — | Medium |
| 3.2 | **Visual Genome** | 108K images, dense region descriptions | **CC BY 4.0** per the author page; images are drawn from the YFCC100M/COCO intersection, so image rights vary per image. | [author page](https://homes.cs.washington.edu/~ranjay/visualgenome/index.html); [HF card](https://huggingface.co/datasets/ranjaykrishna/visual_genome) | — | Medium |
| 3.3 | **Conceptual Captions** | ~3.3M image-URL/caption pairs from alt text | No dataset licence statement found in the repository README. Only URLs are distributed; the images are not. The captions are cleaned web alt text, so their own chain of title is the page owner's. | [repo](https://github.com/google-research-datasets/conceptual-captions) | 2018-09-05 | Low |
| 3.4 | **Flickr30k** | Flickr images with 5 captions each | **Non-commercial research and education only**; images remain under **Flickr Terms of Use**; "We do not own the copyright of the images." | [dataset page](https://shannon.cs.illinois.edu/DenotationGraph/) | — | High |
| 3.5 | **TextCaps** | captioning with in-image text, built on Open Images | No dataset licence found on the project page. Images inherit Open Images (CC BY 2.0); annotations are not clearly licensed. | [textvqa.org/textcaps](https://textvqa.org/textcaps/) | — | Low |
| 3.6 | **Localized Narratives** | 671K Open Images plus COCO, Flickr30k and ADE20K images, with narrated descriptions | **Annotations CC BY 4.0**; images inherit the source dataset's licences. | [project page](https://google.github.io/localized-narratives/) | — | High |
| 3.7 | **Open Images** | Large image set with boxes and labels | **Annotations CC BY 4.0; images CC BY 2.0**; the page says to verify each image. | [facts and figures](https://storage.googleapis.com/openimages/web/factsfigures.html) | — | High |
| 3.8 | **LAION-400M / LAION-5B** | 400M / 5B CLIP-filtered image-text pairs | **Metadata CC BY 4.0**; only links and metadata are distributed; "LAION has never distributed image content itself." Images keep their owners' rights. | [LAION-5B announcement](https://laion.ai/laion-5b-a-new-era-of-open-large-scale-multi-modal-datasets) | 2022-03-31 | High |
| 3.9 | **LAION takedown and Re-LAION-5B** | CSAM findings and re-release | **19 December 2023:** LAION took down all known accessible LAION-5B datasets and derivatives after Stanford Internet Observatory reported **1,008** links to CSAM or likely CSAM. **30 August 2024:** Re-LAION-5B removed **2,236** links matched against IWF, C3P and Stanford hash lists; a "research-safe" variant also filters NSFW. | [Re-LAION blog](https://laion.ai/blog/relaion-5b); [Stanford report](https://purl.stanford.edu/kh752sm9123) | 2023-12-19 / 2024-08-30 | High |
| 3.10 | **DataComp-1B** | 1B-scale Common Crawl image-text pairs | **Metadata CC BY 4.0**; "The individual images are under their own copyrights." | [HF card](https://huggingface.co/datasets/mlfoundations/datacomp_1b) | — | High |
| 3.11 | **PixelProse** | Synthetic-caption image-text pairs | **CC BY 4.0** metadata; images from Common Crawl, own rights. | [HF card](https://huggingface.co/datasets/tomg-group-umd/pixelprose) | — | High |
| 3.12 | **RedCaps** | Reddit image-caption pairs | **CC BY 4.0** (not MIT; images remain Reddit users'/hosts' rights). | [HF API](https://huggingface.co/api/datasets/red_caps) | — | High |
| 3.13 | **Wikipedia / Wikimedia Commons pairs** | Article text with Commons images; WIT-style corpora | **Wikipedia text CC BY-SA 4.0**; Commons images are **per-file** (CC BY, CC BY-SA, public domain). The text share-alike is incompatible with the MIT-only rule for training. | [wikipedia.org](https://www.wikipedia.org/) | — | High (text); Medium (WIT) |
| 3.14 | **Screen-reader and web alt text** | The alt attributes and accessibility text authors write on pages | No blanket licence. Alt text is authored expression owned by the page author; scraping it is not licensed by the page's terms. It is also often personal or low quality. | Analysis; the Copyright Office report treats web text as copyrighted (Section 6) | 2025-05-09 | Low |
| 3.15 | **Foundation-1** | Named in the brief as an image-description corpus | **Not found** in this strand's searches. No licence, size or publisher verified. Treat as unverified and unusable until a primary card is read. | — | — | Low |
| 3.16 | **MIT/CC0 image-description corpora** | A corpus whose images, not only annotations, are MIT or CC0 | **None found at scale.** Every large corpus found is URL-only metadata under CC BY 4.0, or images under Flickr/Commons/per-file licences, or research-only. | This table | — | Medium |

**What CC BY 4.0 annotations mean for weights trained on them.** CC BY 4.0 grants reproduction, adaptation and commercial use **with attribution**, and has no share-alike. Whether model weights are an "adaptation" of the annotations is unsettled: if they are, the attribution condition would attach to the weights and the model card would have to carry it; if they are not — the majority practice, and the premise of the Copyright Office's market analysis — the annotation licence does not reach the weights. The decisive problem is not the annotation licence but the **images**: COCO's terms do not grant rights to the images, and the images carry per-image Flickr licences. A model trained on COCO images therefore has no clean chain of title, whatever the annotation licence says. Attribution alone would not satisfy Izcal's rule in any case: **CC BY is not MIT**.

**Implications:** No crawled or Flickr-derived image-description corpus can be **training** data under the rule. COCO, Visual Genome, Flickr30k, Localized Narratives, TextCaps, Open Images, LAION, DataComp, PixelProse and RedCaps are all disqualified at the image level; their annotations are disqualified at the licence level. They remain useful as **evaluation** sets under an OD-L2-style exception, which the ML spec should state explicitly. The only clean captioning-data route is **commissioned photography with commissioned descriptions** (Section 8), or synthetic images from Izcal's own renderer with commissioned captions. LAION is additionally excluded by the CSAM and provenance findings: a browser vendor cannot defend shipping weights trained on a corpus that required a CSAM takedown.

## 4. OCR and document data

| # | Source | What it is | Licence actually verified | Source | Date | Conf. |
|---|---|---|---|---|---|---|
| 4.1 | **IAM Handwriting** | Handwritten English forms and lines | **Non-commercial research only.** The FKI page states the database "may be used for non-commercial research purpose only." | [FKI page](https://fki.tic.heia-fr.ch/databases/iam-handwriting-database) (certificate expired; exact line via search result) | — | Medium |
| 4.2 | **FUNSD** | 199 scanned forms with entity labels | **No licence found.** The site's "License" link leads to a broken path; the GitHub repository returns 404 for the licence file. | [project page](https://guillaumejaume.github.io/FUNSD/) | — | Low |
| 4.3 | **SROIE** | ~1,000 scanned receipts, ICDAR 2019 | **No dataset licence stated.** Download requires an RRC portal login and the portal's terms; the paper states none. | [RRC task](https://rrc.cvc.uab.es/?ch=13); [paper](https://arxiv.org/abs/2103.10213) | 2019 | Low |
| 4.4 | **DocVQA** | Document images with questions | **No dataset licence stated** on the page or paper; download through the RRC portal after login. | [docvqa.org](https://www.docvqa.org/datasets) | — | Low |
| 4.5 | **PDF/A and PDF/UA corpora (veraPDF)** | ISO 19005 / ISO 14289 test files | **No licence field** in the repository; GitHub reports `license: null`. | [veraPDF corpus](https://api.github.com/repos/veraPDF/veraPDF-corpus) | — | Low |
| 4.6 | **Common Crawl text** | Web pages, monthly crawls | **Limited, non-exclusive licence**; crawled content "may be subject to separate terms of use"; users must "respect the copyrights and other applicable rights of third parties." Not an open licence. | [terms of use](https://commoncrawl.org/terms-of-use) | — | High |
| 4.7 | **Synthetic text rendering** | Izcal-generated page images from text and fonts | The output images are Izcal's own work; the **fonts** are the licence issue. OFL and Apache fonts permit commercial use and do not make the rendered image a derivative, but they are not MIT and cannot ship as fonts under OD-L7. MIT-licensed fonts exist but none was verified here. | Analysis; [licence audit](../Phase%202%20reconciliation/licence_audit.md), OD-L7 | 2026-10-19 | Medium |
| 4.8 | **Wikipedia text** | Article text for tokenizers, language data and OCR text corpora | **CC BY-SA 4.0.** Share-alike; not MIT. | [wikipedia.org](https://www.wikipedia.org/) | — | High |
| 4.9 | **Project Gutenberg** | ~75K public-domain ebooks | **Public domain in the United States** (some titles by permission); the **Project Gutenberg licence and trademark** attach to the ebook files. Strip the header and references to get the unrestricted text; check non-US copyright. | [licence page](https://www.gutenberg.org/policy/license.html) | — | High |
| 4.10 | **Government documents** | Laws, reports, forms, gazettes | **US federal works are public domain** (17 U.S.C. §105); UK Crown copyright is under the **Open Government Licence**, which is not MIT; EU member states vary. | General legal framework | — | Low |
| 4.11 | **MIT document models** | Florence-2, Donut | **MIT** weights (Section 2), usable for OCR/document understanding without a training-data licence if shipped as-is. | [Florence-2](https://huggingface.co/microsoft/Florence-2-large); [Donut](https://huggingface.co/naver-clova-ix/donut-base) | — | High |

**Implications:** OCR training data must be **commissioned or synthetic**. IAM, SROIE, DocVQA and the veraPDF corpus are unusable for training: IAM is explicitly non-commercial, and the others have no licence that a vendor could rely on. Common Crawl cannot supply OCR ground truth because the pages carry third-party rights and no per-item licence. The clean pipeline is: **Izcal's own page renderer + Izcal's own fonts** (OD-L7) to generate printed-text images with exact ground truth, plus **commissioned handwriting and form scans** with signed releases for the handwriting and any personal data. Project Gutenberg and US federal documents supply clean text for the language side, after stripping the PG licence and checking non-US status.

## 5. Speech corpora

| # | Source | What it is | Licence actually verified | Source | Date | Conf. |
|---|---|---|---|---|---|---|
| 5.1 | **LibriSpeech** | Read audiobooks from LibriVox | **CC BY 4.0** | [OpenSLR 12](https://www.openslr.org/12/) | — | High |
| 5.2 | **LibriLight** | Large unlabelled audiobook corpus | **CC BY 4.0** (commonly stated; the HF card is gated and returned 401, so the licence text was not read here) | [OpenSLR 31](https://www.openslr.org/31/) | — | Medium |
| 5.3 | **Common Voice** | 30,000+ hours of crowdsourced read speech, 100+ languages | **CC0 1.0.** Mozilla states the dataset is "free under a CC0 license"; since October 2025 it is distributed through the Mozilla Data Collective, not Hugging Face. | [Mozilla blog](https://www.mozillafoundation.org/en/blog/common-voice-18-dataset-release); [CC0 playbook](https://common-voice.github.io/community-playbook/sub_pages/cc0waiver_process.html) | 2024-06-19 | High |
| 5.4 | **FLEURS** | 102-language read speech | **CC BY 4.0** (dataset card metadata) | [HF API](https://huggingface.co/api/datasets/google/fleurs) | — | High |
| 5.5 | **Multilingual LibriSpeech (MLS)** | Multilingual read audiobooks | **CC BY 4.0** | [OpenSLR 94](https://www.openslr.org/94/) | — | High |
| 5.6 | **TED-LIUM 3** | 452 hours of TED talks | **CC BY-NC-ND 3.0** — non-commercial, no derivatives. Unusable for training or shipping. | [TensorFlow Datasets catalog](https://www.tensorflow.org/datasets/catalog/tedlium); [OpenSLR 51](https://www.openslr.org/51) | — | Medium |
| 5.7 | **GigaSpeech** | Audiobooks, podcasts and YouTube audio | The card says **Apache-2.0**, but the access terms require the researcher to use the database **"only for non-commercial research and educational purposes."** The access terms are the operative restriction. | [HF card](https://huggingface.co/datasets/speechcolab/gigaspeech) | — | High |
| 5.8 | **Emilia** | In-the-wild speech | The metadata says **cc-by-4.0**, but the card text says use is permitted **"only for non-commercial purposes under the CC BY-NC-4.0 license."** The card text controls; treat as non-commercial. | [HF card](https://huggingface.co/datasets/amphion/Emilia-Dataset) | — | High |
| 5.9 | **YouTube-derived corpora** | Audio pulled from YouTube (e.g. AudioSet-style) | YouTube's Terms of Service prohibit downloading except through the service; the audio remains the uploader's. No licence flows to the corpus builder. | YouTube Terms (general) | — | Low |
| 5.10 | **Podcast corpora** | Spotify Podcast Dataset and similar | Research licence; not redistributable; episode rights remain with publishers. | General; not re-verified here | — | Low |
| 5.11 | **Commissioned recording** | Voice talent reading Izcal scripts | Market rates: **$200–$500 per finished hour** for audiobook-grade narration; **$225 per raw hour** non-edited; some quotes start at **$150 per finished hour**. | [Voicecrafters rate guide](https://www.voicecrafters.com/industry-standard-voice-over-rates); [Voice Acting 101](https://voiceacting101.com/voice-over-rates) | — | Medium |
| 5.12 | **LibriVox** | Volunteer public-domain audiobooks (the source of LibriSpeech) | **Public domain** recordings; the readers waive rights. The underlying texts are usually PD in the US. | [LibriVox](https://librivox.org/) | — | Medium |
| 5.13 | **Transcripts and source audio** | The transcript of a recording inherits that recording's licence; a third-party transcript cannot be licensed more permissively than the audio it transcribes. Commissioned transcripts of commissioned audio are D0. | Analysis; no dataset found that grants a transcript licence broader than its audio licence | — | High |

**Implications:** Under the literal rule, **no large speech corpus can train shipped weights**: LibriSpeech, LibriLight, FLEURS and MLS are CC BY 4.0; Common Voice is CC0 but not MIT; TED-LIUM, GigaSpeech and Emilia are non-commercial; YouTube and podcast corpora have no valid chain. The ML spec must either (a) take **Q-D1** (CC0/PD allowed) and use **Common Voice** plus LibriVox, which unlocks a viable ASR training set at zero data cost, or (b) commission read speech at roughly **$200–$500 per finished hour** plus transcription and QC. If the owner also takes **Q-D2** (CC BY allowed), LibriSpeech and MLS become available; that is the larger unlock and should be decided explicitly, not assumed.

## 6. Text for tokenizers and language models

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 6.1 | **Tokenizers inherit the corpus licence.** Training a BPE tokenizer on Wikipedia makes the tokenizer a derived artifact of CC BY-SA text; training it on Gutenberg or commissioned text avoids the question. The **tokenizer code** licence is separate: SentencePiece and Hugging Face `tokenizers` are Apache-2.0 (not MIT), `tiktoken` is MIT. Verify each at implementation. | — | General; code licences from memory, not read here | — | Low |
| 6.2 | **Wikipedia text is CC BY-SA 4.0.** Share-alike is incompatible with the MIT-only rule if the tokenizer or weights are treated as adaptations. | — | [wikipedia.org](https://www.wikipedia.org/) | — | High |
| 6.3 | **Common Crawl is not an open licence.** It grants a limited licence and requires respect for third-party rights; the pages themselves remain copyrighted. | — | [terms](https://commoncrawl.org/terms-of-use) | — | High |
| 6.4 | **Project Gutenberg text is public domain in the US**, with the PG licence and trademark on the files. Strip the boilerplate; check non-US copyright. | — | [licence](https://www.gutenberg.org/policy/license.html) | — | High |
| 6.5 | **US Copyright Office, Part 3 (pre-publication, 9 May 2025).** Training involves *prima facie* reproduction. Fair use turns on the works used, their source, the purpose and the controls on output. Research and analysis uses are likely transformative; **commercial use of vast troves to produce expressive content that competes in existing markets, especially through illegal access, goes beyond established fair-use boundaries.** The Office recommends letting licensing markets develop, with extended collective licensing only if market failure appears. | 108 pages; conclusion at pp. 107–108 | [report PDF](https://www.copyright.gov/ai/Copyright-and-Artificial-Intelligence-Part-3-Generative-AI-Training-Report-Pre-Publication-Version.pdf); [report page](https://www.copyright.gov/ai/) | 2025-05-09 | High |
| 6.6 | **EU AI Act Article 53.** Providers of general-purpose models must (c) put in place a copyright policy that identifies and honours reservations of rights under Article 4(3) of Directive (EU) 2019/790, including machine-readable means; and (d) publish a **sufficiently detailed summary of training content** using the AI Office template. Applies from **2 August 2025**. The Commission published the template on **24 July 2025**. | — | [Article 53](https://artificialintelligenceact.eu/article/53/); [template report](https://www.lawsociety.ie/gazette/top-stories/2025/july/ai-models-must-now-set-out-training-data-provenance) | 2025-07-24 / 2025-08-02 | High (Art. 53); Medium (template date) |
| 6.7 | **Hamburg, Kneschke v LAION.** The Regional Court (27 September 2024, 310 O 227/23) dismissed the claim: LAION's reproduction of a photograph to build a research dataset fell under the **§60d TDM exception** for scientific research. The appeal was dismissed by the Higher Regional Court on **10 December 2025**. This validates **dataset creation for research**, not commercial model training. | — | [WIPO judgment record](https://www.wipo.int/wipolex/en/judgments/details/2381); [appeal report](https://www.technologyslegaledge.com/2025/12/robert-kneschke-v-laion-judgment-of-10-december-2025-ref-5-u-104-24) | 2024-09-27 / 2025-12-10 | High (first instance); Medium (appeal) |
| 6.8 | **US training cases.** *Kadrey v. Meta* (N.D. Cal., 25 June 2025) granted Meta summary judgment on fair use for training. *Bartz v. Anthropic* (N.D. Cal., 23 June 2025) held training on lawfully acquired books fair use but training on a pirated library not fair use; the case settled for **$1.5 billion** in September 2025. The line is **lawful source**, not "training is always fair." | $1.5B | Secondary reports; not read in the opinions | 2025-06-23 / 2025-06-25 / 2025-09 | Medium |
| 6.9 | **Getty Images v Stability AI** (UK High Court, [2025] EWHC 2863 (Ch), **4 November 2025**). The judgment date is verified; the outcome was not read in this strand. Secondary reports describe a mixed result, with Getty's copyright training claim failing and a trade-mark finding. Treat the outcome as **unverified** until the judgment is read. | — | [judgment page](https://www.judiciary.uk/judgments/getty-images-v-stability-ai/) | 2025-11-04 | Low (date High) |
| 6.10 | **Deduplication and PII scrubbing are not optional.** They are quality and lawful-basis controls: duplicates amplify memorisation, and personal data in training text triggers GDPR obligations. The Copyright Office report treats the amount and source of works as fair-use factors. | — | [report](https://www.copyright.gov/ai/) | 2025-05-09 | High (principle); Low (method) |

**Implications:** The ML spec's training plan should adopt the **Copyright Office's line as its own**: train on commissioned, MIT, CC0/public-domain or otherwise clearly licensed text; do not rely on fair use for crawled text; honour machine-readable TDM reservations (Article 53(1)(c)); and publish an Article 53(1)(d) summary using the 24 July 2025 template. For tokenizers, prefer **Project Gutenberg, US government text and commissioned text** over Wikipedia and Common Crawl, so the tokenizer's provenance is clean. Treat the Hamburg ruling as support for a **research dataset**, not as permission to train a shipped model on crawled images.

## 7. Consent, provenance and attribution

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 7.1 | **Datasheets for Datasets** propose documenting motivation, composition, collection, preprocessing, uses, distribution and maintenance for every dataset. This is the minimum disclosure Izcal should attach to each commissioned corpus. | — | [arXiv 1803.09010](https://arxiv.org/abs/1803.09010) | 2018-03-23; CACM 2021-12 | High |
| 7.2 | **Data Statements for NLP** propose documenting language, speaker/annotator demographics, provenance and curation for NLP data. Use them for speech and text corpora. | — | [TACL 2018](https://aclanthology.org/Q18-1041/) | 2018 | High |
| 7.3 | **Dataset Providers Alliance (DPA).** Formed **26 June 2024**; the AI Data Licensing Position Paper was published **4 September 2024** (not 2025). It favours direct, free-market licensing over compulsory collective licensing; **robust opt-in** for people's data and likeness; and rights metadata such as IPTC photo metadata. | — | [DPA launch](https://www.thedpa.ai/post/leading-dataset-licensors-unite-to-launch-the-dataset-providers-alliance-dpa); [position paper](https://www.thedpa.ai/post/dpa-unveils-comprehensive-ai-data-licensing-position-paper) | 2024-06-26 / 2024-09-04 | High |
| 7.4 | **C2PA Content Credentials 2.1** is a provenance manifest for media: who made it, with what tools, and what edits were applied. It does **not** record training-data licences; it can record assertions about generated outputs. The specification page shows version 2.1 but no publication date. | — | [C2PA 2.1](https://spec.c2pa.org/specifications/specifications/2.1/index.html) | — | Medium |
| 7.5 | **robots.txt and opt-outs are weak but relevant.** The Copyright Office report records that robots.txt is widely used but "has significant limitations," is not designed for generative-AI scraping, and cannot bind platforms whose content is copied elsewhere. The EU route is stronger: Article 4(3) TDM reservations and Article 53(1)(c) require honouring machine-readable opt-outs. | — | [report p.102](https://www.copyright.gov/ai/); [Article 53](https://artificialintelligenceact.eu/article/53/) | 2025-05-09 | High |
| 7.6 | **What a browser vendor can defensibly commission.** Own photography and screen recordings with signed contributor releases; commissioned read speech and handwriting; public-domain scans; synthetic renders from Izcal's own fonts. Faces need a **model release**; minors need verifiable parental consent (COPPA in the US, GDPR Article 8 in the EU); medical images need explicit consent and de-identification (HIPAA in the US, GDPR Article 9 in the EU); biometric identifiers can trigger BIPA in Illinois. | — | Legal framework; no single source | — | Low |
| 7.7 | **Crowd-work terms.** Pay at least the local living wage, not piece rates that fall below it; obtain informed consent that names AI training as the use; give a withdrawal route; do not use minors; keep payment and consent records in the ledger. | — | Analysis; DPA opt-in principle | — | Low |
| 7.8 | **User browsing data is not a training source.** The ML service runs on user pages under the isolation and privacy principles (P1, B10). Training on pages a user visits would require explicit, separate consent and would create a cross-site data flow the architecture forbids. | — | [HGBA-2 P1, B10]; HGBA-3 B10 | — | High (architecture) |

**Implications:** The ML spec should require a **Datasheet and a Data Statement per corpus**, a **signed release per contributor**, and a **rights-metadata record per item**. It should also state, in the privacy section, that the captioning model never trains on user browsing data, and that accessibility repairs on user pages are inference-only. The DPA's opt-in position is the closest industry norm to the owner's rule and should be cited as the consent basis for commissioned data.

## 8. Commissioned-data costs, throughput and quality

| # | Finding | Numbers | Source | Date | Conf. |
|---|---|---|---|---|---|
| 8.1 | **Annotation labour rates.** Managed-service annotator hours cluster at **$4–$12/hour**; generalist rates run **$8–$20/hour**. Per-label pricing: bounding boxes **$0.03–$0.12 per box**, classification **~$0.03 per image**, polygons from **$0.06**, per-image pricing **$0.01–$5.00** depending on task. | $4–20/hr; $0.03–0.12/box | [HabileData](https://www.habiledata.com/blog/image-annotation-pricing); [Basic.ai](https://www.basic.ai/blog-post/how-much-do-data-annotation-services-cost-complete-guide); [Annotation Box](https://annotationbox.com/pricing) | 2026 | Medium |
| 8.2 | **Voice recording rates.** Audiobook-grade narration **$200–$500 per finished hour**; non-edited raw recording **~$225/hour**; lower quotes **$150/hour**. A "finished hour" is edited, mastered audio and typically takes 2–4 studio hours per finished hour. | $200–500/finished hour | [Voicecrafters](https://www.voicecrafters.com/industry-standard-voice-over-rates); [Voice Acting 101](https://voiceacting101.com/voice-over-rates) | — | Medium |
| 8.3 | **Transcription rates.** Human transcription and QC **$10–$60 per audio hour**, depending on language, speaker count and turnaround. | $10–60/audio hour | [Sonix](https://sonix.ai/pricing); vendor rate guides | 2026 | Medium |
| 8.4 | **Throughput (estimates set by this strand, not measured).** Short image captions: **20–40 images/hour/annotator**. Classification or box verification: **100–300 images/hour**. Full OCR ground truth: **1–3 pages/hour**. Audio verification: **2–4 minutes per audio minute**. | † | This strand | 2026-10-21 | Low |
| 8.5 | **Caption agreement.** COCO's convention is **5 independent captions per image**; that is the minimum for measuring inter-annotator agreement. Report pairwise CIDEr/SPICE and a human adequacy rating; set the threshold after a pilot rather than inventing one. | 5 captions/image | [COCO](https://cocodataset.org/#termsofuse) | — | Medium |
| 8.6 | **Transcript quality.** Double-transcribe a **5% sample** and report WER. Proposed targets: **≤2%†** for commissioned read speech, **≤5%†** for spontaneous speech. These are targets, not measured results. | † | This strand | 2026-10-21 | Low |
| 8.7 | **Worked example (arithmetic, not a quote).** 10,000 images × 5 captions = 50,000 captions; at 30 captions/hour that is **1,667 annotator-hours**, or **$13k–$20k** at $8–$12/hour, before photography and review. 1,000 hours of commissioned speech at $250/finished hour is **$250k** for voice and studio; transcription at $60/audio hour adds **$60k**; 20% QC adds about **$62k**; total roughly **$370k†**. | † | This strand | 2026-10-21 | Low (arithmetic) |
| 8.8 | **Labour and consent.** Pay above the local minimum, disclose that the work trains AI, allow withdrawal, do not use minors, and keep payment records. These are conditions of a defensible commission, not optional ethics. | — | DPA opt-in principle; legal framework | 2024-09-04 | Low |

**Implications:** The training plan should budget commissioned data as a **first-class cost**, not a rounding error: captions are cheap per item but add up at corpus scale, and speech is dominated by voice and studio time, not annotation. Use the throughput figures only as planning estimates† and replace them with measured rates after the first 1,000 items. The commission contract must carry the consent and withdrawal terms, because a withdrawal after training cannot be undone in the weights — only excluded from future runs.

## 9. What the rule implies operationally: the data ledger

The rule "training data must be Izcal-commissioned or MIT-licensed with the licence recorded" is only enforceable if **every item** has a recorded basis and the training pipeline can read nothing else. The audit's class system is the precedent: each code dependency gets a class; each data item should get a **D-class**.

| D-class | Meaning | Training | Shipping | Precedent |
|---|---|---|---|---|
| **D0** | **Commissioned** by Izcal: own photography, own recordings, own synthetic renders, work-for-hire with signed releases | Allowed | Allowed | New; the data analogue of "build our own" |
| **D1** | **MIT** or dual-licensed with MIT as one choice | Allowed | Allowed | Audit class 1 |
| **D2** | **CC0 1.0 / public domain** | Allowed only if **Q-D1** is answered yes | Allowed only with Q-D1 | Audit class 4 analogue (OD-L6) |
| **D3** | **CC BY** (any version) | **Not allowed** unless **Q-D2** is answered yes | Not allowed | Audit class 2 analogue |
| **D4** | **Research / non-commercial / no licence** (IAM, Flickr30k, TED-LIUM, Emilia, SROIE, DocVQA) | **Never** | Never | Audit class 3 (OD-L2); eval only |
| **D5** | **Unknown or conflicting** (Foundation-1, TrOCR weights, Whisper's conflict, FUNSD) | **Quarantine** | Quarantine | Fail closed |

**Per-item record (JSON Lines or a SQLite table), one row per image, audio file, document or text shard:**

| Field | Type | Rule |
|---|---|---|
| `item_id` | `sha256:<hex>` of the bytes | Content-addressed; duplicates collapse to one row |
| `source_id`, `source_url`, `retrieved_at` | string, URL, ISO date | No item without a source |
| `licence_spdx` | SPDX ID or `commissioned` / `PD` | Must be in the allowlist for its D-class |
| `licence_url`, `licence_sha256` | URL, hash | The licence text is stored and hashed, not just named |
| `rights_basis` | D0–D5 | Set by the licence gate, reviewed by a human |
| `consent_id`, `release_id` | UUID | Required for D0 with people, faces, minors, medical data |
| `subject_flags` | `face`, `minor`, `medical`, `biometric` | Any flag forces a consent review |
| `annotator_id_hash`, `task_id`, `rate_usd_hr` | hash, string, decimal | Labour record; no raw worker identity in the training set |
| `pii_flags`, `retention_until` | list, date | PII minimised; retention enforced |
| `review_status`, `reviewer`, `reviewed_at` | enum, hash, date | `approved`, `quarantined` or `rejected` |
| `dataset_id`, `dataset_version` | string, semver | Ties the item to a manifest |

**Per-dataset manifest:** `dataset_id`, item count, `manifest_sha256`, counts by D-class and SPDX ID, excluded items and reasons, reviewer, date. **Per-model manifest:** `model_id`, version, `weights_sha256`, base weights and their licence (if any), `training_manifest_sha256`, eval sets and their licences, model-card version, Article 53 summary version, C2PA signer, release date.

**Gate rules for the training pipeline:**

1. **R1 — no licence, no ingestion.** An item without an allowlisted `licence_spdx` and a stored `licence_sha256` is rejected.
2. **R2 — no URL-only image sources by default.** Flickr, LAION, DataComp, PixelProse, RedCaps and Common Crawl are denied unless a per-item licence is captured; the default is deny.
3. **R3 — commissioned means released.** A D0 item needs a signed release and a payment record; a D0 item with a person needs a model release.
4. **R4 — PII gates.** Any `face`, `minor`, `medical` or `biometric` flag needs a consent record and a retention date.
5. **R5 — training reads only from a signed manifest.** The trainer refuses a directory that is not a manifest; CI recomputes the manifest hash from the shards.
6. **R6 — release needs the full disclosure pack.** Model card, Datasheet, Data Statement, Article 53 summary, ledger digest and weight hash must all be present before a model enters the ML inventory.

**How to prove no non-MIT data entered.** This is a negative claim, so the proof is procedural, not mathematical: a **complete item-level ledger**, a **content-addressed store**, a **signed manifest per dataset**, a **trainer that fails closed**, a **training log that records the manifest hash**, a **release artifact that carries the ledger digest and weight hash**, and an **independent sample audit** that re-checks a random 1-in-N items against their licence URLs. Add a **deny-list of sources** (YouTube, Flickr, Reddit, Common Crawl, Wikipedia, IAM, SROIE, DocVQA) that the ingestion job cannot override without a written exception. The licence audit's class system is the model: the ledger should be reviewable the same way `tools/deny.toml` is reviewable, with the same fail-closed default.

**Retention and withdrawal.** Keep raw commissioned data only as long as needed; keep the ledger, hashes and consent records for the audit period. A contributor withdrawal removes the item from future datasets and future training runs, but **cannot remove it from already-trained weights**; the consent form must say so. For PII, minimise at collection; for faces, consider storing only the derived annotation where the model does not need the pixels.

**Implications:** The ML spec should make the ledger a **ship blocker**, not a documentation nicety: no model enters the B10 inventory without a signed model manifest and a ledger digest. It should also adopt the D-class vocabulary in its data section so the owner can see, at a glance, how much of each model's training set is D0/D1 versus D2/D3, and how much is eval-only D4. The Article 53 summary and the model card should be generated from the same manifest, so the public disclosure cannot drift from the internal record.

## 10. Open questions and unverified claims

- **Q-D1. Do CC0 1.0 and public domain count as allowed training data?** They impose no conditions but are not MIT. Recommendation: yes, as a named exception recorded in the ledger, mirroring OD-L6 for standards data. Without it, Common Voice and Project Gutenberg are excluded.
- **Q-D2. Does CC BY 4.0 count for training?** Recommendation: no for shipped weights; eval only. If the owner wants LibriSpeech, FLEURS, MLS, COCO annotations or Open Images annotations as training data, this is the decision that admits them, with an attribution ledger.
- **Q-D3. How is Whisper's conflict resolved?** The weight card says Apache-2.0; the README says code and weights are MIT. Recommendation: treat the weights as Apache-2.0 until OpenAI confirms otherwise in writing, and do not ship them.
- **Q-D4. Are MIT weights trained on non-MIT data acceptable?** Under the letter of the rule, yes: the shipped artifact is MIT. But the distributor's grant may not cure upstream defects. Recommendation: require a provenance note for every third-party weight and prefer weights whose training data is documented (Florence-2, Donut, BLIP-2, GIT, Moonshine).
- **Q-D5. What is the commissioned-data budget?** The worked example in 8.7 is tens to hundreds of thousands of dollars for a small corpus; the spec needs a number before it commits to an ASR or captioning training run.

**Unverified or weakly verified in this strand:**

- **Foundation-1**: not found; no card, licence or size verified.
- **FUNSD, SROIE, DocVQA, veraPDF corpus**: no licence read; treat as D4/D5.
- **Getty v Stability outcome**: judgment date verified (4 November 2025); the result was not read in the judgment.
- **LibriLight**: CC BY 4.0 is commonly stated, but the gated card returned 401 and the licence text was not read.
- **C2PA 2.1**: version page read, publication date not shown.
- **COCO terms page**: did not render; the annotation/image split is Medium, from reproduced wording.
- **Conceptual Captions and TextCaps**: no dataset licence found.
- **WIT / Wikipedia image-text corpora**: text licence High, dataset-level licence not verified.
- **Vosk models**: the licence column was read, but not every model card.
- **Tokenizer code licences** (SentencePiece, Hugging Face `tokenizers`, `tiktoken`): from memory, not read here; verify before the M-1 section is written.
- **Annotation throughput, IAA thresholds and WER targets**: estimates set by this strand (†), not measured.
- **2026 case law**: the note verified the 2024–2025 decisions above; it did not exhaustively survey 2026 appellate developments. Any spec claim about "the law as of October 2026" should be re-checked by the fact-checker.
