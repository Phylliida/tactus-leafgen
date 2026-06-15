# Anatomically-grounded procedural flowers — literature survey

**Goal:** ground `tactus-leafgen`'s flower work in the real botany/CG literature, so
the generator is anatomically faithful rather than ad-hoc. Our flowers currently
reuse the leaf engine (a petal = a `Blade`, placed radially) — this survey says
how to make that a proper *floral-formula-driven* model.

**Provenance & caveat.** Sources were fetched live by the `deep-research`
workflow (run `wf_5ae6ce55-ae2`); the **adversarial cross-verification step was
API-rate-limited and did not complete**, so the extracted claims are
*source-grounded but not triple-verified*. They have been cross-checked against
domain knowledge; the foundational references (ABOP, Vogel, Coen & Meyerowitz,
Ijiri 2005, Prusinkiewicz 2007) are well-established. Exact years/venues of the
two journal PDFs should be confirmed before formal citation. Downloaded papers
live in [`docs/refs/`](refs/).

---

## The one-line answer

There is **no single algorithm**. The field is a *stack*: a **botanical spec**
(floral formula / floral diagram) → an **arrangement rule** (phyllotaxis:
whorled, or Vogel's golden-angle for heads) → **organ geometry** (petals etc. as
surfaces) → optionally a **developmental grammar** (L-systems) or **genetic
identity logic** (ABC model). The single most directly applicable work is
**Ijiri et al. 2005**, which drives interactive flower modeling from floral
diagrams + inflorescence types — and a *floral diagram is itself a 2D top-view
schematic*, i.e. almost exactly what our generator draws.

---

## Sources (downloaded → `docs/refs/`)

| File | Reference |
|---|---|
| `prusinkiewicz-lindenmayer-1990_algorithmic-beauty-of-plants.pdf` | Prusinkiewicz & Lindenmayer, *The Algorithmic Beauty of Plants* (Springer 1990) — the L-systems bible |
| `prusinkiewicz-etal-2001_positional-information.pdf` | Prusinkiewicz, Mündermann, Karwowski, Lane, "The use of positional information in the modeling of plants", SIGGRAPH 2001 |
| `runions-etal-2005_leaf-venation.pdf` | Runions et al., "Modeling and visualization of leaf venation patterns", SIGGRAPH 2005 (the venation engine we already use) |
| `ijiri-etal-2005_floral-diagrams.pdf` | Ijiri, Owada, Okabe, Igarashi, "Floral diagrams and inflorescences: interactive flower modeling using botanical structural constraints", ACM TOG / SIGGRAPH 2005, doi:10.1145/1073204.1073253 |
| `zju_flower-boundary.pdf` | Xu et al., "Boundary-dominant flower blooming simulation" (ZJU) — petal blooming/curling |
| `frontiers-2012_fpls-organ-shape.pdf` | Frontiers in Plant Science (2012) plant-modeling review, doi:10.3389/fpls.2012.00076 *(title to confirm)* |
| `njp-2012_phyllotaxis.pdf` | New J. Phys. 14 (2012) 085014, phyllotaxis physics *(title to confirm)* |

Paywalled (cited, not downloaded): Vogel 1979 (*Math. Biosci.* 44:179); Douady &
Couder 1992 (*PRL* 68:2098, doi:10.1103/PhysRevLett.68.2098); Coen & Meyerowitz
1991 "The war of the whorls" (*Nature* 353:31); the floral-quartet/MADS review
(*Development* 143:3259, 2016); Prusinkiewicz, Erasmus, Lane, Harder, Coen 2007
"Evolution and development of inflorescence architectures" (*Science* 316:1452,
doi:10.1126/science.1140429); Ijiri sketch-flower (SIGGRAPH 2004) and X-ray-CT
flower modeling (SIGGRAPH 2014). Books: Bell, *Plant Form* (branching-pattern
taxonomy Ijiri draws on); Ronse De Craene, *Floral Diagrams* (Cambridge 2010).

---

## The layers

### 1. Structural / developmental frameworks — L-systems
**ABOP** (Prusinkiewicz & Lindenmayer 1990) is the canonical framework: parallel
string-rewriting grammars (bracketed for branching; **parametric**,
**context-sensitive**, **stochastic** variants) that model plant *development* —
a meristem produces internodes → a bud → whorls of organs. **Positional
information** (Prusinkiewicz et al. 2001) lets organ size/shape vary smoothly
along an axis, and supplies the packing used for indefinite organ counts.
- *Anatomically faithful because:* it simulates the actual growth process, so
  correct structure emerges rather than being drawn.
- *2D applicability:* the **grammar/whorl-rule idea** transfers (whorls as
  production rules); full L-system *growth* is 3D/developmental — **overkill for
  a 2D schematic**, useful only as conceptual scaffolding.

### 2. Floral formulas & diagrams → diagram-driven modeling  ← **the key layer**
A **floral formula** (e.g. `✶ K5 C5 A∞ G(5)`) and **floral diagram** (concentric
cross-section schematic) are botany's compact anatomical spec — the flower analog
of leaf-architecture vocabulary. **Ijiri et al. 2005** builds flowers from
exactly this: a **floral-diagram editor** (four concentric whorl regions —
pistil/stamen/petal/sepal — on a receptacle, radial symmetry, indefinite counts
auto-packed via Prusinkiewicz 2001) plus an **inflorescence editor** (8 of the 22
branching patterns from Bell). Crucially it **separates structure from
geometry**: specify anatomy first, drape organ shapes after.
- *Anatomically faithful because:* the floral diagram *is* the botanical spec.
- *2D applicability:* **direct and central.** Their output is 3D, but the floral
  diagram is a 2D top-view schematic — i.e. our generator should *be* a
  floral-diagram renderer. **Adopt this as our architecture.**

