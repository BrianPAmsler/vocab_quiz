const { invoke } = window.__TAURI__.tauri;

const list = document.getElementById("users");

let users = await invoke("get_users");
users.sort();

let buttons = [];

async function update_buttons() {
    let current_user = await invoke("get_current_user");
    
    buttons.forEach((button) => {
        if (button.innerText === current_user.name) {
            button.classList.add("selected");
        } else {
            button.classList.remove("selected");
        }
    })
}

let plus = document.getElementById("add-user");
function make_user_button(user) {
    let button = document.createElement("button");
    button.innerText = user.name;

    button.onclick = async () => {
        await invoke("set_current_user", {user: user});
        await update_buttons();
    }

    list.insertBefore(button, plus);
    buttons.push(button);
}

window.back = () => {
    window.location.replace("index.html");
}

users.forEach(make_user_button);
await update_buttons();