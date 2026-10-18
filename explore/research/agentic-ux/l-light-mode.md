# Acan light mode

## Summary

[inferred] Add a daylight edition of direction D, not a replacement identity.
[inferred] Use Live vellum `#F4F1EA` and engraved brass `#795A27`.
[inferred] Keep four papers, catalog silhouettes, mounted gauges, and a flat plan.
[sourced] Primary ink reaches at least 12.65:1 and APCA Lc +90.9 across four papers, computed with `/tmp/acan-light-contrast/node_modules/apca-w3/src/apca-w3.js`.
[inferred] Preserve state meanings with darker inks, explicit marks, and readable labels.
[inferred] Offer System, Light, and Dark, with a local window override for the second monitor.
[inferred] Give exports an explicit publication appearance independent of the operator's screen.

## 1. Research

### Reading performance is not personal preference

[sourced] Piepenbrock, Mayr, and Buchner's 2014 experiment found smaller pupils and better proofreading with dark text on light backgrounds. The results supported display luminance as a mechanism, rather than establishing a universal preference. https://pubmed.ncbi.nlm.nih.gov/25135324/

[sourced] Their 2013 study found a positive polarity advantage for younger and older adults. https://pubmed.ncbi.nlm.nih.gov/23654206/

[sourced] Buchner and Baumgartner found better proofreading with positive polarity in darkness and office illumination. Their measured strain and self-reported discomfort did not vary with polarity. https://pubmed.ncbi.nlm.nih.gov/17510822/

[inferred] Acan's labels, budgets, and inspector deserve positive polarity support. These studies do not establish faster detection of a failing machine at second-monitor distance.

[sourced] H Locke describes halation as bright letters spreading against a dark surround, with particular difficulty for some people with astigmatism. The article also acknowledges that the frequently repeated Harrison explanation is not an accessible controlled study. https://medium.com/@h_locke/why-dark-mode-causes-more-accessibility-issues-than-it-solves-54cddf6466f5

[inferred] Treat halation as a credible user need, not proof that every astigmatic person needs light mode. Do not claim either polarity prevents eye disease or fatigue.

### Two contrast checks have different jobs

[sourced] WCAG 2.2 SC 1.4.3 requires 4.5:1 for ordinary text and 3:1 for large text, subject to its exceptions. https://www.w3.org/WAI/WCAG22/Understanding/contrast-minimum.html

[sourced] SC 1.4.11 requires 3:1 for necessary graphical information and control indicators against adjacent colors. Its guidance warns that thin antialiased strokes can appear weaker than their nominal colors. https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html

[sourced] APCA accounts for polarity and associates Lc with text size and weight. Its guidance prefers Lc 90 for fluent reading and gives lower thresholds with size restrictions. APCA is not the normative WCAG 2.2 contrast method. https://git.apcacontrast.com/documentation/APCA_in_a_Nutshell.html

[inferred] Keep WCAG as the conformance check and APCA as an additional design diagnostic. A passing ratio does not approve a 10.5 px chip.

### Instruments change presentation without changing meaning

[sourced] Garmin's G1000 FliteCharts offers Day, Night, and Auto. Day uses a white background, Night black, and Auto can switch at a selected backlight percentage. This is a chart example within a glass cockpit, not evidence that every flight instrument inverts. https://www.manualsdir.com/manuals/250394/garmin-g1000-quest-kodiak.html?page=415

[sourced] IHO's ECDIS Presentation Library governs symbols, line styles, and day, dusk, and night colors together. https://iho.int/uploads/user/Services%20and%20Standards/ENC_ECDIS/PSC%20Advice%20IHO_Ed%202.1_Final.pdf

[inferred] Borrow the coordinated palette change and stable symbol grammar. Do not copy certified navigation colors or imply navigation-grade validation.

[sourced] Google Maps on iOS separates exploration appearance from navigation colors and offers explicit overrides and device-following options. https://support.google.com/maps/answer/10189433?hl=en&co=GENIE.Platform%3DiOS

