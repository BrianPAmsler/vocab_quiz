const { invoke } = window.__TAURI__.core;

const params = new Proxy(new URLSearchParams(window.location.search), {
    get: (searchParams, prop) => searchParams.get(prop),
});

let word_obj = null;

const yes_no = document.getElementById("yesno");
const chk = document.getElementById("check");
const word = document.getElementById("word");
const pron = document.getElementById("pronunciation");
const def = document.getElementById("definition");

word.style.display = "";

async function next(result) {
    await invoke("practice_current_word", {result: result});

    let count = await invoke("get_remaining_words");

    if (count > 0) {
        await invoke("pick_next_word");

        window.location.replace("question.html");
    } else {
        await invoke("conclude_session");
        await invoke("save_current_user");

        window.location.replace("index.html");
    }
}

function yes() {
    next(true);
}

function no() {
    next(false);
}

function check() {
    chk.style.display = "none";
    yes_no.style.display = "";

    def.style.display = "";

    if (word_obj.pronunciation != null) {
        pron.style.display = "";
    }
}

window.yes = yes;
window.no = no;
window.check = check;

async function main() {
    word_obj = await invoke("get_current_word");

    word.innerText = word_obj.text;
    pron.innerHTML = "Pronunciation:<br>" + word_obj.pronunciation;
    def.innerHTML = "Definition:<br>" + word_obj.definition;
}

main();