(function () {
    var prevUnits = app.preferences.rulerUnits;
    var prevDialogs = app.displayDialogs;
    var doc = null;
    app.preferences.rulerUnits = Units.PIXELS;
    app.displayDialogs = DialogModes.NO;

    function bounds(values) {
        var result = [];
        for (var i = 0; i < values.length; i++) {
            result.push(Math.round(values[i].as("px")));
        }
        return result.join(",");
    }

    function maskBounds(layer) {
        doc.activeLayer = layer;
        var ref = new ActionReference();
        ref.putEnumerated(charIDToTypeID("Lyr "), charIDToTypeID("Ordn"), charIDToTypeID("Trgt"));
        var descriptor = executeActionGet(ref);
        var hasMask = stringIDToTypeID("hasUserMask");
        if (!descriptor.hasKey(hasMask) || !descriptor.getBoolean(hasMask)) {
            return "";
        }
        try {
            var channel = new ActionReference();
            channel.putEnumerated(charIDToTypeID("Chnl"), charIDToTypeID("Chnl"), charIDToTypeID("Msk "));
            var select = new ActionDescriptor();
            select.putReference(charIDToTypeID("null"), channel);
            select.putBoolean(stringIDToTypeID("makeVisible"), false);
            executeAction(charIDToTypeID("slct"), select, DialogModes.NO);

            var selection = new ActionReference();
            selection.putProperty(charIDToTypeID("Chnl"), charIDToTypeID("fsel"));
            var load = new ActionDescriptor();
            load.putReference(charIDToTypeID("null"), selection);
            load.putReference(charIDToTypeID("T   "), channel);
            executeAction(charIDToTypeID("setd"), load, DialogModes.NO);
            return " mask=" + bounds(doc.selection.bounds);
        } catch (e) {
            return " mask=?";
        } finally {
            doc.selection.deselect();
            doc.activeChannels = doc.componentChannels;
        }
    }

    function walk(layers, indent, lines) {
        for (var i = 0; i < layers.length; i++) {
            var layer = layers[i];
            var group = layer.typename == "LayerSet";
            var name = layer.name.replace(/\\/g, "\\\\").replace(/'/g, "\\'").replace(/[\r\n]/g, " ");
            var line = indent + (group ? "group" : "layer") + " '" + name + "'";
            line += " opacity=" + Math.round(layer.opacity * 2.55);
            line += " blend=" + String(layer.blendMode).replace(/^BlendMode\./, "");
            line += " visible=" + layer.visible + " bounds=" + bounds(layer.bounds);
            line += maskBounds(layer);
            lines.push(line);
            if (group) {
                walk(layer.layers, indent + "  ", lines);
            }
        }
    }

    try {
        if (typeof OUT == "undefined") {
            throw new Error("OUT is required");
        }
        var input = new File(OUT);
        for (var i = 0; i < app.documents.length; i++) {
            var openPath = null;
            try {
                openPath = app.documents[i].fullName.fsName;
            } catch (e) {}
            if (openPath !== null && openPath.toLowerCase() == input.fsName.toLowerCase()) {
                throw new Error("Input PSD is already open");
            }
        }
        doc = app.open(input);
        var lines = [];
        walk(doc.layers, "", lines);
        var opts = new ExportOptionsSaveForWeb();
        opts.format = SaveDocumentType.PNG;
        opts.PNG8 = false;
        opts.transparency = true;
        doc.exportDocument(new File(OUT + ".png"), ExportType.SAVEFORWEB, opts);
        return lines.join("\n");
    } finally {
        try {
            if (doc !== null) {
                doc.close(SaveOptions.DONOTSAVECHANGES);
            }
        } finally {
            app.preferences.rulerUnits = prevUnits;
            app.displayDialogs = prevDialogs;
        }
    }
}());
