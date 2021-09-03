
async function main() {
  let timerHandle = null;

  function setRandom(founds) {
      const index = Math.floor(Math.random() * founds.items.length)
      // const index = 0
      const path = founds.items[index].file.path;
      document.getElementById('image').src = `/file?path=${encodeURIComponent(path)}`
  }

  function toggleFullScreen() {
    if (!document.fullscreenElement) {
        document.documentElement.requestFullscreen();
    } else {
      if (document.exitFullscreen) {
        document.exitFullscreen();
      }
    }
  }

  function toggleClass(element, name) {
    let classes = element.getAttribute('class', '').split(/ +/);
    let found = classes.some(it => it == name);
    if (found)
      element.setAttribute('class', classes.filter(it => it != name).join(' '))
    else
      element.setAttribute('class', classes + ' ' + name)
  }

  async function search(expression) {
    if (timerHandle)
      clearInterval(timerHandle)

    const founds = await fetch('/search', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({expression}),
    }).then(it => it.json());

    setRandom(founds)
    timerHandle = setInterval(_ => setRandom(founds), 10 * 1000)

  }

  function onMenuSwitch(e) {
    Array.from(document.querySelectorAll('.menu-panel')).forEach(it => toggleClass(it, 'hidden'));
    e.preventDefault()
    e.stopPropagation()
  }

  function onSearchButton(e) {
    e.preventDefault()
    e.stopPropagation()
    const expression = document.querySelector('#search-expression').value
    search(expression)
    onMenuSwitch()
  }

  search(`path like '%wallpaper%'`)
  document.querySelector('#image').addEventListener('click', e => toggleFullScreen(), true);
  document.querySelector('#search-button').addEventListener('click', onSearchButton, true);
  Array.from(document.querySelectorAll('.menu-switch')).forEach(it => it.addEventListener('mouseenter', onMenuSwitch, false));
  Array.from(document.querySelectorAll('.menu-panel')).forEach(it => it.addEventListener('mouseleave', onMenuSwitch, true));
}

main()
