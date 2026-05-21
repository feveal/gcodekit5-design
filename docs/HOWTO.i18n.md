# HOWTO: i18n / Translations (gettext)

This project uses **gettext** via the `gettext-rs` crate, with a small translation macro `t!("...")`.

## Where translations live

- Template (source strings): `po/gcodekit5.pot`
- Per-language translations: `po/<lang>.po` (e.g. `po/fr.po`)
- Runtime setup (bindtextdomain/textdomain): `crates/gcodekit5-ui/src/i18n.rs`
- Extraction/update script: `scripts/update-po.sh`

## How strings are translated in code

Use `t!(...)` for any user-visible string:

```rust
use crate::t;

let label = gtk::Label::new(Some(&t!("Machine Control")));
```

Formatting is supported (gettext returns a format string):

```rust
let msg = t!("Zoom: {0}%", zoom_pct);
```

Notes:
- Avoid embedding `\n` or other control characters unless you really want them in the UI.
- Don’t pass strings containing interior NUL bytes (`\0`) to GTK/glib APIs.

## Updating translations after UI changes

When you add/modify strings wrapped in `t!(...)`, update the POT and merge changes into all PO files:

```bash
./scripts/update-po.sh
```

This runs `xgettext` across `crates/gcodekit5-ui/src/**` and then `msgmerge`/`msginit` for the configured languages.

### Requirements

You need gettext tooling installed (xgettext/msgmerge/msginit). On Fedora:

```bash
sudo dnf install gettext
```

## Editing translations

Open the appropriate `po/<lang>.po` and translate the `msgstr` values.

Example:

```po
msgid "Visualizer"
msgstr "Visualiseur"
```

After editing, you typically want to validate PO syntax:

```bash
msgfmt --check po/fr.po
```

## Adding a new language

1) Add the language code to `scripts/update-po.sh`:

Location:
- `scripts/update-po.sh` → `LANGS=("fr" "de" "es" "pt" "it")`

Add your new language (example `nl`):

```bash
LANGS=("fr" "de" "es" "pt" "it" "nl")
```

2) Generate/init the new PO:

```bash
./scripts/update-po.sh
```

This will create `po/nl.po` if it doesn’t exist.

3) Translate `po/nl.po`.

## Running the app in a specific language

The UI calls `i18n::init(Some(language))` and will set:
- `LANGUAGE=<lang>`
- `LANG=<lang>.UTF-8`

You can also force language from the environment when launching:

```bash
LANGUAGE=fr LANG=fr_FR.UTF-8 cargo run
```

If the app exposes a UI preference that passes a language string into `i18n::init`, use that instead.

## Troubleshooting

### Strings not translating

Checklist:
- The string is wrapped in `t!("...")`.
- `po/gcodekit5.pot` contains the `msgid` (run `./scripts/update-po.sh`).
- Your `po/<lang>.po` has a non-empty `msgstr`.
- You are running with `LANGUAGE=<lang>` (or app preference set).

### Gettext domain / locale dir issues

Runtime binding is done in:
- `crates/gcodekit5-ui/src/i18n.rs`

It uses:
- domain: `gcodekit5`
- locale dir: `env!("LOCALE_DIR")` (provided at build time)

If translations still do not load, verify what `LOCALE_DIR` points to in `build.rs` and whether compiled catalogs are available there.
