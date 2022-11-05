const { invoke } = window.__TAURI__.tauri;

const list = document.getElementById("users");

let users = await invoke("get_users");
users.sort();

let plus = document.getElementById("add-user");
function make_user_button(name) {
    let button = document.createElement("button");
    button.innerText = name;

    list.insertBefore(button, plus);
}

users.forEach(make_user_button);