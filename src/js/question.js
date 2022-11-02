const { invoke } = window.__TAURI__.tauri;

const params = new Proxy(new URLSearchParams(window.location.search), {
    get: (searchParams, prop) => searchParams.get(prop),
});

let word_obj = await invoke("get_current_word", {});

const yes_no = document.getElementById("yesno");
const chk = document.getElementById("check");
const word = document.getElementById("word");
const pron = document.getElementById("pronunciation");
const def = document.getElementById("definition");

word.innerText = word_obj.text;
pron.innerHTML = "Pronunciation:<br>" + word_obj.pronunciation;
def.innerHTML = "Definition:<br>" + word_obj.definition;

async function next() {
    await invoke("pick_next_word", {});

    window.location.replace("question.html");
}

function yes() {
    next();
}

function no() {
    next();
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