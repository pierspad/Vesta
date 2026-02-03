# Developer Guide

## Localization
Translation files are located in:
`srt-gui/src/lib/i18n/`

To add a new language or modify existing translations, edit the corresponding JSON files in this directory.

## Predefined Models
Predefined models and provider configurations are defined in:
`srt-gui/src/lib/models.ts`

To add information for new models (e.g., context window, recommended status) or update provider endpoints, modify the `modelsByProvider` object in this file.
