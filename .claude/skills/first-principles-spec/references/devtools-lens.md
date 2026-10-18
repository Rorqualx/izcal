# The developer-tools lens

Questions to work through for any web developer tool: inspectors, debuggers, profilers, network tools, protocol tools, agent integrations. They come from the project owner's direction for Izcal's DevTools (17 October 2026). Treat them as prompts for research and design, not as a checklist to tick off: an answer of "not worth it, because..." is a good answer when the evidence supports it.

## See what current tools cannot

- What happens in a page that no current tool shows, or shows only in fragments? Consider the network stack below HTTP (DNS, TLS, QUIC, connection reuse), process and site-isolation boundaries, scheduling and task attribution, the rendering pipeline's invalidation reasons ("why did this restyle, relayout or repaint?"), memory attribution per frame or origin, service-worker lifecycles, storage partitioning, privacy-relevant data flows (who received what), the accessibility tree and how it changes, and WebAssembly at source level.
- What do developers repeatedly try to find out and fail? Look for evidence: surveys, issue trackers, Stack Overflow, forum threads.
- What can only be answered after the fact? Bugs that do not reproduce, races and timing problems are where today's tools fail most.

## Expose what is currently hidden

- Which engine data could be exposed safely that other browsers keep internal? Izcal's own architecture is the main source of new answers (see `izcal-capabilities.md`).
- How is it exposed: as views, as queryable data, as streams, as exports to standard formats?
- What must never be exposed, or only with consent: other origins' data, credentials, anything that breaks HGBA-2 P1 (security and privacy)?

## Interact with internet protocols in new ways

- Beyond watching requests: replaying, editing and resending them; simulating network conditions per protocol layer; inspecting and comparing protocol versions (HTTP/1.1, HTTP/2, HTTP/3); seeing certificate chains and transparency status, DNS resolution, ECH and connection coalescing.
- Which standard formats and protocols should the tools speak natively: HAR, qlog for QUIC, OpenTelemetry trace context, Perfetto traces, source maps (ECMA-426), the Reporting API?
- Can the developer drive the network as well as observe it, for example with mock servers or fault injection, using Izcal's simulation substrate (B1)?

## Visualize for people, not only for experts

- Question the side panel. Is a developer tool a docked pane, a full workspace, a second window, an overlay on the page, a timeline you scrub, a graph of cause and effect? Which one depends on the task, and the design should follow the task.
- Make tools friendlier without making them weaker. A newcomer and an expert should both get answers, through progressive disclosure, explanations in plain language ("this layout shift happened because...") and links back to the specification.
- Show causes, not only state: why something happened, what triggered it, what it affected.
- The tools themselves must be accessible (keyboard, screen reader, contrast), because the shell already is (HGBA shell UI design).
- Learn from visualizations that failed. Find out why removed features were removed (usage data, maintenance cost, wrong abstraction) before reviving the idea.

## Connect with the rest of the developer's world

- **Agents:** expose the tools to AI agents directly, for example as an MCP server, so an agent can inspect, debug and test a page. The design must respect Izcal's agent principal and origin capabilities (HGBA-3 B9), AI-off by default (B10), prompt-injection risk from page content, and user consent.
- **Editors and IDEs:** VS Code and others, through standard protocols (Debug Adapter Protocol, Language Server Protocol, the Chrome DevTools Protocol for existing clients) rather than one-off extensions.
- **Other tools and APIs:** test runners (WebDriver BiDi, WPT), CI pipelines, observability systems (OpenTelemetry), design tools, and the developer's own scripts.
- **Openness:** protocols documented and stable enough that others can build on them. Learn why earlier protocols fragmented (one per engine) and what the standards efforts (WebDriver BiDi) fixed.

## Question the assumptions of today's tools

For each one, decide to keep it or break it, and record why:
- One page, one tab, one tool window at a time.
- The developer is a human at a keyboard.
- The tools are part of the browser, not a separate product.
- Inspection is live only; history is lost when the page reloads.
- Each engine has its own protocol.
- DevTools are for professionals only.
