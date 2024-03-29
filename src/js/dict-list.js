const { invoke } = window.__TAURI__.tauri;
const { open, save }  = window.__TAURI__.dialog;

const list = document.getElementById("dicts");

const fit = window.fit;

let dicts = await invoke("get_dict_list");
dicts.sort();

async function pick_dict(dict) {
    await invoke("set_dict", {dict: dict});
    
    let r = await invoke("start_practice_session")

    if (r) {
        await invoke("pick_next_word", {});
        window.location.replace("question.html");
    }
}

let plus = document.getElementById("add-dict");
console.log(plus.clientHeight);
function make_dict_button(dict, count=0) {
    let button = document.createElement("button");
    let div = document.createElement("div");
    button.appendChild(div);
    div.innerText = dict.name;
    div.classList.add("button");
    button.onclick = async (e) => {
        await pick_dict(dict);
    }

    console.log("count: " + count);
    if (count > 0) {
        let notification_count = document.createElement("div");
        button.appendChild(notification_count);
        notification_count.id = "notification-count";
        notification_count.innerText = count;
    }

    list.insertBefore(button, plus);
    
    fit(div, 4);
}

dicts.forEach(async (dict) => {
    let count = await invoke("get_pool_size", {dict: dict});
    make_dict_button(dict, count);
});

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