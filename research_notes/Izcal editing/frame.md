# Izcal editing: frame

Written 20 October 2026 (phase 1 of the first-principles-spec method).

## The problem and who has it

Every web page that takes text from a person goes through editing: a search box, a login form, a chat message, a comment, a spreadsheet cell, a document in an online word processor, a code editor. Editing is where the browser meets the person's hands, their language and their operating system's input methods, and where text leaves or enters the page through the clipboard and drag and drop. No accepted Izcal spec owns it. The Phase 3 ledger records the gap as **G-16** ("Editing (caret, selection, editing commands, spellcheck budget)") and the spellcheck conflict as **C-22** ("no owner, stage or budget"; the ledger recommends that TX's placement stay and that the stage and budget become the editing spec's). Stage 1's plan relied on Parley for IME composition, and HGBA-2 2.3.0 removed Parley; the fonts and text spec took the text side of input (TX-D23, 6.8a) and left "the editing commands themselves" to this spec. The DOM spec (draft 0.2, 6.6 and 6.18) takes the Selection **data model**, `selectionchange`, composition-event plumbing and `beforeinput`/`input` dispatch, and gives this spec "caret movement in editable content, editing commands and `execCommand`, `contenteditable` behaviour, `beforeinput`/`input` semantics for edits, IME composition beyond the event plumbing, spellchecking, and the visual selection painted by graphics". The accessibility spec (draft 0.2, 6.8, 6.9) exposes editable text and routes AT edits through `beforeinput`/`input`, and says "the future editing spec owns the commands; this spec owns their exposure".

Scope of this spec, as assigned:
- **Selection and ranges** beyond DOM's data model: caret and selection behaviour in editable and non-editable content, caret movement (logical and visual, bidi), selection extension and granularity, the caret and selection painting hand-off to graphics, `::selection` and the Highlight API's editing-related highlights.
- **`contenteditable` and `designMode`**: the editing model. execCommand (legacy, unfinished spec), Input Events level 2 `beforeinput`, EditContext: evaluated from first principles and evidence, with a decision.
- **Text controls** (`input`, `textarea`): their editing engine, value model, selection API, undo stack, autofill boundary.
- **IME and composition**: the renderer side and the cross-process protocol with the broker (TX-D23), for Windows TSF, macOS `NSTextInputClient`, Linux Wayland text-input with IBus and Fcitx, Android `InputConnection`.
- **Spellcheck and grammar**: Izcal's own spellchecker and dictionaries (MIT only; TX-D14's language-data track); no cloud.
- **Clipboard**: async Clipboard API, `execCommand('copy')` and clipboard events, sanitization with the style spec's sanitizer, permissions, cross-process.
- **Drag and drop**, **undo/redo**, **find in page**, **autocorrect and autocapitalize**, **virtual keyboards** (VirtualKeyboard API, `inputmode`, `enterkeyhint`).
- **Accessibility of editing** (with AX), **security** (paste injection, clipboard snooping, drag-and-drop leaks, IME code), **determinism** (input as recorded events), **observability**.

