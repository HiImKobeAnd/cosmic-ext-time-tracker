import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Widgets

ColumnLayout {
    id: root

    property var pluginApi: null
    property var config: pluginApi?.pluginSettings || ({})

    readonly property var integrations: {
        let list = config.integrations || [];
        return list.map(i => ({
                    "key": String(i),
                    "name": String(i)
                }));
    }

    NComboBox {
        label: "Integration"
        description: "Select integration"
        model: integrations
        currentKey: config.selectedIntegration
        onSelected: key => {
            Qt.callLater(() => {
                config.selectedIntegration = key;
                pluginApi.saveSettings();
            });
        }
    }

    function saveSettings() {
        pluginApi.saveSettings();
    }
}
