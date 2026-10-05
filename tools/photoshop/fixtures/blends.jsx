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
        doc = app.documents.add(32, 32, 72, "blends", NewDocumentMode.RGB, DocumentFill.TRANSPARENT);
        var blends = [["Normal", BlendMode.NORMAL],["Dissolve", BlendMode.DISSOLVE],["Darken", BlendMode.DARKEN],["Multiply", BlendMode.MULTIPLY],["ColorBurn", BlendMode.COLORBURN],["LinearBurn", BlendMode.LINEARBURN],["DarkerColor", BlendMode.DARKERCOLOR],["Lighten", BlendMode.LIGHTEN],["Screen", BlendMode.SCREEN],["ColorDodge", BlendMode.COLORDODGE],["LinearDodge", BlendMode.LINEARDODGE],["LighterColor", BlendMode.LIGHTERCOLOR],["Overlay", BlendMode.OVERLAY],["SoftLight", BlendMode.SOFTLIGHT],["HardLight", BlendMode.HARDLIGHT],["VividLight", BlendMode.VIVIDLIGHT],["LinearLight", BlendMode.LINEARLIGHT],["PinLight", BlendMode.PINLIGHT],["HardMix", BlendMode.HARDMIX],["Difference", BlendMode.DIFFERENCE],["Exclusion", BlendMode.EXCLUSION],["Subtract", BlendMode.SUBTRACT],["Divide", BlendMode.DIVIDE],["Hue", BlendMode.HUE],["Saturation", BlendMode.SATURATION],["Color", BlendMode.COLORBLEND],["Luminosity", BlendMode.LUMINOSITY]];
        for (var i = 0; i < blends.length; i++) {
            var layer = paint(doc, blends[i][0], [i,0,i+1,1], [255,0,0]); layer.blendMode = blends[i][1];
        }
        var options = new PhotoshopSaveOptions(); options.layers = true; options.embedColorProfile = true;
        doc.saveAs(new File(OUT), options, true, Extension.LOWERCASE);
        return "saved blends" + (note ? ": " + note : "");
    } finally {
        try { if (doc !== null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits = prevUnits; app.displayDialogs = prevDialogs; }
    }
}());
