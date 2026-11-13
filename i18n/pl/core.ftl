# pl translation of en/core.ftl. An ID missing here is shown in English.

## Counts

count-episodes =
    { $count ->
        [one] { $count } odcinek
        [few] { $count } odcinki
       *[many] { $count } odcinków
    }
count-shows =
    { $count ->
        [one] { $count } serial
        [few] { $count } seriale
       *[many] { $count } seriali
    }
