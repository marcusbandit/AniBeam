// Text cut to one of the roles (Tokens.roles): display, title, ui, ledger, and figure, the
// numerals face for numbers on their own.
// The role sets the width and weight axes, the size and, for the ledger, tabular figures;
// any of them can still be overridden per instance (font.pointSize: theme.typeNormal on a
// title that sits in a small tile, say). Colour defaults to the theme's text.
import QtQuick

Text {
    property string role: "ui"
    // The words by message ID (#792), `msgArgs` filled in; `text` set directly wins
    property string msg: ""
    property var msgArgs: ({})
    text: msg ? words.t(msg, msgArgs) : ""
    color: theme.text
    font.family: theme.roleFamily(role)
    font.pointSize: theme.roleSize(role)
    font.weight: theme.roleWeight(role)
    font.variableAxes: theme.roleAxes(role)
    font.features: theme.roleFeatures(role)
}
