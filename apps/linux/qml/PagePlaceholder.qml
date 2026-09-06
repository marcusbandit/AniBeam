// A page that is not built yet. Each page task replaces its use in Frame's page map.
import QtQuick

PageScroll {
    id: page
    property var props: ({})
    property string title: frame.nav.current.label
    Text { text: page.title; color: theme.text; font.family: theme.fontSans; font.pointSize: theme.typeLarge; font.weight: Font.Bold }
    Text { text: "Not built yet"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
}
