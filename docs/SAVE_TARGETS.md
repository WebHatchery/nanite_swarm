# Save behavior by target

| Target | Menu Quit | OS window close | Storage |
| --- | --- | --- | --- |
| Native desktop | Saves the active slot before quitting | No reliable macroquad callback is exposed, so autosave and menu transitions are the protection | Toolkit native key store |
| Browser/WASM | The Quit control is omitted because a browser tab cannot be closed safely | `beforeunload` is not a reliable persistence contract for this game | Toolkit local storage |

Native autosaves also run on the configured cadence, before leaving the
planetary screen, after travel/launch, and when the campaign ends. The visible
slot retains three rotated recovery generations. A browser player should use
the menu or leave the tab after the autosave marker appears rather than rely on
the browser closing event.
