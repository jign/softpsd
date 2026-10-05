(function () {
    var previousDialogs = app.displayDialogs;
    var doc = null;
    app.displayDialogs = DialogModes.NO;
    try {
        var input = new File(OUT);
        for (var i = 0; i < app.documents.length; i++) {
            var path = null;
            try { path = app.documents[i].fullName.fsName; } catch (e) {}
            if (path !== null && path.toLowerCase() == input.fsName.toLowerCase()) {
                return "already open; skipped";
            }
        }
        doc = app.open(input);
        return "opens: " + doc.layers.length + " root layers";
    } catch (e) {
        return "error: " + String(e).replace(/[\r\n\t]/g, " ");
    } finally {
        try { if (doc !== null) { doc.close(SaveOptions.DONOTSAVECHANGES); } }
        finally { app.displayDialogs = previousDialogs; }
    }
}());