[sourced] Apple Maps on Mac has an "Always use light map appearance" setting independent of the broader dark appearance. https://support.apple.com/en-ca/guide/maps/mpsaf9c43d8f/mac

[sourced] Bloomberg tested alternative and high-contrast color sets with clients who had color vision deficiencies. Clients were more accurate and confident with those sets than with the current set. https://www.bloomberg.com/company/stories/designing-the-terminal-for-color-accessibility/

[inferred] Bloomberg supports testing actual tasks and offering alternatives. Its familiar dark terminal is not evidence against a daylight instrument.

### Professional tools separate appearance from shared content

[sourced] Figma offers Light, Dark, and System, keeps appearance device-specific, and provides enhanced contrast in both polarities. Changing appearance does not change collaborators' appearance. https://help.figma.com/hc/en-us/articles/5576781786647-Change-themes-in-Figma

[sourced] Linear offers light and dark presets, custom themes, and system preference. https://linear.app/docs/account-preferences

[sourced] VS Code supports OS scheme detection and preferred light, dark, and high-contrast themes. https://code.visualstudio.com/docs/configure/themes

[sourced] GitHub offers light and dark colorblind themes and high-contrast options, with independently selected day and night themes when following the system. https://docs.github.com/en/get-started/accessibility/managing-your-theme-settings

[inferred] Appearance belongs to the operator's device, not the blueprint YAML or an agent's change.

### White drawing paper has its own conventions

[sourced] Het Nieuwe Instituut distinguishes whiteprints, colored lines on white, from negative blueprints. It describes whiteprints as useful for annotations and color schemes, and diazotype as one whiteprinting technique. https://nieuweinstituut.nl/en/projects/invented-from-copies/de-witdruk

[sourced] Drafting guidance describes narrow and wide line groups, including 0.25 mm and 0.5 mm, rather than one weight for everything. https://cadsetterout.com/drawing-standards/line-weights/

[inferred] Light Blueprint should resemble a blue-ink whiteprint. On vellum, keep silhouettes heavier than internal construction and dimensions. Physical millimeters are an analogy, not a CSS conversion.

[sourced] `prefers-color-scheme` expresses an OS or user-agent preference, not a measured room illumination. https://developer.mozilla.org/en-US/docs/Web/CSS/@media/prefers-color-scheme

[sourced] `AmbientLightSensor` has limited availability, requires permission, and measures light around its hosting device. https://developer.mozilla.org/en-US/docs/Web/API/AmbientLightSensor

[inferred] Follow the OS by default, but never equate it with sunlight at a second monitor.

[sourced] Research used WebSearch full text on 2026-10-17 without rate-limit errors. Representative search evidence is `/home/factory-user/.factory/artifacts/scripts/08356d2d-72c3-43a6-897a-154726ab24bf/call_kxFU55vDWxqw8hWj7EWc2rOu.log`. The studies above were available as abstracts, not full experimental datasets.
[inferred] Remaining gaps are Acan user testing, calibrated display measurements, CVD simulation of the rendered drawing, and reliable native per-display identifiers.

## 2. Design

### Four light papers, one identity

[sourced] A5, K15, U4, S1, S2, and R1 lock direction D, four mode papers, distinct silhouettes, mounted gauges, and a flat plan. `/home/factory-user/acan/explore/map.md:40-55`, `/home/factory-user/acan/explore/map.md:114-143`.

[inferred] Add an appearance axis independent of mode. Keep dark unchanged. Make the light papers solid colors, avoiding a vignette that complicates contrast.

