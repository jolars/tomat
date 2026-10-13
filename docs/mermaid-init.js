// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

(async () => {
    if (!document.querySelector(".mermaid")) return;

    // Resolve against this script so nested chapters use the same local library.
    const libraryUrl = new URL("mermaid.min.js", document.currentScript.src);
    try {
        await new Promise((resolve, reject) => {
            const script = document.createElement("script");
            script.src = libraryUrl.href;
            script.onload = resolve;
            script.onerror = reject;
            document.head.append(script);
        });
    } catch (error) {
        console.error("Unable to load Mermaid diagrams.", error);
        return;
    }
    const darkThemes = ['ayu', 'navy', 'coal'];
    const lightThemes = ['light', 'rust'];

    const classList = document.getElementsByTagName('html')[0].classList;

    let lastThemeWasLight = true;
    for (const cssClass of classList) {
        if (darkThemes.includes(cssClass)) {
            lastThemeWasLight = false;
            break;
        }
    }

    const theme = lastThemeWasLight ? 'default' : 'dark';
    mermaid.initialize({ startOnLoad: false, theme });
    await mermaid.run();

    // Reloading makes Mermaid redraw the diagrams with the newly selected theme.

    for (const darkTheme of darkThemes) {
        const button = document.getElementById(`mdbook-theme-${darkTheme}`) ?? document.getElementById(darkTheme);
        button?.addEventListener('click', () => {
            if (lastThemeWasLight) {
                window.location.reload();
            }
        });
    }

    for (const lightTheme of lightThemes) {
        const button = document.getElementById(`mdbook-theme-${lightTheme}`) ?? document.getElementById(lightTheme);
        button?.addEventListener('click', () => {
            if (!lastThemeWasLight) {
                window.location.reload();
            }
        });
    }
})();
