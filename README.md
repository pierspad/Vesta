# Vesta Application

## Standard Naming Convention for Series Mode
When processing a series in the Flashcards tab or using multiple media files, it is highly recommended to follow the standard naming convention so that the application can accurately detect the season and episode numbers and auto-pair the files.

**Format:**
`name_[season<number>]_[ep]<number>.<extension>`
or simply
`<name>_S<season>E<episode>.<extension>`

**Examples:**
- `12_angry_men_[season01]_[ep]01.mp4`
- `breaking_bad_s01e05.mp4`

This ensures that numbers that are part of the title (like "12 Angry Men") are not accidentally interpreted as the season or episode number. In this way, when separating generated APKGs, they will be cleanly exported as `<DeckName>_<EpisodeNumber>.apkg`.
