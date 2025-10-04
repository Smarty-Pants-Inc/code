## @just-every/code v0.2.186
This release adds an upgrade flow, improves model/context-window accuracy, and polishes TUI history and core execution.

### Changes
- TUI: add upgrade settings and auto-upgrade flow; support guided commands in terminal popup.
- Core/Model: support provider-qualified slugs and family-based model info for accurate context-window lookup.
- TUI/Footer: derive context window from model and show total tokens; fix missing X% left.
- TUI/History: refine exec states—hide spinner on call id and suppress fallback cells for finalized runs.
- Core: treat git grep as a search command; forward cwd to process calls; add semantic approval prefix matching and OS/tool reporting.

### Install
```
npm install -g @just-every/code@latest
code
```

### Thanks
Thanks to @jimmyfraiture2 for contributions!

Compare: https://github.com/Smarty-Pants-Inc/code/compare/v0.2.149...v0.2.186
