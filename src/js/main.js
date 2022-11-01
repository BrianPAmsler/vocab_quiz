const { invoke } = window.__TAURI__.tauri;
const { appWindow } = window.__TAURI__.window;
await appWindow.show();

document.addEventListener('contextmenu', event => event.preventDefault());

let greetMsgEl = document.getElementById("greet-msg");

async function greet(name) {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsgEl.textContent = await invoke("greet", { name: name });
}

window.greet = greet;

let form = document.getElementById("greet-form");
let input = document.getElementById("greet-input");

form.addEventListener('submit', (e) => {
  e.preventDefault();
  
  greet(input.value);

  input.value = ""; 
});