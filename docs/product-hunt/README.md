# Product Hunt Launch Kit

This folder contains the launch copy, checklists, and source templates for a `peep` Product Hunt launch aimed at AI coding power users.

## Current Product Hunt Constraints

Checked on 2026-03-31 against Product Hunt's official guide:

- Tagline: max 60 characters
- Description: max 500 characters
- Thumbnail: square, recommended 240x240, under 3 MB
- Gallery: at least 2 images, recommended 1270x760
- Video: optional, YouTube URL only

Source: <https://www.producthunt.com/launch/preparing-for-launch>

## Files In This Kit

- `copy.md`: Product Hunt submission copy, social copy, and FAQ replies
- `checklist.md`: T-5, T-1, and launch-day operating checklist
- `release-notes-template.md`: Human-readable GitHub release draft for launch week
- `templates/`: Product Hunt thumbnail and gallery HTML sources

## Generated Assets

Run the generator to refresh the launch assets and README screenshots:

```bash
./scripts/generate_product_hunt_assets.sh
```

This produces:

- `assets/screenshot.png`
- `assets/screenshot-focus.png`
- `assets/screenshot-empty.png`
- `assets/product-hunt/thumbnail.png`
- `assets/product-hunt/gallery-01-hero.png`
- `assets/product-hunt/gallery-02-zero-config.png`
- `assets/product-hunt/gallery-03-visibility.png`
- `assets/product-hunt/gallery-04-focus.png`
- `assets/product-hunt/gallery-05-terminal.png`
- `assets/product-hunt/demo.mp4`

## Verification

Run the local verification pass before tagging a release:

```bash
./scripts/verify_launch_paths.sh
```

That script verifies:

- tests pass
- `cargo install --path .` works
- a locally packaged release tarball extracts and runs
- optional Homebrew reachability can be checked with `VERIFY_BREW=1`