### 3. Phyllotaxis — organ arrangement
**Vogel 1979**: the n-th organ at angle `n · 137.5°` (golden angle) and radius
`r = c·√n` — produces the sunflower/daisy capitulum with Fibonacci parastichies.
**Douady & Couder 1992** show the golden angle *emerges* from primordia repulsion
(the mechanistic "why"). Arrangement is otherwise **whorled** (fixed N per ring,
adjacent whorls usually alternating) vs **spiral**.
- *Anatomically faithful because:* matches real organ packing and the observed
  137.5° divergence.
- *2D applicability:* **direct** — Vogel is a 2D top-view formula. **Must
  implement** for composite heads (Asteraceae: ray + disk florets) and spiral
  perianths. Whorled placement we already do.

### 4. Floral anatomy — what the spec must capture (standard botany)
Concentric whorls, outside→in: **calyx** (sepals) · **corolla** (petals)
[together = perianth; *tepals* when undifferentiated] · **androecium** (stamens =
filament + anther) · **gynoecium** (carpels = ovary + style + stigma), on a
**receptacle**. Parameters: **merosity** (3 monocots / 4–5 eudicots / many),
**symmetry** (actinomorphic radial vs **zygomorphic** bilateral — orchid,
pea, snapdragon), **fusion** (sympetaly = fused petals → tube/bell; syncarpy =
fused carpels), **aestivation** (bud overlap).
- *2D applicability:* **these are exactly our parameters** — counts, radii,
  symmetry, fusion all live in the 2D diagram.

### 5. Developmental / genetic accuracy — the ABC(DE) model
**Coen & Meyerowitz 1991** ("war of the whorls"): genes determine whorl organ
identity — A→sepal, A+B→petal, B+C→stamen, C→carpel (A/C mutually antagonistic).
Extended to **ABCDE / floral-quartet** (MADS-box transcription factors; adds D =
ovule, E = SEPALLATA). Computational developmental models simulate these.
- *Anatomically faithful because:* it's the genetic logic generating the whorls.
- *2D applicability:* don't simulate the genetics — but **use the ABC *logic* as
  the clean rule for "which organ in which whorl,"** which makes mutants/variants
  (e.g. doubled flowers = petaloid stamens) fall out naturally. Full
  developmental sim = beyond 2D.

### 6. Organ-shape modeling
Petals/sepals as parametric/subdivision **surfaces** (3D) or, for us, **2D blade
outlines** (we have this). **Petal venation** = the same auxin-canalization model
as leaves (Runions 2005 — already in our engine). **Growth-based / biomechanical**
petal shape & curling — *Boundary-dominant flower blooming simulation* (ZJU) and
Liang & Mahadevan, "Growth, geometry and mechanics of a blooming lily" (PNAS
2011): differential edge growth drives blooming/ruffling.
- *2D applicability:* petal-as-blade ✓ and petal venation ✓ are **ours already**;
  curling/blooming is **3D/biomechanical — out of scope** for a flat schematic.

### 7. Inflorescence architecture
Types: raceme, spike, panicle, corymb, umbel, cyme (determinate), **capitulum/
head**, spadix, catkin — split into **monopodial/indeterminate (racemose)** vs
**sympodial/determinate (cymose)**. **Prusinkiewicz et al. 2007** (*Science*)
unify racemes/cymes/panicles from a few developmental parameters (timing of the
vegetative→flowering transition). Ijiri 2005 offers 8 patterns interactively.
- *2D applicability:* the **arrangement topology** (which flower where) is
  2D-drawable; the developmental *timing* model is the rigorous backing but not
  needed to render the layout. A **later** feature (multi-flower scenes).

---

## Recommended pipeline (anatomically-grounded, 2D, floral-formula-driven)

Mirror **Ijiri 2005's structure→geometry split**, in 2D:

1. **Input = a floral formula / diagram.** Symmetry (actino/zygo) + per-whorl
   spec `{organ type (ABC identity), merosity, fusion, radius, size, color}`.
2. **Receptacle** at the center.
3. **Whorls placed concentrically**, outside→in: calyx → corolla → androecium →
   gynoecium; adjacent whorls **alternate** angular offset (petals between sepals).
4. **Phyllotaxis:** whorled (even N) by default; **Vogel golden-angle** for
   capitula (ray + disk florets) and spiral perianths.
5. **Organs:** sepal/petal = `Blade` (have it, + petal venation); **stamen** =
   filament (line) + anther (small shape); **pistil** = central ovary/style/stigma.
6. **Symmetry:** actinomorphic = uniform organs; **zygomorphic** = differentiate
   organs by angular position (pea banner/wing/keel; orchid lip).
7. **Fusion:** sympetalous → merge corolla into one tube/bell outline; syncarpous
   → single central gynoecium.
8. **Inflorescence (later):** arrange multiple flowers per Ijiri/Bell patterns or
   the Prusinkiewicz-2007 grammar (raceme / cyme / umbel / capitulum).

### Where we are vs. what to add
- **Have:** one corolla whorl, radial placement, petal = blade, petal midrib
  veins, colored `Scene` renderer, stem.
- **Add for anatomy:** the **floral-formula spec object**; the **sepal/androecium/
  gynoecium whorls** (stamens + pistil are the detail that makes it read as a real
  flower); **Vogel** for composite heads; **zygomorphic** mode; **fusion**.
- **Skip (out of 2D scope):** L-system developmental growth, biomechanical petal
  curling, 3D organ surfaces, CT-scan data-driven geometry.
