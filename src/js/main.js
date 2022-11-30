document.addEventListener('contextmenu', event => event.preventDefault());

function fit(element, initial_size) {
    var s = initial_size;
    while (element.scrollHeight > element.clientHeight) {
        s -= 0.1;
        element.style.fontSize = s + "vw";
    }

    console.log(element.scrollHeight + ", " + element.clientHeight + ", " + s);
}

window.fit = fit;