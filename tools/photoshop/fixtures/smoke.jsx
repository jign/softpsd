// Reference fixture: group > painted layer, 60% Multiply, raster mask. Saves to OUT (run.ps1 -Out).
app.displayDialogs = DialogModes.NO;
var prevUnits = app.preferences.rulerUnits; app.preferences.rulerUnits = Units.PIXELS;
var out = new File(OUT);
var doc = app.documents.add(64, 64, 72, "smoke", NewDocumentMode.RGB, DocumentFill.TRANSPARENT);
var grp = doc.layerSets.add(); grp.name = "Group A";
var lyr = grp.artLayers.add(); lyr.name = "Painted"; lyr.opacity = 60; lyr.blendMode = BlendMode.MULTIPLY;
doc.activeLayer = lyr;
doc.selection.select([[8,8],[40,8],[40,40],[8,40]]);
var c = new SolidColor(); c.rgb.red = 200; c.rgb.green = 30; c.rgb.blue = 30;
doc.selection.fill(c);
doc.selection.select([[16,16],[48,16],[48,48],[16,48]]);
var d = new ActionDescriptor();
d.putClass(charIDToTypeID("Nw  "), charIDToTypeID("Chnl"));
var r = new ActionReference(); r.putEnumerated(charIDToTypeID("Chnl"), charIDToTypeID("Chnl"), charIDToTypeID("Msk "));
d.putReference(charIDToTypeID("At  "), r);
d.putEnumerated(charIDToTypeID("Usng"), charIDToTypeID("UsrM"), charIDToTypeID("RvlS"));
executeAction(charIDToTypeID("Mk  "), d, DialogModes.NO);
doc.selection.deselect();
var o = new PhotoshopSaveOptions(); o.layers = true; o.embedColorProfile = true;
doc.saveAs(out, o, true, Extension.LOWERCASE);
doc.close(SaveOptions.DONOTSAVECHANGES);
var d2 = app.open(out);
var s = "reopened " + d2.name + " " + d2.width + "x" + d2.height + " mode=" + d2.mode + " profile=" + d2.colorProfileName + "\n";
for (var i = 0; i < d2.layerSets.length; i++) { var ls = d2.layerSets[i]; s += "  set " + ls.name + "\n";
  for (var j = 0; j < ls.artLayers.length; j++) { var l = ls.artLayers[j]; s += "    layer " + l.name + " opacity=" + l.opacity + " blend=" + l.blendMode + " bounds=" + l.bounds + "\n"; } }
d2.close(SaveOptions.DONOTSAVECHANGES);
app.preferences.rulerUnits = prevUnits;
s;
