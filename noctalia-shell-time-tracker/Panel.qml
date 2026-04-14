import QtQuick
import QtQuick.Layouts
import qs.Commons
import qs.Widgets

Item {
    id: root

    property var pluginApi: null

    readonly property var geometryPlaceholder: panelContainer
    readonly property bool allowAttach: true

    property real contentPreferredWidth: 420 * Style.uiScaleRatio
    property real contentPreferredHeight: mainLayout.implicitHeight + (Style.marginL * 2)

    readonly property var projects: {
        let list = pluginApi?.pluginSettings?.projects || [];
        return list.map(p => ({
                    "key": String(p.id),
                    "name": p.name
                }));
    }
    readonly property var activities: {
        let list = pluginApi?.pluginSettings?.activities || [];
        let filteredList = list.filter(a => {
            return String(a.project_id) === pluginApi.pluginSettings.selectedProject;
        });
        return filteredList.map(p => ({
                    "key": String(p.id),
                    "name": p.name
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
                label: "Projects"
                description: "Select project"
                model: projects
                currentKey: pluginApi.pluginSettings.selectedProject
                onSelected: key => {
                    Qt.callLater(() => {
                        pluginApi.pluginSettings.selectedProject = key;
                        pluginApi.saveSettings();
                    });
                }
            }
            NComboBox {
                label: "Activiy"
                description: "Select activity"
                model: activities
                currentKey: pluginApi.pluginSettings.selectedActivity
                onSelected: key => {
                    Qt.callLater(() => {
                        pluginApi.pluginSettings.selectedActivity = key;
                        pluginApi.saveSettings();
                    });
                }
            }
        }
    }
}
