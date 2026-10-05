(function () {
    var previousUnits=app.preferences.rulerUnits;
    var previousDialogs=app.displayDialogs;
    var doc=null;
    app.preferences.rulerUnits=Units.PIXELS;
    app.displayDialogs=DialogModes.NO;
    try {
        if (typeof OUT=="undefined") { throw new Error("OUT is required"); }
        doc=app.documents.add(30001,8,72,"psb",NewDocumentMode.RGB,DocumentFill.TRANSPARENT);
        var layer=doc.artLayers.add(); layer.name="Wide"; doc.activeLayer=layer;
        doc.selection.select([[0,0],[30001,0],[30001,8],[0,8]]);
        var colour=new SolidColor(); colour.rgb.red=255; colour.rgb.green=0; colour.rgb.blue=0;
        doc.selection.fill(colour); doc.selection.deselect();
        var save=new ActionDescriptor();
        save.putObject(charIDToTypeID("As  "),stringIDToTypeID("largeDocumentFormat"),new ActionDescriptor());
        save.putPath(charIDToTypeID("In  "),new File(OUT));
        save.putBoolean(charIDToTypeID("Cpy "),true);
        save.putBoolean(stringIDToTypeID("lowerCase"),true);
        executeAction(charIDToTypeID("save"),save,DialogModes.NO);
        return "saved psb";
    } finally {
        try { if (doc!==null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits=previousUnits; app.displayDialogs=previousDialogs; }
    }
}());
