import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Widgets
import qs.Services.UI
import "."

Rectangle {
    id: root

    property var pluginApi: null

    property ShellScreen screen
    property string widgetId: ""
    property string section: ""
    property int sectionWidgetIndex: -1
    property int sectionWidgetsCount: 0

    // readonly property int counter: pluginApi?.pluginSettings?.counter || 0
    readonly property var runningEntry: pluginApi?.pluginSettings?.runningEntry
    readonly property var runningEntrysActivity: {
        pluginApi.pluginSettings.activities.find(p => String(p.id) == runningEntry.context.activity_id);
    }
    readonly property var selectedActivity: {
        pluginApi.pluginSettings.activities.find(p => String(p.id) == pluginApi.pluginSettings.selectedActivity);
    }
    property var dateTimeStart: new Date("2026-04-12")
    property string displayTime: ""
    implicitWidth: row.implicitWidth + Style.marginM * 2
    implicitHeight: Style.barHeight

    color: Style.capsuleColor
    radius: Style.radiusM

    function updateDifference() {
        let start = new Date(runningEntry.start_time);
        let now = new Date();
        let diffMs = now - start;

        let seconds = Math.floor((diffMs / 1000) % 60);
        let minutes = Math.floor((diffMs / (1000 * 60)) % 60);
        let hours = Math.floor((diffMs / (1000 * 60 * 60)) % 24);
        let days = Math.floor(diffMs / (1000 * 60 * 60 * 24));

        displayTime = `${hours}h ${minutes}m ${seconds}s`;
    }

    Timer {
        interval: 1000
        running: runningEntry != null
        repeat: true
        triggeredOnStart: true
        onTriggered: updateDifference()
    }

    RowLayout {
        id: row
        anchors.centerIn: parent
        spacing: Style.marginS

        Rectangle {
            width: root.implicitWidth
            height: root.implicitWidth
            radius: width / 2
            color: runningEntrysActivity?.color || selectedActivity.color
        }

        NText {
            text: if (runningEntry != null) {
                root.displayTime;
            } else {
                "No running entry";
            }
            color: Color.mOnSurface
            pointSize: Style.fontSizeS
        }
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onClicked: mouse => {
            if (pluginApi && mouse.button == Qt.LeftButton) {
                if (runningEntry) {
                    Backend.stopCurrentTimeEntry();
                } else {
                    Backend.startTimeEntry();
                }
            } else if (pluginApi && mouse.button == Qt.RightButton) {
                pluginApi.openPanel(root.screen, root);
            }
        }
    }

    Component.onCompleted: {
        Backend.pluginApi = root.pluginApi;
        Logger.i("Plugin", "Widget loaded");
        Logger.d("Plugin", "ID:", pluginApi?.pluginId);
        Logger.d("Plugin", "Version:", pluginApi?.manifest?.version);
        Logger.d("Plugin", "Language:", pluginApi?.currentLanguage);
        Logger.d("Plugin", "Counter:", root.counter);
    }
}
