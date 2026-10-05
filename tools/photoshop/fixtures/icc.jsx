(function () {
    var previousUnits=app.preferences.rulerUnits;
    var previousDialogs=app.displayDialogs;
    var doc=null;
    app.preferences.rulerUnits=Units.PIXELS;
    app.displayDialogs=DialogModes.NO;
    try {
        if (typeof OUT=="undefined") { throw new Error("OUT is required"); }
        doc=app.documents.add(32,32,72,"icc",NewDocumentMode.RGB,DocumentFill.TRANSPARENT);
        doc.convertProfile("Adobe RGB (1998)",Intent.RELATIVECOLORIMETRIC,false,false);
        var layer=doc.artLayers.add(); layer.name="Dot"; doc.activeLayer=layer;
        doc.selection.select([[4,4],[12,4],[12,12],[4,12]]);
        var colour=new SolidColor(); colour.rgb.red=0; colour.rgb.green=120; colour.rgb.blue=255;
        doc.selection.fill(colour); doc.selection.deselect();
        var options=new PhotoshopSaveOptions(); options.layers=true; options.embedColorProfile=true;
        doc.saveAs(new File(OUT),options,true,Extension.LOWERCASE);
        return "saved icc";
    } finally {
        try { if (doc!==null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.preferences.rulerUnits=previousUnits; app.displayDialogs=previousDialogs; }
    }
}());
