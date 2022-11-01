const { appWindow } = window.__TAURI__.window;

var html = `<div class="titlebar-button" id="titlebar-minimize">
<img
    src="https://api.iconify.design/mdi:window-minimize.svg"
    alt="minimize"
/>
</div>
<div class="titlebar-button" id="titlebar-maximize">
<img
    src="https://api.iconify.design/mdi:window-maximize.svg"
    alt="maximize"
/>
</div>
<div class="titlebar-button" id="titlebar-close">
<img src="https://api.iconify.design/mdi:close.svg" alt="close" />
</div>`

let node = document.createElement('div');
node.setAttribute('data-tauri-drag-region', '');
node.setAttribute('class', 'titlebar');

node.innerHTML = html;

let bg = document.getElementById("bg");
document.body.insertBefore(node, bg);
    
document
.getElementById('titlebar-minimize')
.addEventListener('click', () => appWindow.minimize());
document
.getElementById('titlebar-maximize')
.addEventListener('click', () => appWindow.toggleMaximize());
document
.getElementById('titlebar-close')
.addEventListener('click', () => appWindow.close());

document.addEventListener('keydown', (e) => {
    if (e.altKey) {
      //e.preventDefault();
    }
  });