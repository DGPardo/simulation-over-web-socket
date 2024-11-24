import React from "react";
import * as wasm from "wasm-nbody";
import { WasmSimulation } from "wasm-nbody";

const wasmState = await wasm.default();

const Surface: React.FC = () => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null);

  React.useEffect(() => {
    if (canvasRef.current) {
      const simulation = new WasmSimulation(); // Initialize the simulation
      const canvas = canvasRef.current;
      const context = canvas.getContext("2d");

      if (!context) {
        console.error("Could not get 2D rendering context");
        return;
      }

      simulation.addBody(+10, +10, 1);
      simulation.addBody(-10, +10, 1);
      simulation.addBody(-10, -10, 1);
      simulation.addBody(+10, -10, 1);

      // Draw function
      const draw = () => {
        // Clear the canvas
        context.clearRect(0, 0, canvas.width, canvas.height);

        const nBodies: number = simulation.numberOfBodies();
        const xPosition = new Float32Array(wasmState.memory.buffer, simulation.xPosition(), nBodies);
        const yPosition = new Float32Array(wasmState.memory.buffer, simulation.yPosition(), nBodies);

        // Draw each body
        for (let i = 0; i < nBodies; i++) {
          const x = xPosition[i];
          const y = yPosition[i];
          const radius = 5;

          context.beginPath();
          context.arc(x, y, radius, 0, Math.PI * 2);
          context.fillStyle = "white"; // Adjust the color if needed
          context.fill();
          context.closePath();
        };

        // Request the next frame
        requestAnimationFrame(draw);
      };

      // Start the animation
      draw();
    }
  }, [canvasRef]);

  return (
    <canvas ref={canvasRef} width={800} height={600} style={{ background: "black" }} />
  );
};

export default Surface;
