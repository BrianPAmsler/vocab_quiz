const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;
const appWindow = getCurrentWindow();

async function word_page() {
    window.location.replace("dict-list.html");
}

window.word_page = word_page;

async function main() {
    await appWindow.show();
}

main();