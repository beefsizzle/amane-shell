import QtQuick
import QtQml
import QtQml.Models

/*
 * daru's lock screen as SDDM's login screen. the wallpaper and colours come from theme.conf.user, which the shell
 * rewrites whenever the wallpaper changes (its `sddm` recolor target), so this shows the wallpaper you last had.
 * only plain QtQuick is used: the greeter runs before anything else, often on X11, and a theme that fails to load
 * leaves no way to log in
 */
Rectangle {
    id: root

    width: 1920
    height: 1080
    color: colour("backgroundColor", "#151411")

    // a colour from the theme's config, or the fallback when it's missing or isn't one
    function colour(key, fallback) {
        const value = String(config.stringValue(key) || "")

        return /^#[0-9a-fA-F]{6}$/.test(value) ? value : fallback
    }

    function faded(color, amount) {
        return Qt.rgba(color.r, color.g, color.b, amount)
    }

    readonly property color textColor: colour("text", "#edebe8")
    readonly property color dimColor: colour("dim", "#9b9996")
    readonly property color faintColor: colour("faint", "#6d6b68")
    readonly property color accentColor: colour("accent", "#edc26e")
    readonly property color dangerColor: colour("danger", "#f17477")

    readonly property string textFont: "Adwaita Sans"
    readonly property string iconFont: "JetBrainsMono Nerd Font"

    property int userIndex: Math.max(userModel.lastIndex, 0)
    property int sessionIndex: Math.max(sessionModel.lastIndex, 0)
    property bool busy: false
    property bool failed: false
    property date now: new Date()

    // the models hand their rows only to delegates, so these keep one small object per row to read the names from
    Instantiator {
        id: users

        model: userModel

        delegate: QtObject {
            readonly property string name: model.name
        }
    }

    Instantiator {
        id: sessions

        model: sessionModel

        delegate: QtObject {
            readonly property string name: model.name
        }
    }

    // the rows arrive after the screen is built, so these follow the counts as well as the chosen index
    readonly property var user: users.count > 0 ? users.objectAt(Math.min(userIndex, users.count - 1)) : null
    readonly property var session: sessions.count > 0 ? sessions.objectAt(Math.min(sessionIndex, sessions.count - 1)) : null

    readonly property string userName: user ? user.name : ""
    readonly property string shownName: user ? user.name : ""
    readonly property string sessionName: session ? session.name : ""

    function login() {
        if (busy || password.text.length === 0) {
            return
        }

        busy = true
        failed = false

        sddm.login(root.userName, password.text, root.sessionIndex)
    }

    Connections {
        target: sddm

        function onLoginFailed() {
            root.busy = false
            root.failed = true

            password.text = ""
            password.forceActiveFocus()
        }

        function onLoginSucceeded() {
            root.busy = false
        }
    }

    Timer {
        interval: 1000
        running: true
        repeat: true

        onTriggered: root.now = new Date()
    }

    // the shell blurs the wallpaper before it's copied here, so this needs no GPU effect
    Image {
        anchors.fill: parent

        source: config.stringValue("background") ? "file://" + config.stringValue("background") : ""
        fillMode: Image.PreserveAspectCrop
        smooth: true
        cache: false
    }

    Rectangle {
        anchors.fill: parent

        color: "#000000"
        opacity: 0.48
    }

    Column {
        anchors.centerIn: parent

        spacing: 12

        Text {
            anchors.horizontalCenter: parent.horizontalCenter

            text: Qt.formatTime(root.now, "HH:mm")
            color: root.textColor
            font.family: root.textFont
            font.pixelSize: 128
            font.weight: Font.Light
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter

            text: Qt.formatDate(root.now, "dddd d MMMM")
            color: root.dimColor
            font.family: root.textFont
            font.pixelSize: 22
        }

        Item {
            width: 1
            height: 48
        }

        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter

            width: 76
            height: 76
            color: root.faded(root.accentColor, 0.22)
            border.width: 1.5
            border.color: root.faded(root.accentColor, 0.6)

            Text {
                anchors.centerIn: parent

                text: root.shownName.charAt(0).toUpperCase()
                color: root.accentColor
                font.family: root.textFont
                font.pixelSize: 32
                font.bold: true
            }
        }

        // with more than one account, the arrows beside the name switch between them
        Row {
            anchors.horizontalCenter: parent.horizontalCenter

            spacing: 10

            Text {
                anchors.verticalCenter: parent.verticalCenter

                visible: users.count > 1
                text: "\u{f0141}"
                color: previousUser.containsMouse ? root.textColor : root.faintColor
                font.family: root.iconFont
                font.pixelSize: 16

                MouseArea {
                    id: previousUser

                    anchors.fill: parent
                    anchors.margins: -6
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor

                    onClicked: {
                        root.userIndex = (root.userIndex + users.count - 1) % users.count
                        root.failed = false
                        password.forceActiveFocus()
                    }
                }
            }

            Text {
                text: root.shownName
                color: root.textColor
                font.family: root.textFont
                font.pixelSize: 17
                font.bold: true
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter

                visible: users.count > 1
                text: "\u{f0142}"
                color: nextUser.containsMouse ? root.textColor : root.faintColor
                font.family: root.iconFont
                font.pixelSize: 16

                MouseArea {
                    id: nextUser

                    anchors.fill: parent
                    anchors.margins: -6
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor

                    onClicked: {
                        root.userIndex = (root.userIndex + 1) % users.count
                        root.failed = false
                        password.forceActiveFocus()
                    }
                }
            }
        }

        Item {
            width: 1
            height: 6
        }

        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter

            width: 340
            height: 46
            color: "#59000000"
            border.width: 1
            border.color: root.failed ? root.dangerColor : root.faded(root.textColor, 0.25)

            TextInput {
                id: password

                anchors.fill: parent
                anchors.leftMargin: 14
                anchors.rightMargin: 14

                verticalAlignment: TextInput.AlignVCenter
                echoMode: TextInput.Password
                passwordCharacter: "•"
                color: root.textColor
                selectionColor: root.faded(root.accentColor, 0.4)
                font.family: root.textFont
                font.pixelSize: 16
                focus: true
                enabled: !root.busy
                clip: true

                onAccepted: root.login()
                onTextEdited: root.failed = false

                Keys.onEscapePressed: text = ""
            }

            Text {
                anchors.left: parent.left
                anchors.leftMargin: 14
                anchors.verticalCenter: parent.verticalCenter

                visible: password.text.length === 0
                text: "Password"
                color: root.faded(root.textColor, 0.45)
                font.family: root.textFont
                font.pixelSize: 16
            }
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter

            text: root.busy ? "Logging in…"
                : root.failed ? "Wrong password, try again"
                : keyboard.capsLock ? "Caps Lock is on"
                : "Enter your password to log in"
            color: root.failed ? root.dangerColor : keyboard.capsLock ? root.accentColor : root.dimColor
            font.family: root.textFont
            font.pixelSize: 13
        }
    }

    // the session on the left, which a click switches, and the power buttons on the right
    Item {
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 28

        height: 30

        Row {
            anchors.left: parent.left
            anchors.verticalCenter: parent.verticalCenter

            spacing: 8

            Text {
                anchors.verticalCenter: parent.verticalCenter

                text: "\u{f0379}"
                color: sessionSwitch.containsMouse ? root.accentColor : root.faintColor
                font.family: root.iconFont
                font.pixelSize: 14
            }

            Text {
                anchors.verticalCenter: parent.verticalCenter

                text: root.sessionName
                color: sessionSwitch.containsMouse ? root.textColor : root.faintColor
                font.family: root.textFont
                font.pixelSize: 13
            }
        }

        MouseArea {
            id: sessionSwitch

            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom

            width: 240
            hoverEnabled: true
            cursorShape: sessions.count > 1 ? Qt.PointingHandCursor : Qt.ArrowCursor

            onClicked: {
                const count = sessions.count

                if (count > 1) {
                    root.sessionIndex = (root.sessionIndex + 1) % count
                }

                password.forceActiveFocus()
            }
        }

        Text {
            anchors.centerIn: parent

            text: sddm.hostName
            color: root.faintColor
            font.family: root.textFont
            font.pixelSize: 12
        }

        Row {
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter

            spacing: 6

            Repeater {
                model: [
                    { glyph: "\u{f04b2}", allowed: sddm.canSuspend, act: function () { sddm.suspend() } },
                    { glyph: "\u{f0709}", allowed: sddm.canReboot, act: function () { sddm.reboot() } },
                    { glyph: "\u{f0425}", allowed: sddm.canPowerOff, act: function () { sddm.powerOff() } },
                ]

                delegate: Rectangle {
                    required property var modelData

                    visible: modelData.allowed
                    width: 30
                    height: 30
                    color: button.containsMouse ? root.faded(root.textColor, 0.12) : "transparent"

                    Text {
                        anchors.centerIn: parent

                        text: modelData.glyph
                        color: button.containsMouse ? root.textColor : root.faintColor
                        font.family: root.iconFont
                        font.pixelSize: 15
                    }

                    MouseArea {
                        id: button

                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor

                        onClicked: modelData.act()
                    }
                }
            }
        }
    }

    Component.onCompleted: password.forceActiveFocus()
}
