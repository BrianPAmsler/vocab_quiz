const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

await appWindow.show();

async function word_page() {
    window.location.replace("dict-list.html");
}

window.word_page = word_page;