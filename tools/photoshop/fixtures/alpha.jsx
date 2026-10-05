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
        doc = app.documents.add(32, 32, 72, "alpha", NewDocumentMode.RGB, DocumentFill.TRANSPARENT);
        paint(doc, "Dot", [4,4,12,12], [0,120,255]);
        var options = new PhotoshopSaveOptions(); options.layers = true; options.embedColorProfile = true;
        doc.saveAs(new File(OUT), options, true, Extension.LOWERCASE);
        return "saved alpha" + (note ? ": " + note : "");
    } finally {
        try { if (doc !== null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits = prevUnits; app.displayDialogs = prevDialogs; }
    }
}());
