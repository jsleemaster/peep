# Launch Checklist

## Before Asset Freeze

- Run `cargo test`
- Run `./scripts/verify_launch_paths.sh`
- Run `./scripts/generate_product_hunt_assets.sh`
- Manually sanity-check `peep --mock`, empty state, dark theme, and light theme
- Confirm README screenshots, Product Hunt assets, and copy all reflect the same release

## 3-5 Days Before Launch

- Get feedback from 5-10 heavy users of Claude Code, Codex, or Gemini
- Pull 2-3 short quotes that can be reused in replies or social posts
- Finalize maker profile, Product Hunt draft, and gallery order
- Prepare links you will reference repeatedly:
  - repo
  - latest release
  - Homebrew tap
  - README install section

## Day Before Launch

- Tag the launch release
- Publish the GitHub release with the launch-friendly notes template
- Confirm the latest release page and README install commands agree on artifact names
- Double-check the Product Hunt thumbnail is under 3 MB
- Queue the maker comment and first comment in Product Hunt draft

## Launch Time

- Best default: schedule for 12:01am Pacific Time
- Korea time reference:
  - during U.S. daylight saving time, 12:01am Pacific is 4:01pm KST
  - during U.S. standard time, 12:01am Pacific is 5:01pm KST
- Post the maker comment immediately after launch goes live
- Spend the first 2-3 hours replying quickly to installation, privacy, and support questions
- When users ask how to try it fast, point them to `brew install peep` and `peep --mock`

## Metrics To Watch

- GitHub stars
- release downloads
- Homebrew install feedback
- Product Hunt upvotes and comments
- first-run failure reports
- requests for new tool integrations
