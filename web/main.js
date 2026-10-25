// Loads the WebAssembly module and starts the game in #game.
import init, { start } from './pkg/through_the_solar_system.js';

const container = document.getElementById('game');

try {
  await init();
  start(container, { assets: './textures/' });
  container.focus();
} catch (err) {
  console.error(err);
  if (!container.textContent) {
    container.textContent = 'The game could not start in this browser.';
  }
}
