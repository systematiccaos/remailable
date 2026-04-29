import QtQuick 2.17
import QtQuick.Window 2.17

// Import the CXX-Qt QML module defined in build.rs
import io.remailable.Remailable 1.0

Window {
    id: root
    visible: true
    width: 1620
    height: 2160
    color: "#ffffff"
    title: "remailable"

    // AppLoad required signals for reMarkable Paper Pro compatibility
    signal close
    function unloading() {
        // Cleanup — called by AppLoad before unloading the frontend.
        // Future phases will add state persistence here.
    }
}