| Mode | Dark paper stops [sourced][B] | Dark grid RGB and three alphas [sourced][B] | Light paper [inferred] | Light grid minor, major, axis [inferred] |
|---|---|---|---|---|
| Live | `#141414`, `#0A0A0A` | `255,255,255`; `.035,.075,.12` | Vellum `#F4F1EA` | `#E5E0D5`, `#D6CFBF`, `#A69E8D` |
| Blueprint | `#0F2136`, `#07111D` | `140,190,240`; `.06,.13,.22` | Whiteprint `#EDF3F8` | `#DCE7F0`, `#C7D7E5`, `#92A7BA` |
| Simulate | `#141B17`, `#090D0B` | `170,230,190`; `.04,.09,.15` | Graph `#F0F4EE` | `#DFE8DC`, `#CAD8C6`, `#94A68E` |
| Replay | `#1B160F`, `#0D0A07` | `240,210,160`; `.04,.09,.15` | Archive `#F5ECDC` | `#E7DCC7`, `#D8C7A8`, `#AD9875` |

[inferred] These rows map `--paper`, `--grid-minor`, `--grid-major`, and `--grid-axis`. Keep mode names and timestamps visible because paper tint alone cannot identify mode.

[sourced] The original vellum was `#F4F1EA`, brass `#B08D57`, and brass highlight `#D9B97A`. `/home/factory-user/acan/explore/stage3-directions.html:9`, `/home/factory-user/acan/explore/stage3-refined.html:11`.

[inferred] Use `#795A27` for brass outlines and emphasis, `#E1D1AA` for broad metal fill, and primary ink for lettering. Brass should describe the bezel, not coat every chassis.

### Complete token mapping

[sourced] B is the UI root and paper definitions. L is `STATE_CSS`. I is the inherited iso kit. F is the Floor's hardcoded colors. Dark columns below transcribe those files.
[inferred] Every light column is a proposed value. Comma-separated dark values use Live, Blueprint, Simulate, Replay order. A single value is shared.

| Token | Dark [sourced][B][L] | Light [inferred] |
|---|---|---|
| `--glass` | `rgba(18,18,18,.7)` | `rgba(252,250,245,.96)` |
| `--glass-hi` | `rgba(26,26,26,.82)` | `#FCFAF5` |
| `--line` | white at `.08` | `#D4CFC3` |
| `--line2` | white at `.13` | `#BDB5A5` |
| `--text` | `#D6D6D6` | `#24282D` |
| `--dim` | `#7C7C7C` | `#50575E` |
| `--faint` | `#4A4A4A` | `#60656B` |
| `--ok` | `#35B7A2` | `#17685D` |
| `--strain`, `--wait` | `#E1A43C` | `#805400` |
| `--fail` | `#E8604A` | `#AC392B` |
| `--idle` | `#4A4A4A` | `#59616A` |
| `--ghost` | `#7AB8E6` | `#28658B` |
| `--shadow` | B's dark inset and drop shadows | `0 1px 0 #FFFFFF80 inset,0 8px 24px #29251F14,0 1px 3px #29251F0D` |
| `--sans`, `--mono`, `--ease` | B's font stacks and easing | Unchanged |

