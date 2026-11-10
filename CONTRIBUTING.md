# Contributing / ការចូលរួម

**ខ្មែរ:** សូមអរគុណ! ការចូលរួមចំណែកទាំងអស់ (កូដ ការបកប្រែ ឯកសារ) ស្ថិតក្រោម **RPL-1.5** ដូចកូដរបស់គម្រោង។ តាម RPL-1.5 §6.0 អ្នកផ្ដល់អាជ្ញាប័ណ្ណលើការកែប្រែរបស់អ្នកទៅ Licensor និងភាគីទាំងអស់ក្រោមលក្ខខណ្ឌដដែល។ ការផ្ញើ pull request មានន័យថាអ្នកយល់ព្រមលើចំណុចនេះ ហើយអ្នកមានសិទ្ធិផ្ដល់កូដនោះ។

**English:** Thank you! Every contribution (code, translation, docs) is under **RPL-1.5**, like the project. Under RPL-1.5 §6.0 you license your changes to the Licensor and to everyone on the same terms. Opening a pull request means you agree to this and have the right to contribute that code.

## Before you start

- Read `docs/GDM-Blueprint.html` (requirements, threat model, test cards §12). A change that touches a card must keep or add its test.
- Khmer first: every visible string is in `src/locales/km.json` and `en.json`. No machine-translated Khmer.
- Design rule: no glow, no outline/border/ring anywhere; state is shown by fill and colour (`design/check.cjs` and `tests/ui.spec.ts` enforce it).
- New dependencies need a reason in the PR and must pass `cargo deny` (licence allow-list in `deny.toml`) and `pnpm audit`.

## Checks (same as CI)

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny --config deny.toml check && cargo audit
pnpm install && pnpm lint && pnpm exec tsc --noEmit && pnpm audit
pnpm build && pnpm test:ui        # Playwright: km/en x dark/light, axe, clipping, XSS
pnpm ext                           # extension builds
```

Commits must not add `Co-Authored-By` or other credit lines; authorship of the project is GoldenIQ (see NOTICE.md). Security issues: see SECURITY.md, never a public issue.
