// A page that is not built yet. Each page task replaces its use in Frame's page map.
import QtQuick

PageScroll {
    id: page
    property var props: ({})
    property string title: frame.nav.current.label
    Text { text: page.title; color: theme.text; font.family: theme.fontDisplay; font.pointSize: theme.typeLarge; font.weight: Font.DemiBold }
    Text { text: "Coming soon"; color: theme.textDim; font.family: theme.fontSans; font.pointSize: theme.typeNormal }
}
