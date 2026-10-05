# Driving Photoshop from a script

Photoshop on Windows is a COM server. From PowerShell:

```powershell
$ps = New-Object -ComObject Photoshop.Application
$ps.DoJavaScript($js)   # $js is an ExtendScript string; its last expression is returned
```

Write the whole job in ExtendScript and return a string. Calling COM methods such as
`Documents.Add` directly from PowerShell fails on argument types.

Rules inside the script:

- `app.preferences.rulerUnits = Units.PIXELS` first, restore after. If the user's units are
  percent, `documents.add(64, 64, ...)` throws "Illegal Argument".
- `app.displayDialogs = DialogModes.NO`.
- Save with `doc.saveAs(file, opts, true, Extension.LOWERCASE)` (asCopy) and close with
  `SaveOptions.DONOTSAVECHANGES`. Never touch the user's open documents.
- `layer.bounds` is the pixel rect cut by the layer mask, for Photoshop's own files too.
  The stored rect is the `boundsNoMask` key of the layer descriptor from `executeActionGet`;
  the DOM has no property for it.
- Setting `doc.activeLayer` to a hidden layer makes it visible. Read layers through
  `executeActionGet` with `putIdentifier("Lyr ", layer.id)` and take `visible` from the
  descriptor; export the PNG before selecting anything.
- Strings returned to PowerShell are Unicode, but a child process's stdout is decoded with
  `[Console]::OutputEncoding`, cp1252 by default. Gate scripts set it to UTF-8 themselves.
- Scripting cannot give a mask's stored rect. Loading the mask channel as a selection gives
  the bounds of the selected area, which depends on the mask's content, not its rect. Hide
  Selection is `Usng` = `HdSl`; the enabled state is the `userMaskEnabled` descriptor key.
- A raster mask from the selection needs Action Manager:
  `Mk` with `Nw`=`Chnl`, `At`=mask channel, `Usng`=`UsrM`/`RvlS`. See `tools/photoshop/fixtures/smoke.jsx`.

Photoshop must be running with a user logged in. It is not headless. `tools/photoshop/run.ps1`
runs any .jsx and prints what it returns. Environment variables set in the shell do not reach the running Photoshop; `run.ps1 -Out` injects a path as `OUT` instead.

Verified on Photoshop 27.5.0.
