(function () {
    var prevUnits = app.preferences.rulerUnits;
    var prevDialogs = app.displayDialogs;
    var doc = null;
    var note = "";
    app.preferences.rulerUnits = Units.PIXELS;
    app.displayDialogs = DialogModes.NO;
    function paint(parent, name, bounds, rgb) {
        var layer = parent.artLayers.add(); layer.name = name;
        doc.activeLayer = layer;
        doc.selection.select([[bounds[0],bounds[1]],[bounds[2],bounds[1]],
            [bounds[2],bounds[3]],[bounds[0],bounds[3]]]);
        var colour = new SolidColor();
        colour.rgb.red = rgb[0]; colour.rgb.green = rgb[1]; colour.rgb.blue = rgb[2];
        doc.selection.fill(colour); doc.selection.deselect();
        return layer;
    }
    try {
        if (typeof OUT == "undefined") { throw new Error("OUT is required"); }
        doc = app.documents.add(32, 32, 72, "nested", NewDocumentMode.RGB, DocumentFill.TRANSPARENT);
        var outer = doc.layerSets.add(); outer.name = "Outer";
        var middle = outer.layerSets.add(); middle.name = "Middle";
        var inner = middle.layerSets.add(); inner.name = "Inner";
        paint(inner, "Deep", [4,4,12,12], [255,0,0]);
        try {
            var reference = new ActionReference();
            reference.putIdentifier(charIDToTypeID("Lyr "), middle.id);
            var layer = new ActionDescriptor();
            layer.putBoolean(stringIDToTypeID("layerSectionExpanded"), false);
            var change = new ActionDescriptor();
            change.putReference(charIDToTypeID("null"), reference);
            change.putObject(charIDToTypeID("T   "), charIDToTypeID("Lyr "), layer);
            executeAction(charIDToTypeID("setd"), change, DialogModes.NO);
            var state = executeActionGet(reference);
            var expanded = stringIDToTypeID("layerSectionExpanded");
            note = state.hasKey(expanded) && !state.getBoolean(expanded)
                ? "Middle layerSectionExpanded=false requested; check saved lsct"
                : "layerSectionExpanded unavailable; groups left open";
        } catch (error) {
            note = "layerSectionExpanded unavailable; groups left open: " + error;
        }
        var options = new PhotoshopSaveOptions(); options.layers = true; options.embedColorProfile = true;
        doc.saveAs(new File(OUT), options, true, Extension.LOWERCASE);
        return "saved nested" + (note ? ": " + note : "");
    } finally {
        try { if (doc !== null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits = prevUnits; app.displayDialogs = prevDialogs; }
    }
}());
