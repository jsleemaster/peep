# Peep Pixel Sprite Redesign

Date: 2026-04-17
Status: Approved for planning

## Summary

Redesign the terminal pixel-art system for `peep` so the leader chicken and party sprites feel cuter, softer, and more expressive without sacrificing legibility in the stage layout. The redesign should improve four areas together:

- silhouette quality
- color and shading
- animation personality
- rendering density

The chosen direction is a dual-tier expressive system:

- leader sprites get the richest rendering treatment and stronger animation
- party sprites keep the same visual language but remain compact and fast to read

This is a sprite and rendering redesign, not a stage layout redesign.

## Goals

1. Make the leader chicken visibly cuter and more alive than the current `half block` compact rendering.
2. Unify leader and party sprites under one visual style system.
3. Increase sprite expressiveness with stronger animation and better shading.
4. Allow higher-density rendering where it materially improves the art.
5. Preserve stage readability and avoid render breakage in narrow terminals.

## Non-Goals

- Rework the rankings panel layout
- Change analytics, rankings, or party data semantics
- Introduce new gameplay or interaction mechanics
- Redesign the entire global theme system
- Depend on browser-only or image-only rendering paths

## Current Context

The current sprite system lives across a few tightly coupled files:

- `src/tui/sprites/chicken.rs`
- `src/tui/sprites/renderer.rs`
- `src/tui/widgets/stage.rs`
- `src/tui/theme.rs`

Current issues:

1. `src/tui/sprites/chicken.rs` mixes palette constants, sprite pixel data, animation frames, and growth-stage logic in one file.
2. `src/tui/sprites/renderer.rs` only exposes doubled-width and compact `half block` rendering, which limits visual expressiveness.
3. `src/tui/widgets/stage.rs` chooses sprites and also implicitly carries rendering assumptions like footprint and compactness.
4. Leader and party sprites are related in concept, but not organized as a deliberate style system.

## Design Direction

The approved visual direction is:

- softer and cuter pixel art
- stronger character animation
- leader and party redesigned together
- leader allowed more visual budget than party
- rendering expression preferred over strict conservative compatibility

The approved product stance is:

- leader uses the richest rendering path available
- party keeps dense information layouts readable
- the system still needs a safe fallback for constrained environments

## Rendering Strategy

### Dual-Tier Rendering

The sprite system will be split into two visual tiers:

#### Leader Tier

- Higher logical sprite resolution
- More expressive silhouette and facial detail
- Richer animation set
- Uses the highest-density supported terminal glyph strategy selected for the redesign

#### Party Tier

- Same visual language as the leader
- Smaller logical footprint
- Simplified details for fast recognition
- Strong stage-state readability over maximum detail

### Renderer Profiles

The renderer will support two runtime profiles:

#### Expressive

- Default profile
- Intended for the redesigned experience
- Uses the richer glyph strategy selected for the redesign
- Prioritizes visual quality

#### Safe

- Fallback profile
- Used when terminal size or rendering constraints would make expressive output unreadable
- Preserves sprite identity, state readability, and layout stability

### Glyph Strategy

The redesign should not be locked to the current compact `half block` approach. It should introduce a richer rendering path and keep a simpler fallback. The recommended upper bound for the new expressive renderer is a quadrant-style strategy rather than jumping immediately to more fragile ultra-dense symbol sets.

This keeps the design ambitious without making font support the only thing deciding whether the UI looks correct.

## Art Direction

### Silhouette

Leader silhouette should clearly read as:

- comb
- head
- beak
- chest and belly mass
- wing mass
- tail mass

Party silhouettes should keep the same language with reduced complexity:

- egg
- cracked egg
- chick peeking
- chick active
- chick waiting
- chick done

The redesign should reduce noisy one-pixel protrusions and use smoother stepped curves that read as rounded forms in the terminal.

### Color and Shading

The palette should be reorganized around a style system rather than ad hoc color constants.

For each sprite family, shading should distinguish:

- highlight
- base
- warm or cool midtone
- shadow
- accent

Specific requirements:

