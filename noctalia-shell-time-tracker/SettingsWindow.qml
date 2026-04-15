import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Widgets

ColumnLayout {
    id: root

    property var pluginApi: null
    property var config: pluginApi?.pluginSettings || ({})

    NText {
        text: "Hello"
        color: Color.mOnSurface
        pointSize: Style.fontSizeS
    }

    function saveSettings() {
        pluginApi.saveSettings();
    }
}
