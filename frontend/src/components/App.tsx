import React from 'react';
import Canvas from './Canvas';
import RightPanel from './RightPanel';
import simulation from '../stores/SimulationStore';
import { Box } from '@mui/material';
import { InteractionService } from '../services/InteractionService';

function App() {

  const interactionService = new InteractionService(simulation, window.innerWidth, window.innerHeight);

  return (
    <Box style={{ display: 'flex' }}>
      <Canvas simulation={simulation} interaction={interactionService}/>
      <RightPanel simulation={simulation}/>
    </Box>
  );
}

export default App;
