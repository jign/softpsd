(function () {
    var previousUnits = app.preferences.rulerUnits;
    var previousDialogs = app.displayDialogs;
    var doc = null;
    app.preferences.rulerUnits = Units.PIXELS;
    app.displayDialogs = DialogModes.NO;

    function addLayer(name, hideSelection) {
        var layer = doc.artLayers.add();
        layer.name = name;
        doc.activeLayer = layer;
        doc.selection.selectAll();
        var red = new SolidColor();
        red.rgb.red = 255; red.rgb.green = 0; red.rgb.blue = 0;
        doc.selection.fill(red);
        doc.selection.select([[8,8],[24,8],[24,24],[8,24]]);
        var make = new ActionDescriptor();
        make.putClass(charIDToTypeID("Nw  "), charIDToTypeID("Chnl"));
        var mask = new ActionReference();
        mask.putEnumerated(charIDToTypeID("Chnl"), charIDToTypeID("Chnl"), charIDToTypeID("Msk "));
        make.putReference(charIDToTypeID("At  "), mask);
        make.putEnumerated(charIDToTypeID("Usng"), charIDToTypeID("UsrM"), charIDToTypeID(hideSelection ? "HdSl" : "RvlS"));
        executeAction(charIDToTypeID("Mk  "), make, DialogModes.NO);
        if (!hideSelection) {
            var select = new ActionDescriptor();
            select.putReference(charIDToTypeID("null"), mask);
            executeAction(charIDToTypeID("slct"), select, DialogModes.NO);
            var black = new SolidColor();
            black.rgb.red = 0; black.rgb.green = 0; black.rgb.blue = 0;
            doc.selection.fill(black);
            var set = new ActionDescriptor();
            var ref = new ActionReference();
            ref.putIdentifier(charIDToTypeID("Lyr "), layer.id);
            set.putReference(charIDToTypeID("null"), ref);
            var properties = new ActionDescriptor();
            properties.putBoolean(stringIDToTypeID("userMaskEnabled"), false);
            set.putObject(charIDToTypeID("T   "), charIDToTypeID("Lyr "), properties);
            executeAction(charIDToTypeID("setd"), set, DialogModes.NO);
        }
        doc.selection.deselect();
        doc.activeChannels = doc.componentChannels;
    }

    try {
        if (typeof OUT == "undefined") { throw new Error("OUT is required"); }
        doc = app.documents.add(32,32,72,"mask-white",NewDocumentMode.RGB,DocumentFill.TRANSPARENT);
        addLayer("White", true);
        addLayer("Disabled", false);
        var options = new PhotoshopSaveOptions();
        options.layers = true; options.embedColorProfile = true;
        doc.saveAs(new File(OUT), options, true, Extension.LOWERCASE);
        return "saved mask-white";
    } finally {
        try { if (doc !== null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits = previousUnits; app.displayDialogs = previousDialogs; }
    }
}());
