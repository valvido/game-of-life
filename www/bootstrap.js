// A dependency graph that contains any wasm must all be imported
// asynchronously. This `bootstrap.js` file does the single async import, so
// that no one else needs to worry about it again.
import("../pkg/wasm_game_of_life.js").then(({ Universe }) => {
  console.log("WebAssembly Loaded!");

  let universe;
  let intervalId;

  // Initialize the simulation and start the button functionality
  document.getElementById("startButton").addEventListener("click", () => {
    const width = parseInt(document.getElementById("width").value);
    const height = parseInt(document.getElementById("height").value);
    const aliveCount = parseInt(document.getElementById("aliveCount").value);
    const iterations = parseInt(document.getElementById("iterations").value);

    // Initialize the universe with the user inputs
    universe = Universe.new(width, height, aliveCount);

    // Start the continuous simulation with an interval
    if (intervalId) {
      clearInterval(intervalId); // Clear any previous interval
    }

    let count = 0;
    intervalId = setInterval(() => {
      if (count >= iterations) {
        clearInterval(intervalId); // Stop the simulation after the specified iterations
      } else {
        // Run the simulation for one iteration
        universe.tick();
        count++;

        // Display the updated universe grid as a string
        document.getElementById("game-of-life-canvas").textContent = universe.render();
      }
    }, 400); // Update every 100ms (you can adjust this speed)
  });
}).catch((err) => {
  console.error("Failed to initialize WebAssembly:", err);
});