| Token | Dark [sourced][B][I][L] | Light [inferred] |
|---|---|---|
| `--anatomy-page` | `#0B0B0B` | `#F4F1EA` |
| `--anatomy-card` | `#131313` | `#FCFAF5` |
| `--anatomy-text` | `#A3A3A3` | `#50575E` |
| `--anatomy-strong` | `#E6E6E6` | `#24282D` |
| `--anatomy-muted` | `#6E6E6E` | `#60656B` |
| `--anatomy-paper` | `#131313`, `#0D1A2A`, `#111714`, `#16120D` | `#F2EEE4` |
| `--anatomy-face` | `#1B1B1B`, `#11223A`, `#17201B`, `#1E1912` | `#F8F5ED` |
| `--anatomy-top` | `#1C1C1C`, `#13263F`, `#18221C`, `#201A13` | `#FCFAF5` |
| `--anatomy-shade-0` | `#0F0F0F`, `#0A1524`, `#0D120F`, `#120E0A` | `#C5C0B4` |
| `--anatomy-shade-1` | `#121212`, `#0C182A`, `#0F1511`, `#14100B` | `#D5D0C5` |
| `--anatomy-shade-2` | `#151515`, `#0E1C30`, `#121914`, `#17130E` | `#E4E0D6` |
| `--anatomy-shade-3` | `#181818`, `#102035`, `#141C17`, `#1A1510` | `#EEEBE3` |
| `--anatomy-lit-top` | `#262626` | `#FCFAF5` |
| `--anatomy-lit-shade` | `#1D1D1D` | `#E4E0D6` |
| `--anatomy-hi` | `#D4D4D4`, `#CFE0F2`, `#CFE0D4`, `#E3D3B8` | `#343A40`, Blueprint `#27445D` |
| `--anatomy-mid` | `#7A7A7A`, `#5F7A96`, `#62786A`, `#7F6F58` | `#59616A` |
| `--anatomy-lo` | `#474747`, `#344C66`, `#36463C`, `#4A3F31` | `#929187` |
| `--anatomy-faint` | `#2A2A2A`, `#203449`, `#233029`, `#2E271E` | `#CBC7BD` |
| `--anatomy-lit` | `#FFFFFF` | `#1B1F24` |
| `--anatomy-halo` | `#050505`, `#030A12`, `#050505`, `#050505` | `#7B7465` |
| `--anatomy-halo-opacity` | `.9` | `.12` |
| `--anatomy-dot` | `#E6E6E6`, `#CFE0F2`, `#E6E6E6`, `#E6E6E6` | `#795A27` |
| `--anatomy-red` | `#E8604A` | `#AC392B` |
| `--anatomy-green` | `#6BE0A0` | `#17685D` |
| `--anatomy-blue` | `#6B9BFF` | `#28658B` |
| `--anatomy-weight` | `.75px` | `1px` |
| `--anatomy-lift` | I's dark shadow stack | `0 2px 5px #29251F14` |
| `--anatomy-ease`, `--anatomy-mono`, `--pulse-at` | I's easing, font, runtime delay | Unchanged |

[inferred] Neutral chassis materials stay shared across light papers. Restrict `--anatomy-lo` and `--anatomy-faint` to decorative seams, never required boundaries or text. Use `--anatomy-hi` for silhouettes.
[inferred] Bind essential `.iso-face-text` to `--text`, not geometry tone defaults. At normal zoom use 1.5 px silhouettes, 1 px internal lines, and 0.75 px drafting dimensions. Keep required strokes non-scaling.

[sourced] Several required roles are hardcoded, not existing custom properties. `/home/factory-user/acan/explore/ui-v1/build.mjs:43-64`, `/home/factory-user/acan/explore/ui-v1/lib.mjs:19-74`, `/home/factory-user/acan/explore/ui-v1/floor.mjs:35-58`.

