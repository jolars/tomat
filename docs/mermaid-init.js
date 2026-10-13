// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

(() => {
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
    mermaid.initialize({ startOnLoad: true, theme });

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
