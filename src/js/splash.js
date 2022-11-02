const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;

await appWindow.show();

async function word_page() {
    let dicts = await invoke("get_dict_list", {});

    await invoke("set_dict", {dict: dicts[0]});
    await invoke("pick_next_word", {});

    window.location.replace("question.html");
}

window.word_page = word_page;