| Role | Dark implementation [sourced][B][L][F][R] | Light mapping [inferred] |
|---|---|---|
| Working, strained, failing, waiting, idle | `ok`, `strain`, `fail`, `wait`, `idle` | Five inks above, with check, triangle, cross, pause, and hollow circle |
| Ghost | Blue dashed lines at `.85`, faint blue fill | `#28658B` at full opacity, dashed empty bezel, fill `#28658B08` |
| Dark | Whole machine opacity `.32`, lit `#777` | `#59616A` power-slash marker, opaque label, `#EEEBE3` body |
| Fogged | Blur and opacity `.7` | `#626973` question marker and dotted boundary, opaque label, `#E4E0D6` body |
| Run auxiliary | Top `#CFE6FB`, shade `#56718A` | Top `#28658B`, shade `#204F6D`, play mark |
| Lamp off | Top `#3A3A3A`, shade `#262626` | Top `#D4D0C6`, shade `#AAA498`, ink outline |
| Lamp shades | `#1F6E62`, `#8A6322`, `#8B3427` | `#124E46`, `#624000`, `#812B20` |
| Belts | Shade-2 bed, lo rails, hi flow, red errors | `#DED9CE` bed, `#59616A` rails, brass flow, fail errors |
| Selected belts | White lit strokes | `#1B1F24` double outline and arrow |
| Chip shell, text, reading | Black `.72`, `#D8D8D8`, `#8A8A8A` | `#FCFAF5`, primary ink, dim ink |
| Chip badge | `#E8E8E8` on `#0A0A0A` | `#E1D1AA` with `#24282D` lettering |
| State chip fills | Bright strain, fail, run | `#F2E8CC`, `#F6E3DC`, `#E2EDF4`, primary lettering and state outline |
| Chip leaders, selection | White gradients and white `.4` | `#929187` leader, brass selection plus primary outer ring |
| Egress pill | Transparent, dim text, ok dot | `#FCFAF5`, dim text, `#17685D` check and "local"; fail cross for outbound warning |
| Callouts and compass | White text, black backplates | Primary text, `#FCFAF5` plates, `#59616A` ticks |
| Hover and active fill | White `.04` through `.1` | `#24282D0A`, active `#E1D1AA` |
| Drone body and eye | Black gradients, glowing white eye | Four chassis tones, brass bezel, `#24282D` eye, no glow |
| Smoke and threads | `#8D8D8D`, white `.35` | `#706B61` smoke, `#59616A` threads |
| Seal | `#C9A466`, `#6E5530`, `#E2C48C` | `#E1D1AA`, `#795A27`, `#795A27` |
| Territory neutral | White `.016`, lines `#5C5C5C`, text `#6C6C6C` | Fill `#24282D06`, ink `#59616A`, unclaimed dashed boundary |
| Territory building, verifying | Amber `.045`, amber `.55`, text `#B98D47` | Fill `#8054000A`, outline `#805400`, single diagonal building hatch, crosshatch verifying |
| Territory passed, failed, stale | Brass `.05`, `#A88A55`, fail, stale `7 3` | Fill `#795A270A`, brass seal, fail cross, stale broken boundary and "stale" |
| Floor scout, build, verify | Gray stripes, `#CFCFCF`, gray diagonal | `#59616A` stripes, `#59616A` solid, `#59616A` hatch over `#E4E0D6` |
| Replay `--k` for recover, deploy, agent, CI | `#9FD3B0`, `#D9B26A`, `#E3D3B8`, `#9CC7EE` | Ok ink, brass, primary ink, blue ink, with event labels |

[inferred] Territory hatches are new, not a claim about existing UI. Keep hatch strokes decorative and sparse. Their labeled boundary carries meaning. Keep ordinary chip labels primary ink, including state chips.
[inferred] Keep runtime geometry variables `--lift`, `--i`, and `--d` unchanged. Render meaningful flow dots at full opacity rather than inheriting decorative dot opacity.

[sourced] Color cannot be the sole carrier of information under SC 1.4.1. https://www.w3.org/WAI/WCAG22/Understanding/use-of-color.html
[inferred] Preserve the same hues' meanings across polarities, not identical hex values. Validate protan, deutan, tritan, and grayscale renderings. Do not call this palette CVD-safe before that test.

### Computed contrast

[sourced] The table reports WCAG ratio followed by signed APCA Lc. Calculations used sRGB, opaque foregrounds, solid papers, and `apca-w3@0.1.9`, algorithm 0.0.98G-4g. `/tmp/acan-light-contrast/node_modules/apca-w3/package.json`, `/tmp/acan-light-contrast/node_modules/apca-w3/src/apca-w3.js:143-171`.