Not in scope: the DOM's Selection object and range registry (DM 6.6), event dispatch (DM 6.5), composition rendering, shaping and the movement rules' Unicode data (TX 6.8a, 6.6), OS IME framework hosting (broker, TX-D23), painting machinery (GR), the Sanitizer API's parser integration (ST 6.5), the PDF viewer's form fields (PD, which uses this spec's services), the shell's own UI (shell design), the password manager's store (security scope), autofill beyond its boundary with form controls.

## Who has the problem

- **People typing**, in every script, with every input method: Latin keyboards with dead keys and autocorrect, CJK IMEs with candidate windows and reconversion, Indic and South-East Asian scripts whose grapheme clusters are built from several keystrokes, right-to-left and mixed-direction text, handwriting and voice dictation, on-screen keyboards on touch devices, switch and voice-control users through AT.
- **Authors of rich-text editors** (ProseMirror, Lexical, Slate, CKEditor, Quill, TipTap, Google Docs, Microsoft Word for the web, Notion, code editors such as Monaco and CodeMirror): they fight `contenteditable`'s differences between engines and its unspecified behaviour, and some left the DOM for canvas.
- **Authors of ordinary forms**: they need `input` and `textarea` to behave, autofill to fill, undo to work, IME not to break their key handlers.
- **People copying and pasting**: they expect copy and paste to work everywhere and pages not to read their clipboard or write malware commands into it.
- **Assistive-technology users** (with AX): screen readers announce edits; voice and switch users edit through AT actions.
- **Developers and agents** debugging editors: "why did this keystroke produce this DOM change?", "what did the IME send?", "why did undo do that?", reproducing IME bugs that need a specific OS and IME.

## What Izcal's specs already decide

- **M-1 (HGBA-2 2.3.2).** No library from a browser project or doing engine work; MIT only for everything shipped (code and data); another engine's source is never evidence. Spellcheck engines and dictionaries from browser projects or under other licences (Hunspell: MPL/GPL/LGPL; most Hunspell dictionaries: LGPL, GPL, MPL or CC BY-SA) are therefore not shipped; OS interfaces (TSF, `NSTextInputClient`, Wayland protocols, Android `InputConnection`, OS spell-checking services, OS clipboards) are allowed as OS interfaces.
- **HGBA-3 B1** (recorded inputs, seeded scheduler, virtual clock): every key, composition update, clipboard read and drop is a recorded input. **B2** typed capability IPC. **B4** mutation log. **B8** semantic plane. **B9, B10** agent principal, AI off by default: no ML autocorrect or cloud grammar by default.
- **I-1** one renderer per (site, container), origin-keyed (FP-D8; DM-D6 always origin). The clipboard and drag data are cross-origin data: they must be mediated by the broker.
- **DM (draft 0.2):** 6.5 (`hg-input` turns OS input into UI Events; every input event a recorded B1 input with the broker's sequence number; user activation minted by the broker, DM-D20, which gates clipboard writes); 6.6 (Selection boundary above); 6.12 (forms are `hg-forms`; autofill writes values through `hg-forms` from the broker, never by script; `execCommand` and `contenteditable` are this spec's; 1.19% of page loads call `execCommand`); 6.17 (`hg-input`, `hg-forms`, `hg-webapi` includes Clipboard's DOM side and activation checks); 6.18 (Clipboard: "DOM side and activation checks" in `hg-webapi`; "Broker stores, prompts and OS integration (shell, security scope)").
- **AX (draft 0.2):** 6.8 (text positions, editable text exposure; AT edits go through `beforeinput`/`input`); 6.9 (AT set-value as `insertReplacementText`; set-caret through the selection's user path); 6.12 (caret browsing over DM's Selection model).
- **TX (accepted 0.3.4):** TX-D23 (broker owns OS IME frameworks; no IME component in a renderer; TSF DLLs in the broker are inventoried; composition rendering and caret and selection movement rules are TX's); TX-16 (no page-controlled string reaches an OS text API in the broker); TX-D14 (Izcal authors its own linguistic data with a language-data track; no third-party word lists); TX-D12 (dictionary segmentation for SA scripts); FP-D13 (every dictionary in every build).
- **ST (accepted):** the Sanitizer API and `SanitizingSink` (sanitizing inside the parser); one HTML parser (G23).
- **LY (accepted):** `caretPositionFromPoint` in `hg-layout-query`; scroll anchoring prioritises "the focused editable element, then the active find-in-page match"; form controls are UA shadow trees with Izcal-defined metrics (LY-D11, including caret width); "Editing (future spec): caret and selection rectangles from fragments and the text stack".
- **GR (accepted):** display lists in chunks, property trees, compositor-mutable nodes capped at 16† per document; compositor animations for `transform` and `opacity`; no caret or highlight design exists there.
- **MD (accepted):** every image byte, clipboard images included, is decoded only in a media process (MD-20).
- **Stage 1 13.2:** clipboard, drag and drop and find in page are Stage 1; "spellcheck through OS services" was Stage 2; IME integration is M6.

## What success means, measurably

- WPT `editing/`, `selection/`, `input-events/`, `clipboard-apis/`, `html/editing/` (dnd, editing hosts, `inputmode`), `edit-context/`, `css/css-highlight-api/`, `uievents/` composition at or above the best shipping engine's rate per directory.
- Every keystroke's effect explainable: the input record → `beforeinput` → the edit applied → DOM mutation records → `input`, inspectable and replayable from a recording.
- IME correctness across the three desktop OSes and Android on a published IME matrix (Microsoft Pinyin and Japanese IME, Apple's Kotoeri and Pinyin, IBus and Fcitx with Anthy/Mozc and Rime, Gboard and Samsung Keyboard), measured with black-box tests.
- No clipboard read without a gesture and the user's consent; no clipboard write without transient activation; pasted HTML sanitized by the one parser; zero renderer access to another origin's drag or clipboard data.
- Spellcheck in every language with an Izcal dictionary, offline, with no text leaving the device, and results invisible to script.
- Keystroke-to-paint latency in editable text bounded and measured.
