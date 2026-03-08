1) al momento il drag and drop dei file srt o media è disattivato, se vado per trascinare qualche media sopra mi segna l'icona del divieto e non posso dropparlo

vorrei poter droppare i film seguendo queste regole:


2) se l'utente droppa 3 film nominati tipo

1. <nome>.native.<lingua>.srt
2. <nome>.<lingua>.srt
3. <nome>.<lingua>.mp4 / mp3 o quello chevuoi

sai bene che 1) va in original subtitle track, 2) va in reference translation e 3) in media file


nella modalità serie tv invece metti un singolo tasto dove si possono selezionare più file (o sempre lascia la posisibilità di trascinarli dentro) e assicurati che vengano accoppiati secondo questa logica



3) nella tab translation, nel pannello translation options di base non selezionare nulla, al momento pare che sia selezionato Local LLM di default. Solo se l'utente clicca su local LLM deve illuinarsi e selezionarsi

se clicca sul tasto per cambiare modello qualunque modello sia stato scelto deve anche deselezionarsi

4) implementa il drag and drop dei file srt anche nella tab translation e assicurati che il caricamento funzioni, perchè al moemnto sto provando a caricarli anche dal file chooser e non pare funzionare


5) quando faccio fetch models lm studios tampa nel termianle che la richiesta l'ha ricevuta e ha risposto, ma l'app stampa load failed, fixa e assicurati che funzioni



6) nella tab settings, nel pannello card style, se vado sulle info di styling si vede tutto correttamente con il tooltip che va verso sopra, mentre se vado su front e back non si vede nulla, perchè il tooltip va sempre verso sopra ma sopra fc'è il limite della ui

puoi fare in modo che solo per front e back il tooltip vada verso sotto?

puoi fare anche in modo che ci sia il syntax highlighting di html e css nelle rispettive celle (anche qualcosa di base giusto per) e la numerazione del rigo per ogni new line?


7) fixa l'errore causato nella action per la build di windows 

 Compiling urlencoding v2.1.3
   Compiling hound v3.5.1
error[E0432]: unresolved import `srt_sync`
 --> apps\srt-gui\src-tauri\src\state.rs:5:5
  |
5 | use srt_sync::SyncEngine;
  |     ^^^^^^^^ use of unresolved module or unlinked crate `srt_sync`
  |
  = help: if you wanted to use a crate named `srt_sync`, use `cargo add srt_sync` to add it to your `Cargo.toml`

error[E0432]: unresolved import `srt_parser`
  --> apps\srt-gui\src-tauri\src\commands\translate.rs:11:5
   |
11 | use srt_parser::SrtParser;
   |     ^^^^^^^^^^ use of unresolved module or unlinked crate `srt_parser`
   |
   = help: if you wanted to use a crate named `srt_parser`, use `cargo add srt_parser` to add it to your `Cargo.toml`

error[E0432]: unresolved import `srt_translate`
  --> apps\srt-gui\src-tauri\src\commands\translate.rs:12:5
   |
12 | use srt_translate::{
   |     ^^^^^^^^^^^^^ use of unresolved module or unlinked crate `srt_translate`
   |
   = help: if you wanted to use a crate named `srt_translate`, use `cargo add srt_translate` to add it to your `Cargo.toml`

error[E0432]: unresolved import `srt_sync`
 --> apps\srt-gui\src-tauri\src\commands\sync.rs:7:5
  |
7 | use srt_sync::{SamplerStrategy, SyncEngine};
  |     ^^^^^^^^ use of unresolved module or unlinked crate `srt_sync`
  |
  = help: if you wanted to use a crate named `srt_sync`, use `cargo add srt_sync` to add it to your `Cargo.toml`

error[E0282]: type annotations needed for `Result<_, _>`
   --> apps\srt-gui\src-tauri\src\commands\translate.rs:244:9
    |
244 |     let translated = translate_subtitles_with_rate_limit_cancellable(
    |         ^^^^^^^^^^
...
261 |             let error_str = e.to_string();
    |                             - type must be known at this point
    |
help: consider giving `translated` an explicit type, where the placeholders `_` are specified
    |
244 |     let translated: Result<_, E> = translate_subtitles_with_rate_limit_cancellable(
    |                   ++++++++++++++

error[E0282]: type annotations needed
   --> apps\srt-gui\src-tauri\src\commands\translate.rs:258:9
    |
258 |     let translated = match translated {
    |         ^^^^^^^^^^
...
284 |         message: format!("Traduzione completata: {} sottotitoli", translated.len()),
    |                                                                   ---------- type must be known at this point
    |
help: consider giving `translated` an explicit type
    |
258 |     let translated: /* Type */ = match translated {
    |                   ++++++++++++

Some errors have detailed explanations: E0282, E0432.
For more information about an error, try `rustc --explain E0282`.
error: could not compile `vesta` (lib) due to 6 previous errors
failed to build app: failed to build app
       Error failed to build app: failed to build app
Error: Process completed with exit code 1.