- white leader chicken should show volume instead of mostly flat cream fill
- chick sprites should separate face, belly, and wing mass more clearly
- comb, beak, and feet should remain high-signal accents
- all shades should remain readable against the existing dark and light themes

### Animation Tone

Animation should lean into character rather than subtle motion only.

Leader animation set should support:

- idle
- peck
- blink
- sleep
- done or proud pose

Party animation set should support:

- wobble for egg
- crack and peek rhythm for hatching states
- hop or bob for active chick
- more charming waiting pose
- celebratory or proud done pose

Animation timing should favor short acting beats and brief holds instead of constant motion noise.

## Architecture and File Responsibilities

The redesigned sprite system should separate style, art assets, rendering, and placement.

### Proposed File Split

- `src/tui/sprites/style.rs`
  - Shared palette and shading rules
  - Character-wide visual tokens
- `src/tui/sprites/leader.rs`
  - Leader sprite data and leader animation frames
- `src/tui/sprites/party.rs`
  - Egg, hatch, chick, waiting, and done sprite data
- `src/tui/sprites/renderer.rs`
  - Glyph-profile rendering only
  - No state semantics
- `src/tui/sprites/chicken.rs`
  - Either removed or reduced to compatibility re-exports during migration
- `src/tui/widgets/stage.rs`
  - Chooses what to draw and where
  - Does not own sprite art rules

### Responsibility Boundaries

The system should answer four questions in separate places:

1. What should this character look like?
2. What palette and shading language does it use?
3. How is that pixel grid mapped to terminal glyphs?
4. Where does the sprite sit in the stage layout?

This separation is necessary so the redesign can evolve later without reopening the whole stage widget each time a sprite changes.

## Compatibility and Fallback Rules

Even with expression prioritized, the system must keep two invariants:

1. no render panic
2. no unreadable sprite-state output

### Fallback Conditions

The safe profile should be used when:

- the viewport is too narrow for the expressive footprint
- the chosen glyph strategy would collide with labels or status lines
- a render path cannot guarantee readable output within the assigned area

### Fallback Behavior

- leader downgrades to a simpler profile while preserving identity
- party stays compact and readable
- layout stability wins over maximal detail

## Testing Strategy

### Render Safety Tests

Add tests for:

- narrow terminal sizes
- medium terminal sizes
- representative leader and party states
- non-empty output for all sprite families
- safe fallback activation under constrained layouts

### Visual Regression Coverage

Create snapshot-style checks for:

- leader idle
- leader peck
- egg
- cracked egg
- peeking chick
- active chick
- waiting chick
- done chick

These checks should run in both dark and light themes where practical.

### Behavioral Checks

Validate that:

- sprite selection still matches agent stage and waiting or completed state
- fallback mode does not change semantic stage mapping
- narrow layouts remain stable and do not clip or panic

## Rollout Plan

### Phase 1: Structural Split

- Introduce style, leader, party, and renderer boundaries
- Keep behavior equivalent while separating responsibilities

### Phase 2: Art and Animation Replacement

- Replace leader art first in the new structure
- Replace party stage sprites next
- Wire expressive and safe rendering profiles

### Phase 3: Regression and Tuning

- Tune narrow terminal behavior
- Tune dark and light palette contrast
- Confirm stage readability under real usage

## Success Criteria

The redesign is successful when:

1. the leader chicken is noticeably cuter and more alive than the current rendering
2. party stages are easier to distinguish at a glance
3. leader and party feel like one coherent sprite family
4. expressive rendering improves the look without destabilizing the layout
5. narrow terminals still render safely without clipping or panics

## Risks and Mitigations

### Risk: richer glyph strategy looks inconsistent across fonts

Mitigation:

- keep a safe renderer profile
- test representative terminal widths
- constrain the expressive renderer to a density level that still behaves predictably

### Risk: leader quality improves but party readability regresses

Mitigation:

- keep separate leader and party tiers
- measure party changes against stage density, not against standalone beauty

### Risk: architecture cleanup expands into unrelated widget refactoring

Mitigation:

- keep the work scoped to sprite assets, rendering, and stage integration seams
- do not redesign rankings or unrelated stage behavior in this effort
