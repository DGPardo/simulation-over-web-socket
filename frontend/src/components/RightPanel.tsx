import React, { useEffect, useRef, useState } from 'react';
import { Simulation } from '../stores/SimulationStore';
import { Box, Button } from '@mui/material';

const REFRESH_INTERVAL = 50;

const FpsCounter = () => {
  const [fps, setFps] = useState(0);
  const lastFrameTimeRef = useRef(performance.now());
  const frameCountRef = useRef(0);
  const fpsIntervalRef = useRef<number | null>(null);

  useEffect(() => {
    const calculateFps = () => {
      const now = performance.now();
      frameCountRef.current += 1;

      // Check time difference
      if (now - lastFrameTimeRef.current >= 1000) { // Update FPS every second
        setFps(frameCountRef.current); // Update FPS state
        frameCountRef.current = 0; // Reset frame count
        lastFrameTimeRef.current = now; // Reset last frame time
      }

      requestAnimationFrame(calculateFps); // Request next frame
    };

    // Start the animation loop
    fpsIntervalRef.current = requestAnimationFrame(calculateFps);

    // Cleanup on component unmount
    return () => {
      cancelAnimationFrame(fpsIntervalRef.current!);
    };
  }, []);
  return <div>FPS: {fps}</div>;
};

type NumberOfBodiesProps = {
  simulation: Simulation;
};

const NumberOfBodies: React.FC<NumberOfBodiesProps> = ({ simulation }) => {
  const [numberOfBodies, setNumberOfBodies] = useState(simulation.getNumberOfBodies());
  useEffect(() => {
    const interval = setInterval(() => {
      setNumberOfBodies(simulation.getNumberOfBodies());
    }, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [simulation]);
  return <div>Number of bodies: {numberOfBodies}</div>;
}

type PhysicalTimeProps = {
  simulation: Simulation;
};

const PhysicalTime: React.FC<PhysicalTimeProps> = ({ simulation }) => {
  const [physicalTime, setPhysicalTime] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => {
      setPhysicalTime(simulation.getPhysicalTime());
    }, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [simulation]);
  return <div>Physical time: {physicalTime.toFixed(2)}</div>;
}

const MouseCoords = () => {
  const [mouseCoords, setMouseCoords] = useState([0, 0]);
  useEffect(() => {
    const handleMouseMove = (event: MouseEvent) => {
      setMouseCoords([event.clientX, event.clientY]);
    };
    window.addEventListener('mousemove', handleMouseMove);
    return () => window.removeEventListener('mousemove', handleMouseMove);
  }, []);
  return <div>Mouse coords: {mouseCoords.join(', ')}</div>;
}

type KineticEnergyProps = {
  simulation: Simulation;
};

const KineticEnergy: React.FC<KineticEnergyProps> = ({ simulation }) => {
  const [kineticEnergy, setKineticEnergy] = useState(0);
  useEffect(() => {
    const interval = setInterval(() => {
      const ke = simulation.getKineticEnergy();
      setKineticEnergy(ke);
    }, REFRESH_INTERVAL);
    return () => clearInterval(interval);
  }, [simulation]);
  return <div>Kinetic energy: {kineticEnergy.toFixed(2)}</div>;
}


export type RightPanelProps = {
  simulation: Simulation;
};

const RightPanel: React.FC<RightPanelProps> = ({ simulation }) => {
  return (
    <Box sx={{ position: 'absolute', top: 0, right: 0, padding: 2, zIndex: 10, color: "white" }}>
      <FpsCounter />
      <NumberOfBodies simulation={simulation}/>
      <PhysicalTime simulation={simulation}/>
      <KineticEnergy simulation={simulation}/>
      <MouseCoords />
      <Button onClick={() => simulation.reset()}>
        Reset
      </Button>
    </Box>
  )
};

export default RightPanel;
