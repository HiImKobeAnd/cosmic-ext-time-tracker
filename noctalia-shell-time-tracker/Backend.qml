pragma Singleton
import QtQuick
import Quickshell
import qs.Commons
import Quickshell.Io
import qs.Widgets
import qs.Services.UI

QtObject {
    id: root

    property var pluginApi: null

    property var config: pluginApi?.pluginSettings || ({})
    readonly property var runningEntry: config.runningEntry
    readonly property var scopes: config.scopes
    readonly property var projects: config.projects

    property Process tracker: Process {
        command: [Quickshell.env("HOME") + "/.config/noctalia/plugins/time-tracker/noctalia-shell-time-tracker"] // TODO change to something better
        running: true
        stdinEnabled: true

        stdout: SplitParser {
            onRead: data => {
                console.log("Received from backend: ", data);
                try {
                    let obj = JSON.parse(data);
                    console.log("JSON: " + obj.description);

                    switch (obj.message) {
                    case "get_current_time_entry":
                        if (obj.content == null) {
                            ToastService.showNotice("No running time entry");
                        }
                        config.runningEntry = obj.content;
                        pluginApi.saveSettings();
                        break;
                    case "stop_current_time_entry":
                        config.runningEntry = null;
                        pluginApi.saveSettings();
                        break;
                    case "start_time_entry":
                        config.runningEntry = obj.content;
                        pluginApi.saveSettings();
                        ToastService.showNotice("Running entry");
                        break;
                    case "get_all_integrations":
                        config.integrations = obj.content;
                        pluginApi.saveSettings();
                        break;
                    case "get_all_scopes":
                        config.scopes = obj.content;
                        pluginApi.saveSettings();
                        break;
                    case "get_all_projects":
                        config.projects = obj.content;
                        pluginApi.saveSettings();
                        break;
                    default:
                        console.error("Not a valid message.");
                        break;
                    }
                } catch (e) {
                    console.error("Failed to parse JSON from rustBackend: ", data);
                }
            }
        }
    }

    function getCurrentTimeEntry() {
        let request = {
            "message": "get_current_time_entry",
            "content": null
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }

    function stopCurrentTimeEntry() {
        let entry = root.runningEntry;
        let request = {
            "message": "stop_current_time_entry",
            "content": entry
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }

    function startTimeEntry() {
        let entry = root.runningEntry;
        let selectedScope = config.selectedScope;
        let selectedProject = config.selectedProject;
        let description = config.description;

        if (selectedScope) {
            ToastService.showNotice("Please select a scope");
        }

        let request = {
            "message": "start_time_entry",
            "content": {
                "scope_id": selectedScope,
                "project_id": selectedProject,
                "description": description
            }
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }

    function getAllIntegrations() {
        let request = {
            "message": "get_all_integrations",
            "content": null
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }

    function getAllScopes() {
        let request = {
            "message": "get_all_scopes",
            "content": null
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }

    function getAllProjects() {
        let request = {
            "message": "get_all_projects",
            "content": null
        };
        let jsonString = JSON.stringify(request);
        tracker.write(jsonString + "\n");
    }
}
