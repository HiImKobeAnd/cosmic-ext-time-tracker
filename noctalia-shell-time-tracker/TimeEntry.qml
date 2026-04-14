import QtQuick

QtObject {
    property var id: ""
    property bool billable: false
    property string description: ""
    property var startTime: new Date()
    property var stopTime: new Date()
}
