import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Widgets

Item {
    id: root

    property var pluginApi: null
    property var config: pluginApi?.pluginSettings || ({})

    readonly property var geometryPlaceholder: panelContainer
    readonly property bool allowAttach: true

    property real contentPreferredWidth: 420 * Style.uiScaleRatio
    property real contentPreferredHeight: mainLayout.implicitHeight + (Style.marginL * 2)

    readonly property var scopes: {
        let list = config.scopes || [];
        return list.map(s => ({
                    "key": String(s.id),
                    "name": s.name
                }));
    }
    readonly property var projects: {
        let list = config.projects || [];
        let filteredList = list.filter(s => {
            return String(s.scope_id) === config.selectedScope;
        });
        return filteredList.map(s => ({
                    "key": String(s.id),
                    "name": s.name
                }));
    }

    Rectangle {
        id: panelContainer
        anchors.fill: parent
        anchors.margins: Style.marginL
        color: Color.mSurface
        radius: Style.radiusL

        ColumnLayout {
            id: mainLayout
            spacing: Style.marginL
            anchors.margins: Style.marginL

            NComboBox {
                label: "Scope"
                description: "Select scope"
                model: scopes
                currentKey: config.selectedScope
                onSelected: key => {
                    Qt.callLater(() => {
                        config.selectedScope = key;
                        pluginApi.saveSettings();
                    });
                }
            }

            NComboBox {
                label: "Projects"
                description: "Select project"
                model: projects
                currentKey: config.selectedProject
                onSelected: key => {
                    Qt.callLater(() => {
                        config.selectedProject = key;
                        pluginApi.saveSettings();
                    });
                }
            }
        }
    }
}
