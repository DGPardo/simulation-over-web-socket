import React, { useEffect } from "react";
import { Simulation } from "../stores/SimulationStore";
import { InteractionService } from "../services/InteractionService";
import { Body } from "wasm-bindings";
import { CameraStore } from "../stores/CameraStore";

type CanvasProps = {
  simulation: Simulation;
  interaction: InteractionService;
};

const Canvas: React.FC<CanvasProps> = ({simulation, interaction}) => {
  const canvasRef = React.useRef<HTMLCanvasElement>(null);
  const camera = interaction.camera;

  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }
    const canvas = canvasRef.current;
    const context = canvas.getContext("2d");

    if (!context) {
      console.error("Could not get 2D rendering context");
      return;
    }

    // Draw function
    const draw = () => {
      // Clear the canvas
      context.clearRect(0, 0, canvas.width, canvas.height);

      simulation.step();

      // Draw each body
      for (let i = 0; i < simulation.getNumberOfBodies(); i++) {
        const body: Body = simulation.getBody(i);
        drawBody(context, body, camera);
      };

      if (interaction.stagedBody) {
        // Visualize the staged body
        const body = interaction.stagedBody;
        drawBody(context, body, camera);
        const [x, y] = camera.worldToScreen(body.position[0], body.position[1]);
        const [vx, vy] = body.velocity;
        context.beginPath();
        context.moveTo(x, y);
        context.lineTo(x + vx * camera.zoom, y + vy *  camera.zoom);
        context.strokeStyle = "white";
        context.stroke();
        context.closePath();
      }

      requestAnimationFrame(draw);
    };

    // Start the animation
    draw();
  }, [canvasRef, interaction.stagedBody, simulation, camera]);

  // Resize the canvas when the window changes size
  useEffect(() => {
    const handleResize = () => {
      if (canvasRef.current) {
        canvasRef.current.width = window.innerWidth;
        canvasRef.current.height = window.innerHeight;
        camera.resize(window.innerWidth, window.innerHeight);
      }
    };

    window.addEventListener("resize", handleResize);

    // Initial resize
    handleResize();

    return () => {
      window.removeEventListener("resize", handleResize);
    };
  }, [canvasRef, camera]);
  return (
      <canvas
        tabIndex={0}
        ref={canvasRef}
        width={window.innerWidth}
        height={window.innerHeight}
        onMouseDown={interaction.onMouseDown}
        onMouseMove={interaction.onMouseMove}
        onKeyDown={interaction.onKeyDown}
        onMouseUp={interaction.onMouseUp}
        onWheel={interaction.onWheel}
        onMouseEnter={interaction.onMouseEnter}
        onMouseLeave={interaction.onMouseLeave}
        style={{
          background: "black",
          position: "fixed",
          top: 0,
          left: 0,
          width: "100vw",
          height: "100vh",
          zIndex: 0,
        }}
      />
  );
};

const drawBody = (context: CanvasRenderingContext2D, body: Body, camera: CameraStore): void => {
  context.beginPath();

  const [x, y] = camera.worldToScreen(body.position[0], body.position[1]);

  context.arc(x, y, body.radius * camera.zoom, 0, Math.PI * 2);
  context.fillStyle = `rgba(${body.color[0]}, ${body.color[1]}, ${body.color[2]}, ${body.color[3]})`;
  context.fill();
  context.closePath();
}

export default Canvas;
