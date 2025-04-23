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

window.yes = () => {
    next(true);
}

window.no = () => {
    next(false);
}

window.check = () => {
    chk.style.display = "none";
    yes_no.style.display = "";

    def.style.display = "";

    if (word_obj.pronunciation != null) {
        pron.style.display = "";
    }
}

async function main() {
    let word = await invoke("get_current_word");
    word_obj = word[0];
    let practice_direction = word[1];

    if (practice_direction == "Forward") {
        word.innerText = word_obj.text;
        pron.innerHTML = "Pronunciation:<br>" + word_obj.pronunciation;
        def.innerHTML = "Definition:<br>" + word_obj.definition;
    } else if (practice_direction == "Backward") {
        word.innerText = word_obj.definition;
        pron.innerHTML = "Pronunciation:<br>" + word_obj.pronunciation;
        def.innerHTML = "Word:<br>" + word_obj.text;
    }
}

main();