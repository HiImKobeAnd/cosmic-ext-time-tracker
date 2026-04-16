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
    readonly property var activities: config.activities
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
                        console.log(obj.content);
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
                        break;
                    case "get_all_integrations":
                        config.integrations = obj.content;
                        pluginApi.saveSettings();
                        break;
                    case "get_all_activities":
                        config.activities = obj.content;
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
        let selectedProject = config.selectedProject;
        let selectedActivity = config.selectedActivity;

        if (selectedProject) {
            ToastService.showNotice("Please select a project");
        }
        if (selectedActivity) {
            ToastService.showNotice("Please select an activity");
        }

        let request = {
            "message": "start_time_entry",
            "content": {
                "context": {
                    "activity_id": selectedActivity,
                    "project_id": selectedProject,
                    "workspace_id": null
                },
                "description": "test"
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

    function getAllActivities() {
        let request = {
            "message": "get_all_activities",
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
