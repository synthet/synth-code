# Synth fork icon sources — Zed + Warp

The vector masters that every shipped app icon is rendered from, for both this
checkout and the sibling `../warp` checkout. Started from the JetBrains Darcula
concepts explored in https://github.com/synthet/zed/pull/3; the marks survived
that exploration, the plate did not.

## The plate

Flat `#101014`, inset `64/1024`, corner radius `184`, **transparent surround** —
the same construction Cursor and Photoshop use. Deliberately no gradient, no
accent rim, no bevel.

The transparent surround is the point. The previous Warp rasters were opaque all
the way to the canvas edge, so Windows drew a hard black frame around the tile.
`check_transparent_surround()` in `script/apply_branding_icons.py` now probes all
four corners and all four edge midpoints of every emitted PNG and ICO frame, so
that defect cannot come back silently.

## Palette

| Role | Colour |
| --- | --- |
| Plate | `#101014` |
| Glyph, flat (Windows, small sizes) | `#A9B7C6` |
| Glyph highlight (macOS gradient top) | `#F8FAFC` |
| Preview channel tint | `#6897BB` |
| Nightly channel tint | `#9876AA` |
| Dev channel tint | `#CC7832` |

Glyph colour is the only thing that varies between Synth Code's release channels.
The plate and the mark stay identical across all four.

## Masters

| App | macOS | Windows |
| --- | --- | --- |
| Synth Code | [SVG](zed-macos.svg) / [PNG](zed-macos.png) | [SVG](zed-windows.svg) / [PNG](zed-windows.png) |
| Warp | [SVG](warp-macos.svg) / [PNG](warp-macos.png) | [SVG](warp-windows.svg) / [PNG](warp-windows.png) |

Synth Code is the window mark; Warp (Synth Term) is the two-panel mark. They stay
distinguishable in grayscale and at 16 px, so colour is never the differentiator.

The window mark is a rounded frame (204..820 by 256..768, 72 units thick, outer
radius 80) around a code slash: two chevrons and a slash, point-symmetric about
the plate centre. It follows the owner's reference render. The source files keep
their `zed-*` names because `script/apply_branding_icons.py` and `build.py`
look them up by those names. Warp sits at 222..806, so neither glyph runs to the
plate edge while the other has margin.

The mark does not use the Zed logomark from https://zed.dev/brand, which earlier
versions of these masters placed verbatim. The in-product mark,
[`assets/images/zed_logo.svg`](../../images/zed_logo.svg), is the same geometry in
one colour without the plate.

## Small-size variants

Sizes at or below 32 px render from hand-simplified sources with grid-snapped
geometry, so bars land on whole pixels instead of straddling them:

| Source | Used for | Reduction |
| --- | --- | --- |
| [zed-small-32.svg](zed-small-32.svg) | Synth Code 24 and 32 px | 96-unit frame, heavier chevrons and slash |
| [zed-small-16.svg](zed-small-16.svg) | Synth Code 16 px | 64-unit grid: one-pixel frame, chevrons and slash, one-pixel gaps |
| [warp-small-32.svg](warp-small-32.svg) | Warp 16, 24 and 32 px | Same panels, inter-panel gap widened |

`source_for()` in `script/apply_branding_icons.py` picks between them. Renders
happen at the target size rather than by downscaling from 1024, so grid-snapped
edges land on whole pixels.

## Regenerating

Shipped icons for both checkouts:

```sh
uv run --with pillow python script/apply_branding_icons.py           # Zed + Warp
uv run --with pillow python script/apply_branding_icons.py --zed-only
uv run --with pillow python script/apply_branding_icons.py --check   # validate, no writes
```

The comparison sheet and the 1024 px master PNGs:

```sh
python assets/branding/jetbrains-darcula-concepts/build.py
```

The macOS document icon, `crates/zed/resources/Document.icns`, renders from
[zed-document.svg](zed-document.svg): rasterize it at 1024 px with `magick`, then save
it with Pillow's ICNS writer (`Image.save(path, format="ICNS")`), which emits 16 to
1024 px. Neither script produces it.

Both need ImageMagick 7 with SVG support on `PATH` (verified against 7.1.2 with
librsvg). `build.py` validates the masters, prefixes SVG IDs per sheet instance,
and renders straight from vectors — `comparison-sheet.svg` is generated, so edit
`build.py` for layout, not the sheet.

## Still outstanding

Icon Composer files, an `.icns`, and native OS validation on macOS. Warp's
macOS `AppIcon.icon/` adaptive bundles and its DockTilePlugin variants are a
separate icon system and are not produced here.

The original raster exploration's prompts are kept in [prompts.json](prompts.json)
as design history, not as a build input. The SVG geometry is a reconstruction of
those shapes, not a pixel-exact trace.