| Foreground [inferred] | Live [sourced] | Blueprint [sourced] | Simulate [sourced] | Replay [sourced] |
|---|---|---|---|---|
| Text `#24282D` | 13.14, +93.4 | 13.26, +94.0 | 13.33, +94.4 | 12.65, +90.9 |
| Dim `#50575E` | 6.50, +77.2 | 6.55, +77.8 | 6.59, +78.2 | 6.25, +74.7 |
| Faint `#60656B` | 5.21, +71.2 | 5.26, +71.8 | 5.29, +72.1 | 5.02, +68.7 |
| Brass `#795A27` | 5.63, +73.2 | 5.68, +73.8 | 5.71, +74.2 | 5.42, +70.7 |
| Working `#17685D` | 5.86, +74.1 | 5.91, +74.7 | 5.95, +75.1 | 5.64, +71.6 |
| Strained, waiting `#805400` | 5.84, +74.1 | 5.89, +74.6 | 5.93, +75.0 | 5.62, +71.6 |
| Failing `#AC392B` | 5.52, +71.8 | 5.56, +72.4 | 5.60, +72.7 | 5.31, +69.3 |
| Ghost, run `#28658B` | 5.60, +72.9 | 5.65, +73.5 | 5.68, +73.9 | 5.39, +70.4 |
| Idle, dark `#59616A` | 5.57, +73.1 | 5.62, +73.6 | 5.65, +74.0 | 5.36, +70.5 |
| Fogged `#626973` | 4.92, +69.5 | 4.96, +70.0 | 4.98, +70.4 | 4.73, +67.0 |

[sourced] Reproduce the table after installing the pinned package under `/tmp` with `npm install --prefix /tmp/acan-light-contrast apca-w3@0.1.9 --ignore-scripts --no-audit --no-fund`.

```js
import {APCAcontrast, sRGBtoY} from
	"/tmp/acan-light-contrast/node_modules/apca-w3/src/apca-w3.js";
const rgb = h => h.match(/[a-f0-9]{2}/gi).map(v => parseInt(v, 16));
const y = h => rgb(h).map(v => v / 255)
	.map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4)
	.reduce((s, v, i) => s + v * [.2126, .7152, .0722][i], 0);
const papers = ["#F4F1EA", "#EDF3F8", "#F0F4EE", "#F5ECDC"];
const inks = ["#24282D", "#50575E", "#60656B", "#795A27", "#17685D",
	"#805400", "#AC392B", "#28658B", "#59616A", "#626973"];
for (const ink of inks) console.log(ink, papers.map(paper => [
	((Math.max(y(ink), y(paper)) + .05) /
		(Math.min(y(ink), y(paper)) + .05)).toFixed(2),
	APCAcontrast(sRGBtoY(rgb(ink)), sRGBtoY(rgb(paper))).toFixed(1)
]));
```

[sourced] Black against white produced 21.00:1 and Lc +106.0. Reversal produced 21.00:1 and Lc -107.9. These reference checks passed using the pinned calculator at `/tmp/acan-light-contrast/node_modules/apca-w3/src/apca-w3.js`.

[sourced] Primary ink on brass badge fill produced 9.81:1 and Lc +75.6. Dim ink on opaque glass produced 7.02:1 and Lc +82.5, using `/tmp/acan-light-contrast/node_modules/apca-w3/src/apca-w3.js`.

[inferred] These are mathematical checks, not measured luminance or rendered accessibility certification. Round only for reporting. Check actual composited colors, adjacent machine surfaces, hover, selection, fog, and export.

[inferred] Use primary ink for essential 16 px body text and compact 14 px bold labels. Reserve dim ink for larger secondary labels, and faint ink for nonessential annotation. Increase the current microtype rather than preserving its density at any cost.

### Shading, ink-in, exports, and switching

[sourced] The kit already has a light selector on `.iso[data-theme="light"]`, not merely on the body. Paper overrides also live in mode-specific `.iso` rules. `/home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs:568-588`, `/home/factory-user/acan/explore/ui-v1/build.mjs:195-201`.

[inferred] Apply one resolved theme to DOM, SVG materials, labels, and renderer in one update. Put theme-plus-mode overrides after existing rules. Setting only a body attribute is insufficient.

[inferred] Flip edges from luminous to dark and bodies from charcoal to pale mineral tones. Keep shade-0 darkest and shade-3 lightest. Preserve light direction and face normals. Never invert RGB or reverse physical illumination. The four-tone scheme applies to iso and exploded views, not an excuse to extrude R1's flat plan.

[inferred] Keep ghost-to-live "ink-in". Dashed blue geometry becomes a continuous ink outline, then material fills and its mounted gauge appear. Avoid a white flash or expanding glow. Reduced motion gets the same final state immediately.
[inferred] Switching appearance never triggers ink-in or alters source-matching status.

