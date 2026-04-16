import QtQuick
import Quickshell
import Quickshell.Io
import qs.Services.UI
import "."

Item {
    property var pluginApi: null

    IpcHandler {
        target: "plugin:time-tracker"
        function setCount(count: real) {
            if (pluginApi && count) {
                pluginApi.pluginSettings.counter = count;
                pluginApi.saveSettings();
                ToastService.showNotice("Message updated to: " + count);
            }
        }
        function getCurrentTimeEntry() {
            Backend.getCurrentTimeEntry();
        }
        function stopCurrentTimeEntry() {
            Backend.stopCurrentTimeEntry();
        }
        function startTimeEntry() {
            Backend.startTimeEntry();
        }
        function getAllIntegrations() {
            Backend.getAllIntegrations();
        }
        function getAllScopes() {
            Backend.getAllScopes();
        }
        function getAllProjects() {
            Backend.getAllProjects();
        }
    }

    Component.onCompleted: {
        Backend.pluginApi = root.pluginApi;
    }
}
