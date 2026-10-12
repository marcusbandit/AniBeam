// Text cut to one of the four roles of the face (Tokens.roles): display, title, ui, ledger.
// The role sets the width and weight axes, the size and, for the ledger, tabular figures;
// any of them can still be overridden per instance (font.pointSize: theme.typeNormal on a
// title that sits in a small tile, say). Colour defaults to the theme's text.
import QtQuick

Text {
    property string role: "ui"
    color: theme.text
    font.family: theme.fontFamily
    font.pointSize: theme.roleSize(role)
    font.weight: theme.roleWeight(role)
    font.variableAxes: theme.roleAxes(role)
    font.features: theme.roleFeatures(role)
}
