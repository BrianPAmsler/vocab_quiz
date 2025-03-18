const { invoke } = window.__TAURI__.core;

const form = document.getElementById("user-form");
const err_txt = document.getElementById("error-text");

form.addEventListener("submit", async (e) => {
    e.preventDefault();

    let form_data = new FormData(form);

    var name = form_data.get("name");

    await invoke("create_user", {name: name})
    .then(() => document.location.replace("user-list.html"))
    .catch((error_message) => {
        err_txt.innerText = error_message;
    });
});