const { invoke } = window.__TAURI__.tauri;

const params = new Proxy(new URLSearchParams(window.location.search), {
    get: (searchParams, prop) => searchParams.get(prop),
  });

let word = await invoke("get_word", {word: params.word});

let word_el = document.getElementById("word");
let def_el = document.getElementById("definition");
let pron_el = document.getElementById("pronunciation");

word_el.innerText = "Word: " + word.text;
def_el.innerText = "Definition: " + word.definition;
pron_el.innerText = "Pronunciation: " + word.pronunciation;