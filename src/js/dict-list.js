const { invoke } = window.__TAURI__.tauri;
const { open, save }  = window.__TAURI__.dialog;

const list = document.getElementById("dicts");

let dicts = await invoke("get_dict_list");
dicts.sort();

async function pick_dict(dict) {
    await invoke("set_dict", {dict: dict});
    await invoke("pick_next_word", {});

    window.location.replace("question.html");
}

let plus = document.getElementById("add-dict");
function make_dict_button(dict) {
    let button = document.createElement("button");
    button.innerText = dict.name;
    button.onclick = async (e) => {
        await pick_dict(dict);
    }

    list.insertBefore(button, plus);
}

dicts.forEach(make_dict_button);

async function load_dict() {
    let files = await open({
        multiple: true,
        filters: [{
          name: 'Dictionary',
          extensions: ['dct', 'xml']
        }]
      });

    console.log(files);
    files.forEach(async (f) => {
        await invoke("import_dict", {filename: f});
    })

    await invoke("reload_files");
    window.location.reload();
}

window.load_dict = load_dict;