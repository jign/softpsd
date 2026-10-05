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

    function layerDescriptor(layer) {
        var ref = new ActionReference();
        ref.putIdentifier(charIDToTypeID("Lyr "), layer.id);
        return executeActionGet(ref);
    }

    function maskState(descriptor) {
        var hasMask = stringIDToTypeID("hasUserMask");
        if (!descriptor.hasKey(hasMask) || !descriptor.getBoolean(hasMask)) {
            return "";
        }
        return " mask=" + (descriptor.getBoolean(stringIDToTypeID("userMaskEnabled")) ? "on" : "off");
    }

    function layerBounds(layer, descriptor) {
        var key = stringIDToTypeID("boundsNoMask");
        if (!descriptor.hasKey(key)) {
            return bounds(layer.bounds);
        }
        var rect = descriptor.getObjectValue(key);
        var keys = ["left", "top", "right", "bottom"];
        var result = [];
        for (var i = 0; i < keys.length; i++) {
            result.push(Math.round(rect.getUnitDoubleValue(stringIDToTypeID(keys[i]))));
        }
        return result.join(",");
    }

    function walk(layers, indent, lines) {
        for (var i = 0; i < layers.length; i++) {
            var layer = layers[i];
            var descriptor = layerDescriptor(layer);
            var group = layer.typename == "LayerSet";
            var name = layer.name.replace(/\\/g, "\\\\").replace(/'/g, "\\'").replace(/[\r\n]/g, " ");
            var line = indent + (group ? "group" : "layer") + " '" + name + "'";
            line += " opacity=" + Math.round(layer.opacity * 2.55);
            line += " blend=" + String(layer.blendMode).replace(/^BlendMode\./, "");
            line += " visible=" + descriptor.getBoolean(stringIDToTypeID("visible"));
            line += " bounds=" + layerBounds(layer, descriptor);
            line += maskState(descriptor);
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
        doc.saveAs(new File(OUT + ".png"), new PNGSaveOptions(), true, Extension.LOWERCASE);
        var lines = [];
        walk(doc.layers, "", lines);
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
