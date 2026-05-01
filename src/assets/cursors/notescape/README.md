# NoteScape Cursor Set

SVG cursor assets designed for NoteScape's light canvas UI.

Suggested CSS hotspots:

| Cursor | File | Hotspot |
| --- | --- | --- |
| Default | `default.svg` | `12 8` |
| Pointer | `pointer.svg` | `20 12` |
| Text | `text.svg` | `24 24` |
| Grab | `grab.svg` | `24 24` |
| Grabbing | `grabbing.svg` | `24 24` |
| Move | `move.svg` | `24 24` |
| Crosshair | `crosshair.svg` | `24 24` |
| Resize NWSE | `resize-nwse.svg` | `24 24` |
| Not Allowed | `not-allowed.svg` | `24 24` |

Example:

```css
body {
  cursor: url("./assets/cursors/notescape/default.svg") 12 8, auto;
}

button {
  cursor: url("./assets/cursors/notescape/pointer.svg") 20 12, pointer;
}
```
