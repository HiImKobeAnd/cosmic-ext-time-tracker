pragma Singleton

QtObject {
    id: root

    property var pluginApi: null
    property var config: pluginApi?.pluginSettings || ({})
}
