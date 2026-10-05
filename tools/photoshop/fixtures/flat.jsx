(function () {
    var previousUnits=app.preferences.rulerUnits;
    var previousDialogs=app.displayDialogs;
    var doc=null;
    app.preferences.rulerUnits=Units.PIXELS;
    app.displayDialogs=DialogModes.NO;
    try {
        if (typeof OUT=="undefined") { throw new Error("OUT is required"); }
        doc = app.documents.add(32,32,72,"flat",NewDocumentMode.RGB,DocumentFill.WHITE);
        doc.activeLayer=doc.backgroundLayer;
        doc.selection.select([[8,8],[24,8],[24,24],[8,24]]);
        var colour=new SolidColor(); colour.rgb.red=255; colour.rgb.green=0; colour.rgb.blue=0;
        doc.selection.fill(colour); doc.selection.deselect();
        var options=new PhotoshopSaveOptions(); options.layers=true; options.embedColorProfile=true;
        doc.saveAs(new File(OUT),options,true,Extension.LOWERCASE);
        return "saved flat";
    } finally {
        try { if (doc!==null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits=previousUnits; app.displayDialogs=previousDialogs; }
    }
}());