[sourced] S3 requires share-safe PNG, SVG, and video frames. R2 defines the default redaction set and keeps numbers. `/home/factory-user/acan/explore/map.md:130-143`.

[inferred] Default exports to a stable Light publication profile, with Current appearance and Dark alternatives. Preview the chosen paper and persist the export choice separately. Bake SVG colors and background instead of inheriting the recipient's OS. Lock appearance through a video take. Redaction is identical under every theme.

[inferred] Offer System, Light, and Dark in Appearance and the command palette. F8 toggles the focused window between resolved Light and Dark, leaving System via an explicit local override. The menu returns to System. Preserve existing saved Dark choices.

[inferred] Store appearance locally per window, never in project history. An always-open second-monitor window can remain Dark while the editing window follows System. For a later display profile, resolve window override before display override before System. A spanning window gets one appearance based on its host-selected monitor, never a split canvas.

[inferred] Defer automatic monitor-profile persistence until native display identifiers survive disconnects and docking. A per-window override meets the immediate second-monitor need without pretending CSS knows each monitor's light level.

## 3. What to avoid

[inferred] Avoid pure `#FFFFFF` as the full-canvas default. This preserves Acan's paper identity and lowers emitted luminance at equal display settings. It is a design choice, not a claim that white is medically harmful.

[sourced] Original brass `#B08D57` on Live vellum yields only 2.74:1 and Lc +49.5. Its highlight `#D9B97A` yields 1.67:1 and Lc +27.3, using the calculation above.
[inferred] Do not retain that brass for required outlines or small text. Do not darken every metal face into brown. Use engraved dark brass edges around pale fill.

[sourced] Current working teal `#35B7A2` yields 2.20:1 and Lc +40.2 on Live vellum.
[inferred] Do not transfer dark-theme emissive state colors unchanged.

[inferred] Avoid state signals based only on motion, hue, or whole-object opacity. Keep labels and required outlines opaque when dark, fogged, deselected, or idle. Decorative body fading must not erase a failure.

[inferred] Avoid translucent panels over arbitrary belts without a contrast floor. Use near-opaque glass and opaque chips. Test worst-case composites instead of quoting the paper-only table.

[inferred] Avoid animated crossfades between polarities, ambient sensor polling, and automatic time-of-day switching separate from System. Apply tokens together, invalidate once, and return to idle.

[sourced] U2 requires distance readability and low idle cost. `/home/factory-user/acan/explore/map.md:115`.
[inferred] White mode must not become a continuously rendered texture or another reason to wake the canvas.

## Recommended v0.1 scope

[inferred] Ship eight resolved combinations from four modes and two appearances. Include System, the focused-window override, F8, and export appearance preview. Do not ship custom palette editing, dusk, sensor automation, or automatic display profiles.

[inferred] Replace hardcoded color roles before wiring the switch. Preserve R1, the redaction set, keyboard focus, and the ghost lifecycle. Add high-contrast handling independently of polarity, with solid boundaries and opaque panels.

[inferred] Gate release on screenshot checks for every paper and state, primary-label contrast, glass composites, keyboard focus, reduced motion, grayscale and CVD views, SVG colors, and a redacted video-frame export.

[guess] A daylight desk session and a dim-room second-monitor session will expose failures that swatch review misses. Compare failure detection and budget reading with users, including people who report halation.

[inferred] Implementation, live display tests, and participant validation remain open. This document proposes the palette and verifies its color arithmetic only.

[B]: /home/factory-user/acan/explore/ui-v1/build.mjs
[L]: /home/factory-user/acan/explore/ui-v1/lib.mjs
[I]: /home/factory-user/.factory/skills/anatomy/kit/iso-kit.mjs
[F]: /home/factory-user/acan/explore/ui-v1/floor.mjs
[R]: /home/factory-user/acan/explore/ui-v1/replay.mjs
