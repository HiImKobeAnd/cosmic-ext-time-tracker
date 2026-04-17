import QtQuick
import QtQuick.Layouts
import Quickshell
import qs.Commons
import qs.Widgets
import qs.Services.UI
import "."

Rectangle {
    id: root
    implicitWidth: row.implicitWidth + Style.marginM * 2
    implicitHeight: Style.barHeight
    color: Style.capsuleColor
    radius: Style.radiusM

    property var pluginApi: null
    property ShellScreen screen

    property var config: pluginApi?.pluginSettings || ({})
    readonly property var runningEntry: config.runningEntry
    readonly property var runningEntrysScope: {
        config.scopes.find(s => String(s.id) == runningEntry?.scope_id);
    }
    readonly property var runningEntrysProject: {
        config.projects.find(p => String(p.id) == runningEntry?.project_id);
    }
    readonly property var selectedScope: {
        config.scopes.find(s => String(s.id) == config.selectedScope);
    }
    readonly property var selectedProject: {
        config.projects.find(p => String(p.id) == config.selectedProject);
    }
    property string displayTime: ""

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
            color: runningEntrysProject?.color || selectedProject?.color
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
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton

        onEntered: {
            let rows = [];
            rows.push(["Scope", runningEntrysScope?.name || selectedScope?.name]);
            rows.push(["Project", runningEntrysProject?.name || selectedProject?.name]);
            TooltipService.show(root, rows, BarService.getTooltipDirection(root.screen?.name));
        }

        onExited: {
            TooltipService.hide();
        }

        onClicked: mouse => {
            if (pluginApi && mouse.button == Qt.LeftButton) {
                pluginApi.openPanel(root.screen, root);
            } else if (pluginApi && mouse.button == Qt.RightButton) {
                PanelService.showContextMenu(contextMenu, row, screen);
            } else if (pluginApi && mouse.button == Qt.MiddleButton) {
                if (runningEntry) {
                    Backend.stopCurrentTimeEntry();
                } else {
                    Backend.startTimeEntry();
                }
            }
        }
    }

    NPopupContextMenu {
        id: contextMenu

        model: [
            {
                "label": "Open Settings",
                "action": "settings",
                "icon": "settings"
            },
            {
                "label": runningEntry ? "Stop running timer" : "Start running timer",
                "action": runningEntry ? "stop-timer" : "start-timer",
                "icon": runningEntry ? "media-pause" : "media-play"
            },
        ]

        onTriggered: action => {
            contextMenu.close();
            PanelService.closeContextMenu(screen);

            if (action === "settings") {
                BarService.openPluginSettings(screen, pluginApi.manifest);
            } else if (action === "start-timer") {
                Backend.startTimeEntry();
            } else if (action === "stop-timer") {
                Backend.stopCurrentTimeEntry();
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
